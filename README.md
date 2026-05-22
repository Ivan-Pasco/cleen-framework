# Frame - Full-Stack Framework for Clean Language

**Version 2.0**

Frame is the official full-stack framework for Clean Language, providing a unified programming model for building web applications that compile to WebAssembly.

## What's New in Version 2.0

Frame 2.0 introduces a **plugin-based architecture** where framework functionality is provided by Clean Language plugins rather than Rust crates:

| Component | v1 (Rust Crates) | v2 (Clean Plugins) |
|-----------|------------------|-------------------|
| Web Server | `frame-server` | `frame.web` plugin |
| ORM/Database | `frame-data` | `frame.data` plugin |
| Authentication | `frame-auth` | `frame.auth` plugin |
| UI Components | `frame-ui` | `frame.ui` plugin |
| CLI | `frame-cli` | `cleen` (external) |

This enables:
- **Self-hosting**: Framework extends itself using Clean Language
- **Simpler authoring**: Write plugins in Clean, not Rust
- **Sandboxed execution**: Plugins run in WASM sandbox
- **Easy distribution**: Single `.wasm` file + manifest

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     CLEAN LANGUAGE SOURCE                                │
│                      (with import: block)                                │
│                                                                          │
│   import:                                                                │
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
│                          COMPILER                                        │
│                                                                          │
│   1. Parse source file                                                   │
│   2. Load plugins from ~/.cleen/plugins/                                 │
│   3. Expand framework blocks via plugin.expand_block()                   │
│   4. Continue normal compilation → WASM                                  │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        HOST BRIDGE                                       │
│                      (Rust runtime)                                      │
│                                                                          │
│   Provides: HTTP, Database, Crypto, Auth, File I/O, etc.                │
└─────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

```bash
# Install Clean Manager (cleen)
curl -sSL https://cleanlanguage.org/install.sh | bash

# Install the compiler
cleen install cln

# Install Frame plugins
cleen plugin install frame.web frame.data frame.auth frame.ui
```

### Create a Simple Server

```clean
// main.cln
import:
    frame.web
    frame.data

model: name="User" table="users"
    string email
    string name

server: port=3000
    route: method="GET" path="/users"
        return json(User.all())

    route: method="POST" path="/users"
        user = User()
        user.email = request.body("email")
        user.name = request.body("name")
        user.save()
        return json(user)
```

### Compile and Run

```bash
# Compile with plugins
cln compile main.cln -o app.wasm --plugins

# Run with host bridge
./host-bridge run app.wasm
```

## Project Structure

```
clean-framework/
├── host-bridge/              # Runtime imports (Rust crate)
│   └── src/
├── plugins/                  # Clean Language plugins
│   ├── frame.web/
│   │   ├── plugin.toml
│   │   ├── src/main.cln
│   │   ├── build.sh
│   │   └── README.md
│   ├── frame.data/
│   ├── frame.auth/
│   └── frame.ui/
├── examples/
│   ├── todo-app/
│   └── api-server/
├── scripts/
│   ├── build-all-plugins.sh
│   ├── install-plugins.sh
│   └── test-plugins.sh
└── documents/
    └── specification/
```

## Official Plugins

### frame.web

Web server DSL with routing and middleware.

**Blocks:** `server`, `route`, `middleware`

```clean
import:
    frame.web

server: port=3000
    middleware:
        _log_info("Request: " + request.path)
        return next(request)

    route: method="GET" path="/hello"
        return {"message": "Hello World"}
```

### frame.data

ORM with automatic CRUD methods.

**Blocks:** `model`, `query`, `transaction`

```clean
import:
    frame.data

model: name="Post" table="posts"
    string title
    string content
    boolean published = false
```

### frame.auth

Authentication with JWT and sessions.

**Blocks:** `auth`, `protected`, `login`

```clean
import:
    frame.auth

auth: strategy="jwt" secret="$JWT_SECRET"

protected:
    route: method="GET" path="/profile"
        return request.user
```

### frame.ui

UI components with SSR and hydration.

**Blocks:** `component`, `layout`, `page`

```clean
import:
    frame.ui

component: name="Counter" client="on"
    state:
        integer count = 0

    render()
        return "<button onclick=\"increment()\">" + count().toString() + "</button>"

    increment()
        setCount(count() + 1)
```

## Host Bridge

The Host Bridge provides runtime capabilities to WASM modules:

| Namespace | Functions |
|-----------|-----------|
| `_http_*` | listen, route, middleware, request |
| `_db_*` | query, insert, update, delete, transaction |
| `_auth_*` | create_token, verify_token, hash_password |
| `_file_*` | read, write, exists, delete |
| `_env_*` | get, set |
| `_log_*` | info, warn, error |
| `_crypto_*` | random, hash, sign, verify |
| `_time_*` | now, sleep |

## Building from Source

```bash
# Clone the repository
git clone https://github.com/clean-language/frame
cd frame

# Build host-bridge
cargo build --release

# Build all plugins
./scripts/build-all-plugins.sh

# Install plugins locally
./scripts/install-plugins.sh
```

## Plugin Development

Create your own plugins to extend the framework:

```bash
# Create plugin scaffold
cleen plugin create my-plugin

# Edit src/main.cln
# Implement expand_block function

# Build
cleen plugin build

# Install locally
cleen plugin install ./
```

### Plugin API

```clean
// Required: expand_block function
expand_block(block_name: string, attributes: string, body: string) -> string
    // Return Clean Language source code
    return generated_code

// Optional: validate_block function
validate_block(block_name: string, attributes: string, body: string) -> string
    // Return empty string if valid, error message otherwise
    return ""
```

## Documentation

- [Architecture Overview](documents/ARCHITECTURE.md)
- [Frame Overview](documents/specification/01_frame_overview.md)
- [Compiler Plugins](documents/specification/10_compiler_plugins.md)
- [Plugin System](documents/specification/07_frame_plugins.md)
- [Server/Routing](documents/specification/03_frame_server.md)
- [ORM/Data](documents/specification/04_frame_data.md)
- [Authentication](documents/specification/06_frame_auth.md)
- [UI Components](documents/specification/05_frame_ui.md)
- [Host Bridge Contracts](documents/specification/frame_bridge_contracts.md)

## Examples

- [Todo App](examples/todo-app/) - Full-stack todo with SSR and REST API
- [API Server](examples/api-server/) - REST API with authentication

## License

MIT OR Apache-2.0

## Links

- [Clean Language](https://cleanlanguage.org)
- [Clean Language Compiler](https://github.com/clean-language/cln)
- [Clean Manager](https://github.com/clean-language/cleen)
