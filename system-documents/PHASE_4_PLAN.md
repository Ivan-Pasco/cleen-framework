# Phase 4: Frame Server Implementation Plan

**Project:** Frame Framework
**Phase:** 4 - HTTP Server & WASM Runtime
**Status:** Planning
**Start Date:** November 20, 2025
**Dependencies:** Phase 3 (Frame Data ORM) ✅ Complete

---

## Overview

Frame Server is the HTTP server component that:
1. Executes WASM modules compiled from Clean Language
2. Routes HTTP requests to appropriate WASM functions
3. Bridges WASM ↔ Host system operations via Host Bridge
4. Renders server-side React/UI components (SSR)
5. Handles authentication, sessions, and middleware

**Reference:** `/documents/specification/03_frame_server.md`

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     HTTP Request                         │
│                  (GET /api/users)                        │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                  Frame Server (Rust)                     │
│  ┌──────────────────────────────────────────────────┐  │
│  │           Axum HTTP Server                       │  │
│  │  - Router (file-based routing)                   │  │
│  │  - Middleware (logging, CORS, auth)              │  │
│  │  - Request/Response parsing                      │  │
│  └──────────────────┬───────────────────────────────┘  │
│                     │                                    │
│                     ▼                                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │        Wasmtime Runtime                          │  │
│  │  - Load WASM modules (server.wasm, ui.wasm)     │  │
│  │  - Execute WASM functions                        │  │
│  │  - Memory isolation per request                  │  │
│  └──────────────────┬───────────────────────────────┘  │
│                     │                                    │
│                     ▼                                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │           Host Bridge                            │  │
│  │  - bridge:db (database operations)               │  │
│  │  - bridge:http (HTTP client)                     │  │
│  │  - bridge:env (environment vars)                 │  │
│  │  - bridge:log (structured logging)               │  │
│  │  - bridge:time (time operations)                 │  │
│  │  - bridge:crypto (cryptographic ops)             │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│               System Resources                           │
│  - Database (PostgreSQL, MySQL, SQLite)                 │
│  - Filesystem                                            │
│  - Network                                               │
│  - Environment Variables                                 │
└─────────────────────────────────────────────────────────┘
```

---

## Goals

1. **WASM Execution**: Load and execute Clean Language WASM modules
2. **HTTP Routing**: File-based routing with `endpoints:` blocks
3. **Host Bridge**: Secure interface for system operations
4. **Request Handling**: Parse HTTP requests into Clean types
5. **Response Helpers**: `json()`, `html()`, `redirect()`, `notFound()`
6. **Middleware**: Logging, CORS, authentication, rate limiting
7. **Performance**: < 50ms first request (SSR), > 10k req/sec

---

## Implementation Plan

### Week 1: WASM Runtime Foundation

**Goal:** Execute WASM modules with Wasmtime and call functions

**Tasks:**
1. Implement `WasmRuntime` with Wasmtime
   - Engine configuration (optimizations, memory limits)
   - Module loading and caching
   - Store creation per request (isolation)
   - Function invocation with type conversion

2. Implement Host Bridge integration for server runtime
   - Link Host Bridge functions into WASM instance
   - JSON envelope serialization/deserialization
   - Error handling and propagation

3. Test WASM function execution
   - Simple "hello world" endpoint
   - JSON serialization roundtrip
   - Error handling

**Files:**
- `frame-server/src/runtime.rs` - WASM runtime
- `frame-server/src/bridge.rs` - Host Bridge integration
- `frame-server/tests/runtime_tests.rs` - Runtime tests

**Success Criteria:**
- Can load WASM module
- Can call WASM function by name
- Can pass JSON parameters
- Can receive JSON response
- Errors propagate correctly

---

### Week 2: HTTP Router with Axum

**Goal:** Route HTTP requests to WASM functions using Axum

**Tasks:**
1. Implement `FrameRouter` with Axum
   - Route registration (GET, POST, PUT, PATCH, DELETE)
   - Path parameters extraction (`/users/:id`)
   - Query string parsing
   - Request body parsing (JSON, form data)

2. Implement file-based routing
   - Scan `/app/api/*.cln` for `endpoints:` blocks
   - Parse endpoint declarations (METHOD + PATH)
   - Generate route handlers dynamically

3. Implement request context
   - `Request` type with headers, params, query, body
   - `Response` builders (`json()`, `html()`, `redirect()`)
   - Status code helpers

**Files:**
- `frame-server/src/router.rs` - HTTP router
- `frame-server/src/request.rs` - Request context
- `frame-server/src/response.rs` - Response builders
- `frame-server/tests/router_tests.rs` - Router tests

**Success Criteria:**
- Can register routes programmatically
- Can extract path parameters
- Can parse JSON/form bodies
- Can return JSON responses
- Can return HTML responses
- Can redirect

---

### Week 3: Middleware & Server Lifecycle

**Goal:** Add middleware support and complete server lifecycle

**Tasks:**
1. Implement middleware system
   - Logging middleware (request/response timing)
   - CORS middleware (configurable origins)
   - Authentication middleware (session/JWT validation)
   - Error handling middleware

2. Implement server lifecycle
   - Graceful shutdown (drain connections)
   - Hot reload support (watch WASM files)
   - Health check endpoint (`/health`)
   - Metrics endpoint (`/metrics`)

3. Implement static file serving
   - Serve files from `/public` directory
   - Cache headers (ETags, Last-Modified)
   - Gzip compression

**Files:**
- `frame-server/src/middleware.rs` - Middleware implementations
- `frame-server/src/server.rs` - Server lifecycle
- `frame-server/src/static_files.rs` - Static file handler
- `frame-server/tests/middleware_tests.rs` - Middleware tests

**Success Criteria:**
- Logs all requests with timing
- CORS headers work correctly
- Auth middleware blocks unauthorized requests
- Graceful shutdown drains connections
- Static files served with caching
- Gzip compression works

---

### Week 4: Integration & Performance

**Goal:** End-to-end integration tests and performance tuning

**Tasks:**
1. Integration tests
   - Test full request → WASM → database → response flow
   - Test authentication flows
   - Test error handling paths
   - Test concurrent requests

2. Performance optimization
   - Connection pooling for database
   - WASM module caching
   - Response caching middleware
   - Benchmark and profile

3. Documentation
   - API documentation generation
   - Deployment guide
   - Performance tuning guide

**Files:**
- `frame-server/tests/integration_tests.rs` - Integration tests
- `frame-server/benches/server_bench.rs` - Benchmarks
- `frame-server/examples/full_server.rs` - Complete example

**Success Criteria:**
- All integration tests passing
- < 50ms first request latency (SSR)
- < 10ms p95 API latency (simple endpoint)
- < 100ms p95 API latency (with database)
- > 10k req/sec throughput
- Documentation complete

---

## Technical Specifications

### WASM Runtime

**Wasmtime Configuration:**
```rust
let mut config = Config::new();
config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
config.wasm_multi_memory(true);
config.async_support(true);
config.consume_fuel(true); // Resource limiting

let engine = Engine::new(&config)?;
```

**Per-Request Isolation:**
- Each HTTP request gets its own `Store`
- Memory is isolated between requests
- Fuel limits prevent infinite loops
- Timeout after 30 seconds

**Function Invocation:**
```rust
// WASM exports: handle_request(request_json: string) -> string
let handle_request = instance
    .get_typed_func::<u32, u32>(&mut store, "handle_request")?;

let request_json = serde_json::to_string(&request)?;
let request_ptr = allocate_string(&mut store, &instance, &request_json)?;

let response_ptr = handle_request.call(&mut store, request_ptr)?;
let response_json = read_string(&mut store, &instance, response_ptr)?;

let response: Response = serde_json::from_str(&response_json)?;
```

### HTTP Router

**Route Registration:**
```rust
let router = Router::new()
    .route("/api/users", get(get_users).post(create_user))
    .route("/api/users/:id", get(get_user).put(update_user).delete(delete_user))
    .layer(LoggingMiddleware::new())
    .layer(CorsMiddleware::new());
```

**Request Context:**
```rust
pub struct Request {
    pub method: Method,
    pub path: String,
    pub params: HashMap<String, String>,   // Path params: /users/:id
    pub query: HashMap<String, String>,     // Query string: ?page=1
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,   // Parsed JSON body
    pub user: Option<User>,                 // From auth middleware
}
```

**Response Builders:**
```rust
// JSON response
pub fn json<T: Serialize>(data: T) -> Response {
    Response {
        status: 200,
        headers: HashMap::from([("Content-Type", "application/json")]),
        body: serde_json::to_string(&data).unwrap(),
    }
}

// HTML response
pub fn html(content: String) -> Response {
    Response {
        status: 200,
        headers: HashMap::from([("Content-Type", "text/html")]),
        body: content,
    }
}

// Redirect
pub fn redirect(url: String) -> Response {
    Response {
        status: 302,
        headers: HashMap::from([("Location", url)]),
        body: String::new(),
    }
}

// Error responses
pub fn notFound() -> Response { /* 404 */ }
pub fn badRequest(msg: String) -> Response { /* 400 */ }
pub fn unauthorized() -> Response { /* 401 */ }
pub fn internalError(msg: String) -> Response { /* 500 */ }
```

### Host Bridge Integration

**Server-Specific Bridge Functions:**

1. **bridge:http** - HTTP client operations
   ```json
   {
     "fn": "bridge:http.fetch",
     "args": {
       "url": "https://api.example.com/data",
       "method": "GET",
       "headers": {"Authorization": "Bearer ..."},
       "body": null,
       "timeout": 30000
     }
   }
   ```

2. **bridge:env** - Environment variables
   ```json
   {
     "fn": "bridge:env.get",
     "args": {
       "key": "DATABASE_URL"
     }
   }
   ```

3. **bridge:log** - Structured logging
   ```json
   {
     "fn": "bridge:log.info",
     "args": {
       "message": "User logged in",
       "context": {"user_id": 123, "ip": "192.168.1.1"}
     }
   }
   ```

4. **bridge:time** - Time operations
   ```json
   {
     "fn": "bridge:time.now",
     "args": {}
   }
   ```

---

## Dependencies

### Existing (Already in Workspace)
- `tokio` - Async runtime
- `axum` - HTTP server framework
- `hyper` - HTTP primitives
- `tower` - Middleware framework
- `tower-http` - HTTP middleware utilities
- `wasmtime` - WASM runtime
- `serde/serde_json` - Serialization

### New Dependencies Needed
- `http` - HTTP types (already in Cargo.toml)
- `bytes` - Byte buffer utilities (already in Cargo.toml)
- `pin-project` - Pin projection (already in Cargo.toml)

### Internal Dependencies
- `frame-data` - ORM integration
- `frame-auth` - Authentication
- `host-bridge` - System interface

---

## Testing Strategy

### Unit Tests
- WASM runtime (module loading, function calls)
- Router (route matching, parameter extraction)
- Request/Response builders
- Middleware (logging, CORS, auth)

### Integration Tests
- Full HTTP request flow
- Database operations via Host Bridge
- Authentication flows
- Error handling

### Performance Tests
- Benchmark simple endpoint (JSON response)
- Benchmark database query endpoint
- Benchmark SSR page render
- Load testing (concurrent requests)

**Test Coverage Goal:** > 80%

---

## Performance Targets

### Latency
- **First request (SSR page):** < 50ms
- **Simple API endpoint (no DB):** < 1ms
- **Database query endpoint:** < 10ms (p95)
- **Complex query endpoint:** < 100ms (p95)

### Throughput
- **Simple endpoints:** > 10k req/sec
- **Database endpoints:** > 1k req/sec

### Resource Usage
- **Memory per request:** < 1MB
- **Concurrent requests:** > 10k
- **WASM module size:** < 10MB

---

## Example: Complete Server

**File:** `frame-server/examples/full_server.rs`

```rust
use frame_server::{FrameServer, RouterBuilder, WasmRuntime};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load WASM module
    let runtime = WasmRuntime::new("target/wasm32-wasi/release/server.wasm")?;

    // Create router
    let router = RouterBuilder::new()
        .runtime(runtime)
        .route("/api/*", "handle_api_request")
        .route("/pages/*", "handle_page_request")
        .static_files("/public")
        .build()?;

    // Create server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = FrameServer::new(router, addr)?;

    println!("🚀 Frame server listening on http://{}", addr);

    // Start server
    server.start().await?;

    Ok(())
}
```

---

## Success Criteria

### Functionality
- ✅ Load and execute WASM modules
- ✅ Route HTTP requests to WASM functions
- ✅ Parse request parameters, query, body
- ✅ Return JSON, HTML, redirects
- ✅ Host Bridge integration working
- ✅ Middleware system functional
- ✅ Graceful shutdown
- ✅ Static file serving

### Performance
- ✅ Meets latency targets
- ✅ Meets throughput targets
- ✅ Memory usage acceptable
- ✅ No memory leaks

### Testing
- ✅ > 80% test coverage
- ✅ All unit tests passing
- ✅ All integration tests passing
- ✅ Performance benchmarks meet targets

### Documentation
- ✅ API documentation complete
- ✅ Deployment guide written
- ✅ Examples provided

---

## Timeline

- **Week 1 (Nov 20-26):** WASM Runtime Foundation
- **Week 2 (Nov 27-Dec 3):** HTTP Router with Axum
- **Week 3 (Dec 4-10):** Middleware & Server Lifecycle
- **Week 4 (Dec 11-17):** Integration & Performance

**Phase 4 Target Completion:** December 17, 2025

---

## Risks & Mitigations

### Risk: WASM Memory Management Complexity
**Mitigation:** Use proven patterns from wasmtime examples, extensive testing

### Risk: Performance Not Meeting Targets
**Mitigation:** Early benchmarking, profiling, connection pooling, caching

### Risk: Host Bridge Security Vulnerabilities
**Mitigation:** Strict allowlisting, input validation, sandboxing

### Risk: Integration with Frame Data ORM
**Mitigation:** Frame Data already tested and working, clear interfaces

---

## Next Phase Preview

**Phase 5: Frame UI (SSR/CSR Components)**
- Component system with SSR
- Islands architecture (selective hydration)
- Event handling
- Theme system
- Integration with Frame Server

---

**Document Status:** Initial Plan
**Last Updated:** November 20, 2025
**Next Review:** November 27, 2025 (End of Week 1)
