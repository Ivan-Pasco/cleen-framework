# Prompts to Fix Runtime Errors

These prompts should be used with Claude Code to fix the discovered runtime errors. Execute them in order.

---

## Prompt 1: Fix Clean Server Memory Allocation (Priority: CRITICAL)

**Repository:** `clean-server`

```
Fix the WASM string memory allocation bug in clean-server.

## Problem
The host functions `_req_param`, `_req_body`, `_req_query`, `_req_header`, `_req_method`, and `_req_path` in `src/bridge.rs` are crashing with WASM backtrace errors. The issue is in `write_string_to_caller()` function.

## Root Cause
The function allocates memory using `state.memory.allocate()` which uses a host-side bump allocator, but the WASM module expects strings in its own linear memory space. The pointer returned is invalid from WASM's perspective.

## Files to Modify
- `src/bridge.rs` - Fix `write_string_to_caller()` function (lines 1378-1412)
- `src/memory.rs` - May need updates to memory management

## Current Broken Code (bridge.rs:1378-1412)
```rust
fn write_string_to_caller(caller: &mut Caller<'_, WasmState>, s: &str) -> i32 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total_size = STRING_LENGTH_PREFIX_SIZE + len;

    let state = caller.data_mut();
    let ptr = state.memory.allocate(total_size);  // BUG: allocates in host memory

    if let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) {
        let len_bytes = (len as u32).to_le_bytes();
        let _ = memory.write(&mut *caller, ptr, &len_bytes);
        let _ = memory.write(&mut *caller, ptr + STRING_LENGTH_PREFIX_SIZE, bytes);
    }

    ptr as i32
}
```

## Required Fix
1. Call the WASM module's exported `malloc` or `mem_alloc` function to allocate memory INSIDE the WASM linear memory
2. Write the string data to that allocated memory
3. Return the pointer that the WASM module can use

## Expected Solution Pattern
```rust
fn write_string_to_caller(caller: &mut Caller<'_, WasmState>, s: &str) -> i32 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total_size = STRING_LENGTH_PREFIX_SIZE + len;

    // Get WASM memory
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return 0,
    };

    // Call WASM's malloc function to allocate in WASM memory
    let malloc = match caller.get_export("malloc").and_then(|e| e.into_func()) {
        Some(f) => f,
        None => {
            // Fallback: try mem_alloc from memory_runtime
            // Or allocate at end of current memory
            return allocate_at_memory_end(caller, s);
        }
    };

    // Call malloc(size) -> ptr
    let mut results = [wasmtime::Val::I32(0)];
    if malloc.call(&mut *caller, &[wasmtime::Val::I32(total_size as i32)], &mut results).is_err() {
        return 0;
    }

    let ptr = results[0].unwrap_i32() as usize;

    // Write string to WASM memory
    let len_bytes = (len as u32).to_le_bytes();
    let _ = memory.write(&mut *caller, ptr, &len_bytes);
    let _ = memory.write(&mut *caller, ptr + STRING_LENGTH_PREFIX_SIZE, bytes);

    ptr as i32
}
```

## Test Criteria
After fixing, these tests should pass:
1. `curl http://localhost:3333/api/todos/123` should return `id:123,...`
2. `curl -X POST -d "hello" http://localhost:3333/api/echo` should return `echo:hello`
3. `curl http://localhost:3333/api/params?foo=bar` should return `param1:bar,...`

## Test Command
```bash
cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-framework/tests/framework
./run-http-tests.sh
```

Expected: All 115 tests should pass (currently 46 pass, 69 fail).
```

---

## Prompt 2: Verify Compiler String Return Handling (Priority: MEDIUM)

**Repository:** `clean-language-compiler`

```
Verify and fix the string return value handling for host functions in the Clean Language compiler.

## Context
The clean-server's host functions (`_req_param`, `_req_body`, etc.) return string pointers. We need to verify the compiler generates correct code to handle these return values.

## Files to Investigate
1. `src/codegen/mir_codegen.rs` - Lines 1823-1835 handle request context functions
2. `src/codegen/mod.rs` - Lines 8571-8609 define the imports
3. `src/semantic/mod.rs` - Lines 1239-1251 define function signatures

