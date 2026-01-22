# Frame Framework Architecture

**Technical Deep Dive into Frame's Design and Implementation**

**Version:** 2.0

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Plugin System](#plugin-system)
4. [Compilation Pipeline](#compilation-pipeline)
5. [Runtime Model](#runtime-model)
6. [Host Bridge](#host-bridge)
7. [Security Model](#security-model)
8. [Performance Characteristics](#performance-characteristics)
9. [Extensibility](#extensibility)

---

## Overview

Frame 2.0 is built on a layered architecture where **Clean Language** source code compiles to **WebAssembly (WASM)** modules that run on any WASI-compatible host. The framework separates concerns into distinct layers while maintaining a unified type system and compilation pipeline.

### Version 2.0 Architecture Changes

| Aspect | v1 | v2 |
|--------|-----|-----|
| **Plugin Language** | Rust crates | Clean Language |
| **Plugin Format** | Compile-time Rust | WASM modules |
| **Plugin Execution** | Part of compiler build | Loaded at compile-time |
| **CLI Tool** | `frame` (Rust binary) | `cleen` (package manager) |
| **Plugin Location** | `frame-*` crates | `~/.cleen/plugins/` |

### Core Principles

- **WASM-First**: All application logic compiles to portable WebAssembly
- **Host Agnostic**: Same code runs on Node.js, Rust, Deno, browsers, mobile, and desktop
- **Type Safety**: End-to-end type checking from database to UI
- **Sandboxed Execution**: Controlled system access through the Host Bridge
- **Deterministic Compilation**: Same source always produces same output
- **Self-Hosting**: Framework plugins written in Clean Language itself

---

## System Architecture

### High-Level Layers (v2)

```
┌─────────────────────────────────────────────────────────────────┐
│                      Developer Code                              │
│                (Clean Language .cln files)                       │
│                                                                  │
│   import:                                                        │
│       frame.web                                                  │
│       frame.data                                                 │
│                                                                  │
│   server: port=3000                                              │
│       route: method="GET" path="/users"                          │
│           return User.all()                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CLEAN LANGUAGE COMPILER                       │
│                         (cln binary)                             │
│                                                                  │
│   1. Parse source files                                          │
│   2. Load plugins from ~/.cleen/plugins/                         │
│   3. Expand framework blocks via plugin WASM                     │
│   4. Semantic analysis and type checking                         │
│   5. Generate WebAssembly                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       FRAME PLUGINS                              │
│                  (Clean Language → WASM)                         │
│                                                                  │
│   ┌──────────────┬──────────────┬──────────────┬──────────────┐ │
│   │  frame.web   │  frame.data  │  frame.auth  │  frame.ui    │ │
│   │  (server,    │  (model,     │  (auth,      │  (component, │ │
│   │   route)     │   query)     │   protected) │   layout)    │ │
│   └──────────────┴──────────────┴──────────────┴──────────────┘ │
│                                                                  │
│   Location: ~/.cleen/plugins/<name>/<version>/                   │
│   Format: plugin.toml + plugin.wasm                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        HOST BRIDGE                               │
│                      (Rust crate)                                │
│                                                                  │
│   Provides runtime imports to WASM:                              │
│   ┌──────┬──────┬────────┬────────┬──────┬──────┬──────┐        │
│   │ HTTP │  DB  │ Crypto │  Time  │ Log  │  Env │  FS  │        │
│   └──────┴──────┴────────┴────────┴──────┴──────┴──────┘        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     HOST ENVIRONMENT                             │
│             (Node.js, Rust, Deno, Browser, Tauri)                │
└─────────────────────────────────────────────────────────────────┘
```

### Current Implementation Architecture

The following diagram shows the **actual working implementation** as of v0.5.x:

```
┌─────────────────────────────────────────────────────────────────┐
│                        clean-manager (cleen)                     │
│  ┌───────────────┐    ┌──────────────┐    ┌─────────────────┐  │
│  │   Discovery   │───▶│   Codegen    │───▶│  Compile WASM   │  │
│  │  (find files) │    │ (HTML→Clean) │    │   (cleanc)      │  │
│  └───────────────┘    └──────────────┘    └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      clean-server (runtime)                      │
│  ┌───────────────┐    ┌──────────────┐    ┌─────────────────┐  │
│  │  Axum HTTP    │───▶│    Router    │───▶│  WASM Instance  │  │
│  │   Server      │    │  (matchit)   │    │   (wasmtime)    │  │
│  └───────────────┘    └──────────────┘    └─────────────────┘  │
│                              │                     │            │
│                              ▼                     ▼            │
│                       ┌──────────────┐    ┌─────────────────┐  │
│                       │ host-bridge  │◀───│  Bridge Funcs   │  │
│                       │   (shared)   │    │  (_http_route)  │  │
│                       └──────────────┘    └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

#### Build Flow

1. **Discovery** (`clean-manager/src/core/discovery.rs`)
   - Scans `app/ui/pages/` → Page routes
   - Scans `app/ui/components/` → Component functions
   - Scans `app/server/api/` → API routes
   - Scans `app/server/models/` → Data models

2. **Code Generation** (`clean-manager/src/core/codegen.rs`)
   - Converts HTML pages to Clean string concatenation
   - Expands `<app-header>` → `__component_Header_render()`
   - Extracts route params from `[slug].html` → `_req_param("slug")`
   - Generates `start()` function with route registrations

3. **Compilation**
   - Clean compiler (`cleanc`) compiles generated `.cln` to WASM
   - WASM includes all handlers as exported functions

4. **Runtime** (`clean-server`)
   - Loads WASM module, calls `start()` to register routes
   - Axum handles HTTP requests, matches routes via `matchit`
   - Calls WASM handler function by index
   - Returns response string as HTML/JSON

#### Request Processing Flow

```
GET /blog/hello-world
        │
        ▼
┌─────────────────────────────────────────────┐
│ Axum matches route /blog/:slug              │
│ Router finds handler_index = 2              │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│ WASM function __route_handler_2() called    │
│   string slug = _req_param("slug")          │
│   // slug = "hello-world"                   │
│   string html = "<!DOCTYPE html>..."        │
│   html = html + __component_Header_render() │
│   html = html + "<h1>" + slug + "</h1>"     │
│   return html                               │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│ Axum returns Response with HTML body        │
│ Content-Type: text/html                     │
└─────────────────────────────────────────────┘
```

#### Currently Working Features

| Feature | Status | Implementation |
|---------|--------|----------------|
| HTTP Server | ✅ Working | Axum on port 3000 |
| Routing (GET/POST/PUT/DELETE) | ✅ Working | `_http_route(method, path, handler_id)` |
| Route Parameters | ✅ Working | `/blog/:slug` → `_req_param("slug")` |
| Request Headers | ✅ Working | `_req_header("name")` |
| Query Params | ✅ Working | `_req_query("key")` |
| Request Body | ✅ Working | `_req_body()` |
| Static Files | ✅ Working | `/public/*` from `public/` folder |
| SSR Components | ✅ Working | Function call expansion |
| Database | ✅ Working | `_db_query()`, `_db_execute()` |
| Session Auth | ✅ Working | `_auth_get_session()` |
| Protected Routes | ✅ Working | `_http_route_protected()` |

---

### Component Interaction

```
┌────────────────┐        ┌────────────────┐
│  cleen CLI     │───────▶│ Plugin Install │
│  (manager)     │        │ ~/.cleen/      │
└────────────────┘        └────────────────┘
                                 │
                                 ▼
┌────────────────┐        ┌────────────────┐
│   cln CLI      │───────▶│    Compiler    │
│  (compiler)    │        │  + Plugin Load │
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

## Plugin System

### Plugin Architecture (v2)

Frame 2.0 uses **Clean Language plugins** that compile to WASM. Plugins extend the compiler by transforming DSL blocks into standard Clean code at compile-time.

```
┌─────────────────────────────────────────────────────────────────┐
│                        SOURCE CODE                               │
│                                                                  │
│   import:                                                        │
│       frame.web                                                  │
│                                                                  │
│   server: port=3000                                              │
│       route: method="GET" path="/hello"                          │
│           return {"message": "Hello World"}                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        COMPILER                                  │
│                                                                  │
│   1. Parse import: blocks                                        │
│   2. Load plugins from ~/.cleen/plugins/                         │
│   3. Find framework blocks (server:, route:)                     │
│   4. Call plugin.expand_block("server", attrs, body)             │
│   5. Replace block with returned Clean code                      │
│   6. Continue compilation                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     EXPANDED CODE                                │
│                                                                  │
│   // Generated by frame.web                                      │
│   start()                                                        │
│       print("Server starting on localhost:3000")                 │
│       _http_route("GET", "/hello", (request) -> any              │
│           return {"message": "Hello World"}                      │
│       )                                                          │
│       _http_listen("localhost", 3000)                            │
└─────────────────────────────────────────────────────────────────┘
```

### Plugin Directory Structure

Plugins are installed to `~/.cleen/plugins/<name>/<version>/`:

```
~/.cleen/plugins/
├── frame.web/
│   └── 1.0.0/
│       ├── plugin.toml       # Plugin manifest
│       └── plugin.wasm       # Compiled plugin
├── frame.data/
│   └── 1.0.0/
│       ├── plugin.toml
│       └── plugin.wasm
├── frame.auth/
│   └── 1.0.0/
│       ├── plugin.toml
│       └── plugin.wasm
└── frame.ui/
    └── 1.0.0/
        ├── plugin.toml
        └── plugin.wasm
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"      # Optional
get_keywords = "get_keywords"    # Optional - for IDE support

[blocks]
handles = ["server", "route", "middleware"]

[paths]
owns = ["src/endpoints"]
auto_create = true
patterns = ["*.cln"]
implicit_import = true
```

### Folder Conventions

Plugins declare **folder ownership** to provide convention-over-configuration semantics. Files in owned folders automatically receive the plugin's context.

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROJECT STRUCTURE                             │
│                                                                  │
│   myapp/                                                         │
│   ├── frame.toml          # Declares: plugins = [...]            │
│   ├── src/                                                       │
│   │   ├── ui/             # Owned by frame.ui                    │
│   │   │   └── Button.cln  # Implicitly a component               │
│   │   ├── data/           # Owned by frame.data                  │
│   │   │   └── User.cln    # Implicitly a model                   │
│   │   ├── endpoints/      # Owned by frame.httpserver            │
│   │   │   └── users.cln   # Implicitly API routes                │
│   │   └── auth/           # Owned by frame.auth                  │
│   │       └── config.cln  # Auth configuration                   │
│   └── pages/              # Owned by frame.ui                    │
│       └── index.html      # Page with UI components              │
└─────────────────────────────────────────────────────────────────┘
```

| Plugin | Owned Folders | File Types | Purpose |
|--------|---------------|------------|---------|
| `frame.ui` | `src/ui/`, `pages/` | `.cln`, `.html` | UI components, pages |
| `frame.data` | `src/data/` | `.cln` | Data models, queries |
| `frame.httpserver` | `src/endpoints/` | `.cln` | API routes, middleware |
| `frame.auth` | `src/auth/` | `.cln` | Auth configuration |
| `frame.canvas` | `src/canvas/` | `.cln` | Canvas rendering, animation |

When `implicit_import = true`, files in owned folders don't need explicit import statements:

```clean
// src/data/User.cln
// No import needed - frame.data is implicit

model: name="User"
    string email
    string name
```

### Plugin API

Every plugin must export an `expand_block` function:

```clean
expand_block(block_name: string, attributes: string, body: string) -> string
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `block_name` | string | The DSL block identifier (e.g., "server") |
| `attributes` | string | JSON object of block attributes |
| `body` | string | Raw content inside the block |
| **Returns** | string | Clean Language source code |

### Official Frame Plugins

| Plugin | Blocks | Purpose |
|--------|--------|---------|
| `frame.web` | server, route, middleware | Web server and routing |
| `frame.data` | model, query, transaction | ORM and database |
| `frame.auth` | auth, protected, login | Authentication |
| `frame.ui` | component, layout, page | UI components and SSR |
| `frame.canvas` | canvasScene, draw, onFrame | Canvas rendering and animation |

---

## Compilation Pipeline

### Clean to WASM Flow (v2)

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
│ Import Detection│  → Extract plugin requirements (NEW in v2)
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Plugin Loading  │  → Load WASM plugins from ~/.cleen/plugins/ (NEW in v2)
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Block Expansion │  → Transform DSL blocks to Clean code (NEW in v2)
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
│ MIR Generation  │  → Mid-level Intermediate Representation
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
    .wasm Binary
```

### Compiler Stages

#### 1. Parsing

- **Grammar**: Pest-based parser with error recovery
- **Output**: Abstract Syntax Tree (AST)
- **Features**: Indentation-based blocks, type annotations, function signatures

#### 2. Plugin Expansion (NEW in v2)

- **Input**: AST with framework blocks (server:, model:, etc.)
- **Process**: Call plugin's `expand_block` function via WASM runtime
- **Output**: AST with expanded Clean code

#### 3. Semantic Analysis

- **Type Checking**: Strong static typing with inference
- **Scope Management**: Variable and function resolution
- **Validation**: Class inheritance, method signatures

#### 4. Code Generation

- **Target**: WebAssembly (WASM) using `wasm-encoder`
- **Memory**: Linear memory with string pooling
- **Instructions**: Type-specific WASM instruction generation

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

---

## Host Bridge

### Architecture

The Host Bridge is the **only** interface between WASM modules and system resources. It provides a sandboxed, type-safe API for system operations.

```
┌──────────────────────────────────────────┐
│          WASM Module (Clean)             │
│                                          │
│  User.find(id)                           │
│  ↓ Calls _db_query_one                   │
└──────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│           Host Bridge Interface          │
│                                          │
│  _db_query_one(sql, params)              │
└──────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│        Host Implementation               │
│      (Node.js, Rust, Deno, etc.)         │
│                                          │
│  - Execute SQL via pg/mysql driver       │
│  - Apply connection pooling              │
│  - Return result                         │
└──────────────────────────────────────────┘
```

### Bridge Functions

Host Bridge functions are prefixed with `_` to indicate they are runtime imports.

| Namespace | Functions | Purpose |
|-----------|-----------|---------|
| `_http_*` | listen, route, middleware, request | HTTP server and client |
| `_db_*` | query, query_one, insert, update, delete, transaction | Database operations |
| `_auth_*` | create_token, verify_token, hash_password, verify_password | Authentication |
| `_file_*` | read, write, exists, delete | Filesystem (optional) |
| `_env_*` | get, set | Environment variables |
| `_log_*` | info, warn, error | Structured logging |
| `_crypto_*` | random, hash, sign, verify | Cryptographic operations |
| `_time_*` | now, sleep | Time operations |

### Security Guarantees

- **Sandboxed**: WASM modules cannot access system resources directly
- **Allowlisted**: Each bridge function must be explicitly implemented by host
- **Validated**: All bridge calls are type-checked at compile time
- **Auditable**: Bridge calls are logged for security monitoring
- **Isolated**: Each request runs in a separate execution context

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
│      Plugin Sandbox (Compile-time)         │  ← WASM isolation (NEW in v2)
└────────────────────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│         WASM Sandbox (Runtime)             │  ← Memory isolation
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

### Plugin Security (NEW in v2)

Plugins run in a WASM sandbox during compilation:

- **No filesystem access** (except reading source)
- **No network access**
- **No system calls**
- **Memory isolated** per plugin
- **Deterministic** execution

Generated code from plugins goes through full type checking, preventing plugins from bypassing the type system.

---

## Performance Characteristics

### Compilation

- **Incremental**: Only recompile changed modules
- **Parallel**: Utilize multiple CPU cores
- **Caching**: Cache compiled WASM modules
- **Plugin overhead**: < 10ms per framework block expansion

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

---

## Extensibility

### Plugin Development (v2)

Creating plugins in Clean Language:

```bash
# 1. Create plugin project
cleen plugin create my-plugin

# 2. Edit src/main.cln
# Implement expand_block function

# 3. Build plugin
cleen plugin build

# 4. Install locally
cleen plugin install ./

# 5. Use in project
# import:
#     my-plugin
```

### Plugin Project Structure

```
my-plugin/
├── plugin.toml           # Manifest
├── src/
│   └── main.cln          # Plugin source
├── tests/
│   └── test_expand.cln   # Plugin tests
└── README.md             # Documentation
```

### Example Plugin

```clean
// my-plugin/src/main.cln

expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "myblock"
        return expand_myblock(attributes, body)
    return body

expand_myblock(attrs: string, body: string) -> string
    return "// Generated by my-plugin\n" + body
```

---

## Component Summary (v2)

### What's Included

| Component | Location | Purpose |
|-----------|----------|---------|
| **Host Bridge** | `host-bridge/` | Runtime imports for WASM modules |
| **frame.web Plugin** | `plugins/frame.web/` | Server and routing DSL |
| **frame.data Plugin** | `plugins/frame.data/` | ORM and database DSL |
| **frame.auth Plugin** | `plugins/frame.auth/` | Authentication DSL |
| **frame.ui Plugin** | `plugins/frame.ui/` | UI components DSL |

### What's External

| Component | Binary | Purpose |
|-----------|--------|---------|
| **Clean Compiler** | `cln` | Compiles .cln to .wasm, loads plugins |
| **Package Manager** | `cleen` | Plugin installation, compiler management |

---

## Summary

Frame 2.0's architecture provides:

1. **Portability**: Same code runs everywhere via WASM
2. **Safety**: Sandboxed execution with typed interfaces
3. **Performance**: Compiled code with predictable behavior
4. **Simplicity**: One language, one type system, one compiler
5. **Self-Hosting**: Framework plugins written in Clean Language
6. **Extensibility**: Plugin system for custom DSL blocks

The architecture is designed for both human developers and AI-assisted development, with deterministic compilation, typed interfaces, and clear module boundaries.

---

For more details on specific components:
- [Compiler Plugins Specification](./specification/10_compiler_plugins.md)
- [Frame Plugins Specification](./specification/07_frame_plugins.md)
- [Frame Server Specification](./specification/03_frame_server.md)
- [Frame Data Specification](./specification/04_frame_data.md)
- [Frame UI Specification](./specification/05_frame_ui.md)
- [Host Bridge Contracts](./specification/frame_bridge_contracts.md)

---

**End of Document (v2)**
