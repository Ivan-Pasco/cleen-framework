# Frame Bridge Contracts

## Architecture Overview

Clean Language WASM modules have two types of external interactions:

### 1. Native WASM (No Bridge Needed)

These functions are compiled directly into WASM bytecode by the compiler:

| Category | Functions |
|----------|-----------|
| **String** | `string.split`, `string.concat`, `string.trim`, `string.indexOf`, `string.substring`, `string.size`, `string.startsWith`, `string.endsWith`, `string.contains`, `string.toUpperCase`, `string.toLowerCase`, `string.replace`, `string.charAt` |
| **List** | `list.length`, `list.get`, `list.set`, `list.push`, `list.pop`, `list.concat`, `list.slice`, `list.indexOf`, `list.contains`, `list.reverse`, `list.join` |
| **Math** | `math.abs`, `math.floor`, `math.ceil`, `math.round`, `math.pow`, `math.sqrt`, `math.sin`, `math.cos`, `math.tan`, `math.log`, `math.exp` |
| **Conversion** | `int_to_string`, `float_to_string`, `bool_to_string`, `string_to_int`, `string_to_float` |
| **Memory** | `mem_alloc`, `mem_retain`, `mem_release` |

These work identically everywhere - browser, server, CLI, mobile - because they're pure WASM.

### 2. Bridge Functions (Platform Services)

These require host implementation because they access platform resources:

| Category | Functions | Browser | Server | CLI | Mobile |
|----------|-----------|---------|--------|-----|--------|
| **I/O** | `print`, `printl`, `input` | console | stdout | stdout | log |
| **Filesystem** | `file.read`, `file.write`, `file.exists`, `file.delete` | localStorage | filesystem | filesystem | app storage |
| **HTTP Client** | `http.get`, `http.post`, `http.put`, `http.delete` | fetch | reqwest | reqwest | native |
| **HTTP Server** | `_http_route`, `_http_listen`, `_req_param`, `_req_body` | - | hyper | - | - |
| **Environment** | `env.get`, `env.list` | - | env vars | env vars | app config |
| **Time** | `time.now`, `time.sleep` | Date | chrono | chrono | native |
| **Crypto** | `crypto.random`, `crypto.hash`, `crypto.sign` | WebCrypto | ring | ring | native |

---

## Bridge Implementation Layers

### Low-Level Bridge (WASM Imports)

Direct function imports used by compiled WASM:

```wasm
(import "env" "printl" (func $printl (param i32)))
(import "env" "file_read" (func $file_read (param i32) (result i32)))
(import "env" "_http_route" (func $_http_route (param i32 i32 i32) (result i32)))
```

Each platform implements these in its native language:
- **Server (Rust)**: `linker.func_wrap("env", "printl", |ptr| { ... })`
- **Browser (JS)**: `{ env: { printl: (ptr) => console.log(readString(ptr)) } }`

### High-Level Bridge (JSON Envelopes)

For complex operations, use JSON message passing:

```json
{ "fn": "host:<namespace>.<function>", "args": { ... } }
```

Response format:

```json
// Success
{ "ok": true, "data": { ... } }

// Error
{ "ok": false, "err": { "code": "ERROR_CODE", "message": "...", "details": {} } }
```

---

## String Passing Convention

All string parameters use a **pointer + length** format at the WASM level.

### Clean to WASM Type Mapping

| Clean Type | WASM Type | Notes |
|------------|-----------|-------|
| `string` | `(i32, i32)` | (pointer, length) |
| `integer` | `i32` | 32-bit signed integer |
| `number` | `f64` | 64-bit float |
| `boolean` | `i32` | 0 = false, non-zero = true |
| `void` | none | No return value |

### Passing Strings to Bridge Functions

When Clean code calls a bridge function with a string parameter:
1. Compiler allocates memory for the string
2. Writes UTF-8 bytes to memory
3. Passes `(pointer, length)` as two i32 parameters

```
Clean:    _req_param("id")
WASM:     call $_req_param (i32.const 1024) (i32.const 2)
                           ↑ pointer       ↑ length
```

### Receiving Strings from Bridge Functions

Bridge functions return strings as **length-prefixed** pointers:

```
Memory Layout:
[ptr+0..ptr+3]  : Length (u32, little-endian)
[ptr+4..ptr+4+L]: UTF-8 string bytes
```

The runtime allocates memory using WASM's `malloc` if available, otherwise uses host allocation with `memory.grow`.

