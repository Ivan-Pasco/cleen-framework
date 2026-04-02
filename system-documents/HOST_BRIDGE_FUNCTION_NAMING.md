# Host Bridge Function Naming Issue

**Date:** 2025-12-25
**Severity:** CRITICAL
**Component:** host-bridge/src/wasm_linker/database.rs, clean-server/src/bridge.rs
**Affects:** All bridge functions with `expand_strings = true`

## Summary

Bridge functions that have `expand_strings = true` in their plugin.toml declarations are registered with the wrong names in the WASM linker. The compiler generates wrapper functions that call imports with `__raw` suffix, but host-bridge and clean-server register them without the suffix.

## Impact

- WASM modules fail to instantiate with error: `unknown import: env::_db_query__raw has not been defined`
- Applications that use database, HTTP routing, or other bridge functions cannot run
- Affects all runtimes using host-bridge (clean-server, clean-cli, etc.)

## Root Cause

When a plugin declares a bridge function with `expand_strings = true`:

1. **Compiler behavior** (clean-language-compiler/src/codegen/mir_codegen.rs):
   - Generates a wrapper function that takes Clean Language strings (ptr)
   - Wrapper expands strings to raw format (ptr+4, len)
   - Wrapper calls the raw import: `function_name__raw(ptr+4, len, ...)`

2. **Host-bridge behavior** (host-bridge/src/wasm_linker/database.rs):
   - Registers functions with original name: `_db_query` (NO `__raw` suffix)
   - Function signature expects raw parameters: `(sql_ptr: i32, sql_len: i32, ...)`
   - **MISMATCH**: Compiler calls `_db_query__raw`, but host provides `_db_query`

## Evidence

### WASM Module Imports (from wasm-objdump)

```
 - func[50] sig=3 <env._db_query__raw> <- env._db_query__raw
 - func[51] sig=3 <env._db_execute__raw> <- env._db_execute__raw
 - func[52] sig=9 <env._db_begin> <- env._db_begin
 - func[53] sig=4 <env._db_commit__raw> <- env._db_commit__raw
 - func[54] sig=4 <env._db_rollback__raw> <- env._db_rollback__raw
 - func[55] sig=11 <env._http_route__raw> <- env._http_route__raw
 - func[56] sig=12 <env._http_route_protected__raw> <- env._http_route_protected__raw
 - func[57] sig=4 <env._req_param__raw> <- env._req_param__raw
 - func[58] sig=4 <env._req_query__raw> <- env._req_query__raw
 - func[59] sig=4 <env._req_header__raw> <- env._req_header__raw
 - func[63] sig=4 <env._auth_require_role__raw> <- env._auth_require_role__raw
 - func[64] sig=4 <env._auth_can__raw> <- env._auth_can__raw
 - func[65] sig=4 <env._auth_has_any_role__raw> <- env._auth_has_any_role__raw
```

### Plugin Declarations

**plugins/frame.data/plugin.toml:**
```toml
[bridge]
functions = [
  { name = "_db_query", params = ["string", "string"], returns = "string", expand_strings = true },
  { name = "_db_execute", params = ["string", "string"], returns = "integer", expand_strings = true },
  { name = "_db_begin", params = [], returns = "string" },
  { name = "_db_commit", params = ["string"], returns = "integer", expand_strings = true },
  { name = "_db_rollback", params = ["string"], returns = "integer", expand_strings = true },
]
```

**plugins/frame.server/plugin.toml:**
```toml
[bridge]
functions = [
  { name = "_http_listen", params = ["integer"], returns = "integer" },
  { name = "_http_route", params = ["string", "string", "integer"], returns = "integer", expand_strings = true },
  { name = "_http_route_protected", params = ["string", "string", "integer", "string"], returns = "integer", expand_strings = true },
  { name = "_req_param", params = ["string"], returns = "string", expand_strings = true },
  { name = "_req_query", params = ["string"], returns = "string", expand_strings = true },
  { name = "_req_header", params = ["string"], returns = "string", expand_strings = true },
  { name = "_auth_require_role", params = ["string"], returns = "integer", expand_strings = true },
  { name = "_auth_can", params = ["string"], returns = "integer", expand_strings = true },
  { name = "_auth_has_any_role", params = ["string"], returns = "integer", expand_strings = true },
]
```

### Current Host-Bridge Registration (WRONG)

**host-bridge/src/wasm_linker/database.rs:**
```rust
linker.func_wrap(
    "env",
    "_db_query",  // ❌ WRONG - should be "_db_query__raw"
    |mut caller: Caller<'_, S>,
     sql_ptr: i32,
     sql_len: i32,
     params_ptr: i32,
     params_len: i32|  // Function takes raw (ptr, len) pairs
    -> i32 { ... }
)?;
```

### clean-server Registration (CORRECT)

