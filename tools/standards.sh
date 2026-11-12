#!/bin/sh
# Product tool environment; the pinned shared launcher owns engine execution.
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
NODE_PATH="$project_root/packages/expo-oriha/node_modules${NODE_PATH:+:$NODE_PATH}"
export NODE_PATH
CRAFTGUILD_TOKEI=${CRAFTGUILD_TOKEI:-"$project_root/tools/tokei.py"}
export CRAFTGUILD_TOKEI
cd "$project_root"
exec python3 standards/launcher.py "$@"
