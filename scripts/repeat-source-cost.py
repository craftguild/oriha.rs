#!/usr/bin/env python3
"""Alternate three runs of frozen baseline/final source_cost executables."""
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys

binaries = dict(zip(["baseline", "after"], map(Path, sys.argv[1:3])))
assert len(binaries) == 2, "supply baseline and final executable paths"
result = {"binaries": {key: hashlib.sha256(path.read_bytes()).hexdigest()
                       for key, path in binaries.items()}, "runs": []}
assert len(set(result["binaries"].values())) == 2, "baseline and final executables are identical; check build isolation"
for run in range(3):
    phases = ["baseline", "after"] if run % 2 == 0 else ["after", "baseline"]
    for phase in phases:
        for mode, args in [("original", []), ("scaling", ["--scaling"])]:
            output = subprocess.check_output([str(binaries[phase].resolve())] + args, text=True)
            result["runs"].append({"run": run, "phase": phase, "mode": mode,
                                   "rows": list(csv.DictReader(io.StringIO(output)))})
print(json.dumps(result, indent=2))
