#!/usr/bin/env python3
"""Check the package's complete native/data/icon notice inventory by reproduction."""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix="oriha-license-check-") as temporary:
    subprocess.run(["node", "packages/expo-oriha/scripts/generate-third-party-licenses.mjs",
                    "--output-dir", temporary], cwd=ROOT, check=True)
    actual = ROOT / "packages/expo-oriha/licenses"
    for generated in Path(temporary).iterdir():
        path = actual / generated.name
        if not path.is_file() or generated.read_bytes() != path.read_bytes():
            raise ValueError(f"Stale third-party notices: {path.relative_to(ROOT)}")
print("PASS: target-graph license inventory and retained icon license reproduce exactly")
