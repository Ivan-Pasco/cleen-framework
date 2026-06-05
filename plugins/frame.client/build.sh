#!/bin/bash
set -e
cd "$(dirname "$0")"

CLN=cln
if ! command -v "$CLN" &>/dev/null; then
    echo "ERROR: cln compiler not found in PATH"
    exit 1
fi

"$CLN" compile src/main.cln -o plugin.wasm --target=plugin
echo "Built frame.client plugin -> plugin.wasm (compiler $("$CLN" --version 2>/dev/null | awk '{print $NF}'))"
