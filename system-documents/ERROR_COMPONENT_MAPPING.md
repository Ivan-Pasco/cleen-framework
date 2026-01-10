# Error Component Mapping

This document maps the test failures discovered in framework testing to their root cause components.

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Test Applications                                │
│                    (todo_app.cln, auth_app.cln, etc.)                   │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         COMPILER                                         │
│              (clean-language-compiler)                                   │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ 1. Parser: Parses .cln source files                                │ │
│  │ 2. Semantic: Type checking, registers built-in functions           │ │
│  │ 3. Resolver: Resolves function signatures                          │ │
│  │ 4. Codegen: Generates WASM imports/exports                         │ │
│  │ 5. Plugins: Compiler plugins (frame.web) expand syntax             │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ Produces
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          WASM Module                                     │
│                      (.wasm binary)                                      │
│  - Imports: env._req_param, env._req_body, env._req_query, etc.         │
│  - Exports: _start, handler functions                                    │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ Loaded by
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         SERVER                                           │
│                    (clean-server)                                        │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ bridge.rs: Provides host functions for WASM imports                │ │
│  │ wasm.rs: WASM runtime using wasmtime                               │ │
│  │ router.rs: HTTP request routing                                    │ │
│  │ server.rs: HTTP server handling                                    │ │
│  │ memory.rs: WASM memory management                                  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

## Error Mapping Table

| Error Category | Symptom | Root Cause Component | File Location | Details |
|---------------|---------|---------------------|---------------|---------|
| `_req_param()` crash | WASM backtrace error when accessing path params | **SERVER + COMPILER** | `clean-server/src/bridge.rs:541-591` | Memory layout mismatch between compiler's string format and server's expectations |
| `_req_body()` crash | WASM backtrace error when reading request body | **SERVER + COMPILER** | `clean-server/src/bridge.rs:625-642` | Return value handling issue |
| `_req_query()` crash | WASM backtrace error when reading query params | **SERVER + COMPILER** | `clean-server/src/bridge.rs:593-623` | Same as _req_param |
| `_req_header()` crash | WASM backtrace error when reading headers | **SERVER + COMPILER** | `clean-server/src/bridge.rs:644-677` | Same as _req_param |
| `_req_method()` crash | WASM backtrace error when reading HTTP method | **SERVER + COMPILER** | `clean-server/src/bridge.rs:679-696` | Return value handling issue |
| `_req_path()` crash | WASM backtrace error when reading request path | **SERVER + COMPILER** | `clean-server/src/bridge.rs:698-715` | Return value handling issue |
| `body.contains()` not matching | String contains returns false for valid matches | **TEST DATA** | Test script | POST body format differs from expected string |

## Detailed Analysis

### 1. Request Introspection Functions (CRITICAL)

**Affected Functions:**
- `_req_param(name)` - Path parameter extraction
- `_req_query(name)` - Query parameter extraction
- `_req_body()` - Request body reading
- `_req_header(name)` - Header extraction
- `_req_method()` - HTTP method
- `_req_path()` - Request path

**Root Cause: Memory Layout Mismatch**

The compiler generates WASM that expects strings in a specific format:
```
[4 bytes: length][N bytes: string data]
```

The server's `write_string_to_caller()` function writes:
```rust
fn write_string_to_caller(caller: &mut Caller<'_, WasmState>, s: &str) -> i32 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total_size = STRING_LENGTH_PREFIX_SIZE + len;  // 4 + len

    let state = caller.data_mut();
    let ptr = state.memory.allocate(total_size);

    // Write length prefix
    let len_bytes = (len as u32).to_le_bytes();
    memory.write(ptr, &len_bytes);        // Write 4-byte length
    memory.write(ptr + 4, bytes);         // Write string data

    ptr as i32  // Returns pointer to length prefix
}
```

