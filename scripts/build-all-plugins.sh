#!/bin/bash
# Build all Frame plugins
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGINS_DIR="$SCRIPT_DIR/../plugins"

echo "Building Frame plugins..."
echo "========================="
echo ""

# Track success/failure
BUILT=0
FAILED=0

for plugin_dir in "$PLUGINS_DIR"/*/; do
    plugin_name=$(basename "$plugin_dir")
    echo "Building $plugin_name..."

    cd "$plugin_dir"
    if [ -f "build.sh" ]; then
        if bash build.sh; then
            echo "  ✓ $plugin_name built successfully"
            ((BUILT++))
        else
            echo "  ✗ $plugin_name build failed"
            ((FAILED++))
        fi
    else
        echo "  ⚠ $plugin_name has no build.sh, attempting direct compile..."
        if cln compile src/main.cln -o plugin.wasm; then
            echo "  ✓ $plugin_name built successfully"
            ((BUILT++))
        else
            echo "  ✗ $plugin_name build failed"
            ((FAILED++))
        fi
    fi
    echo ""
done

echo "========================="
echo "Build complete: $BUILT succeeded, $FAILED failed"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
