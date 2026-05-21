#!/usr/bin/env python3
"""Strip server-specific bridge imports from plugin WASM files.

The Clean Language compiler emits imports for server-only host functions
unconditionally, even when building a plugin (not a server application).
The plugin loader doesn't provide these functions, causing instantiation failures.

This script removes the named server-only imports and their direct wrapper
functions from the WAT, then renumbers all function references.

Usage:
    python3 strip-unused-wasm-imports.py input.wat output.wat
"""

import re
import sys


# Server-specific runtime functions that the plugin loader never provides.
# These must be stripped from plugin WASMs unconditionally.
FORCE_STRIP_NAMES = {
    '_server_sleep',
    '_async_fire',
    '_async_await',
}


def find_func_body_end(content, start):
    """Return the index one past the closing ) of the func body starting at start."""
    depth = 0
    i = start
    while i < len(content):
        if content[i] == '(':
            depth += 1
        elif content[i] == ')':
            depth -= 1
            if depth == 0:
                end = i + 1
                if end < len(content) and content[end] == '\n':
                    end += 1
                return end
        i += 1
    return len(content)


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} input.wat output.wat", file=sys.stderr)
        sys.exit(1)

    input_path, output_path = sys.argv[1], sys.argv[2]

    with open(input_path, 'r') as f:
        content = f.read()

    # Parse all function imports
    imports = []
    for m in re.finditer(
        r'\(import "(\w+)" "([^"]+)" \(func \(;(\d+);\) \(type (\d+)\)\)\)',
        content
    ):
        imports.append({
            'module': m.group(1),
            'name': m.group(2),
            'idx': int(m.group(3)),
            'type_idx': int(m.group(4)),
            'full_match': m.group(0),
        })

    if not imports:
        with open(output_path, 'w') as f:
            f.write(content)
        return

    # Find the indices to force-strip
    force_strip_indices = {
        imp['idx'] for imp in imports if imp['name'] in FORCE_STRIP_NAMES
    }

    if not force_strip_indices:
        with open(output_path, 'w') as f:
            f.write(content)
        print("No server-specific imports found.", file=sys.stderr)
        return

    # Find wrapper functions: functions exported as the same name AND whose
    # entire body is just a direct call to a force-stripped import.
    # These become invalid after the import is removed, so they must go too.
    wrapper_indices = set()
    for idx in force_strip_indices:
        # A wrapper function directly calls `call <idx>` and is re-exported
        # Find functions that call this import
        for m in re.finditer(
            r'  \(func \(;(\d+);\)[^\n]*\n(.*?)\n  \)',
            content, re.DOTALL
        ):
            func_idx = int(m.group(1))
            body = m.group(2)
            calls = re.findall(r'\bcall (\d+)\b', body)
            # If this function ONLY calls the force-stripped import (and nothing else
            # meaningful), it's a wrapper — mark it for removal.
            if calls and all(int(c) == idx or int(c) in force_strip_indices
                             for c in calls):
                # Verify it's exported (wrappers are always exported)
                if re.search(
                    r'\(export "[^"]*" \(func ' + str(func_idx) + r'\)\)',
                    content
                ):
                    wrapper_indices.add(func_idx)

    to_remove = force_strip_indices | wrapper_indices
    removed_sorted = sorted(to_remove)

    def new_idx(n):
        """Map old function index to new index after removals."""
        if n in to_remove:
            return None
        return n - sum(1 for r in to_remove if r < n)

    # Remove import lines and their re-export lines
    for imp in imports:
        if imp['idx'] in force_strip_indices:
            content = content.replace('  ' + imp['full_match'] + '\n', '')
            content = re.sub(
                r'  \(export "[^"]*" \(func ' + str(imp['idx']) + r'\)\)\n',
                '',
                content
            )

    # Remove wrapper function bodies and their export lines
    # Collect spans first, then remove in reverse order
    wrapper_spans = []
    for widx in wrapper_indices:
        # Remove export for this wrapper
        content = re.sub(
            r'  \(export "[^"]*" \(func ' + str(widx) + r'\)\)\n',
            '',
            content
        )
        # Find and mark function body for removal
        m = re.search(r'  \(func \(;' + str(widx) + r';\)', content)
        if m:
            end = find_func_body_end(content, m.start())
            wrapper_spans.append((m.start(), end))

    for start, end in sorted(wrapper_spans, reverse=True):
        content = content[: start] + content[end:]

    # Renumber all function references
    def renumber_func_def(m):
        old = int(m.group(1))
        ni = new_idx(old)
        return f'(func (;{ni};)' if ni is not None else m.group(0)

    def renumber_call(m):
        old = int(m.group(1))
        ni = new_idx(old)
        return f'call {ni}' if ni is not None else m.group(0)

    def renumber_export(m):
        name, old = m.group(1), int(m.group(2))
        ni = new_idx(old)
        return (f'(export "{name}" (func {ni}))' if ni is not None
                else m.group(0))

    def renumber_elem_ref(m):
        old = int(m.group(1))
        ni = new_idx(old)
        return f'ref.func {ni}' if ni is not None else m.group(0)

    content = re.sub(r'\(func \(;(\d+);\)', renumber_func_def, content)
    content = re.sub(r'\bcall (\d+)\b', renumber_call, content)
    content = re.sub(r'\(export "([^"]*)" \(func (\d+)\)\)', renumber_export, content)
    content = re.sub(r'ref\.func (\d+)', renumber_elem_ref, content)

    with open(output_path, 'w') as f:
        f.write(content)

    print(
        f"Stripped {len(force_strip_indices)} server-specific imports + "
        f"{len(wrapper_indices)} wrapper functions "
        f"({len(to_remove)} total removed)",
        file=sys.stderr
    )


if __name__ == '__main__':
    main()