### Memory Allocation Responsibility

| Direction | Allocator | Format |
|-----------|-----------|--------|
| WASM → Host | Compiler | raw (ptr, len) |
| Host → WASM | Runtime | length-prefixed |

---

## Error Envelope Format

All bridge functions that return complex data use the standard envelope:

### Success Response

```json
{
  "ok": true,
  "data": { ... }
}
```

### Error Response

```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable description",
    "details": { ... }
  }
}
```

### Error Codes by Category

| Code | Category | Description |
|------|----------|-------------|
| `DB_ERROR` | Database | General database failure |
| `QUERY_ERROR` | Database | SQL syntax or execution error |
| `CONNECTION_ERROR` | Database | Connection pool or network issue |
| `TRANSACTION_ERROR` | Database | Transaction state error |
| `VALIDATION_ERROR` | Data | Constraint violation, invalid input |
| `AUTH_ERROR` | Auth | Authentication failure |
| `PERMISSION_DENIED` | Auth | Authorization failure |
| `NOT_FOUND` | Resource | Entity not found |
| `NETWORK_FAIL` | Network | HTTP/connection error |
| `TIMEOUT` | System | Operation exceeded time limit |
| `NOT_SUPPORTED` | Platform | Function unavailable on platform |

---

## Database Bridge Functions (frame.data plugin)

Low-level WASM imports for database operations. Implemented in `clean-server/src/bridge.rs`.

### _db_query

Execute a SELECT query and return results as JSON.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_db_query(sql: string, params: string) -> string` |
| **WASM Signature** | `(i32, i32, i32, i32) -> i32` |
| **WASM Params** | `sql_ptr, sql_len, params_ptr, params_len` |
| **Returns** | Pointer to length-prefixed JSON string |

**Params Format:** JSON array of values
```json
[42, "active", true]
```

**Success Response:**
```json
{
  "ok": true,
  "data": {
    "rows": [
      {"id": 1, "name": "Alice", "email": "alice@example.com"},
      {"id": 2, "name": "Bob", "email": "bob@example.com"}
    ],
    "count": 2
  }
}
```

**Error Response:**
```json
{
  "ok": false,
  "err": {
    "code": "QUERY_ERROR",
    "message": "syntax error at or near 'SELEC'",
    "details": {}
  }
}
```

**Validation:** Only accepts queries starting with `SELECT` or `WITH`.

---

### _db_execute

Execute an INSERT, UPDATE, DELETE, or DDL statement.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_db_execute(sql: string, params: string) -> integer` |
| **WASM Signature** | `(i32, i32, i32, i32) -> i32` |
| **WASM Params** | `sql_ptr, sql_len, params_ptr, params_len` |
| **Returns** | Number of affected rows, or -1 on error |

**Params Format:** JSON array of values
```json
["Alice", "alice@example.com", 30]
```

**Example:**
```clean
affected = _db_execute(
  "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)",
  "[\"Alice\", \"alice@example.com\", 30]"
)
```

**Validation:** Only accepts `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `DROP`, `ALTER`, `TRUNCATE`.

---

### _db_begin

Begin a new database transaction.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_db_begin() -> string` |
| **WASM Signature** | `() -> i32` |
| **Returns** | Pointer to length-prefixed transaction ID string |

**Response:** Transaction ID string (e.g., `"tx_abc123def456"`)

**Usage:**
```clean
tx_id = _db_begin()
// ... perform operations ...
_db_commit(tx_id)
```

---

### _db_commit

Commit a transaction, executing all queued operations atomically.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_db_commit(tx_id: string) -> integer` |
| **WASM Signature** | `(i32, i32) -> i32` |
| **WASM Params** | `tx_id_ptr, tx_id_len` |
| **Returns** | 0 on success, -1 on error |

**Errors:**
- `TRANSACTION_ERROR`: Transaction not found, already committed, or already rolled back

---

### _db_rollback

Rollback a transaction, discarding all queued operations.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_db_rollback(tx_id: string) -> integer` |
| **WASM Signature** | `(i32, i32) -> i32` |
| **WASM Params** | `tx_id_ptr, tx_id_len` |
| **Returns** | 0 on success, -1 on error |

---

## HTTP Bridge Functions (frame.server plugin)

Low-level WASM imports for HTTP server operations. Implemented in `clean-server/src/bridge.rs`.

### _http_listen

