# Compiler Plugin Architecture (10)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.0
**Location:** `/docs/specification/10_compiler_plugins.md`

---

## 1. Introduction

This document describes the **new compiler plugin architecture** where plugins are written **in Clean Language itself** and compiled to WebAssembly. This replaces the previous Rust-based plugin system.

### Key Changes from Previous Architecture

| Aspect | Previous (v1) | New (v2) |
|--------|---------------|----------|
| **Plugin Language** | Rust | Clean Language |
| **Plugin Format** | Rust crate | `.cln` source → `.wasm` binary |
| **Execution** | Compile-time Rust | Compile-time WASM |
| **Installation** | Part of framework build | External via `cleen plugin install` |
| **Location** | `frame-compiler-plugins/` | `~/.cleen/plugins/` |

### Benefits

- **Self-Hosting**: Clean Language plugins extend Clean Language
- **Simpler Authoring**: No Rust knowledge required
- **Portable**: Same plugin works across all compiler installations
- **Sandboxed**: WASM execution provides security isolation
- **Dynamic Loading**: Plugins loaded at compile-time from registry

---

## 2. Plugin Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CLEAN LANGUAGE SOURCE                             │
│                         (with plugins: block)                            │
│                                                                          │
│   plugins:                                                               │
│       frame.web                                                          │
│       frame.data                                                         │
│                                                                          │
│   server: port=3000                                                      │
│       route: method="GET" path="/users"                                  │
│           return User.all()                                              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           COMPILER                                       │
│                                                                          │
│   1. Parse source file                                                   │
│   2. Extract plugins: blocks → ["frame.web", "frame.data"]              │
│   3. Load plugins from ~/.cleen/plugins/                                 │
│   4. For each framework block (server:, route:, model:):                │
│      → Call plugin.expand_block(name, attrs, body)                      │
│      → Replace block with generated Clean code                          │
│   5. Continue normal compilation                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          WASM OUTPUT                                     │
│                    (includes generated code)                             │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Plugin Structure

### Directory Layout

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
└── frame.auth/
    └── 1.0.0/
        ├── plugin.toml
        └── plugin.wasm
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language - provides server, route, and middleware DSL blocks"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand"                # Required: block expansion function
validate = "validate"            # Optional: validation function
get_keywords = "get_keywords"    # Optional: for IDE syntax highlighting

[handles]
blocks = ["server", "route", "middleware", "endpoints"]

# Bridge function declarations - Host Bridge functions this plugin's generated code requires
# The compiler reads these to generate correct WASM imports
[bridge]
functions = [
  { name = "_http_listen", params = ["integer"], returns = "integer", description = "Start HTTP server on port" },
  { name = "_http_route", params = ["string", "string", "integer"], returns = "integer", description = "Register route handler" },
  { name = "_req_body", params = [], returns = "string", description = "Get request body" },
]

[paths]
# Folders owned by this plugin - auto-created by CLI
owns = ["app/api"]

# Automatically create folders when plugin is imported
auto_create = true

# File patterns recognized in owned folders
patterns = ["*.cln"]

# Files in owned folders automatically import this plugin
implicit_import = true
```

> **Note:** The `[bridge]` section is required for all plugins. It declares Host Bridge functions that the generated code will call at runtime. The compiler uses these declarations to generate WASM imports, and the runtime provides implementations.

### Plugin Source (Clean Language)

Plugins are written in Clean Language and export an `expand_block` function:

```clean
// plugins/frame.web/src/main.cln

// Main entry point - called by compiler for each framework block
expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "server"
        return expand_server(attributes, body)
    if block_name == "route"
        return expand_route(attributes, body)
    if block_name == "middleware"
        return expand_middleware(attributes, body)
    return body

// Server block expansion
expand_server(attrs: string, body: string) -> string
    port = get_attr(attrs, "port", "8080")
    host = get_attr(attrs, "host", "localhost")

    return "
