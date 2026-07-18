# Frame Framework Overview (01)

**Project:** Frame – Full-Stack Framework for Clean Language
**Location:** `/documents/specification/01_frame_overview.md`

---

## 1. Introduction

Frame is the official full-stack framework for **Clean Language**, created to unify frontend, backend, and data logic under one compiler, one runtime, and one mental model.

The framework compiles entirely to **WebAssembly (WASM)**, which means that applications built with Frame can run seamlessly on any host—Node.js, Rust, Deno, Tauri, or future WASI environments.

Frame embodies the **Clean Language philosophy**: simple, declarative, and transparent code that's easy to reason about and verify.

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

## 3. Architecture Overview

Frame is divided into three main components:

```
┌────────────────────────────────────────────────────────────────────┐
│                     CLEAN LANGUAGE COMPILER                         │
│                         (cln binary)                                │
│                                                                     │
│   - Parses .cln source files                                       │
│   - Loads plugins declared in the plugins: block in main.cln        │
│   - Expands framework blocks via plugin WASM                       │
│   - Compiles to WebAssembly                                        │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                      FRAME PLUGINS                                  │
│               (Clean Language → WASM)                               │
│                                                                     │
│   ┌────────────┬────────────┬────────────┬────────────┬────────────┬────────────┐  │
│   │frame.server│ frame.data │ frame.auth │  frame.ui  │frame.canvas│frame.client│  │
│   │(endpoints) │  (data,    │  (auth,    │(component, │(canvasScene│(api.*,     │  │
│   │            │  migrate)  │ protected, │ screen,    │ draw,      │ live.*,    │  │
│   │            │            │  login,    │ page, html,│ onFrame,   │ feed.*)    │  │
│   │            │            │  roles)    │ styles, ui)│ onKey*, ..)│            │  │
│   └────────────┴────────────┴────────────┴────────────┴────────────┴────────────┘  │
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

## 4. Plugin System

### How Plugins Work

1. Developer declares plugins in `main.cln` and writes Clean code with framework blocks in the appropriate folder:
   ```clean
   // app/server/api/hello.cln
   // No import needed — frame.server is declared in main.cln and owns this folder

   endpoints:
       GET "/hello":
           return json({"message": "Hello World"})
   ```

2. Compiler reads the `plugins:` block in `main.cln`, loads `frame.server`, and uses folder ownership to determine which files that plugin processes

3. Plugin's `expand_block` function transforms each block:
   ```
   Input:  endpoints: GET "/hello" : ...
   Output: Generated Clean code with _http_listen(), _http_route(), etc.
   ```

4. Expanded code is compiled to WASM

5. At runtime, Host Bridge provides the `_http_*` functions

### Plugin Installation

Plugins are managed via the `cleen` CLI:

```bash
cleen plugin add frame.server       # Install from registry
cleen plugin add frame.data@1.0.0      # Specific version
cleen plugin list                       # Show installed
cleen plugin create my-plugin           # Create new plugin
```

Plugins are stored in `~/.cleen/plugins/<name>/<version>/`.

### Writing Plugins

Plugins are Clean Language files that export an `expand` function. The function receives a **single JSON string** containing the block name, content, and attributes, and returns a JSON result string:

```clean
// my-plugin/src/main.cln
// plugin.toml declares: [exports] expand = "expand"

functions:
    string expand(string input)
        string blockName = _json_get(input, "name")
        string content = _json_get(input, "content")
        if blockName == "myblock"
            return expand_myblock(content)
        return "{\"error\":\"Unknown block: " + blockName + "\"}"

    string expand_myblock(string body)
        return "{\"statements\":\"// Generated by my-plugin\n" + body + "\"}"
```

See [10_compiler_plugins.md](./10_compiler_plugins.md) for the full plugin ABI specification.

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
│ Plugin Expansion│  → Framework blocks → Clean code
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

## 6. Project Structure

> **Canonical reference:** [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) — complete folder reference with examples for each section.

Frame uses a clean architecture-based folder structure with clear separation of concerns:

```
myapp/
├── main.cln                 # Project metadata + plugins: block
│
└── app/
    ├── ui/
    │   ├── pages/          # SSR pages (frame.ui)
    │   │   ├── index.html  # HTML template with {{ }} and cl-* directives
    │   │   ├── index.cln   # Companion data loader (load/guard functions)
    │   │   ├── about.html
    │   │   └── blog/
    │   │       ├── [slug].html
    │   │       └── [slug].cln
    │   │
    │   ├── components/     # Reusable UI components (frame.ui)
    │   │   ├── Header.cln
    │   │   └── Footer.cln
    │   │
    │   └── layouts/        # Page layout wrappers (frame.ui)
    │       └── main.html
    │
    ├── server/             # HTTP server (frame.server)
    │   ├── api/            # API endpoints
    │   │   ├── users.cln
    │   │   └── posts.cln
    │   ├── services/       # Business logic
    │   └── middleware/     # Middleware
    │
    ├── data/               # Data models/ORM (frame.data)
    │   ├── models/
    │   │   ├── User.cln
    │   │   └── Post.cln
    │   ├── queries/        # Custom queries
    │   ├── migrations/     # Schema migrations
    │   └── repositories/   # Data access patterns
    │
    ├── auth/               # Auth configuration (frame.auth)
    │   └── auth.cln
    │
    └── canvas/             # Canvas applications (frame.canvas)
        ├── scenes/
        ├── sprites/
        └── audio/

public/                     # Static assets (served as-is, project root)
└── css/
    └── style.css