Start the HTTP server on the specified port.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_http_listen(port: integer) -> integer` |
| **WASM Signature** | `(i32) -> i32` |
| **Returns** | 0 on success |

---

### _http_route

Register a route handler.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_http_route(method: string, path: string, handler_idx: integer) -> integer` |
| **WASM Signature** | `(i32, i32, i32, i32, i32) -> i32` |
| **WASM Params** | `method_ptr, method_len, path_ptr, path_len, handler_idx` |
| **Returns** | 0 on success |

**Path Parameters:** Use `:name` syntax (e.g., `/users/:id`)

**Example:**
```clean
_http_route("GET", "/api/users/:id", 0)
```

---

### _http_route_protected

Register a route handler with role-based protection.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_http_route_protected(method: string, path: string, handler_idx: integer, role: string) -> integer` |
| **WASM Signature** | `(i32, i32, i32, i32, i32, i32, i32) -> i32` |
| **WASM Params** | `method_ptr, method_len, path_ptr, path_len, handler_idx, role_ptr, role_len` |
| **Returns** | 0 on success |

---

### _req_param

Get a path parameter from the current request.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_param(name: string) -> string` |
| **WASM Signature** | `(i32, i32) -> i32` |
| **WASM Params** | `name_ptr, name_len` |
| **Returns** | Pointer to length-prefixed parameter value |

**Example:** For route `/users/:id` with request `/users/42`:
```clean
id = _req_param("id")  // Returns "42"
```

---

### _req_query

Get a query string parameter from the current request.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_query(name: string) -> string` |
| **WASM Signature** | `(i32, i32) -> i32` |
| **WASM Params** | `name_ptr, name_len` |
| **Returns** | Pointer to length-prefixed parameter value |

**Example:** For request `/search?q=hello`:
```clean
query = _req_query("q")  // Returns "hello"
```

---

### _req_header

Get a header value from the current request.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_header(name: string) -> string` |
| **WASM Signature** | `(i32, i32) -> i32` |
| **WASM Params** | `name_ptr, name_len` |
| **Returns** | Pointer to length-prefixed header value |

---

### _req_body

Get the request body as a string.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_body() -> string` |
| **WASM Signature** | `() -> i32` |
| **Returns** | Pointer to length-prefixed body string |

---

### _req_method

Get the HTTP method of the current request.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_method() -> string` |
| **WASM Signature** | `() -> i32` |
| **Returns** | Pointer to length-prefixed method string (GET, POST, etc.) |

---

### _req_path

Get the path of the current request.

| Property | Value |
|----------|-------|
| **Clean Signature** | `_req_path() -> string` |
| **WASM Signature** | `() -> i32` |
| **Returns** | Pointer to length-prefixed path string |

---

## Plugin Bridge Declaration Format

Plugins declare their bridge function dependencies in `plugin.toml`:

```toml
[bridge]
functions = [
  { name = "_db_query", params = ["string", "string"], returns = "string", description = "..." },
  { name = "_db_execute", params = ["string", "string"], returns = "integer", description = "..." },
]
```

The compiler reads these declarations to:
1. Register functions for semantic analysis (type checking)
2. Generate correct WASM import signatures
3. Validate function calls at compile time

---

## Version Compatibility

| Bridge Version | Compiler Version | Server Version |
|----------------|------------------|----------------|
| 1.0.0 | >= 0.15.0 | >= 1.0.0 |

Bridge functions are versioned with the framework. Breaking changes require a major version bump.

---

## Bridge Namespaces

### 1. I/O Bridge (host:io)

Console and user input operations.

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `print` | `(ptr: i32)` | Print string without newline |
| `printl` | `(ptr: i32)` | Print string with newline |
| `input` | `(prompt: i32) -> i32` | Read string input from user |
| `input_integer` | `(prompt: i32) -> i32` | Read integer input from user |
| `input_float` | `(prompt: i32) -> f64` | Read float input from user |
| `input_yesno` | `(prompt: i32) -> i32` | Read yes/no (boolean) input |

**Platform Notes:**
- Browser: `console.log`, `window.prompt`
- Server: `stdout`, `stdin`
- Mobile: Logging framework, input dialogs

---

### 2. HTTP Client Bridge (host:http)

Outbound HTTP requests.

