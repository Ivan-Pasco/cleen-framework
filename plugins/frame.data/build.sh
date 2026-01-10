#!/bin/bash
set -e
cd "$(dirname "$0")"
cln compile src/main.cln -o plugin.wasm
echo "Built frame.data plugin -> plugin.wasm"