// Generated server code
start()
    print(\"Server starting on " + host + ":" + port + "\")
    " + body + "
    _http_listen(\"" + host + "\", " + port + ")
"

// Route block expansion
expand_route(attrs: string, body: string) -> string
    method = get_attr(attrs, "method", "GET")
    path = get_attr(attrs, "path", "/")

    return "
_http_route(\"" + method + "\", \"" + path + "\", (request) -> any
    " + body + "
)
"

// Helper to parse JSON attributes
get_attr(json: string, key: string, default_val: string) -> string
    // Simplified - real implementation would parse JSON
    return default_val
```

---

## 4. Compilation Flow

### Step 1: Plugin Detection

The compiler parses `plugins:` blocks to identify required plugins:

```clean
plugins:
    frame.web
    frame.data
```

This tells the compiler to load `frame.web` and `frame.data` plugins.

> **Note:** The `plugins:` block is specifically for loading framework plugins. File imports use `import "path/to/file.cln"` syntax instead.

### Step 2: Plugin Loading

```
1. For each import name:
   a. Look in ~/.cleen/plugins/<name>/
   b. Find latest compatible version
   c. Load plugin.toml manifest
   d. Initialize WASM module from plugin.wasm
   e. Register plugin in PluginRegistry
```

### Step 3: Block Expansion

When the compiler encounters a framework block:

```clean
server: port=3000
    route: method="GET" path="/users"
        return User.all()
```

It calls the plugin's `expand_block` function:

```
Input:
  - block_name: "server"
  - attributes: '{"port": "3000"}'
  - body: 'route: method="GET" path="/users"\n    return User.all()'

Output:
  Clean Language source code that replaces the block
```

### Step 4: Recursive Expansion

Generated code may contain nested framework blocks. The compiler recursively expands until no framework blocks remain.

### Step 5: Normal Compilation

The fully expanded Clean code continues through the standard compilation pipeline:
- Semantic Analysis
- Type Checking
- MIR Generation
- WASM Output

---

## 5. Plugin API

### Required Export: `expand_block`

```clean
expand_block(block_name: string, attributes: string, body: string) -> string
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `block_name` | string | The block identifier (e.g., "server", "route") |
| `attributes` | string | JSON object of block attributes |
| `body` | string | Raw content inside the block |
| **Returns** | string | Clean Language source code |

### Optional Export: `validate_block`

```clean
validate_block(block_name: string, attributes: string, body: string) -> string
```

Returns empty string if valid, or error message if invalid. Called before `expand_block`.

### Optional Export: `get_keywords`

```clean
get_keywords() -> string
```

Returns JSON array of keywords for IDE syntax highlighting:
```json
["server", "route", "middleware", "GET", "POST", "PUT", "DELETE"]
```

---

## 6. Host Bridge Integration

Plugins generate code that calls **Host Bridge functions** at runtime. The generated code uses these imports.

### 6.1 Bridge Function Declarations

Plugins **must** declare their Host Bridge function dependencies in the `[bridge]` section of `plugin.toml`. This enables:

- **Decoupled Architecture**: Plugins explicitly declare what they need
- **Compiler Automation**: Compiler generates WASM imports from declarations
- **Runtime Validation**: Runtime can verify all required functions are provided
- **Documentation**: Self-documenting plugin requirements

> **Required**: All plugins must include a `[bridge]` section declaring the Host Bridge functions their generated code uses.

#### Schema Definition

```toml
[bridge]
functions = [
  { name = "<function_name>", params = ["<type>", ...], returns = "<type>", module = "<module>", description = "<description>" },
]
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | ✅ | - | Function name (e.g., `"_db_query"`) |
| `params` | array | ✅ | - | Parameter types as strings |
| `returns` | string | ✅ | - | Return type (`"void"` for no return) |
| `module` | string | ❌ | `"env"` | WASM import module |
| `description` | string | ❌ | - | Human-readable description |

> **Note:** The `module` field defaults to `"env"`, which is the standard WASM import module for Host Bridge functions.

#### Supported Types

Types must match the Clean Language type system:

| Type | WASM Type | Description |
|------|-----------|-------------|
| `"string"` | `(i32, i32)` | Pointer and length (see calling convention below) |
| `"integer"` | `i32` | 32-bit signed integer |
| `"number"` | `f64` | 64-bit floating point |
| `"boolean"` | `i32` | 0 = false, 1 = true |
| `"void"` | (none) | No return value |
| `"any"` | `i32` | Pointer to boxed value |
| `"list"` | `i32` | Pointer to list structure |

#### WASM Calling Convention

**String Parameters:**
- Each `string` parameter expands to two WASM i32 parameters: `(pointer, length)`
- Example: `_db_query(sql: string, params: string)` becomes WASM signature `(i32, i32, i32, i32) -> i32`

**String Returns:**
- String return values use length-prefixed format in linear memory
- First 4 bytes: little-endian u32 length
- Remaining bytes: UTF-8 string data
- The function returns a pointer to this structure

**Example Type Expansion:**
```
Clean: _db_query(sql: string, params: string) -> string
WASM:  (func (param i32 i32 i32 i32) (result i32))
       ↑     sql_ptr sql_len params_ptr params_len   ↑ returns length-prefixed string pointer
```

See [frame_bridge_contracts.md](frame_bridge_contracts.md) for detailed calling conventions.

#### Compilation Flow

```
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│   plugin.toml   │ ──▶  │    Compiler     │ ──▶  │   WASM Module   │
│                 │      │                 │      │                 │
│  [bridge]       │      │  1. Parse decl  │      │  (import "env"  │
│  functions = [  │      │  2. Map types   │      │    "_db_query"  │
│    _db_query    │      │  3. Gen imports │      │    (func ...))  │
│  ]              │      │                 │      │                 │
└─────────────────┘      └─────────────────┘      └─────────────────┘
                                                          │
                                                          ▼
                                                  ┌─────────────────┐
                                                  │    Runtime      │
                                                  │                 │
                                                  │  Host provides  │
                                                  │  implementations│
                                                  └─────────────────┘
```

**Flow Details:**

1. **Plugin Loading**: Compiler reads `plugin.toml` and extracts `[bridge]` section
2. **Type Mapping**: Clean types are mapped to WASM types (string → i32 pointer, etc.)
3. **Import Generation**: Compiler generates WASM import declarations in output module
4. **Runtime Linking**: Host Bridge runtime provides function implementations at execution

#### Example: frame.data Plugin

```toml
[plugin]
name = "frame.data"
version = "1.0.0"
description = "ORM and database plugin for Clean Language - provides model, query, and transaction DSL blocks"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
get_keywords = "get_keywords"

[blocks]
handles = ["model", "query", "transaction"]

[bridge]
functions = [
  # Query and Execute
  { name = "_db_query", params = ["string", "string"], returns = "string", description = "Execute SELECT query, returns JSON result with rows" },
  { name = "_db_execute", params = ["string", "string"], returns = "integer", description = "Execute INSERT/UPDATE/DELETE, returns affected row count" },

  # Transaction Control
  { name = "_db_begin", params = [], returns = "string", description = "Begin transaction, returns transaction ID" },
  { name = "_db_commit", params = ["string"], returns = "integer", description = "Commit transaction by ID, returns 0 on success" },
  { name = "_db_rollback", params = ["string"], returns = "integer", description = "Rollback transaction by ID, returns 0 on success" },
]
```

#### Example: frame.web Plugin

```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language - provides server, route, and middleware DSL blocks"
author = "Clean Language Team"

[compatibility]
min_compiler_version = "0.15.0"

[handles]
blocks = ["server", "route", "middleware", "endpoints"]

[exports]
expand = "expand"
validate = "validate"
get_keywords = "get_keywords"

[bridge]
functions = [
  # HTTP Server
  { name = "_http_listen", params = ["integer"], returns = "integer", description = "Start HTTP server on specified port" },
  { name = "_http_route", params = ["string", "string", "integer"], returns = "integer", description = "Register route: method, path, handler_idx" },
  { name = "_http_route_protected", params = ["string", "string", "integer", "string"], returns = "integer", description = "Register protected route with role requirement" },

  # Request Context - Parameters
  { name = "_req_param", params = ["string"], returns = "string", description = "Get path parameter by name" },
  { name = "_req_query", params = ["string"], returns = "string", description = "Get query parameter by name" },
  { name = "_req_header", params = ["string"], returns = "string", description = "Get request header by name" },

  # Request Context - Data
  { name = "_req_body", params = [], returns = "string", description = "Get request body as string" },
  { name = "_req_method", params = [], returns = "string", description = "Get HTTP method (GET, POST, etc.)" },
  { name = "_req_path", params = [], returns = "string", description = "Get request path" },
]
```

#### Example: frame.auth Plugin

```toml
[plugin]
name = "frame.auth"
version = "1.0.0"
description = "Authentication and authorization plugin for Clean Language"
author = "Clean Language Team"

[compatibility]
min_compiler_version = "0.15.0"

[blocks]
handles = ["auth", "protected", "login", "guard"]

[exports]
expand = "expand_block"
validate = "validate_block"
get_keywords = "get_keywords"

[bridge]
functions = [
  # Token management
  { name = "_auth_create_token", params = ["string", "string"], returns = "string", description = "Create JWT token with payload and secret" },
  { name = "_auth_verify_token", params = ["string", "string"], returns = "string", description = "Verify JWT token, returns payload JSON or empty on failure" },

  # Password hashing
  { name = "_auth_hash_password", params = ["string"], returns = "string", description = "Hash password using bcrypt" },
  { name = "_auth_verify_password", params = ["string", "string"], returns = "boolean", description = "Verify password against hash" },

  # Session management
  { name = "_session_create", params = ["string"], returns = "string", description = "Create session with user data, returns session ID" },
  { name = "_session_get", params = ["string"], returns = "string", description = "Get session data by ID, returns JSON or empty" },
  { name = "_session_destroy", params = ["string"], returns = "integer", description = "Destroy session by ID, returns 0 on success" },
]
```

### 6.2 Available Host Bridge Functions

The following Host Bridge functions are available for plugins to declare. See [frame_bridge_contracts.md](frame_bridge_contracts.md) for detailed signatures and JSON formats.

#### HTTP Functions (frame.web)

```clean
// Server lifecycle
_http_listen(port: integer) -> integer
_http_route(method: string, path: string, handler_idx: integer) -> integer
_http_route_protected(method: string, path: string, handler_idx: integer, role: string) -> integer

// Request context
_req_param(name: string) -> string
_req_query(name: string) -> string
_req_header(name: string) -> string
_req_body() -> string
_req_method() -> string
_req_path() -> string
```

#### Database Functions (frame.data)

```clean
// Query operations
_db_query(sql: string, params: string) -> string    // Returns JSON with rows
_db_execute(sql: string, params: string) -> integer // Returns affected row count

// Transaction control
_db_begin() -> string                               // Returns transaction ID
_db_commit(tx_id: string) -> integer                // Returns 0 on success
_db_rollback(tx_id: string) -> integer              // Returns 0 on success
```

#### Auth Functions (frame.auth)

```clean
// Token management
_auth_create_token(payload: string, secret: string) -> string
_auth_verify_token(token: string, secret: string) -> string

// Password hashing
_auth_hash_password(password: string) -> string
_auth_verify_password(password: string, hash: string) -> boolean

// Session management
_session_create(user_data: string) -> string
_session_get(session_id: string) -> string
_session_destroy(session_id: string) -> integer
```

#### File Functions (frame.fs)

```clean
_file_read(path: string) -> string
_file_write(path: string, content: string) -> integer
_file_exists(path: string) -> boolean
_file_delete(path: string) -> integer
```

These functions are implemented by the **clean-server** runtime and linked when the WASM module is instantiated.

---

## 7. Plugin Paths Configuration

Plugins can define folder ownership through the `[paths]` section in `plugin.toml`. This enables automatic folder scaffolding and implicit imports.

### 7.1 Paths Schema

```toml
[paths]
# Folders owned by this plugin - auto-created by CLI
owns = ["app/api"]

# Automatically create folders when plugin is imported
auto_create = true

# File patterns recognized in owned folders
patterns = ["*.cln"]

# Files in owned folders automatically import this plugin
implicit_import = true
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `owns` | array | ✅ | - | Folders owned by this plugin |
| `auto_create` | boolean | ❌ | `false` | Create folders on plugin install |
| `patterns` | array | ❌ | `["*.cln"]` | File patterns recognized |
| `implicit_import` | boolean | ❌ | `false` | Auto-import plugin in owned folders |

### 7.2 Official Plugin Folder Ownership

| Plugin | Owned Folders |
|--------|---------------|
| `frame.ui` | `app/pages/`, `app/components/`, `app/layouts/` |
| `frame.httpserver` | `app/api/` |
| `frame.data` | `app/data/` |
| `frame.auth` | `app/auth/` |
| `frame.canvas` | `app/canvas/` |

### 7.3 Folder Creation

Folders are created by the CLI at these points:

1. **Project Creation**: `cleen project create myapp --plugins=frame.data,frame.httpserver`
2. **Plugin Installation**: `cleen plugin add frame.data`

Example output:
```bash
$ cleen project create myapp --plugins=frame.data,frame.httpserver,frame.ui

Creating project 'myapp'...
  [frame.data] Creating app/data/
  [frame.httpserver] Creating app/api/
  [frame.ui] Creating app/pages/
  [frame.ui] Creating app/components/
  [frame.ui] Creating app/layouts/

Project created successfully!
```

### 7.4 Implicit Imports

When `implicit_import = true`, files in owned folders don't need explicit `plugins:` declarations:

```clean
// app/data/User.cln
// No need for 'plugins: frame.data' - it's implicit because
// the file is in app/data/ which is owned by frame.data

data User
    integer id : pk, auto
    string email : unique
    string name
```

The compiler detects the file is in a plugin-owned folder and automatically applies the plugin.

---

## 8. Language Server Integration

Plugins can provide language definitions for IDE support (syntax highlighting, autocompletion, diagnostics) through the `[language]` section in `plugin.toml`.

### 8.1 Language Schema

```toml
[language]
# Keywords introduced by this plugin
keywords = ["data", "endpoints", "component"]

# Block definitions with their syntax
blocks = [
  { name = "data", attributes = ["name"], body = "fields", description = "Define a data model" },
  { name = "endpoints", attributes = [], body = "routes", description = "Define HTTP endpoints" },
]

# Field/property modifiers
modifiers = [
  { name = "pk", description = "Primary key" },
  { name = "auto", description = "Auto-increment" },
  { name = "unique", description = "Unique constraint" },
]

# Types introduced by this plugin
types = ["datetime", "uuid"]

# Completions for specific contexts
completions = [
  { context = "block_name", items = ["data", "endpoints"] },
  { context = "field_type", items = ["integer", "string", "boolean", "datetime"] },
  { context = "field_modifier", items = ["pk", "auto", "unique", "default"] },
]

# Snippets for autocompletion
snippets = [
  { trigger = "data", body = "data ${1:ModelName}\n\tinteger id : pk, auto\n\t${0}", description = "Create a data model" },
  { trigger = "endpoint", body = "${1:GET} /${2:path}:\n\thandle:\n\t\t${0}", description = "Create an endpoint" },
]

# Diagnostics rules
diagnostics = [
  { pattern = "data without pk", severity = "warning", message = "Data model should have a primary key" },
]
```

### 8.2 Language Schema Reference

| Field | Type | Description |
|-------|------|-------------|
| `keywords` | array | Keywords introduced by the plugin |
| `blocks` | array | Block definitions with syntax rules |
| `modifiers` | array | Field/property modifiers |
| `types` | array | Custom types introduced |
| `completions` | array | Context-aware completion items |
| `snippets` | array | Code snippets for autocompletion |
| `diagnostics` | array | Custom diagnostic rules |

### 8.3 Language Server Discovery Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         LANGUAGE SERVER                                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  1. DETECT PROJECT                                                       │
│     - Read project.toml → get plugins list                              │
│     - OR scan app.cln → parse plugins: block                            │
│     - OR check file path → implicit plugin from folder ownership        │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  2. LOAD PLUGIN DEFINITIONS                                              │
│     For each active plugin:                                              │
│     - Find plugin at ~/.cleen/plugins/<name>/<version>/                 │
│     - Read plugin.toml                                                   │
│     - Extract [language] section                                         │
│     - Cache definitions per project                                      │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  3. MERGE DEFINITIONS                                                    │
│     - Base Clean Language definitions (always active)                   │
│     - + Plugin keywords, blocks, types, modifiers                       │
│     - + Plugin completions, snippets, diagnostics                       │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  4. PROVIDE LSP FEATURES                                                 │
│     - textDocument/completion    → merged completions                   │
│     - textDocument/hover         → plugin documentation                 │
│     - textDocument/diagnostic    → plugin-specific errors               │
│     - textDocument/definition    → go to plugin source                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.4 Project Plugin Detection

The language server detects active plugins using these methods (in order):

1. **project.toml** (recommended)
   ```toml
   [project]
   name = "myapp"

   [plugins]
   frame.data = "2.0.0"
   frame.httpserver = "2.0.0"
   frame.ui = "2.1.0"
   ```

2. **app.cln plugins: block**
   ```clean
   plugins:
       frame.data
       frame.httpserver
   ```

3. **Folder ownership** (implicit)
   - File in `app/data/` → frame.data is active
   - File in `app/api/` → frame.httpserver is active

### 8.5 Caching Strategy

```
~/.cleen/cache/
└── lsp/
    └── <project-hash>/
        ├── plugins.json      # List of active plugins
        └── definitions.json  # Merged language definitions
```

The cache is invalidated when:
- project.toml changes
- app.cln plugins: block changes
- A plugin is installed/updated/removed

---

## 9. Plugin Development Workflow

### Creating a New Plugin

```bash
# 1. Create plugin project
cleen plugin create my-plugin

# 2. Edit src/main.cln
# Implement expand_block function

# 3. Build plugin
cleen plugin build
# Compiles src/main.cln → plugin.wasm

# 4. Test locally
cleen plugin install ./
# Installs to ~/.cleen/plugins/my-plugin/0.1.0/

# 5. Use in project
# Add to your .cln file:
# plugins:
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
├── build.sh              # Build script
└── README.md             # Documentation
```

### Build Script (build.sh)

```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"
cln compile src/main.cln -o plugin.wasm
echo "Built $(basename $(pwd)) plugin → plugin.wasm"
```

---

## 8. Official Frame Plugins

### frame.web

Handles web server DSL blocks.

**Blocks:** `server`, `route`, `middleware`

**Example:**
```clean
plugins:
    frame.web

server: port=3000
    middleware:
        print("Request: " + request.path)
        return next(request)

    route: method="GET" path="/"
        return {"message": "Hello World"}
```

### frame.data

Handles database/ORM DSL blocks.

**Blocks:** `model`, `query`, `transaction`

**Example:**
```clean
plugins:
    frame.data

model: name="User" table="users"
    string email
    string name
    boolean active = true
```

### frame.auth

Handles authentication DSL blocks.

**Blocks:** `auth`, `protected`, `login`

**Example:**
```clean
plugins:
    frame.auth

auth: strategy="jwt" secret="$JWT_SECRET"

protected:
    route: method="GET" path="/profile"
        return request.user
```

### frame.ui

Handles UI component DSL blocks.

**Blocks:** `component`, `layout`, `page`

**Example:**
```clean
plugins:
    frame.ui

component: name="Button"
    props:
        string label
        string variant = "primary"

    render()
        return <button class="btn btn-{variant}">{label}</button>
```

---

## 9. Migration from Rust Plugins

### What's Being Replaced

| Old (Rust) | New (Clean) |
|------------|-------------|
| `frame-compiler-plugins/` | `plugins/frame.web/src/main.cln` |
| `frame-plugins/` | Deleted - logic moves to Clean plugins |
| Rust `FrameworkPlugin` trait | Clean `expand_block` function |
| Compile-time Rust execution | Compile-time WASM execution |

### What's Being Kept

| Component | Status |
|-----------|--------|
| `host-bridge/` | **KEPT** - Provides runtime imports |
| Host Bridge API | **KEPT** - Same function signatures |
| Plugin concept | **KEPT** - Same DSL block expansion |

### Migration Steps

1. Each Rust plugin crate → Clean Language source file
2. `FrameworkPlugin::expand()` → `expand_block()` function
3. Plugin manifest: `plugin.cln` → `plugin.toml`
4. Build: Rust compilation → `cln compile`

---

## 10. Security Model

### Compile-Time Isolation

- Plugins run in WASM sandbox during compilation
- No filesystem access (except reading source)
- No network access
- No access to compiler internals

### Generated Code Safety

- Generated code goes through full type checking
- Invalid code caught by semantic analysis
- No way to bypass type system

### Plugin Trust

- Plugins installed from registry are signed
- Local plugins require explicit installation
- Version pinning supported for reproducibility

---

## 11. CLI Integration

### Compiler Flags

```bash
# Enable plugin loading (auto-detects from imports)
cln compile app.cln -o app.wasm --plugins

# Specify custom plugin directory
cln compile app.cln -o app.wasm --plugins --plugin-dir ./local-plugins/
```

### Plugin Management (via cleen)

```bash
cleen plugin install frame.web          # Install from registry
cleen plugin install frame.web@1.0.0    # Specific version
cleen plugin install ./my-plugin        # Local plugin
cleen plugin list                       # List installed
cleen plugin remove frame.web           # Uninstall
cleen plugin create my-plugin           # Scaffold new plugin
cleen plugin build                      # Build current plugin
```

---

## 12. AI Development Notes

This architecture is designed for AI-assisted development:

- **Deterministic expansion**: Same input → same output
- **Simple API**: Single `expand_block` function
- **Self-documenting**: Plugins written in Clean are readable
- **Type-safe output**: Generated code is type-checked

When prompting AI:
- Include this document for plugin development
- Include `frame_bridge_contracts.md` for Host Bridge API
- Include specific plugin source for modifications

---

**End of Document 10**