**Low-Level Imports:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `http_get` | `(url: i32) -> i32` | GET request, returns response body |
| `http_post` | `(url: i32, body: i32) -> i32` | POST request |
| `http_put` | `(url: i32, body: i32) -> i32` | PUT request |
| `http_patch` | `(url: i32, body: i32) -> i32` | PATCH request |
| `http_delete` | `(url: i32) -> i32` | DELETE request |
| `http_post_json` | `(url: i32, json: i32) -> i32` | POST with JSON content-type |
| `http_get_with_headers` | `(url: i32, headers: i32) -> i32` | GET with custom headers |
| `http_set_timeout` | `(ms: i32)` | Set request timeout |
| `http_get_response_code` | `() -> i32` | Get last response status code |
| `http_get_response_headers` | `() -> i32` | Get last response headers |
| `http_encode_url` | `(str: i32) -> i32` | URL encode a string |
| `http_decode_url` | `(str: i32) -> i32` | URL decode a string |

**High-Level JSON:**

```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/data",
    "headers": { "Accept": "application/json" },
    "body": null,
    "timeout": 5000
  }
}
```

Response:
```json
{
  "ok": true,
  "data": {
    "status": 200,
    "headers": { "content-type": "application/json" },
    "body": "{ \"msg\": \"OK\" }"
  }
}
```

---

### 3. HTTP Server Bridge (host:server)

Server-side HTTP handling (Frame Server only).

**Low-Level Imports:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `_http_route` | `(method: i32, path: i32, handler: i32) -> i32` | Register route handler |
| `_http_listen` | `(port: i32) -> i32` | Start HTTP server |
| `_req_param` | `(name: i32) -> i32` | Get URL parameter (e.g., `:id`) |
| `_req_query` | `(name: i32) -> i32` | Get query parameter |
| `_req_body` | `() -> i32` | Get request body |
| `_req_header` | `(name: i32) -> i32` | Get request header |
| `_req_method` | `() -> i32` | Get request method |
| `_req_path` | `() -> i32` | Get request path |

**Handler Registration:**

```clean
// In Clean code
endpoints:
    GET /api/users/:id:
        string id = _req_param("id")
        return "User: " + id
```

Compiles to:
```wasm
(call $_http_route
    (i32.const 0)  ;; "GET" string pointer
    (i32.const 10) ;; "/api/users/:id" string pointer
    (i32.const 0)) ;; handler index
```

---

### 4. Filesystem Bridge (host:fs)

File operations (sandboxed).

**Low-Level Imports:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `file_read` | `(path: i32) -> i32` | Read file contents |
| `file_write` | `(path: i32, content: i32) -> i32` | Write file |
| `file_exists` | `(path: i32) -> i32` | Check if file exists |
| `file_delete` | `(path: i32) -> i32` | Delete file |
| `file_append` | `(path: i32, content: i32) -> i32` | Append to file |

**High-Level JSON:**

```json
{ "fn": "host:fs.read", "args": { "path": "data/users.json" } }
```

Response:
```json
{ "ok": true, "data": { "content": "{\"users\": []}" } }
```

**Platform Notes:**
- Browser: `localStorage` or `IndexedDB`
- Server/CLI: Filesystem (sandboxed to project directory)
- Mobile: App-specific storage

---

### 5. Database Bridge (host:db)

SQL database operations.

**High-Level JSON:**

Query:
```json
{
  "fn": "host:db.query",
  "args": { "sql": "SELECT * FROM users WHERE id=$1", "params": [1] }
}
```

Transaction:
```json
{
  "fn": "host:db.tx",
  "args": {
    "ops": [
      { "sql": "INSERT INTO users (name) VALUES ($1)", "params": ["Ana"] },
      { "sql": "INSERT INTO posts (title, user_id) VALUES ($1, $2)", "params": ["Hello", 1] }
    ]
  }
}
```

Error:
```json
{
  "ok": false,
  "err": {
    "code": "DB_ERROR",
    "message": "unique violation on users.email",
    "details": { "constraint": "users_email_key" }
  }
}
```

---

### 6. Environment Bridge (host:env)

Environment variables and configuration.

```json
{ "fn": "host:env.get", "args": { "name": "JWT_SECRET" } }
```

Response:
```json
{ "ok": true, "data": { "value": "my-secret-key" } }
```

List all:
```json
{ "fn": "host:env.list", "args": {} }
```

---

### 7. Time Bridge (host:time)

Time and date operations.

```json
{ "fn": "host:time.now" }
```

Response:
```json
{ "ok": true, "data": { "iso": "2025-12-10T20:00:00Z", "epoch": 1733860800 } }
```

Sleep:
```json
{ "fn": "host:time.sleep", "args": { "ms": 500 } }
```

