#!/bin/bash
# Install all Frame plugins to ~/.cleen/plugins/
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGINS_DIR="$SCRIPT_DIR/../plugins"
INSTALL_DIR="$HOME/.cleen/plugins"

echo "Installing Frame plugins..."
echo "==========================="
echo ""

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Track success/failure
INSTALLED=0
FAILED=0

for plugin_dir in "$PLUGINS_DIR"/*/; do
    plugin_name=$(basename "$plugin_dir")

    # Read version from plugin.toml
    if [ -f "$plugin_dir/plugin.toml" ]; then
        VERSION=$(grep '^version' "$plugin_dir/plugin.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    else
        VERSION="1.0.0"
    fi

    TARGET_DIR="$INSTALL_DIR/$plugin_name/$VERSION"

    echo "Installing $plugin_name@$VERSION..."

    # Check if plugin.wasm exists
    if [ ! -f "$plugin_dir/plugin.wasm" ]; then
        echo "  ⚠ plugin.wasm not found, building first..."
        cd "$plugin_dir"
        if [ -f "build.sh" ]; then
            bash build.sh > /dev/null 2>&1 || true
        else
            cln compile src/main.cln -o plugin.wasm > /dev/null 2>&1 || true
        fi
    fi

    if [ -f "$plugin_dir/plugin.wasm" ]; then
        mkdir -p "$TARGET_DIR"
        cp "$plugin_dir/plugin.toml" "$TARGET_DIR/"
        cp "$plugin_dir/plugin.wasm" "$TARGET_DIR/"
        echo "  ✓ Installed to $TARGET_DIR"
        ((INSTALLED++))
    else
        echo "  ✗ Failed - plugin.wasm not found"
        ((FAILED++))
    fi
    echo ""
done

echo "==========================="
echo "Installation complete: $INSTALLED succeeded, $FAILED failed"
echo ""
echo "Installed plugins are available at: $INSTALL_DIR"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
