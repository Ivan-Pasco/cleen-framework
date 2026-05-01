#!/bin/bash
set -e
cd "$(dirname "$0")"

# Use compiler 0.30.48 — fixes NO-OP string stubs from 0.22.0,
# avoids `rules` keyword conflict in 0.30.38,
# avoids substring-in-concat bug in 0.30.7/0.31.0 (corrupts HTML attribute names),
# avoids complex-function empty-return bug in 0.30.49+
CLN="${CLEEN_HOME:-$HOME/.cleen}/versions/0.30.48/cln"
if [ ! -x "$CLN" ]; then
	echo "ERROR: Compiler 0.30.48 not found at $CLN"
	echo "Install it with: cleen install 0.30.48"
	exit 1
fi

"$CLN" compile src/main.cln -o plugin.wasm --target=plugin
echo "Built frame.ui plugin -> plugin.wasm (compiler 0.30.48)"

# Strip unused imports (session, auth, routing) that the compiler emits
# unconditionally but the plugin loader doesn't provide.
# Requires: wasm-tools (cargo install wasm-tools)
if ! command -v wasm-tools &>/dev/null; then
	echo "ERROR: wasm-tools not found. Install with: cargo install wasm-tools"
	echo "This step is required — without it, server bridge imports remain in the plugin"
	echo "and cause instantiation failures in UI-only projects."
	exit 1
fi
if ! command -v python3 &>/dev/null; then
	echo "ERROR: python3 not found. Install Python 3 to continue."
	exit 1
fi
wasm-tools print plugin.wasm > /tmp/frame_ui_strip.wat
python3 ../../scripts/strip-unused-wasm-imports.py /tmp/frame_ui_strip.wat /tmp/frame_ui_stripped.wat
wasm-tools parse /tmp/frame_ui_stripped.wat -o plugin.wasm
rm -f /tmp/frame_ui_strip.wat /tmp/frame_ui_stripped.wat
echo "Stripped unused imports from plugin.wasm"