---

### 8. Crypto Bridge (host:crypto)

Cryptographic operations.

Random bytes:
```json
{ "fn": "host:crypto.random", "args": { "bytes": 32 } }
```

Hash:
```json
{ "fn": "host:crypto.hash", "args": { "algo": "sha256", "data": "base64:SGVsbG8=" } }
```

Verify password:
```json
{ "fn": "host:crypto.verify", "args": { "algo": "bcrypt", "data": "plaintext", "hash": "stored-hash" } }
```

Sign JWT:
```json
{ "fn": "host:crypto.sign", "args": { "data": { "sub": 1 }, "secret": "key", "alg": "HS256" } }
```

---

### 9. Log Bridge (host:log)

Structured logging.

```json
{ "fn": "host:log.info", "args": { "event": "server.start", "port": 8080 } }
{ "fn": "host:log.warn", "args": { "event": "auth.failed", "user": "ana@x.com" } }
{ "fn": "host:log.error", "args": { "event": "db.error", "message": "connection lost" } }
```

---

### 10. System Bridge (host:sys)

System-level operations (CLI/Server only).

Exit:
```json
{ "fn": "host:sys.exit", "args": { "code": 0 } }
```

Platform info:
```json
{ "fn": "host:sys.platform" }
```

Response:
```json
{ "ok": true, "data": { "os": "linux", "arch": "x86_64", "runtime": "clean-server" } }
```

---

### 11. UI Bridge (frame.ui plugin)

Browser-only bridge functions provided by the `frame.ui` plugin via `loader.js`. All functions use WASM pointer+length format for string parameters.

#### Event Registration

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_onEvent` | selector, event_type, handler_idx | integer | Register event handler via document-level delegation |

#### Event Context (available during event handler execution)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_eventAttr` | attr_name | string | Get attribute from current event target |
| `_ui_eventValue` | — | string | Get value (inputs) or textContent (others) of event target |
| `_ui_eventClosestAttr` | selector, attr_name | string | Find closest ancestor matching selector, get its attribute |
| `_ui_eventType` | — | string | Get event type (click, input, etc.) |

#### DOM Manipulation (single element — querySelector)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_updateElement` | selector, content | integer | Set innerHTML of first matching element |
| `_ui_updateAttr` | selector, attr, value | integer | Set attribute on first matching element |
| `_ui_getText` | selector | string | Get textContent of first matching element |
| `_ui_getAttr` | selector, attr | string | Get attribute of first matching element |
| `_ui_toggleClass` | selector, class | integer | Toggle CSS class |
| `_ui_addClass` | selector, class | integer | Add CSS class |
| `_ui_removeClass` | selector, class | integer | Remove CSS class |
| `_ui_setStyle` | selector, property, value | integer | Set inline style |
| `_ui_updateElementSelf` | content | integer | Update current event target's textContent |

#### DOM Batch (querySelectorAll)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_querySetStyle` | selector, property, value | integer | Set style on ALL matching elements |
| `_ui_querySetAttr` | selector, attr, value | integer | Set attribute on ALL matching elements |
| `_ui_queryAddClass` | selector, class | integer | Add class to ALL matching elements |
| `_ui_queryRemoveClass` | selector, class | integer | Remove class from ALL matching elements |
| `_ui_filterByAttr` | selector, attr, value | integer | Show elements matching attr value, hide others (`*` for all) |
| `_ui_filterByText` | selector, name_attr, desc_attr, query | integer | Filter by text search across name/description attributes |

#### Browser APIs

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_clipboardWrite` | text | integer | Copy text to clipboard |
| `_ui_locationHref` | url | integer | Navigate to URL |
| `_ui_locationQuery` | param | string | Get URL query parameter |
| `_ui_locationPath` | — | string | Get current pathname |
| `_ui_observeVisible` | selector, class | integer | Add class when elements scroll into view (one-shot) |
| `_ui_setTimeout` | handler_idx, delay_ms | integer | Call `handle_event_N` after delay |

#### HTML Rendering

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_html_escape` | string | string | Escape HTML entities in string |
| `_html_raw` | string | string | Return raw HTML without escaping |

#### Component Registry

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_registerComponent` | tag, class | integer | Register component by tag name |
| `_ui_getComponent` | tag | string | Get component class by tag name |
| `_ui_setSlot` | name, content | integer | Set slot content |
| `_ui_getSlot` | name | string | Get slot content |

#### State Management

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_setState` | id, json | integer | Set component state as JSON |
| `_ui_getState` | id | string | Get component state JSON |