## Current Code (mir_codegen.rs:1823-1835)
```rust
Some("_req_param") | Some("_req_query") | Some("_req_header") => {
    // Request context functions with string argument: _req_param(name) -> string
    // Need to expand string argument to (ptr, len)
    debug_mir!(": Matched request context function, expanding string argument");
    if !arguments.is_empty() {
        self.load_string_argument_for_print(&arguments[0])?;
    }
}
Some("_req_body") | Some("_req_method") | Some("_req_path") => {
    // Request context functions with no arguments: _req_body() -> string
    debug_mir!(": Matched request context function with no args");
    // No arguments to load
}
```

## Questions to Answer
1. After calling these functions, is the return value (i32 pointer) correctly converted to a Clean string?
2. Does the generated code expect the string format `[4-byte length][data]`?
3. Is there proper error handling if the host function returns 0 (null pointer)?

## Verification Steps
1. Compile a simple test file that uses `_req_param`:
```clean
functions:
    string test()
        string id = _req_param("id")
        return "got:" + id

start()
    printl("test")
```

2. Examine the generated WASM to verify:
   - The call to `_req_param` passes (ptr, len) for the string argument
   - The return value (i32) is stored and used as a string pointer
   - String concatenation with the result works correctly

## Expected Behavior
The compiler should generate code that:
1. Loads the string argument as (ptr, len) pair
2. Calls the host function
3. Stores the returned i32 as a string pointer
4. Uses the pointer correctly in subsequent operations (like string concatenation)

## If Issues Found
Fix the codegen to properly handle string returns from host functions. The return value is a pointer to a string in format `[4-byte length][string data]`.
```

---

## Prompt 3: Add Integration Tests for Host Functions (Priority: HIGH)

**Repository:** `clean-server`

```
Add integration tests for the request introspection host functions in clean-server.

## Context
The functions `_req_param`, `_req_body`, `_req_query`, `_req_header`, `_req_method`, and `_req_path` need proper integration tests to prevent regressions.

## Files to Create/Modify
- Create `tests/host_functions_test.rs`

## Test Cases to Implement

### Test 1: _req_param extracts path parameters
```rust
#[tokio::test]
async fn test_req_param_extracts_path_params() {
    // 1. Compile a test WASM that uses _req_param
    // 2. Start server with the WASM
    // 3. Make request to /test/123
    // 4. Verify response contains "123"
}
```

### Test 2: _req_body reads request body
```rust
#[tokio::test]
async fn test_req_body_reads_post_body() {
    // 1. Compile a test WASM that echoes _req_body()
    // 2. Start server
    // 3. POST "hello world" to endpoint
    // 4. Verify response contains "hello world"
}
```

### Test 3: _req_query extracts query parameters
```rust
#[tokio::test]
async fn test_req_query_extracts_query_params() {
    // 1. Compile test WASM using _req_query("name")
    // 2. Start server
    // 3. GET /test?name=value
    // 4. Verify response contains "value"
}
```

### Test 4: _req_header reads headers
```rust
#[tokio::test]
async fn test_req_header_reads_headers() {
    // 1. Compile test WASM using _req_header("X-Custom")
    // 2. Start server
    // 3. GET with header "X-Custom: test-value"
    // 4. Verify response contains "test-value"
}
```

### Test 5: _req_method returns HTTP method
```rust
#[tokio::test]
async fn test_req_method_returns_method() {
    // 1. Compile test WASM using _req_method()
    // 2. Start server
    // 3. Make POST request
    // 4. Verify response contains "POST"
}
```

### Test 6: _req_path returns request path
```rust
#[tokio::test]
async fn test_req_path_returns_path() {
    // 1. Compile test WASM using _req_path()
    // 2. Start server
    // 3. GET /api/test/path
    // 4. Verify response contains "/api/test/path"
}
```

## Test Helper Functions Needed
```rust
async fn compile_test_wasm(source: &str) -> PathBuf {
    // Compile .cln source to .wasm
}

async fn start_test_server(wasm_path: &Path, port: u16) -> ServerHandle {
    // Start clean-server with WASM on specified port
}

async fn make_request(method: &str, url: &str, body: Option<&str>, headers: Vec<(&str, &str)>) -> Response {
    // Make HTTP request and return response
}
```

