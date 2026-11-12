#!/usr/bin/env python3
"""Probe the actual IR and conversion bodies in a disposable copy of oriha-core."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def check(work):
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"))
    process = subprocess.run(
        ["cargo", "check", "-p", "oriha-core", "--offline", "--message-format=json"],
        cwd=work, env=env, text=True, capture_output=True, check=False,
    )
    errors = []
    for line in process.stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("reason") == "compiler-message" and event["message"]["level"] == "error":
            errors.append(event["message"])
    if process.returncode and not errors:
        raise RuntimeError(process.stderr)
    return process.returncode, errors


def probe(work, enum, targets):
    source = work / "crates/core/src"
    original = {p: p.read_text() for p in source.rglob("*.rs")}
    ir = source / "ir.rs"
    ir.write_text(ir.read_text().replace(f"pub enum {enum} {{", f"pub enum {enum} {{\n    Adr0002Probe,"))
    code, errors = check(work)
    assert code != 0
    matches = []
    for error in errors:
        assert error.get("code", {}).get("code") == "E0004", error
        assert "Adr0002Probe" in error["message"], error
        span = next(s for s in error["spans"] if s["is_primary"])
        path = work / span["file_name"]
        matches.append((path, span["byte_end"], span["line_start"]))
    observed = {str(path.relative_to(source)) for path, _, _ in matches}
    assert set(targets) <= observed, (enum, targets, observed)
    print(json.dumps({"enum": enum, "initialE0004Bodies": [
        {"file": str(p.relative_to(source)), "line": line} for p, _, line in matches
    ]}), flush=True)

    # Supply a deliberate temporary decision in every reported actual match.
    # Panicking probe arms are NEVER copied back to the working source tree.
    patched = {}
    for path in {p for p, _, _ in matches}:
        data = path.read_bytes()
        for _, end, _ in sorted((m for m in matches if m[0] == path), key=lambda m: m[1], reverse=True):
            brace = data.index(b"{", end)
            arm = f'\n crate::ir::{enum}::Adr0002Probe => panic!("temporary enum probe"),\n'.encode()
            data = data[:brace + 1] + arm + data[brace + 1:]
        path.write_bytes(data)
        patched[path] = data
    code, errors = check(work)
    assert code == 0, errors
    print(json.dumps({"enum": enum, "allDecisionsSupplied": "pass"}), flush=True)

    # Remove each mode's decision independently, proving E0004 in its actual
    # conversion body after all unrelated helpers have already been satisfied.
    for target in targets:
        path = source / target
        path.write_text(original[path])
        code, errors = check(work)
        assert code != 0
        assert errors and all(e.get("code", {}).get("code") == "E0004" for e in errors), errors
        assert all(any(s["is_primary"] and s["file_name"].endswith(target) for s in e["spans"]) for e in errors), errors
        print(json.dumps({"enum": enum, "missingDecision": target, "E0004": len(errors)}), flush=True)
        path.write_bytes(patched[path])
    for path, data in original.items():
        path.write_text(data)


def main():
    with tempfile.TemporaryDirectory(prefix="oriha-0002-probe-") as directory:
        work = Path(directory)
        (work / "crates").mkdir()
        shutil.copytree(ROOT / "crates/core", work / "crates/core")
        manifest = (ROOT / "Cargo.toml").read_text()
        manifest = re.sub(r"members = \[.*?\]", 'members = ["crates/core"]', manifest, count=1, flags=re.S)
        # Reuse real, unchanged dependencies; only oriha-core is a source copy.
        manifest = re.sub(r'path = "(crates/[^\"]+|vendor/[^\"]+)"', lambda m: f'path = "{ROOT / m[1]}"', manifest)
        (work / "Cargo.toml").write_text(manifest)
        shutil.copy2(ROOT / "Cargo.lock", work / "Cargo.lock")
        code, errors = check(work)
        assert code == 0, errors
        probe(work, "BlockKind", ["blocks/structured.rs", "render/block_output/rendered.rs", "render/block_renderer.rs"])
        probe(work, "InlineKind", ["blocks/inline.rs", "render/inline.rs"])
        code, errors = check(work)
        assert code == 0, errors
        print("Actual source restored in temporary copy; both enum probes passed.")


if __name__ == "__main__":
    main()
