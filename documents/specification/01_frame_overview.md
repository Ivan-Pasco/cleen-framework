# Frame Framework Overview (01)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.0
**Location:** `/docs/specification/01_frame_overview.md`

---

## 1. Introduction

Frame is the official full-stack framework for **Clean Language**, created to unify frontend, backend, and data logic under one compiler, one runtime, and one mental model.

The framework compiles entirely to **WebAssembly (WASM)**, which means that applications built with Frame can run seamlessly on any host—Node.js, Rust, Deno, Tauri, or future WASI environments.

Frame embodies the **Clean Language philosophy**: simple, declarative, and transparent code that's easy to reason about and verify.

### Version 2.0 Changes

**Frame 2.0 introduces a new plugin architecture:**

| Aspect | v1 | v2 |
|--------|-----|-----|
| **Plugin Language** | Rust crates | Clean Language |
| **Plugin Format** | Compile-time Rust | WASM modules |
| **Installation** | Built into framework | External via `cleen` |
| **Location** | `frame-*` crates | `~/.cleen/plugins/` |

---

## 2. Core Philosophy

| Principle | Description |
|-----------|-------------|
| **Simplicity** | Minimal syntax, no decorators, no boilerplate. One way to do things clearly. |
| **Type Safety** | Every variable, property, and API is typed from source to output. |
| **Transparency** | No hidden magic—what you read is what runs. |
| **Performance** | WASM runtime ensures predictable speed across environments. |
| **Portability** | One binary can run on many hosts with the same behavior. |
| **Security** | Sandboxed execution, clear Host Bridge boundaries, no unsafe access. |
| **Self-Hosting** | Framework plugins written in Clean Language itself. |

---

## 3. Architecture Overview (v2)

Frame 2.0 is divided into three main components:

```
┌────────────────────────────────────────────────────────────────────┐
│                     CLEAN LANGUAGE COMPILER                         │
│                         (cln binary)                                │
│                                                                     │
│   - Parses .cln source files                                       │
│   - Loads plugins based on import: blocks                          │
│   - Expands framework blocks via plugin WASM                       │
│   - Compiles to WebAssembly                                        │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                      FRAME PLUGINS                                  │
│               (Clean Language → WASM)                               │
│                                                                     │
│   ┌──────────────┬──────────────┬──────────────┬──────────────┐   │
│   │  frame.web   │  frame.data  │  frame.auth  │  frame.ui    │   │
│   │  (server,    │  (model,     │  (auth,      │  (component, │   │
│   │   route)     │   query)     │   protected) │   layout)    │   │
│   └──────────────┴──────────────┴──────────────┴──────────────┘   │
│                                                                     │
│   Location: ~/.cleen/plugins/<name>/<version>/                     │
│   Format: plugin.toml + plugin.wasm                                │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                       HOST BRIDGE                                   │
│                    (Rust crate)                                     │
│                                                                     │
│   Provides runtime imports to WASM:                                │
│   ┌──────┬──────┬────────┬────────┬──────┬──────┬──────┐          │
│   │ HTTP │  DB  │ Crypto │  Time  │ Log  │  Env │  FS  │          │
│   └──────┴──────┴────────┴────────┴──────┴──────┴──────┘          │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                    HOST ENVIRONMENT                                 │
│            (Node.js, Rust, Deno, Browser, Tauri)                   │
└────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **Clean Compiler** | Parse source, load plugins, expand blocks, emit WASM |
| **Frame Plugins** | Transform DSL blocks into Clean code at compile-time |
| **Host Bridge** | Provide system access (HTTP, DB, etc.) at runtime |
| **Host Environment** | Execute WASM and implement Host Bridge |

---

## 4. Plugin System (v2)

### How Plugins Work

1. Developer writes Clean code with framework blocks:
   ```clean
   import:
       frame.web

   server: port=3000
       route: method="GET" path="/hello"
           return {"message": "Hello World"}
   ```

2. Compiler detects `import: frame.web` and loads the plugin

3. Plugin's `expand_block` function transforms each block:
   ```
   Input:  server: port=3000 ...
   Output: Generated Clean code with _http_listen(), _http_route(), etc.
   ```

4. Expanded code is compiled to WASM

5. At runtime, Host Bridge provides the `_http_*` functions

### Plugin Installation

Plugins are managed via the `cleen` CLI:

```bash
cleen plugin install frame.web          # Install from registry
cleen plugin install frame.data@1.0.0   # Specific version
cleen plugin list                       # Show installed
cleen plugin create my-plugin           # Create new plugin
```

Plugins are stored in `~/.cleen/plugins/<name>/<version>/`.

### Writing Plugins

Plugins are Clean Language files that export `expand_block`:

```clean
// my-plugin/src/main.cln

expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "myblock"
        return expand_myblock(attributes, body)
    return body

expand_myblock(attrs: string, body: string) -> string
    return "// Generated by my-plugin\n" + body
```

See [10_compiler_plugins.md](./10_compiler_plugins.md) for full details.

---

## 5. Clean-to-WASM Pipeline

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
│ Syntax Parsing  │  → AST Generation
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Plugin Expansion│  → Framework blocks → Clean code (NEW in v2)
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ Semantic        │  → Type checking, scope resolution
│ Analysis        │
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ MIR Generation  │  → Mid-level Intermediate Representation
└─────────────────┘
      │
      ▼
┌─────────────────┐
│ WASM Code Gen   │  → WebAssembly binary
└─────────────────┘
      │
      ▼
    .wasm Binary
```

---

## 6. Project Structure (v2)

Frame uses a clean architecture-based folder structure with clear separation of concerns:

```
myapp/
├── app.cln                 # Main entry point with imports
├── project.toml            # Project configuration
│
└── app/
    ├── pages/              # SSR pages (frame.ui)
    │   ├── index.html      # HTML template with {{ }} and cl-* directives
    │   ├── index.cln       # Companion data loader (load/guard functions)
    │   ├── about.html
    │   └── blog/
    │       ├── [slug].html
    │       └── [slug].cln
    │
    ├── components/         # Reusable UI components (frame.ui)
    │   ├── Header.cln
    │   └── Footer.cln
    │
    ├── layouts/            # Page layout wrappers (frame.ui)
    │   └── main.html
    │
    ├── api/                # HTTP endpoints (frame.httpserver)
    │   ├── users.cln
    │   └── posts.cln
    │
    ├── data/               # Data models/ORM (frame.data)
    │   ├── User.cln
    │   ├── Post.cln
    │   └── migrations/     # Schema migrations
    │
    ├── auth/               # Auth configuration (frame.auth)
    │   └── config.cln
    │
    ├── canvas/             # Canvas applications (frame.canvas)
    │
    └── public/             # Static assets (served as-is)
        └── css/
            └── style.css

├── dist/                   # Compiled WASM output
```

### Folder Ownership by Plugin

| Plugin | Owned Folders |
|--------|---------------|
| `frame.ui` | `app/pages/`, `app/components/`, `app/layouts/` |
| `frame.httpserver` | `app/api/` |
| `frame.data` | `app/data/` |
| `frame.auth` | `app/auth/` |
| `frame.canvas` | `app/canvas/` |

### Plugin Auto-Detection (v2.1)

The compiler automatically detects and loads plugins based on file paths. Files placed in standard folders don't need explicit `plugins:` declarations:

| Folder Pattern | Auto-Detected Plugins |
|----------------|----------------------|
| `/api/`, `/endpoints/` | `frame.httpserver` + `frame.data` + `frame.auth` |
| `/data/` | `frame.data` |
| `/auth/` | `frame.auth` |
| `/canvas/` | `frame.canvas` |
| `/pages/`, `/components/`, `/layouts/` | `frame.ui` |

This means a file at `app/api/users.cln` automatically has access to HTTP, database, and auth bridge functions without any imports.

