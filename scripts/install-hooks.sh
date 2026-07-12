#!/bin/bash
# Install Clean Framework git hooks.
#
# Creates symlinks in .git/hooks/ pointing to scripts/hooks/.
# Idempotent: safe to run repeatedly.
#
# See system-documents/testing/TEST_STRATEGY.md §4 for the hook layer mapping.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

HOOK_SOURCE_DIR="scripts/hooks"
HOOK_TARGET_DIR=".git/hooks"

if [ ! -d "$HOOK_TARGET_DIR" ]; then
    echo "error: $HOOK_TARGET_DIR does not exist. Are you inside a git repository?" >&2
    exit 1
fi

install_hook() {
    local name="$1"
    local src="$REPO_ROOT/$HOOK_SOURCE_DIR/$name"
    local dst="$HOOK_TARGET_DIR/$name"

    if [ ! -f "$src" ]; then
        echo "warning: $src not found; skipping $name" >&2
        return
    fi

    chmod +x "$src"

    if [ -L "$dst" ]; then
        rm "$dst"
    elif [ -f "$dst" ]; then
        mv "$dst" "$dst.backup.$(date +%s)"
        echo "backed up existing $dst"
    fi

    ln -s "../../$HOOK_SOURCE_DIR/$name" "$dst"
    echo "installed: $dst -> $HOOK_SOURCE_DIR/$name"
}

install_hook pre-commit
install_hook pre-push

echo
echo "Hooks installed. Test with: git commit --allow-empty -m test"
echo "See system-documents/testing/TEST_STRATEGY.md §4 for what each hook enforces."