## Success Criteria
All 6 tests pass, demonstrating that the host functions correctly:
1. Receive parameters from WASM
2. Access request context
3. Return strings to WASM in the correct format
```

---

## Prompt 4: Fix Request Context Population (Priority: HIGH)

**Repository:** `clean-server`

```
Fix the request context population in clean-server to ensure it's available to handlers.

## Problem
Debug logs show `_req_param: No request context!` which means the request context isn't being set before handlers are called.

## Files to Investigate
- `src/server.rs` - HTTP request handling
- `src/wasm.rs` - WASM invocation
- `src/bridge.rs` - Request context access

## Current Flow (Suspected)
1. HTTP request arrives at server.rs
2. Router matches the route
3. Handler is invoked via wasm.rs
4. Handler calls _req_param
5. bridge.rs tries to access request_context but it's None

## Required Fix
Ensure `WasmState.request_context` is populated BEFORE invoking the handler:

```rust
// In server.rs or wasm.rs, before calling handler:
state.request_context = Some(RequestContext {
    method: request.method().to_string(),
    path: request.uri().path().to_string(),
    query: parse_query_params(request.uri().query()),
    headers: extract_headers(request.headers()),
    body: read_body(request).await,
    params: extract_path_params(&matched_route, request.uri().path()),
});

// Then call the handler
instance.call_handler(handler_index)?;
```

## Verification
1. Add debug logging before handler invocation
2. Verify request_context is Some
3. Verify all fields are populated correctly

## Test
After fix, `curl http://localhost:3333/api/todos/42` should return `id:42,...` instead of crashing.
```

---

## Prompt 5: Memory Management Audit (Priority: MEDIUM)

**Repository:** `clean-server`

```
Audit and fix the memory management between WASM and host in clean-server.

## Problem
The current memory management has issues where host-allocated memory isn't accessible from WASM.

## Files to Audit
- `src/memory.rs` - Current memory implementation
- `src/bridge.rs` - Memory usage in host functions
- `src/wasm.rs` - WASM memory export handling

## Current memory.rs Analysis Needed
1. What does `MemoryManager::allocate()` actually do?
2. Does it allocate in WASM linear memory or host memory?
3. How does it interact with WASM's memory export?

## Key Questions
1. Is there a `malloc` export from the WASM module we can call?
2. Can we access the WASM memory's data directly?
3. Are we correctly handling memory growth?

## Required Changes

### Option A: Use WASM's malloc
```rust
// In bridge.rs
fn allocate_in_wasm(caller: &mut Caller<WasmState>, size: usize) -> Result<usize> {
    let malloc = caller.get_export("malloc")
        .and_then(|e| e.into_func())
        .ok_or("No malloc export")?;

    let mut results = [Val::I32(0)];
    malloc.call(&mut *caller, &[Val::I32(size as i32)], &mut results)?;
    Ok(results[0].unwrap_i32() as usize)
}
```

### Option B: Track WASM memory usage
```rust
// Track the high-water mark of WASM memory usage
struct WasmMemoryTracker {
    next_free: usize,
}

impl WasmMemoryTracker {
    fn allocate(&mut self, memory: &Memory, caller: &mut Caller, size: usize) -> usize {
        let ptr = self.next_free;
        self.next_free += size;

        // Grow memory if needed
        let required_pages = (self.next_free + 65535) / 65536;
        if memory.size(caller) < required_pages {
            memory.grow(caller, required_pages - memory.size(caller));
        }

        ptr
    }
}
```

## Verification
After fix:
1. Strings returned from host functions should be readable by WASM
2. No WASM trap errors when accessing returned pointers
3. Memory doesn't leak between requests
```

---

## Execution Order

1. **Prompt 4** first - Fix request context (quick fix, unblocks debugging)
2. **Prompt 1** second - Fix memory allocation (main fix)
3. **Prompt 5** third - Audit memory management (ensures robustness)
4. **Prompt 2** fourth - Verify compiler (ensures compatibility)
5. **Prompt 3** last - Add tests (prevents regressions)

## Expected Results After All Fixes

```
Tests: 115
Passed: 115
Failed: 0
```
