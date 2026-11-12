#!/usr/bin/env python3
"""Forward installation through the shared copy-only receipt verifier."""

import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent.parent
os.chdir(root)
os.execv('/bin/sh', ['sh', 'tools/standards.sh', 'install', *sys.argv[1:]])
