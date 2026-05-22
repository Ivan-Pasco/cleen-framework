# Compiler Fix: Remove Hardcoded Bridge Functions

## Issue Summary

The compiler has bridge functions (`_http_route`, `_http_listen`) hardcoded in `builtins/registry.rs`, but these same functions are also declared in plugin `[bridge]` sections. This duplication causes conflicts and the `__raw` suffix bug.

## Current Problem

### Hardcoded in Compiler (`src/builtins/registry.rs` lines 346-377):
```rust
fn register_http_server_functions(&mut self) {
    let http_server_functions = vec![
        BuiltinFunction::new(
            "_http_route",
            vec![BuiltinType::String, BuiltinType::String, BuiltinType::Integer],
            BuiltinType::Integer,
            BuiltinCategory::Http,
        )
        .with_wasm_import("env", "_http_route"),
        BuiltinFunction::new(
            "_http_listen",
            vec![BuiltinType::Integer],
            BuiltinType::Integer,
            BuiltinCategory::Http,
        )
        .with_wasm_import("env", "_http_listen"),
    ];
    // ...
}
```

### Also in Plugin (`~/.cleen/plugins/frame.web/1.0.0/plugin.toml`):
```toml
[bridge]
functions = [
  { name = "_http_listen", params = ["integer"], returns = "integer" },
  { name = "_http_route", params = ["string", "string", "integer"], returns = "integer", expand_strings = true },
  { name = "_req_param", params = ["string"], returns = "string", expand_strings = true },
  # ... more functions
]
```

### And in (`~/.cleen/plugins/frame.data/1.0.0/plugin.toml`):
```toml
[bridge]
functions = [
  { name = "_db_query", params = ["string", "string"], returns = "string", expand_strings = true },
  { name = "_db_execute", params = ["string", "string"], returns = "integer", expand_strings = true },
  # ... more functions
]
```

## Correct Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         COMPILER                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Core Builtins (registry.rs)                            │    │
│  │  - printl, print, input                                 │    │
│  │  - math.*, string.*, list.*, json.*                     │    │
│  │  - NO bridge functions (_http_*, _db_*, _req_*)         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              ↓                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Plugin Loader                                          │    │
│  │  - Reads plugin.toml [bridge] section                   │    │
│  │  - Registers bridge functions dynamically               │    │
│  │  - Handles expand_strings parameter expansion           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                         WASM Output
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      CLEAN-SERVER                                │
│  - Provides runtime implementation of bridge functions          │
│  - _http_route, _http_listen, _req_param, etc.                  │
│  - _db_query, _db_execute, etc.                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Required Changes

### 1. Remove Hardcoded Bridge Functions from `registry.rs`

Delete the `register_http_server_functions()` method and its call:

```rust
// DELETE this entire function (lines ~346-377):
fn register_http_server_functions(&mut self) {
    // ... all of this
}

// DELETE the call to it in new() or register_all():
// self.register_http_server_functions();
```

### 2. Ensure Plugin Bridge Loading Works Correctly

The plugin loader should be the ONLY source of bridge functions. Verify that `src/plugins/` correctly:

1. Reads `[bridge]` section from plugin.toml
2. Registers each function with correct signature
3. Handles `expand_strings = true` by expanding string params to (ptr, len) pairs
4. Generates correct WASM imports with namespace "env"

### 3. Fix `expand_strings` Implementation

When `expand_strings = true`:
- Each `string` parameter becomes TWO i32 parameters: `(ptr, len)`
- The function NAME stays the same (NO `__raw` suffix!)
- Only the parameter count changes in the WASM signature

Example for `_db_query`:
```
Plugin declaration:
  { name = "_db_query", params = ["string", "string"], returns = "string", expand_strings = true }

Should generate WASM import:
  (import "env" "_db_query" (func (param i32 i32 i32 i32) (result i32)))
                                   ^^^^^^^^^ ^^^^^^^^^
                                   sql ptr/len  params ptr/len
```

NOT:
```
  (import "env" "_db_query__raw" ...)  // WRONG - no __raw suffix!
```

### 4. Remove Any `__raw` Suffix Logic

Search for and remove any code that adds `__raw` suffix to function names:

```bash
grep -r "__raw" src/
```

The function name in the WASM import should exactly match what's in plugin.toml.

## Files to Modify

1. **`src/builtins/registry.rs`**
   - Remove `register_http_server_functions()` method
   - Remove call to it from initialization

2. **`src/plugins/plugin_abi.rs`** (or similar)
   - Ensure bridge functions are registered correctly
   - Fix `expand_strings` to only expand params, not rename function

3. **`src/codegen/`** (wherever WASM imports are generated)
   - Remove any `__raw` suffix logic
   - Use exact function names from plugin declarations

## Verification

After fix, this test should work:

```clean
import:
    frame.data
    frame.web

functions:
    string test()
        return "ok"

start()
    printl(test())
```

Compile and check WASM imports:
```bash
cln compile test.cln -o test.wasm --plugins
wasm-validate test.wasm  # Should pass
wasm-objdump -x test.wasm | grep "_db_query\|_http_route"
```

Expected output:
```
func[X] ... <- env._db_query      # NOT env._db_query__raw
func[Y] ... <- env._http_route    # NOT env._http_route__raw
```

## Summary

| What | Before (Wrong) | After (Correct) |
|------|----------------|-----------------|
| Bridge function source | Hardcoded + Plugin | Plugin ONLY |
| Function names | `_db_query__raw` | `_db_query` |
| `expand_strings` effect | Renames function | Expands params only |
| Conflict potential | High (duplicates) | None (single source) |
