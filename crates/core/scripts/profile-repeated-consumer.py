#!/usr/bin/env python3
"""Freeze two probe builds and sample requested bytes separately from process RSS.

Usage: profile-repeated-consumer.py OUTPUT_DIRECTORY [--repeats 3]
Writes only beneath OUTPUT_DIRECTORY and Cargo's normal target directory.
No consumer checkout, hooks, sanitizer, or second parser is required.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--repeats", type=int, default=3)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    tracked = subprocess.check_output(["git", "ls-files", "-co", "--exclude-standard"], cwd=root, text=True).splitlines()
    identity = {
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "platform": platform.platform(), "sampling_ms_nominal": 20,
        "files": {p: sha(root / p) for p in sorted(set(tracked))
                  if p.startswith("crates/core/") or p.endswith("Cargo.toml") or p == "Cargo.lock"},
    }
    for kind, enabled in [("alloc", "1"), ("rss", "0")]:
        env = dict(os.environ, ORIHA_MEMORY_ALLOCATIONS=enabled)
        with (out / f"build-{kind}.log").open("w") as log:
            subprocess.run(["cargo", "build", "-p", "oriha-core", "--example", "repeated_consumer", "--release", "--locked"], cwd=root, env=env, stdout=log, stderr=log, check=True)
        binary = out / f"probe-{kind}"
        shutil.copyfile(root / "target/release/examples/repeated_consumer", binary)
        binary.chmod(0o755)
        identity[f"binary_{kind}_sha256"] = sha(binary)
    (out / "identity.json").write_text(json.dumps(identity, indent=2) + "\n")
    for schedule in ["serial", "two", "queued"]:
        for repeat in range(args.repeats):
            for kind in ["alloc", "rss"]:
                name = f"{schedule}-{repeat}-{kind}"
                samples = []
                with (out / f"{name}.jsonl").open("w") as log:
                    start = time.monotonic_ns()
                    process = subprocess.Popen([str(out / f"probe-{kind}"), schedule], stdout=log)
                    while process.poll() is None:
                        # ps reports KiB on both macOS and Linux. This is sampled RSS,
                        # not an OS high-water mark or allocator-requested byte count.
                        raw = subprocess.run(["ps", "-p", str(process.pid), "-o", "rss="], capture_output=True, text=True, check=False).stdout.strip()
                        if raw:
                            samples.append({"elapsed_ns":time.monotonic_ns()-start, "rss_kib":int(raw)})
                        time.sleep(0.02)
                    if process.returncode:
                        raise RuntimeError(f"{name}: exit {process.returncode}")
                (out / f"{name}-rss.json").write_text(json.dumps(samples) + "\n")
                print(json.dumps({"run":name,"peak_rss_kib":max(s["rss_kib"] for s in samples),"samples":len(samples)}), flush=True)


if __name__ == "__main__":
    main()
