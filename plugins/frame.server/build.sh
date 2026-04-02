#!/bin/bash
set -e
cd "$(dirname "$0")"
cln compile src/main.cln -o plugin.wasm
echo "Built frame.server plugin -> plugin.wasm"
