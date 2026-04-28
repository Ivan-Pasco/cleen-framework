# Compiler Bug: Double String Expansion in Bridge Functions

**Date:** 2025-12-25
**Severity:** CRITICAL
**Component:** Code Generation (MIR Codegen)
**Affects:** Bridge functions with `expand_strings = true`

## Summary

The compiler performs **double string expansion** when calling bridge functions that have `expand_strings = true`. String parameters are expanded at the call site AND again in the wrapper function, causing the wrong memory addresses to be read.

## Impact

- HTTP server routes fail to register (method string reads garbage data)
- Database queries fail (SQL parameter reads garbage data)
- Authentication functions fail (role/permission strings read garbage data)
- **ANY bridge function with `expand_strings = true` is broken**

## Root Cause

When `expand_strings = true`, the compiler:

1. **Creates a wrapper function** that expands Clean Language strings `(ptr)` to raw format `(ptr+4, len)`
2. **Generates call site code** that ALSO expands strings before calling the wrapper

This causes double expansion:
```
Caller:    local.get string_ptr          // 0x1000
          i32.const 4
          i32.add                         // 0x1004
          local.get string_ptr            // 0x1000
          i32.load                        // reads length from 0x1000
          call wrapper                    // passes (0x1004, len)

Wrapper:   local.get 0                    // 0x1004 (already expanded!)
          i32.const 4
          i32.add                         // 0x1008 (WRONG!)
          local.get 0                     // 0x1004
          i32.load                        // reads from 0x1004 (garbage!)
          call raw_import                 // passes wrong data
```

## Evidence

### WASM Disassembly Analysis

Compiled from:
```clean
integer status = _http_route("GET", "/test", 0)
```

**Call site (in start function):**
```wasm
local.get 9              # string_ptr for "GET"
i32.const 4
i32.add                  # ptr+4 (content start)
local.get 9
i32.load                 # length from ptr
local.get 4              # handler_idx
call 321                 # wrapper function - receives (ptr+4, len, handler)
```

**Wrapper function [321]:**
```wasm
func[321]:
  local.get 0            # receives ptr+4 (already content pointer!)
  i32.const 4
  i32.add                # ptr+4+4 = ptr+8 (WRONG!)
  local.get 0
  i32.load               # reads length from ptr+4 (garbage!)
  local.get 1            # path string
  i32.const 4
  i32.add                # expand path too
  local.get 1
  i32.load
  local.get 2            # handler_idx
  call 50                # raw import env._http_route
```

### Server Error Logs

```
[ERROR] Invalid HTTP method '                   /api/articles              '
[ERROR] read_raw_string: out of bounds: 24560..1768997151 (memory size: 1048576)
```

The method string is reading garbage because it's accessing `ptr+8` instead of `ptr+4`.

## Expected Behavior

When `expand_strings = true`:

1. **Caller** should pass the **original Clean Language string pointer** to the wrapper
2. **Wrapper** expands the string to `(ptr+4, len)` and calls the raw import
3. **Raw import** receives correct expanded parameters

**Correct call site:**
```wasm
local.get 9              # string_ptr (NOT expanded)
local.get 10             # path string_ptr (NOT expanded)
local.get 4              # handler_idx
call 321                 # wrapper receives original ptrs
```

**Correct wrapper:**
```wasm
func[321]:
  local.get 0            # original string_ptr
  i32.const 4
  i32.add                # ptr+4 (content start)
  local.get 0
  i32.load               # length from ptr
  local.get 1            # path string_ptr
  i32.const 4
  i32.add                # path ptr+4
  local.get 1
  i32.load               # path length
  local.get 2            # handler_idx
  call 50                # raw import receives correct data
```

## Actual Behavior

Call site expands strings before calling wrapper, causing double expansion.

## Files Involved

### Compiler
- `src/codegen/mir_codegen.rs` - Contains wrapper generation and call site logic
- `src/plugins/plugin_abi.rs` - Defines `expand_strings` flag
- `src/plugins/registry.rs` - Registers bridge functions from plugin.toml

### Framework
- `plugins/frame.server/plugin.toml` - Declares `_http_route` with `expand_strings = true`
- `plugins/frame.data/plugin.toml` - Declares `_db_query`, `_db_execute` with `expand_strings = true`

### Server
- `clean-server/src/bridge.rs` - Provides `_http_route`, `_http_listen` (expects ptr+4, len)
- `clean-server/host-bridge/src/wasm_linker/` - Provides database, file I/O, HTTP client functions

