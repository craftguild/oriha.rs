#!/usr/bin/env python3
"""Adapt the shared compiler argv to cargo-ndk's owned target/manifest selectors."""

import os
import sys


def command(arguments, selected_target):
    args = list(arguments)
    if not args or args[0] != 'build':
        raise ValueError('Android compiler adapter expects the shared build argv')

    def selector(flag):
        if args.count(flag) != 1:
            raise ValueError('Expected exactly one shared selector: ' + flag)
        index = args.index(flag)
        if index + 1 == len(args):
            raise ValueError('Missing shared selector value')
        value = args[index + 1]
        del args[index:index + 2]
        return value

    target = selector('--target')
    manifest = selector('--manifest-path')
    if target != selected_target:
        raise ValueError('Android compiler target differs from resolved context')
    abi = {'aarch64-linux-android': 'arm64-v8a',
           'armv7-linux-androideabi': 'armeabi-v7a',
           'x86_64-linux-android': 'x86_64'}.get(target)
    if abi is None:
        raise ValueError('Unsupported Android distribution target')
    # cargo-ndk supplies these two selectors to Cargo itself. Forwarding them in
    # its trailing Cargo args would duplicate the manifest/target ownership.
    return ['cargo', 'ndk', '--target', abi, '--manifest-path', manifest, *args]


if __name__ == '__main__':
    try:
        argv = command(sys.argv[1:], os.environ['CRAFTGUILD_TARGET'])
        os.execvp(argv[0], argv)
    except (ValueError, OSError, KeyError) as error:
        raise SystemExit(str(error)) from error