#### Form Binding & Validation

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_bindInput` | selector, path | integer | Bind input to state path |
| `_ui_validate` | value, rule | string | Validate value against rule (required, email, url) |

---

## Error Codes

| Code | Meaning |
|------|---------|
| `DB_ERROR` | Database or transaction failure |
| `AUTH_ERROR` | Unauthorized / invalid credentials |
| `NETWORK_FAIL` | Connection or timeout error |
| `VALIDATION_ERROR` | Field or data constraint violation |
| `FILE_NOT_FOUND` | File system access error |
| `PERMISSION_DENIED` | Missing bridge permission |
| `NOT_FOUND` | Resource not found |
| `TIMEOUT` | Operation exceeded allowed time |
| `NOT_SUPPORTED` | Function not available on this platform |

---

## Implementing a Bridge

### Rust (clean-server)

```rust
use wasmtime::*;

fn setup_bridge(linker: &mut Linker<State>) -> Result<()> {
    // I/O
    linker.func_wrap("env", "printl", |caller: Caller<State>, ptr: i32| {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let s = read_string(&mem, ptr);
        println!("{}", s);
    })?;

    // HTTP Server
    linker.func_wrap("env", "_http_route", |mut caller: Caller<State>, method: i32, path: i32, handler: i32| -> i32 {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let method_str = read_string(&mem, method);
        let path_str = read_string(&mem, path);
        caller.data_mut().routes.push((method_str, path_str, handler as usize));
        1 // success
    })?;

    Ok(())
}
```

### JavaScript (Browser)

```javascript
const bridge = {
    "printl": (ptr) => {
        console.log(readString(memory, ptr));
    },

    "http_get": async (urlPtr) => {
        const url = readString(memory, urlPtr);
        const response = await fetch(url);
        return writeString(memory, await response.text());
    },

    "file_read": (pathPtr) => {
        const path = readString(memory, pathPtr);
        const content = localStorage.getItem(`fs:${path}`);
        if (!content) throw new Error("FILE_NOT_FOUND");
        return writeString(memory, content);
    }
};

const wasm = await WebAssembly.instantiate(bytes, { env: bridge });
```

### Swift (iOS)

```swift
let bridge: [String: Any] = [
    "printl": { (ptr: Int32) in
        let str = readString(memory, ptr)
        NSLog("%@", str)
    },

    "http_get": { (urlPtr: Int32) -> Int32 in
        let url = readString(memory, urlPtr)
        // Use URLSession...
    }
]
```

---

## Platform Availability Matrix

| Function | Browser | Server | CLI | iOS | Android |
|----------|---------|--------|-----|-----|---------|
| `printl` | console | stdout | stdout | NSLog | Log.d |
| `input` | prompt | stdin | stdin | alert | dialog |
| `file_read` | localStorage | fs | fs | FileManager | File |
| `file_write` | localStorage | fs | fs | FileManager | File |
| `http_get` | fetch | reqwest | reqwest | URLSession | OkHttp |
| `_http_route` | - | hyper | - | - | - |
| `_http_listen` | - | tokio | - | - | - |
| `env.get` | - | env | env | Bundle | BuildConfig |
| `crypto.random` | WebCrypto | ring | ring | SecRandom | SecureRandom |
| `_ui_loadLayout` | - | server | - | - | - |
| `_ui_injectHeadCss` | - | server | - | - | - |
| `_canvas_on_pointer_down` | browser | - | desktop | - | - |
| `_canvas_on_pointer_move` | browser | - | desktop | - | - |
| `_canvas_on_key_down` | browser | - | desktop | - | - |
| `_canvas_event_x` | browser | - | desktop | - | - |
| `_canvas_event_y` | browser | - | desktop | - | - |
| `_canvas_event_key` | browser | - | desktop | - | - |
| `_db_configure` | - | server | - | - | - |
| `_db_register_migration` | - | server | - | - | - |
| `_db_migration_diff` | - | server | - | - | - |

---

## AI Development Notes

1. All bridge functions are stateless and idempotent where possible
2. JSON shape is deterministic - AI tools can safely build, simulate, and validate calls
3. Always use the `"fn"` and `"args"` properties explicitly
4. Nested operations use `ops` arrays for parsing simplicity
5. Always return Error Envelopes; never throw raw host errors
6. Platform availability should be checked before using platform-specific functions
