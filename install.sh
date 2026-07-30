#!/bin/sh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "${SCRIPT_DIR}/scripts/install.sh" ]; then
    exec sh "${SCRIPT_DIR}/scripts/install.sh" "$@"
else
    exec curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.sh | sh
fi
