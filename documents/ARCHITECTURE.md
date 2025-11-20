# Frame Framework Architecture

**Technical Deep Dive into Frame's Design and Implementation**

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Compilation Pipeline](#compilation-pipeline)
4. [Runtime Model](#runtime-model)
5. [Host Bridge](#host-bridge)
6. [Module Architecture](#module-architecture)
7. [Security Model](#security-model)
8. [Performance Characteristics](#performance-characteristics)
9. [Extensibility](#extensibility)

---

## Overview

Frame is built on a layered architecture where **Clean Language** source code compiles to **WebAssembly (WASM)** modules that run on any WASI-compatible host. The framework separates concerns into distinct layers while maintaining a unified type system and compilation pipeline.

### Core Principles

- **WASM-First**: All application logic compiles to portable WebAssembly
- **Host Agnostic**: Same code runs on Node.js, Rust, Deno, browsers, mobile, and desktop
- **Type Safety**: End-to-end type checking from database to UI
- **Sandboxed Execution**: Controlled system access through the Host Bridge
- **Deterministic Compilation**: Same source always produces same output

---

## System Architecture

### High-Level Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Developer Code                        │
│              (Clean Language .cln files)                 │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   Frame Compiler                         │
│  ┌──────────┬──────────┬──────────┬──────────┐          │
│  │  Parser  │ Semantic │ Type     │   Code   │          │
│  │          │ Analysis │ Checker  │   Gen    │          │
│  └──────────┴──────────┴──────────┴──────────┘          │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                WebAssembly Modules                       │
│     ┌──────────────┬──────────────┬─────────────┐       │
│     │ server.wasm  │  ui.wasm     │ manifest.   │       │
│     │ (Backend)    │  (Frontend)  │ json        │       │
│     └──────────────┴──────────────┴─────────────┘       │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   Frame Runtime                          │
│  ┌──────────┬──────────┬──────────┬──────────┐          │
│  │  WASM    │  Router  │   SSR    │  Host    │          │
│  │  Loader  │          │  Engine  │  Bridge  │          │
│  └──────────┴──────────┴──────────┴──────────┘          │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    Host Environment                      │
│   (Node.js, Rust, Deno, Browser, Tauri, Capacitor)      │
└─────────────────────────────────────────────────────────┘
```

### Component Interaction

```
┌────────────────┐        ┌────────────────┐
│   Frame CLI    │───────▶│    Compiler    │
└────────────────┘        └────────────────┘
                                 │
                                 ▼
┌────────────────────────────────────────────┐
│              WASM Modules                  │
│  ┌──────────────┬──────────────────────┐   │
│  │ Backend      │ Frontend             │   │
│  │ - API Routes │ - UI Components      │   │
│  │ - Auth       │ - SSR/Hydration      │   │
│  │ - Data Layer │ - Event Handlers     │   │
│  └──────────────┴──────────────────────┘   │
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│           Host Bridge Interface            │
│  ┌──────┬──────┬────────┬────────┬──────┐  │
│  │ HTTP │  DB  │ Crypto │  Time  │ Log  │  │
│  └──────┴──────┴────────┴────────┴──────┘  │
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│          System Resources                  │
│  (Network, Database, Filesystem, etc.)     │
└────────────────────────────────────────────┘
```

---

## Compilation Pipeline

### Clean to WASM Flow

```
.cln Source Files
      │
      ▼
┌─────────────────┐
│ Lexical Analysis│  → Tokenization
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Syntax Parsing  │  → AST Generation (using Pest grammar)
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Semantic        │  → Type checking, scope resolution
│ Analysis        │    Variable binding, function resolution
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Type Inference  │  → Constraint solving, generic instantiation
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ CIR Generation  │  → Clean Intermediate Representation
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Optimization    │  → Dead code elimination, inlining
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ WASM Code Gen   │  → Generate WebAssembly instructions
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Module Linking  │  → Combine modules, emit manifest
└─────────────────┘
      │
      ▼
    .wasm Binary + manifest.json
```

### Compiler Stages

#### 1. Parsing

- **Grammar**: Pest-based parser with error recovery
- **Output**: Abstract Syntax Tree (AST)
- **Features**: Indentation-based blocks, type annotations, function signatures

#### 2. Semantic Analysis

- **Type Checking**: Strong static typing with inference
- **Scope Management**: Variable and function resolution
- **Validation**: Class inheritance, method signatures

#### 3. Code Generation

- **Target**: WebAssembly (WASM) using `wasm-encoder`
- **Memory**: Linear memory with string pooling
- **Instructions**: Type-specific WASM instruction generation

#### 4. Module Output

```
dist/
├── server.wasm           # Backend logic (API, Auth, Data)
├── ui.wasm               # Frontend logic (components, hydration)
├── manifest.json         # Route mappings and metadata
└── manifest.islands.json # Client hydration manifest
```

---

## Runtime Model

### Request Lifecycle

```
HTTP Request
    │
    ▼
┌────────────────────┐
│   Host Receiver    │  (Node.js, Rust, etc.)
└────────────────────┘
    │
    ▼
┌────────────────────┐
│  Frame Router      │  Maps path → Clean function
└────────────────────┘
    │
    ▼
┌────────────────────┐
│  Request Sandbox   │  Isolated WASM execution context
└────────────────────┘
    │
    ▼
┌────────────────────┐
│  WASM Execution    │  Run Clean API handler
└────────────────────┘
    │
    ▼
┌────────────────────┐
│  Host Bridge Calls │  (DB, HTTP, Crypto, etc.)
└────────────────────┘
    │
    ▼
┌────────────────────┐
│  Response Builder  │  Serialize to JSON/HTML
└────────────────────┘
    │
    ▼
HTTP Response
```

### WASM Execution Model

**Module Loading**:
- WASM modules are lazy-loaded and cached
- Streaming compilation for large modules
- Per-request execution contexts with isolated memory

**Execution Context**:
```javascript
// Example: Node.js runtime adapter
const wasm = await WebAssembly.compileStreaming(
    fs.createReadStream('dist/server.wasm')
);

const instance = await WebAssembly.instantiate(wasm, {
    host: hostBridgeImplementation,
    wasi_snapshot_preview1: wasi.wasiImport
});

// Each request gets isolated execution
const result = instance.exports.handleRequest(requestData);
```

### File-Based Routing

Routes are automatically generated from file structure:

```
app/api/users.cln       → GET/POST  /api/users
app/api/users/[id].cln  → GET/PUT   /api/users/:id
app/pages/index.cln     → GET       /
app/pages/about.cln     → GET       /about
```

**Route Resolution**:
1. Parse incoming request path
2. Map to file-based route
3. Extract dynamic parameters
4. Deserialize query/body to typed Clean input
5. Execute Clean function
6. Serialize output to JSON/HTML

---

## Host Bridge

### Architecture

The Host Bridge is the **only** interface between WASM modules and system resources. It provides a sandboxed, type-safe API for system operations.

```
┌──────────────────────────────────────────┐
│          WASM Module (Clean)             │
│                                          │
│  User.find:                              │
│      where: active == true               │
│                                          │
│  ↓ Calls host:db.query                │
└──────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│           Host Bridge Interface          │
│                                          │
│  {                                       │
│    "fn": "host:db.query",              │
│    "args": {                             │
│      "sql": "SELECT * FROM users...",    │
│      "params": [true]                    │
│    }                                     │
│  }                                       │
└──────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│        Host Implementation               │
│      (Node.js, Rust, Deno, etc.)         │
│                                          │
│  - Execute SQL via pg/mysql driver       │
│  - Apply connection pooling              │
│  - Return rows                           │
└──────────────────────────────────────────┘
```

### Bridge Namespaces

| Namespace | Functions | Purpose |
|-----------|-----------|---------|
| `host:http` | `request`, `respond`, `redirect` | HTTP operations |
| `host:db` | `query`, `tx`, `prepare` | Database access |
| `host:env` | `get`, `list` | Environment variables |
| `host:log` | `info`, `warn`, `error` | Structured logging |
| `host:time` | `now`, `sleep` | Time operations |
| `host:crypto` | `random`, `hash`, `verify`, `sign` | Cryptographic operations |
| `host:fs` | `read`, `write`, `list` | Filesystem (desktop/CLI only) |
| `host:sys` | `exit`, `platform` | System information |

### Message Format

**Request**:
```json
{
  "fn": "host:db.query",
  "args": {
    "sql": "SELECT * FROM users WHERE id=$1",
    "params": [42]
  }
}
```

**Response (Success)**:
```json
{
  "ok": true,
  "data": {
    "rows": [
      {"id": 42, "name": "Alice", "email": "alice@example.com"}
    ]
  }
}
```

**Response (Error)**:
```json
{
  "ok": false,
  "err": {
    "code": "DB_ERROR",
    "message": "Connection timeout",
    "details": {"timeout": 5000}
  }
}
```

### Security Guarantees

- **Sandboxed**: WASM modules cannot access system resources directly
- **Allowlisted**: Each bridge function must be explicitly implemented by host
- **Validated**: All bridge calls are type-checked at compile time
- **Auditable**: Bridge calls are logged for security monitoring
- **Isolated**: Each request runs in a separate execution context

---

## Module Architecture

### Frame CLI

```
frame-cli/
├── commands/
│   ├── new.rs          # Project scaffolding
│   ├── serve.rs        # Development server
│   ├── build.rs        # Production build
│   └── db.rs           # Database commands
├── compiler/
│   └── adapter.rs      # Clean compiler interface
├── runtime/
│   └── host.rs         # Host bridge initialization
└── main.rs             # Command dispatcher
```

**Responsibilities**:
- Parse CLI arguments and flags
- Invoke Clean compiler with appropriate options
- Manage development server with file watching
- Coordinate database migrations
- Scaffold new projects and plugins

### Frame Server

```
frame-server/
├── router/
│   ├── mapper.rs       # File-based route mapping
│   └── params.rs       # Parameter extraction
├── wasm/
│   ├── loader.rs       # WASM module loading
│   ├── cache.rs        # Module caching
│   └── sandbox.rs      # Execution isolation
├── ssr/
│   └── renderer.rs     # Server-side rendering
└── bridge/
    └── implementation/ # Host bridge adapters
```

**Responsibilities**:
- Load and cache WASM modules
- Route incoming HTTP requests
- Execute WASM functions in isolated contexts
- Implement Host Bridge functions
- Render server-side HTML

### Frame Data (ORM)

```
frame-data/
├── models/
│   ├── parser.rs       # Parse `data` blocks
│   └── validator.rs    # Validate models
├── query/
│   ├── builder.rs      # Query DSL compilation
│   └── executor.rs     # SQL generation
├── migrations/
│   ├── generator.rs    # Generate migrations from diffs
│   └── runner.rs       # Apply migrations
└── bridge/
    └── db.rs           # Database bridge implementation
```

**Responsibilities**:
- Parse Clean `data` block syntax
- Validate models and relationships
- Generate SQL from declarative queries
- Create and run database migrations
- Interface with database drivers via Host Bridge

### Frame UI

```
frame-ui/
├── components/
│   ├── parser.rs       # Parse `component` blocks
│   └── validator.rs    # Validate component props
├── ssr/
│   ├── renderer.rs     # Server-side rendering
│   └── escaping.rs     # HTML escaping
├── hydration/
│   ├── manifest.rs     # Islands manifest generation
│   └── loader.js       # Client-side loader
└── csdr/
    └── dom.rs          # Client-side rendering (optional)
```

**Responsibilities**:
- Parse Clean UI components
- Render components to HTML (SSR)
- Generate islands manifest for hydration
- Provide client-side rendering (optional)
- Manage component lifecycle

### Frame Auth

```
frame-auth/
├── session/
│   ├── manager.rs      # Session lifecycle
│   └── store.rs        # Session storage
├── jwt/
│   ├── encoder.rs      # JWT signing
│   └── decoder.rs      # JWT verification
├── roles/
│   └── guard.rs        # Role-based access control
└── middleware/
    └── auth.rs         # Authentication middleware
```

**Responsibilities**:
- Manage user sessions (cookies)
- Sign and verify JWTs
- Enforce role-based permissions
- Provide authentication middleware

### Frame Plugins

```
frame-plugins/
├── loader/
│   └── discovery.rs    # Plugin discovery
├── hooks/
│   ├── cli.rs          # CLI command hooks
│   ├── ui.rs           # UI component hooks
│   ├── server.rs       # Server route hooks
│   └── data.rs         # ORM lifecycle hooks
└── sandbox/
    └── runtime.rs      # Sandboxed plugin execution
```

**Responsibilities**:
- Discover and load plugins
- Execute plugin hooks at appropriate lifecycle stages
- Sandbox plugin execution
- Validate plugin permissions

---

## Security Model

### Threat Model

Frame applications are designed to defend against:

- **Code Injection**: WASM sandboxing prevents arbitrary code execution
- **Data Leakage**: Type system prevents unauthorized data access
- **CSRF**: Built-in CSRF token validation
- **XSS**: Automatic HTML escaping in templates
- **SQL Injection**: Parameterized queries only
- **Privilege Escalation**: Role-based access control

### Defense Layers

```
┌────────────────────────────────────────────┐
│          Application Code (Clean)          │  ← Type-safe, validated
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│         WASM Sandbox (Isolation)           │  ← Memory isolation
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│    Host Bridge (Controlled Access)         │  ← Allowlisted operations
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│      System Resources (Protected)          │  ← OS-level security
└────────────────────────────────────────────┘
```

### Security Features

**Compile-Time**:
- Type safety eliminates entire classes of bugs
- SQL injection prevention via parameterized queries
- XSS prevention via automatic escaping

**Runtime**:
- WASM sandboxing isolates application code
- Per-request execution contexts prevent state leakage
- Host Bridge enforces access control

**Application**:
- Built-in CSRF protection
- Secure session management (HTTP-only, SameSite cookies)
- JWT signing and verification
- Role-based access control

---

## Performance Characteristics

### Compilation

- **Incremental**: Only recompile changed modules
- **Parallel**: Utilize multiple CPU cores
- **Caching**: Cache compiled WASM modules
- **Typical Times**:
  - Small app (< 1000 LOC): < 1 second
  - Medium app (< 10k LOC): < 5 seconds
  - Large app (> 10k LOC): < 30 seconds

### Runtime

**Cold Start**:
- WASM module loading: 10-50ms (depends on module size)
- First request overhead: 20-100ms
- Subsequent requests: 1-5ms (cached)

**Request Latency** (p95):
- Simple API endpoint: < 5ms
- SSR page render: < 50ms
- Database query: < 20ms (local)
- Full page with auth + data: < 100ms

**Memory**:
- Minimal baseline: ~10MB per process
- Per-request overhead: ~100KB
- Scales linearly with concurrent requests

**Throughput**:
- Simple endpoints: > 10k req/sec (single core)
- SSR pages: > 1k req/sec (single core)
- Database queries: Limited by database, not runtime

### Optimization Strategies

**Compilation**:
- Use `--release` builds for production (optimizations enabled)
- Enable WASM SIMD for numeric computations
- Leverage dead code elimination

**Runtime**:
- Cache WASM modules aggressively
- Use connection pooling for databases
- Enable HTTP caching for GET routes
- Implement pagination for large result sets

**Frontend**:
- Default to SSR for fast first paint
- Hydrate only interactive components
- Lazy-load non-critical WASM bundles
- Use CDN for static assets

---

## Extensibility

### Plugin Architecture

Plugins extend Frame without modifying the core:

```
Plugin Structure
├── plugin.cln           # Manifest and hook registrations
├── ui/                  # UI components
├── server/              # API routes
├── cli/                 # CLI commands
└── data/                # ORM hooks
```

### Hook Points

**CLI Hooks**:
- Add custom commands
- Extend build process
- Implement custom generators

**UI Hooks**:
- Register custom tags/components
- Add layout templates
- Implement theming extensions

**Server Hooks**:
- Add middleware
- Register routes
- Implement custom handlers

**Data Hooks**:
- React to model lifecycle events
- Transform queries
- Implement custom validators

### Extension Example

```clean
plugin Charts
    meta:
        name = "charts"
        version = "1.0.0"

    hooks:
        ui: registerTags
        cli: registerCLI

functions:
    registerTags(tags)
        tags.register("charts-line", "plugins/charts/ui/LineChart.cln")
        tags.register("charts-bar", "plugins/charts/ui/BarChart.cln")

    registerCLI(cli)
        cli.command("charts:add")
            .describe("Add chart component to page")
            .action(addChart)
```

---

## Platform Adaptations

### Node.js Host

```javascript
import { WASI } from 'wasi';
import http from 'http';

const wasi = new WASI({ env: process.env });
const wasm = await WebAssembly.instantiateStreaming(
    fetch('server.wasm'),
    {
        wasi_snapshot_preview1: wasi.wasiImport,
        host: nodeBridgeImplementation
    }
);

http.createServer((req, res) => {
    const result = wasm.instance.exports.handleRequest(
        encodeRequest(req)
    );
    decodeResponse(result, res);
}).listen(8080);
```

### Rust Host (Tauri Desktop)

```rust
use wasmtime::*;

let engine = Engine::default();
let module = Module::from_file(&engine, "server.wasm")?;

let mut store = Store::new(&engine, ());
let instance = Instance::new(&mut store, &module, &[])?;

// Handle window events
let handle_event = instance.get_typed_func::<(i32,), i32>(&mut store, "handle_event")?;
```

### Browser Client

```javascript
const bridge = {
    dom: {
        create: tag => document.createElement(tag),
        setAttr: (el, k, v) => el.setAttribute(k, v),
        append: (parent, child) => parent.appendChild(child)
    }
};

const { instance } = await WebAssembly.instantiateStreaming(
    fetch('/ui.wasm'),
    { bridge }
);

instance.exports.boot('app', {});
```

---

## Summary

Frame's architecture provides:

1. **Portability**: Same code runs everywhere via WASM
2. **Safety**: Sandboxed execution with typed interfaces
3. **Performance**: Compiled code with predictable behavior
4. **Simplicity**: One language, one type system, one compiler
5. **Extensibility**: Plugin system for custom functionality

The architecture is designed for both human developers and AI-assisted development, with deterministic compilation, typed interfaces, and clear module boundaries.

---

For more details on specific components:
- [Frame CLI Specification](./specification/02_frame_cli.md)
- [Frame Server Specification](./specification/03_frame_server.md)
- [Frame Data Specification](./specification/04_frame_data.md)
- [Frame UI Specification](./specification/05_frame_ui.md)
- [Host Bridge Contracts](./specification/frame_bridge_contracts.md)
