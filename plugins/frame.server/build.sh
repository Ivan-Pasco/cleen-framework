#!/bin/bash
set -e
cd "$(dirname "$0")"

# Use compiler 0.31.0 — matches frame.ui build configuration
CLN="${CLEEN_HOME:-$HOME/.cleen}/versions/0.31.0/cln"
if [ ! -x "$CLN" ]; then
	echo "ERROR: Compiler 0.31.0 not found at $CLN"
	echo "Install it with: cleen install 0.31.0"
	exit 1
fi

"$CLN" compile src/main.cln -o plugin.wasm --target=plugin
echo "Built frame.server plugin -> plugin.wasm (compiler 0.31.0)"

# Strip unused imports (session, auth, HTTP client, file I/O, math, etc.)
# that the compiler emits unconditionally but the plugin loader doesn't provide.
# Requires: wasm-tools (cargo install wasm-tools)
if command -v wasm-tools &>/dev/null && command -v python3 &>/dev/null; then
	wasm-tools print plugin.wasm > /tmp/frame_server_strip.wat
	python3 ../../scripts/strip-unused-wasm-imports.py /tmp/frame_server_strip.wat /tmp/frame_server_stripped.wat
	wasm-tools parse /tmp/frame_server_stripped.wat -o plugin.wasm
	rm -f /tmp/frame_server_strip.wat /tmp/frame_server_stripped.wat
	echo "Stripped unused imports from plugin.wasm"
else
	echo "WARNING: wasm-tools or python3 not found, skipping import stripping"
	echo "Plugin may fail to load if compiler emits unused host function imports"
fi