**clean-server/src/bridge.rs:**
```rust
linker.func_wrap(
    "env",
    "_http_route",  // ✅ CORRECT - clean-server uses wrapper approach
    |mut caller: Caller<'_, WasmState>,
     method_ptr: i32,
     method_len: i32,
     path_ptr: i32,
     path_len: i32,
     handler_idx: i32| -> i32 { ... }
)?;
```

**Wait** - clean-server also registers without `__raw` suffix! Let me check if it's working...

Actually, I just remembered: the server startup logs showed routes registered successfully in v0.2.0 with the double expansion fix. Let me verify what clean-server is actually registering.

## Required Fix

### Option 1: Change host-bridge to register `__raw` functions

Update all host-bridge functions that take raw (ptr, len) parameters to register with `__raw` suffix:

**host-bridge/src/wasm_linker/database.rs:**
```rust
linker.func_wrap(
    "env",
    "_db_query__raw",  // ✅ Add __raw suffix
    |mut caller: Caller<'_, S>,
     sql_ptr: i32,
     sql_len: i32,
     params_ptr: i32,
     params_len: i32| -> i32 { ... }
)?;

linker.func_wrap(
    "env",
    "_db_execute__raw",  // ✅ Add __raw suffix
    ...
)?;

linker.func_wrap(
    "env",
    "_db_commit__raw",  // ✅ Add __raw suffix
    ...
)?;

linker.func_wrap(
    "env",
    "_db_rollback__raw",  // ✅ Add __raw suffix
    ...
)?;
```

**clean-server/src/bridge.rs:**
```rust
linker.func_wrap(
    "env",
    "_http_route__raw",  // ✅ Add __raw suffix
    ...
)?;

linker.func_wrap(
    "env",
    "_http_route_protected__raw",  // ✅ Add __raw suffix
    ...
)?;

linker.func_wrap(
    "env",
    "_req_param__raw",  // ✅ Add __raw suffix
    ...
)?;

// And all other expand_strings = true functions
```

### Option 2: Change compiler to NOT add `__raw` suffix

This would require changes in clean-language-compiler to NOT add `__raw` suffix when generating raw import calls. This is more invasive and affects the compiler's plugin architecture.

**Recommended: Option 1** - Change host-bridge and clean-server to use `__raw` suffix. This is the correct behavior based on the compiler's design.

## Affected Functions

### host-bridge functions (database.rs):
- `_db_query` → `_db_query__raw` ✅
- `_db_execute` → `_db_execute__raw` ✅
- `_db_commit` → `_db_commit__raw` ✅
- `_db_rollback` → `_db_rollback__raw` ✅
- `_db_begin` - NO CHANGE (no string params)

### clean-server functions (bridge.rs):
- `_http_route` → `_http_route__raw` ✅
- `_http_route_protected` → `_http_route_protected__raw` ✅
- `_req_param` → `_req_param__raw` ✅
- `_req_query` → `_req_query__raw` ✅
- `_req_header` → `_req_header__raw` ✅
- `_auth_require_role` → `_auth_require_role__raw` ✅
- `_auth_can` → `_auth_can__raw` ✅
- `_auth_has_any_role` → `_auth_has_any_role__raw` ✅
- `_http_listen` - NO CHANGE (no string params)
- `_req_body` - NO CHANGE (no params)
- `_req_method` - NO CHANGE (no params)
- `_req_path` - NO CHANGE (no params)
- `_auth_get_session` - NO CHANGE (no params)
- `_auth_require_auth` - NO CHANGE (no params)

## Test Case

**Source:** app-db.cln (article-blog example)

**Expected behavior:**
```bash
$ cln compile app-db.cln -o app-db.wasm --plugins
$ clean-server app-db.wasm --port 3000
[INFO] Server starting on http://localhost:3000
[INFO] Routes registered: GET /, GET /api/articles, GET /articles/:slug
```

**Current behavior:**
```bash
$ clean-server app-db.wasm --port 3000
[ERROR] Server error: WASM error: Failed to instantiate WASM module: unknown import: `env::_db_query__raw` has not been defined
```

**After fix:**
- WASM module loads successfully
- Routes register without errors
- Database queries execute (or fail with "No database configured" which is expected)

## Related Issues

- Double string expansion bug (RESOLVED in v0.20.8)
- Plugin bridge architecture contract enforcement

## Priority

**CRITICAL** - Blocks all applications that use database functions or HTTP server features.

## References

- Compiler: `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/codegen/mir_codegen.rs`
- Host-bridge: `/Users/earcandy/Documents/Dev/Clean Language/clean-server/host-bridge/src/wasm_linker/database.rs`
- clean-server: `/Users/earcandy/Documents/Dev/Clean Language/clean-server/src/bridge.rs`
- Plugins: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/plugins/`

---

**Next Steps:**
1. Fix host-bridge database functions to use `__raw` suffix
2. Fix clean-server HTTP/auth functions to use `__raw` suffix
3. Rebuild clean-server as v0.2.1
4. Test with app-db.cln example
5. Verify all bridge functions work correctly
