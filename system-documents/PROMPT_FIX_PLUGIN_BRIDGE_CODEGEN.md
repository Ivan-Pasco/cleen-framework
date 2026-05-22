# Compiler Fix: Plugin Bridge Function Code Generation

## Issue Summary

When a Clean Language file imports a plugin (e.g., `import: frame.data` or `import: frame.web`), the compiled WASM has validation errors. The WASM validates correctly when:
- No `--plugins` flag is used
- `--plugins` is used but no `import:` statements exist

## Evidence

### Test 1: No plugins - VALID
```clean
functions:
    string hello()
        return "Hello"
start()
    printl(hello())
```
Compile: `cln compile test.cln -o test.wasm`
Result: ✓ Valid WASM

### Test 2: --plugins flag without import - VALID
```clean
functions:
    string hello()
        return "Hello"
start()
    printl(hello())
```
Compile: `cln compile test.cln -o test.wasm --plugins`
Result: ✓ Valid WASM

### Test 3: frame.data import - INVALID
```clean
import:
    frame.data

functions:
    string hello()
        return "Hello"
start()
    printl(hello())
```
Compile: `cln compile test.cln -o test.wasm --plugins`
Result: ✗ WASM validation errors:
```
error: type mismatch in call, expected [i32, i32, i32] but got [i32]
error: type mismatch at end of function, expected [] but got [i32]
```

### Test 4: frame.web import - INVALID
```clean
import:
    frame.web

functions:
    string hello()
        return "Hello"
start()
    printl(hello())
```
Compile: `cln compile test.cln -o test.wasm --plugins`
Result: ✗ WASM validation errors:
```
error: type mismatch in call, expected [i32, i32] but got [i32]
error: type mismatch at end of function, expected [] but got [i32]
```

## Root Cause Analysis

When plugins are imported, the compiler reads the `[bridge]` section from `plugin.toml` and registers these functions. This registration is corrupting or conflicting with the standard builtin function signatures.

### Plugin Bridge Configuration (frame.data/plugin.toml)
```toml
[bridge]
functions = [
  { name = "_db_query", params = ["string", "string"], returns = "string", expand_strings = true },
  { name = "_db_execute", params = ["string", "string"], returns = "integer", expand_strings = true },
  { name = "_db_begin", params = [], returns = "string" },
  ...
]
```

### Plugin Bridge Configuration (frame.web/plugin.toml)
```toml
[bridge]
functions = [
  { name = "_http_route", params = ["string", "string", "integer"], returns = "integer", expand_strings = true },
  { name = "_req_param", params = ["string"], returns = "string", expand_strings = true },
  ...
]
```

### Observed Problems

1. **Wrong function names in WASM imports**:
   `wasm-objdump` shows `func[67] sig=3 <math_sqrt> <- env._db_query`
   The internal name is `math_sqrt` but the import is `env._db_query` - naming is corrupted.

2. **Standard builtins corrupted**:
   Even simple code like `printl(hello())` fails after plugin import, suggesting that bridge function registration is overwriting or corrupting the builtin function table.

3. **expand_strings not working**:
   Functions with `expand_strings = true` should have their string parameters expanded to (ptr, len) pairs at WASM level, but the codegen isn't generating correct calls.

## Expected Behavior

1. Bridge functions from plugins should be registered in addition to existing builtins, not replacing them
2. Bridge function signatures should correctly reflect `expand_strings` behavior:
   - `expand_strings = true`: Each string param becomes (i32, i32) at WASM level
   - `expand_strings = false/omitted`: Standard string handling
3. Standard builtin functions should not be affected by plugin imports

## Files to Investigate

1. `src/builtins/registry.rs` - `register_plugin_bridge_functions()` method
2. `src/plugins/plugin_abi.rs` - `BridgeFunction::to_builtin_function()` method
3. `src/codegen/` - Function call generation for bridge functions
4. Plugin loading code that handles the `import:` statements

## Suggested Fix Approach

1. **Fix function table corruption**: Ensure bridge functions are added to the registry without overwriting existing entries. The naming collision (math_sqrt appearing for _db_query) suggests an index or hashmap collision.

2. **Implement expand_strings properly**:
   - When generating call instructions for bridge functions with `expand_strings = true`
   - For each string parameter, generate two i32 values: ptr and len
   - Update the function signature in the WASM import section accordingly

3. **Add unit tests**: Create tests that verify:
   - Plugin import doesn't affect non-plugin function calls
   - Bridge functions are registered with correct signatures
   - expand_strings generates correct (ptr, len) pairs

## Verification

After fix, all these should produce valid WASM:

```bash
cln compile test-no-plugins.cln -o test.wasm                    # Already works
cln compile test-with-plugins-flag.cln -o test.wasm --plugins   # Already works
cln compile test-import-data.cln -o test.wasm --plugins         # Should work
cln compile test-import-web.cln -o test.wasm --plugins          # Should work
cln compile app-db.cln -o app.wasm --plugins                    # Should work
```