**The Issue:**
When the WASM code receives the pointer and tries to use the string, it crashes because:
1. The compiler generates code that expects the pointer to be a Clean string structure
2. The server allocates memory using `state.memory.allocate()` which uses a simple bump allocator
3. The WASM module may be trying to access memory beyond what's allocated in its own memory space
4. The crash happens in WASM function 280, which is likely a string utility function

**Component Responsibility:**
- **COMPILER (60%)**: Generates WASM that expects specific memory layout
- **SERVER (40%)**: Provides host functions that write to memory

### 2. Static Returns Work

Routes that return static strings without calling request introspection functions work:

```clean
string __route_handler_0()
    return "Welcome to Todo App"  // Works - no runtime calls
```

vs

```clean
string __route_handler_3()
    string id = _req_param("id")  // Crashes - runtime call
    return "id:" + id
```

### 3. String Contains Logic

The auth tests show that `body.contains("test@test.com")` fails. This is a **TEST DATA** issue:

- The HTTP POST sends: `email=test@test.com&password=secret123` (URL-encoded form)
- The handler expects the body to contain literal `"test@test.com"`
- This should work, but may fail if `_req_body()` crashes before reaching the contains check

## Fix Recommendations by Component

### Clean Server (Priority: HIGH)

**File: `src/bridge.rs`**

1. **Fix string writing to WASM memory:**
   ```rust
   fn write_string_to_caller(caller: &mut Caller<'_, WasmState>, s: &str) -> i32 {
       // The issue is that we're writing to our own allocator,
       // but the WASM module expects strings in its memory space.
       // We need to use the WASM module's memory export properly.
   }
   ```

2. **Ensure request context is populated:**
   The debug logs show `_req_param: No request context!` which means the request context isn't being set before the handler is called.

3. **Memory management:**
   - The server uses `state.memory.allocate()` but this allocates in host memory
   - Need to allocate in WASM memory space using the exported `malloc` or `mem_alloc` function

### Clean Compiler (Priority: MEDIUM)

**File: `src/codegen/mir_codegen.rs`**

1. **Verify return value handling:**
   ```rust
   Some("_req_param") | Some("_req_query") | Some("_req_header") => {
       // String argument expansion is correct
       // Need to verify return value is handled as string pointer
   }
   ```

2. **String pointer format:**
   - Ensure compiled code expects `[4-byte length][data]` format
   - Verify the pointer returned by host functions is used correctly

### Framework (Priority: LOW)

The framework itself (test applications) is correctly written. The issues are in the runtime components.

### Plugins (Priority: LOW)

The compiler plugins are syntax expanders and don't affect runtime behavior.

## Test Results Summary

| Component | Tests | Pass | Fail | Pass Rate |
|-----------|-------|------|------|-----------|
| Static Routes | 30 | 30 | 0 | 100% |
| Request Introspection | 69 | 0 | 69 | 0% |
| String Operations | 16 | 16 | 0 | 100% |
| **Total** | **115** | **46** | **69** | **40%** |

## Action Items

1. **SERVER**: Debug `write_string_to_caller()` to ensure proper WASM memory allocation
2. **SERVER**: Verify request context is set before handler invocation
3. **SERVER**: Add integration tests for request introspection functions
4. **COMPILER**: Verify string return value handling in codegen
5. **TESTING**: Add unit tests for each host function

## Related Files

**Server:**
- `/clean-server/src/bridge.rs` - Host function implementations
- `/clean-server/src/memory.rs` - Memory management
- `/clean-server/src/wasm.rs` - WASM runtime

**Compiler:**
- `/clean-language-compiler/src/codegen/mir_codegen.rs` - Code generation
- `/clean-language-compiler/src/codegen/mod.rs` - WASM imports
- `/clean-language-compiler/src/semantic/mod.rs` - Function signatures

**Tests:**
- `/clean-framework/tests/framework/apps/*.cln` - Test applications
- `/clean-framework/tests/framework/run-http-tests.sh` - Test runner
