#!/usr/bin/env python3
"""Reproduce ADR-0001 probes from current sources or an archived Git revision.

Writes new receipts only. Baseline uses the saved pre-status probe; current uses
the example. Both have identical workload, allocator and timing boundaries.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def output(*args, cwd=ROOT):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def identity(root):
    if root == ROOT:
        names = output("git", "ls-files", "--cached", "--others", "--exclude-standard", "--", "Cargo.toml", "Cargo.lock", "crates", "vendor").splitlines()
    else:
        names = [str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()]
    hashes = {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
              for name in sorted(set(names)) if (root / name).is_file()}
    digest = hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest()
    return {"sha256": digest, "files": hashes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", help="Git revision to archive; omitted for the current worktree")
    parser.add_argument("--output", type=Path, required=True, help="new receipt prefix")
    parser.add_argument("--freeze-binaries", type=Path, help="copy measured executables for paired reruns")
    args = parser.parse_args()
    prefix = args.output.resolve()
    suffixes = [".jsonl", "-original.csv", "-scaling.csv", "-metadata.json", "-build.log"]
    if any(Path(str(prefix) + s).exists() for s in suffixes):
        parser.error("receipt exists; choose a new output prefix")
    prefix.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="adr0001-", dir=ROOT / "target") as tmp:
        source = ROOT
        revision = output("git", "rev-parse", args.baseline or "HEAD")
        if args.baseline:
            source = Path(tmp) / "source"
            source.mkdir()
            archive = Path(tmp) / "baseline.tar"
            subprocess.run(["git", "archive", revision, "Cargo.toml", "Cargo.lock", "crates", "-o", str(archive)], cwd=ROOT, check=True)
            subprocess.run(["tar", "-xf", str(archive), "-C", str(source)], check=True)
            shutil.copyfile(ROOT / "tools/benchmarks/0001-adoption-baseline.rs", source / "crates/core/examples/adoption_cost.rs")
        metadata = {
            "revision": revision, "source": "archive" if args.baseline else "worktree",
            "identity": identity(source), "rustc": output("rustc", "-Vv"),
            "platform": platform.platform(), "cpu": output("sysctl", "-n", "machdep.cpu.brand_string"),
            "samples": {"adoption": 7, "original": 31, "scaling": 7}, "warmups": 1,
            "commands": [],
        }
        env = dict(os.environ)
        # Cargo can consider archived files fresh by mtime when two source trees
        # share a target directory. Give archives a completely fresh build tree.
        build = Path(tmp) / "build" if args.baseline else ROOT / "target"
        env["CARGO_TARGET_DIR"] = str(build)
        env["ORIHA_SAMPLES"] = "7"
        with Path(str(prefix) + "-build.log").open("w") as log:
            for example, flags, suffix in [("adoption_cost", [], ".jsonl"), ("source_cost", [], "-original.csv"), ("source_cost", ["--scaling"], "-scaling.csv")]:
                command = ["cargo", "run", "-p", "oriha-core", "--example", example, "--release", "--locked"]
                if flags:
                    command += ["--"] + flags
                metadata["commands"].append(command)
                with Path(str(prefix) + suffix).open("w") as receipt:
                    subprocess.run(command, cwd=source, env=env, stdout=receipt, stderr=log, check=True)
                if args.freeze_binaries:
                    args.freeze_binaries.mkdir(parents=True, exist_ok=True)
                    binary = build / "release/examples" / example
                    shutil.copy2(binary, args.freeze_binaries / example)
        Path(str(prefix) + "-metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Recorded {prefix}: {metadata['identity']['sha256']}")


if __name__ == "__main__":
    main()
