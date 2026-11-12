#!/usr/bin/env python3
"""Delegate counting to pinned tokei with byte-identical Ruby/Groovy aliases."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    program = shutil.which('tokei')
    if not program or Path(program).resolve() == Path(__file__).resolve():
        raise SystemExit('The real tokei 15.0.0 counter is required on PATH')
    args = sys.argv[1:]
    if args == ['--version']:
        os.execv(program, [program, *args])
    if args[:4] != ['--files', '--output', 'json', '--']:
        os.execv(program, [program, *args])
    with tempfile.TemporaryDirectory(prefix='oriha-tokei-alias-') as temporary:
        mapped = {}
        paths = []
        for index, name in enumerate(args[4:]):
            source = Path(name)
            extension = {'.podspec': '.rb', '.gradle': '.groovy'}.get(source.suffix)
            if extension:
                alias = Path(temporary) / (str(index) + extension)
                shutil.copyfile(source, alias)
                mapped[str(alias)] = name
                paths.append(str(alias))
            else:
                paths.append(name)
        result = subprocess.run([program, *args[:4], *paths], capture_output=True)
        sys.stderr.buffer.write(result.stderr)
        if result.returncode:
            raise SystemExit(result.returncode)
        report = json.loads(result.stdout)

        def restore_names(value):
            if isinstance(value, dict):
                if 'name' in value and value['name'] in mapped:
                    value['name'] = mapped[value['name']]
                for child in value.values():
                    restore_names(child)
            elif isinstance(value, list):
                for child in value:
                    restore_names(child)

        restore_names(report)
        print(json.dumps(report))


if __name__ == '__main__':
    main()
