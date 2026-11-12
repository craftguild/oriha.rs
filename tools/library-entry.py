#!/usr/bin/env python3
"""Retain the public library selector spelling while delegating to standards."""

import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent.parent
args = ['--artifact' if arg == '--library' else arg for arg in sys.argv[1:]]
os.chdir(root)
os.execv('/bin/sh', ['sh', 'tools/standards.sh', *args])
