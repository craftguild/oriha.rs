#!/usr/bin/env python3
"""Derive common ownership evidence; standards still verifies exact reproduction."""

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def owned(name):
    relative = PurePosixPath(name)
    if not name or relative.is_absolute() or '..' in relative.parts or str(relative) != name:
        raise ValueError('Unsafe generation input/output path')
    path = ROOT / name
    if any(parent.is_symlink() for parent in path.parents) or path.is_symlink():
        raise ValueError('Symlink in generation evidence')
    return path


def inventory(name):
    directory = owned(name)
    if not directory.is_dir():
        raise ValueError('Missing generation input directory: ' + name)
    files = []
    for path in sorted(directory.rglob('*')):
        if path.is_symlink():
            raise ValueError('Symlink in generation directory inventory')
        if path.is_file():
            files.append(path.relative_to(ROOT).as_posix())
        elif not path.is_dir():
            raise ValueError('Nonregular generation input')
    return files


def hashes(names):
    return {name: hashlib.sha256(owned(name).read_bytes()).hexdigest()
            for name in sorted(set(names))}


def derive():
    recipe_path = 'assets/expo-generation.json'
    recipe = json.loads(owned(recipe_path).read_text())
    if recipe['schema_version'] != 1 or recipe['owner'] != 'oriha-expo-bridge-and-icons':
        raise ValueError('Unsupported Expo generation recipe')
    files = inventory(recipe['output_root'])
    if set(files) != set(recipe['files']):
        raise ValueError('Expo generated output inventory differs from its recipe')
    names = [recipe_path, *recipe['inputs']]
    for directory in recipe['input_dirs']:
        names.extend(inventory(directory))
    inputs = hashes(names)
    # A content revision identifies the complete generator inputs, not a fake Git commit.
    revision = hashlib.sha256(json.dumps(inputs, sort_keys=True,
                                         separators=(',', ':')).encode()).hexdigest()
    record = {'schema_version': 1, 'owner': recipe['owner'],
              'generator_revision': revision, 'inputs': inputs, 'files': hashes(files)}
    destination = owned('.craftguild/ownership/expo.json')
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as stream:
            temporary = Path(stream.name)
            stream.write((json.dumps(record, indent=2) + '\n').encode())
        os.replace(temporary, destination)
    finally:
        if temporary is not None and temporary.exists():
            temporary.unlink()
    print('Recorded Expo ownership; exact canonical reproduction remains required by standards.')


if __name__ == '__main__':
    derive()
