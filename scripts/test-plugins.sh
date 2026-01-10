#!/bin/bash
# Run tests for all Frame plugins
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGINS_DIR="$SCRIPT_DIR/../plugins"

echo "Testing Frame plugins..."
echo "========================"
echo ""

# Track success/failure
PASSED=0
FAILED=0

for plugin_dir in "$PLUGINS_DIR"/*/; do
    plugin_name=$(basename "$plugin_dir")

    if [ -d "$plugin_dir/tests" ]; then
        echo "Testing $plugin_name..."

        cd "$plugin_dir"
        for test_file in tests/*.cln; do
            if [ -f "$test_file" ]; then
                test_name=$(basename "$test_file")
                if cln test "$test_file"; then
                    echo "  ✓ $test_name passed"
                    ((PASSED++))
                else
                    echo "  ✗ $test_name failed"
                    ((FAILED++))
                fi
            fi
        done
        echo ""
    else
        echo "⚠ $plugin_name has no tests directory"
        echo ""
    fi
done

echo "========================"
echo "Tests complete: $PASSED passed, $FAILED failed"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