## Required Fix

**Location:** `src/codegen/mir_codegen.rs`

### Issue 1: Call Site Generation

When generating calls to bridge functions with `expand_strings = true`, the compiler must:

**CURRENT (WRONG):**
```rust
// Expands strings at call site
for arg in args {
    if arg.type == String {
        instructions.push(LocalGet(arg_idx));
        instructions.push(I32Const(4));
        instructions.push(I32Add);
        instructions.push(LocalGet(arg_idx));
        instructions.push(I32Load(...));
    }
}
instructions.push(Call(wrapper_idx));
```

**REQUIRED (CORRECT):**
```rust
// Pass original pointers to wrapper - let wrapper do expansion
for arg in args {
    instructions.push(LocalGet(arg_idx));
    // NO expansion here!
}
instructions.push(Call(wrapper_idx));
```

### Detection Logic

The compiler needs to check if a function call is targeting a bridge function with `expand_strings = true`:

```rust
fn is_expand_strings_bridge_function(&self, func_name: &str) -> bool {
    self.bridge_functions
        .iter()
        .any(|f| f.name == func_name && f.expand_strings)
}
```

Then in call generation:
```rust
if self.is_expand_strings_bridge_function(&func_name) {
    // Pass original Clean Language string pointers
    for arg in args {
        self.generate_simple_argument(arg);
    }
} else {
    // Normal call - expand strings if needed
    for arg in args {
        self.generate_expanded_argument(arg);
    }
}
```

## Test Case

**Source:** `test-http-route.cln`
```clean
import:
	frame.server

start()
	printl("Testing _http_route")
	integer status = _http_route("GET", "/test", 0)
	printl("Route registered")
```

**Expected WASM (after fix):**
```wasm
# Call site - passes original ptrs
local.get 9        # "GET" string ptr (NOT expanded)
local.get 10       # "/test" string ptr (NOT expanded)
i32.const 0        # handler_idx
call 321           # wrapper

# Wrapper func[321] - does expansion
func[321]:
  local.get 0      # method ptr
  i32.const 4
  i32.add          # method content
  local.get 0
  i32.load         # method length
  local.get 1      # path ptr
  i32.const 4
  i32.add          # path content
  local.get 1
  i32.load         # path length
  local.get 2      # handler
  call 50          # env._http_route
```

**Server should receive:**
- `method_ptr = 0x1004` (content start of "GET")
- `method_len = 3`
- `path_ptr = 0x100C` (content start of "/test")
- `path_len = 5`
- `handler_idx = 0`

## Verification Steps

After fix:

1. Compile test case:
   ```bash
   cln compile test-http-route.cln -o test.wasm --plugins
   ```

2. Check call site (should NOT expand strings):
   ```bash
   wasm-objdump -d test.wasm | grep -B 5 "call.*wrapper"
   # Should show: local.get, local.get, i32.const, call
   # Should NOT show: i32.const 4, i32.add before call
   ```

3. Test with clean-server:
   ```bash
   clean-server test.wasm --port 3000
   # Should register routes without "Invalid HTTP method" errors
   ```

4. Test database operations:
   ```clean
   string result = _db_query("SELECT * FROM users", "[]")
   # Should execute query successfully
   ```

## Related Issues

- Double expansion affects ALL bridge functions with `expand_strings = true`:
  - `_http_route`, `_http_route_protected`
  - `_db_query`, `_db_execute`, `_db_commit`, `_db_rollback`
  - `_auth_require_role`, `_auth_can`, `_auth_has_any_role`
  - `_req_param`, `_req_query`, `_req_header`

## Workaround

None available. Bridge functions with `expand_strings = true` cannot work until this is fixed.

## Priority

**CRITICAL** - Blocks all HTTP server and database functionality in Clean Language applications.

## References

- Platform Architecture: `/Users/earcandy/Documents/Dev/Clean Language/platform-architecture/`
- Host Bridge Spec: `foundation/platform-architecture/HOST_BRIDGE.md`
- Server Extensions: `foundation/platform-architecture/SERVER_EXTENSIONS.md`
- Plugin Bridge Contract: `documents/specification/frame_bridge_contracts.md`

---

**Next Steps:**
1. Fix call site generation to NOT expand strings when calling wrapper functions
2. Add test case to compiler test suite
3. Verify fix with all affected bridge functions
4. Release as patch version (v0.20.8)
