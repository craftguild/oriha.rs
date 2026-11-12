#!/usr/bin/env python3
"""Check existing Swift/Kotlin binding initialization against a built host library."""
import ctypes
import argparse
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("library", type=Path)
parser.add_argument("--swift", type=Path, default=root / "packages/expo-oriha/ios/generated/oriha_native_bridge.swift")
parser.add_argument("--kotlin", type=Path, default=root / "packages/expo-oriha/android/src/main/java/uniffi/oriha_native_bridge/oriha_native_bridge.kt")
args = parser.parse_args()
library = ctypes.CDLL(str(args.library.resolve()))
swift = args.swift.read_text()
kotlin = args.kotlin.read_text()
pattern = r"(uniffi_oriha_native_bridge_checksum_\w+)\(\) != (\d+)"
expected = dict(re.findall(pattern, swift))
assert expected and expected == dict(re.findall(pattern, kotlin)), "binding expectations differ"
contracts = [int(re.search(r"bindings_contract_version = (\d+)", text)[1]) for text in (swift, kotlin)]
contract = library.ffi_oriha_native_bridge_uniffi_contract_version
contract.restype = ctypes.c_uint32
assert contract() == contracts[0] == contracts[1]
results = {}
for name, value in expected.items():
    function = getattr(library, name)
    function.restype = ctypes.c_uint16
    actual = function()
    assert actual == int(value), (name, value, actual)
    results[name] = actual
print(json.dumps({"contract_version": contract(), "checksums": results, "result": "pass"}, indent=2))
