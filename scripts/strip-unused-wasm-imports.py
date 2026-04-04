#!/usr/bin/env python3
"""Strip unreachable imports and dead functions from a WASM text (WAT) file.

The Clean Language compiler emits imports for ALL known host functions
unconditionally, even when the source code doesn't reference them. This causes
plugin instantiation failures when the plugin loader doesn't provide stubs for
every possible host function.

This script performs dead-code elimination at the import level:
1. Builds a call graph from the WAT
2. Finds all functions reachable from exports
3. Removes unreachable imports and their dead wrapper functions
4. Renumbers all function references

Usage:
    python3 strip-unused-wasm-imports.py input.wat output.wat
"""

import re
import sys


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} input.wat output.wat", file=sys.stderr)
        sys.exit(1)

    input_path, output_path = sys.argv[1], sys.argv[2]

    with open(input_path, 'r') as f:
        content = f.read()

    # Parse imports
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

    num_imports = len(imports)
    if num_imports == 0:
        # Nothing to strip
        with open(output_path, 'w') as f:
            f.write(content)
        return

    # Parse function bodies and build call graph
    func_bodies = {}
    for m in re.finditer(r'\(func \(;(\d+);\)[^\n]*\n(.*?)\n  \)', content, re.DOTALL):
        func_bodies[int(m.group(1))] = m.group(2)

    call_graph = {}
    for idx, body in func_bodies.items():
        call_graph[idx] = set(int(x) for x in re.findall(r'\bcall (\d+)\b', body))

    # Find exports
    exports = set()
    for m in re.finditer(r'\(export "[^"]*" \(func (\d+)\)\)', content):
        exports.add(int(m.group(1)))

    # BFS reachability from exports
    reachable = set()
    queue = list(exports)
    while queue:
        f = queue.pop()
        if f in reachable:
            continue
        reachable.add(f)
        if f in call_graph:
            for callee in call_graph[f]:
                if callee not in reachable:
                    queue.append(callee)

    # Identify what to remove
    all_func_indices = set(func_bodies.keys()) | set(range(num_imports))
    unused_imports = {imp['idx'] for imp in imports if imp['idx'] not in reachable}
    dead_funcs = {idx for idx in all_func_indices
                  if idx not in reachable and idx >= num_imports}
    to_remove = unused_imports | dead_funcs

    if not to_remove:
        with open(output_path, 'w') as f:
            f.write(content)
        print("No unused imports found.", file=sys.stderr)
        return

    # Build renumber map
    remap = {}
    new_idx = 0
    for old_idx in range(max(all_func_indices) + 1):
        if old_idx in to_remove:
            continue
        remap[old_idx] = new_idx
        new_idx += 1

    # Remove unused import lines
    for imp in imports:
        if imp['idx'] in unused_imports:
            content = content.replace('  ' + imp['full_match'] + '\n', '')

    # Remove dead function definitions with proper parenthesis matching
    func_spans = []
    for m in re.finditer(r'  \(func \(;(\d+);\)', content):
        func_idx = int(m.group(1))
        if func_idx not in dead_funcs:
            continue
        start = m.start()
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
                    func_spans.append((start, end))
                    break
            i += 1

    # Remove in reverse order to preserve offsets
    for start, end in sorted(func_spans, reverse=True):
        content = content[:start] + content[end:]

    # Renumber all function references
    def renumber_func_def(m):
        old = int(m.group(1))
        return f'(func (;{remap[old]};)' if old in remap else m.group(0)

    def renumber_call(m):
        old = int(m.group(1))
        return f'call {remap[old]}' if old in remap else m.group(0)

    def renumber_export(m):
        old = int(m.group(2))
        return (f'(export "{m.group(1)}" (func {remap[old]}))'
                if old in remap else m.group(0))

    def renumber_elem_ref(m):
        old = int(m.group(1))
        return f'ref.func {remap[old]}' if old in remap else m.group(0)

    content = re.sub(r'\(func \(;(\d+);\)', renumber_func_def, content)
    content = re.sub(r'\bcall (\d+)\b', renumber_call, content)
    content = re.sub(r'\(export "([^"]*)" \(func (\d+)\)\)', renumber_export, content)
    content = re.sub(r'ref\.func (\d+)', renumber_elem_ref, content)

    with open(output_path, 'w') as f:
        f.write(content)

    print(f"Stripped {len(unused_imports)} unused imports + "
          f"{len(dead_funcs)} dead functions "
          f"({len(to_remove)} total removed)", file=sys.stderr)


if __name__ == '__main__':
    main()