dist/                       # Compiled WASM output
```

### Folder Ownership by Plugin

Each plugin declares the folders it owns in its `plugin.toml` file. When a plugin is declared in `main.cln`, the compiler uses folder ownership to determine which files that plugin processes. Files in owned folders do not need an explicit `import` statement for the plugin (`implicit_import = true` controls this).

| Plugin | Owned Folders |
|--------|---------------|
| `frame.ui` | `app/ui/web/pages/`, `app/ui/web/components/`, `app/ui/web/layouts/` |
| `frame.server` | `app/server/`, `app/server/api/`, `app/server/middleware/` |
| `frame.data` | `app/data/`, `app/data/models/`, `app/data/`, `app/data/migrations/`, `app/data/` |
| `frame.auth` | `app/auth/` |
| `frame.canvas` | `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` |

### Plugin Loading

Plugins **must** be declared in `main.cln` via the `plugins:` block:

```clean
// main.cln
plugins:
	frame.server
	frame.data
	frame.auth
	frame.ui
```

Once a plugin is declared, `implicit_import = true` in the plugin's `plugin.toml` means files inside the plugin's owned folders do not need an explicit `import` statement — the plugin is already active for those files by virtue of folder ownership.

| File Location | Plugin that processes it | Import needed? |
|---------------|--------------------------|----------------|
| `app/server/api/users.cln` | `frame.server` (declared in main.cln) | No |
| `app/data/models/User.cln` | `frame.data` (declared in main.cln) | No |
| `app/auth/auth.cln` | `frame.auth` (declared in main.cln) | No |
| `app/ui/web/pages/index.cln` | `frame.ui` (declared in main.cln) | No |
| `app/ui/web/components/Header.cln` | `frame.ui` (declared in main.cln) | No |
| `app/canvas/scenes/main.cln` | `frame.canvas` (declared in main.cln) | No |

This means a file at `app/server/api/users.cln` does not need to import `frame.server` explicitly — but the plugin must still be declared in `main.cln`.

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

| Plugin | Blocks / Namespaces | Purpose |
|--------|---------------------|---------|
| `frame.server` | `endpoints` | HTTP routing and API endpoints |
| `frame.data` | `data`, `migrate` | ORM and database |
| `frame.auth` | `auth`, `protected`, `login`, `roles` | Authentication and authorization |
| `frame.ui` | `component`, `screen`, `page`, `html`, `styles`, `ui` | UI components and SSR |
| `frame.canvas` | `canvasScene`, `draw`, `onFrame`, `onPointerDown`, `onPointerMove`, `onKeyDown` | Canvas rendering and animation |
| `frame.client` | *(no DSL blocks — bridge functions only)* `api.*`, `live.*`, `feed.*` | Client-side HTTP, WebSocket, and SSE communication |

### Example Usage

**`main.cln`** — project entry point:
```clean
plugins:
	frame.server
	frame.data
	frame.auth
	frame.ui
```

> 🚧 The `app/entity/` folder, `class User` + `data User:` pairing, `User.data.findByEmail(...)`, and `Database.save(entity)` shown below target `frame.data` v2, the target for the next release. **Currently shipped `frame.data` v3.0.10** uses bare-field `data User` and block-form mutations (`User.insert:`, `User.update:`, `User.delete:`) — see the plugin's `patterns/` folder for shipping examples. Phase 2 sub-cycles are underway; this section is the plugin's migration target.

**`app/entity/user.cln`** — domain class (processed by `frame.data`, paired by name with data block):
```clean
class User
	integer? id
	string email
	private string passwordHash
	boolean admin
	datetime createdAt
```

**`app/data/models/user.cln`** — data block (processed by `frame.data`, paired with `class User` by name):
```clean
data User:
	fields:
		id primary generated
		email required unique
		passwordHash as "password_hash" required
		admin required default: false
		createdAt as "created_at" required default: time.now()

	queries:
		User? findByEmail(string emailAddress)
			return User.first:
				where:
					email == emailAddress
```

**`app/auth/auth.cln`** — auth configuration (processed by `frame.auth`, declared in main.cln):
```clean
auth:
	session:
		cookie: "frame.sid"
		timeoutMinutes: 60
	jwt:
		alg: "HS256"
		ttlMinutes: 60
```

**`app/server/api/users.cln`** — API endpoints (processed by `frame.server`, declared in main.cln):
```clean
endpoints:
	POST "/login":
		LoginForm body = req.json(LoginForm)
		User? user = User.data.findByEmail(body.email)
		if user == null or not user.verifiesPassword(body.password)
			return badRequest("Invalid credentials")
		Session s = auth.session.create(user.id!, claims: { email: user.email, role: user.role })
		return auth.session.setCookie(s, redirect("/dashboard"))

	GET "/profile" [auth]:
		User user = User.data.findOrFailById(req.context.claims.sub)
		return json(user)
```

---

## 9. Development Workflow

### Development

```bash
# Install plugins
cleen plugin add frame.server frame.data frame.auth frame.ui

# Compile with plugins
cln compile main.cln -o app.wasm --plugins

# Run with host runtime
./host-bridge run app.wasm
```

### Production

```bash
# Build optimized
cln compile main.cln -o app.wasm --plugins -O3

# Deploy to any WASI host
```

---

## 10. Next Documents

| Document | Description |
|----------|-------------|
| [10_compiler_plugins.md](./10_compiler_plugins.md) | Compiler plugin architecture |
| [03_frame_server.md](./03_frame_server.md) | Runtime, routing, Host Bridge |
| [04_frame_data.md](./04_frame_data.md) | ORM and database |
| [frame_bridge_contracts.md](./frame_bridge_contracts.md) | Host Bridge API reference |

---

**End of Document 01**
