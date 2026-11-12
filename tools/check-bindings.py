#!/usr/bin/env python3
"""Generate bindings in an owned root and verify their real host ABI checksums."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
receipt = json.loads((ROOT / '.craftguild/oriha-native-bridge/receipt.json').read_text())
library = next(Path(file) for file in receipt['context']['outputs']
               if file.endswith(('liboriha_native_bridge.dylib', 'liboriha_native_bridge.so')))
with tempfile.TemporaryDirectory(prefix="oriha-bindings-check-") as temporary:
    owned = Path(temporary)
    subprocess.run(["node", "packages/expo-oriha/scripts/generate-uniffi-bindings.mjs",
                    "--output-dir", str(owned)], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "scripts/check-uniffi-checksums.py",
                    str(library),
                    "--swift", str(owned / "swift/oriha_native_bridge.swift"),
                    "--kotlin", str(owned / "kotlin/uniffi/oriha_native_bridge/oriha_native_bridge.kt")],
                   cwd=ROOT, check=True)
