#!/bin/bash
set -e
cd "$(dirname "$0")"

# Pin to compiler 0.22.0 — later versions have a string literal regression
# that causes expand_block to return empty bodies for html: blocks.
# See: system-documents/cross-component-prompts/framework-frame-ui-html-expand-empty-body.md
CLN="${CLEEN_HOME:-$HOME/.cleen}/versions/0.22.0/cln"
if [ ! -x "$CLN" ]; then
	echo "ERROR: Compiler 0.22.0 not found at $CLN"
	echo "Install it with: cleen install 0.22.0"
	exit 1
fi

"$CLN" compile src/main.cln -o plugin.wasm
echo "Built frame.ui plugin -> plugin.wasm (compiler 0.22.0)"
