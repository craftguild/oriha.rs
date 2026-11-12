#!/usr/bin/env python3
"""Check Cargo modules and ABI-preserving included native type definitions."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
subprocess.run(["cargo", "fmt", "--all", "--", "--check"], cwd=ROOT, check=True)
included = sorted((ROOT / "crates/native-bridge/src/types").glob("*.rs"))
subprocess.run(["rustfmt", "--edition", "2021", "--check", *map(str, included)],
               cwd=ROOT, check=True)