### Folder Creation

Folders are auto-created by the CLI when:
1. Creating a new project: `cleen project create myapp --plugins=...`
2. Adding a plugin: `cleen plugin add frame.data`

See [02_frame_cli.md](./02_frame_cli.md) for details on scaffolding.

---

## 7. Host Bridge Summary

The **Host Bridge** connects WASM modules with system capabilities:

| Namespace | Functions | Purpose |
|-----------|-----------|---------|
| `_http_*` | listen, route, middleware, request | HTTP server and client |
| `_db_*` | query, insert, update, delete, transaction | Database operations |
| `_auth_*` | create_token, verify_token, hash_password | Authentication |
| `_file_*` | read, write, exists, delete | Filesystem (optional) |
| `_env_*` | get, set | Environment variables |
| `_log_*` | info, warn, error | Structured logging |
| `_crypto_*` | random, hash, sign, verify | Cryptographic operations |
| `_time_*` | now, sleep | Time operations |

Host Bridge functions are prefixed with `_` to indicate they are runtime imports.

---

## 8. Framework Plugins

### Official Plugins

| Plugin | Blocks | Purpose |
|--------|--------|---------|
| `frame.web` | server, route, middleware | Web server and routing |
| `frame.data` | model, query, transaction | ORM and database |
| `frame.auth` | auth, protected, login | Authentication |
| `frame.ui` | component, layout, page | UI components and SSR |

### Example Usage

```clean
import:
    frame.web
    frame.data
    frame.auth

// Define data model
model: name="User" table="users"
    string email
    string password_hash
    boolean admin = false

// Configure authentication
auth: strategy="jwt" secret="$JWT_SECRET"

// Create web server
server: port=3000

    // Public route
    route: method="POST" path="/login"
        user = User.find_by_email(request.body("email"))
        if user == null
            return {"error": "Invalid credentials"}
        if !_auth_verify_password(request.body("password"), user.password_hash)
            return {"error": "Invalid credentials"}
        token = _auth_create_token(user, _auth_secret)
        return {"token": token}

    // Protected routes
    protected:
        route: method="GET" path="/profile"
            return request.user

        route: method="GET" path="/admin"
            if !request.user.admin
                return {"error": "Forbidden"}
            return {"message": "Admin area"}
```

---

## 9. Migration from v1

### Removed Components

The following Rust crates are replaced by Clean Language plugins:

| Removed Crate | Replacement |
|---------------|-------------|
| `frame-cli` | `cleen` (package manager) |
| `frame-server` | `frame.web` plugin + Host Bridge |
| `frame-data` | `frame.data` plugin + Host Bridge |
| `frame-ui` | `frame.ui` plugin |
| `frame-auth` | `frame.auth` plugin + Host Bridge |
| `frame-plugins` | Deleted (replaced by compiler plugins) |
| `frame-compiler-plugins` | Deleted (replaced by Clean plugins) |

### Kept Components

| Component | Status |
|-----------|--------|
| `host-bridge` | **KEPT** - Provides runtime imports |

---

## 10. Development Workflow

### Development

```bash
# Install plugins
cleen plugin install frame.web frame.data

# Compile with plugins
cln compile app.cln -o app.wasm --plugins

# Run with host runtime
./host-bridge run app.wasm
```

### Production

```bash
# Build optimized
cln compile app.cln -o app.wasm --plugins -O3

# Deploy to any WASI host
```

---

## 11. Next Documents

| Document | Description |
|----------|-------------|
| [10_compiler_plugins.md](./10_compiler_plugins.md) | **NEW** - Compiler plugin architecture |
| [03_frame_server.md](./03_frame_server.md) | Runtime, routing, Host Bridge (updated for v2) |
| [04_frame_data.md](./04_frame_data.md) | ORM and database (updated for v2) |
| [frame_bridge_contracts.md](./frame_bridge_contracts.md) | Host Bridge API reference |

---

**End of Document 01 (v2)**
