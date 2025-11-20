# Frame - The Official Full-Stack Framework for Clean Language

Frame is the official full-stack framework for Clean Language, designed to unify backend, frontend, and data layers under one compiler and one mindset.

## Overview

Frame applications are written entirely in Clean Language and compiled to **WebAssembly (WASM)**, enabling deployment on Node.js, Rust, Deno, or future pure WASI hosts.

### Guiding Principles

- **Simplicity:** One clear way to write code, without decorators or boilerplate
- **End-to-End Type Safety:** Types are shared across ORM, APIs, and UI
- **Transparency:** Human-readable code with no hidden runtime layers
- **Portability:** Runs on any host implementing the Host Bridge
- **Security by Default:** Sandboxed execution, strict defaults, typed host boundaries

## Architecture

Frame consists of several integrated modules:

```
frame/
├── frame-cli/          # CLI tool for project management
├── frame-server/       # WASM runtime and HTTP bridge
├── frame-data/         # ORM and schema management
├── frame-ui/           # Component-based UI system
├── frame-auth/         # Authentication and authorization
├── frame-plugins/      # Plugin system
├── host-bridge/        # Interface between Clean and host runtime
└── examples/           # Example applications
```

### Module Overview

| Module | Purpose | Status |
|--------|---------|--------|
| **frame-cli** | Project scaffolding, compilation, development server | ✅ Foundation |
| **frame-server** | WASM runtime, HTTP routing, host bridge | ✅ Foundation |
| **frame-data** | Type-safe ORM with query builder | ✅ Foundation |
| **frame-ui** | HTML-first component system | ✅ Foundation |
| **frame-auth** | JWT/session authentication, RBAC | ✅ Foundation |
| **frame-plugins** | Extensible plugin system | ✅ Foundation |
| **host-bridge** | Bridge between Clean and host capabilities | ✅ Core Complete |

## Quick Start

### Installation

#### Via Clean Manager (Recommended)

```bash
# 1. Install Clean Manager (one-time setup)
curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash
cleen init

# 2. Install Clean Language compiler
cleen install latest
cleen use latest

# 3. Install Frame Framework
cleen install frame

# 4. Verify installation
frame --version
```

#### From Source

```bash
# Clone the repository
git clone https://github.com/Ivan-Pasco/cleen-framework
cd cleen-framework

# Build the CLI
cargo build --release

# The binary will be at: target/release/frame
```

### Create Your First Project

```bash
# Create a new Frame project
frame new myapp

# Navigate to the project
cd myapp

# Set up the database
frame db:migrate

# Start the development server
frame serve
```

Your application is now running at `http://localhost:3000`!

## CLI Commands

### Project Management

```bash
frame new <name>              # Create a new Frame project
frame serve                   # Start development server
frame build --target=web      # Build for production
```

### Database Operations

```bash
frame db:plan                 # Show pending migrations
frame db:migrate              # Apply migrations
frame db:seed                 # Run database seeding
```

### API Tools

```bash
frame api:spec                # Generate OpenAPI 3.1 spec
frame api:sdk --lang=typescript  # Generate client SDK
```

### Platform Targets

```bash
frame build --target=web      # Static web build
frame build --target=pwa      # Progressive Web App
frame build --target=mobile   # Mobile (Capacitor)
frame build --target=desktop  # Desktop (Tauri)
frame build --target=server   # Server deployment
frame build --target=cli      # CLI application
```

## Project Structure

A typical Frame project:

```
myapp/
├── backend/
│   └── main.cln              # Backend entry point
├── frontend/
│   └── main.cln              # Frontend entry point
├── db/
│   ├── schema.cln            # Database schema
│   └── seed.cln              # Seed data
├── config/
│   ├── app.cln               # Application config
│   ├── auth.cln              # Authentication config
│   └── ui.cln                # UI theme config
├── public/                   # Static assets
│   ├── css/
│   ├── js/
│   └── icons/
└── package.frame.toml        # Package manifest
```

## Example: Full-Stack Application

### Backend (backend/main.cln)

```clean
import Data from "frame/data"
import Server from "frame/server"

class User
    integer id : pk, auto
    string name : min=1, max=80
    string email : email, unique
    boolean active = true

functions:
    list<User> getUsers()
        return User.find:
            where:
                active = true
            order:
                id desc

    User createUser(string name, string email)
        return User.create({ name: name, email: email })

    integer main()
        Server.start(port: 3000)
        return 0
```

### Frontend (frontend/main.cln)

```clean
import UI from "frame/ui"

functions:
    Widget render()
        list<User> users = api.get("/users")

        return (
            <ui-page title="Users">
                <ui-section>
                    <ui-card>
                        <h2>User List</h2>
                        <ui-table>
                            <thead>
                                <tr>
                                    <th>Name</th>
                                    <th>Email</th>
                                </tr>
                            </thead>
                            <tbody>
                                for user in users:
                                    <tr>
                                        <td>{user.name}</td>
                                        <td>{user.email}</td>
                                    </tr>
                            </tbody>
                        </ui-table>
                    </ui-card>
                </ui-section>
            </ui-page>
        )
```

### Database Schema (db/schema.cln)

```clean
class User
    integer id : pk, auto
    string name : min=1, max=80
    string email : email, unique
    boolean active = true
    datetime createdAt : default=now
```

## Frame Data (ORM)

Frame Data provides a type-safe, declarative ORM:

### Schema Definition

```clean
class Post
    integer id : pk, auto
    string title : min=1, max=200
    string content : min=1
    boolean published = false
    integer authorId : foreign=User.id
    User author : related
    datetime createdAt : default=now
```

### Query Operations

```clean
# Find all
list<Post> posts = Post.all()

# Find by ID
Post? post = Post.find(1)

# Query with conditions
list<Post> published = Post.find:
    where:
        published = true
    order:
        createdAt desc
    limit: 10

# Create
Post post = Post.create({
    title: "Hello Frame",
    content: "First post",
    published: true
})

# Update
post.title = "Updated Title"
post.save()

# Delete
Post.delete(1)
```

### Transactions

```clean
Data.tx:
    User user = User.create({ name: "Alice" })
    Post post = Post.create({
        title: "Welcome",
        authorId: user.id
    })
    return { user: user, post: post }
```

## Frame UI

HTML-first component system with Clean syntax:

### Built-in Components

- `<ui-page>` - Page container with title
- `<ui-section>` - Content section
- `<ui-card>` - Card container
- `<ui-button>` - Button with variants
- `<ui-input>` - Form input
- `<ui-form>` - Form container
- `<ui-table>` - Data table
- `<ui-tag>` - Label/badge
- `<ui-modal>` - Modal dialog
- `<ui-tabs>` - Tab navigation

### Custom Components

```clean
class UserBadge
    string name
    string? role

    functions:
        Widget render()
            return (
                <span class="user-badge">
                    {name}
                    if role != null:
                        <ui-tag>{role}</ui-tag>
                </span>
            )
```

## Authentication

Frame Auth provides unified authentication:

### Configuration (config/auth.cln)

```clean
auth:
    session:
        cookie = "frame.sid"
        timeoutMinutes = 60
    jwt:
        secret = env("JWT_SECRET")
        ttlMinutes = 60
```

### Role-Based Access Control (config/roles.cln)

```clean
roles:
    admin:
        - "*"
    editor:
        - "post.create"
        - "post.edit"
    viewer:
        - "post.read"
```

### Usage

```clean
if not auth.can(user, "post.publish")
    return error(403, "Forbidden")
```

## Host Bridge

The Host Bridge defines the boundary between Clean code and the host runtime:

### Available Capabilities

| Namespace | Functions |
|-----------|-----------|
| `bridge:http` | `request`, `respond` |
| `bridge:db` | `query`, `tx` |
| `bridge:env` | `get` |
| `bridge:time` | `now` |
| `bridge:crypto` | `random`, `hash` |
| `bridge:log` | `info`, `warn`, `error` |
| `bridge:sys` | `exit`, `sleep` |

All bridge functions use JSON envelopes for communication:

```json
{ "ok": true, "data": {...} }
{ "ok": false, "err": { "code": "ERROR_CODE", "message": "Error message" } }
```

## Platform Deployment

### Web/PWA

```bash
frame build --target=pwa
# Output: dist/web/
```

### Mobile (Capacitor)

```bash
frame mobile:init
frame mobile:plugin camera
frame build --target=mobile
```

### Desktop (Tauri)

```bash
frame desktop:init
frame build --target=desktop
```

### Server

```bash
frame server:init
frame build --target=server --release
docker-compose up
```

## Development Status

Frame is currently in **active development**. The foundation is in place with:

✅ Complete CLI with all commands
✅ Host Bridge interface
✅ Frame Data ORM core
✅ Frame UI components foundation
✅ Authentication system
✅ Plugin architecture

### Roadmap

- [ ] Complete Clean Language compiler integration
- [ ] Full WASM runtime implementation
- [ ] Live hot reload for development
- [ ] Production deployment tools
- [ ] Plugin marketplace
- [ ] Visual component builder
- [ ] GraphQL auto-generation
- [ ] Pure WASI hosting support

## Contributing

Frame is part of the Clean Language project. Contributions are welcome!

### Development Setup

```bash
# Clone the repository
git clone https://github.com/clean-language/frame
cd frame

# Build all modules
cargo build

# Run tests
cargo test

# Run example
cd examples/full-stack
frame serve
```

## Documentation

Full documentation is available in the `/documents` directory:

- [Frame Framework Specification v2](documents/Frame_Framework_Specification_v2.md)
- [Frame Architecture v10](documents/Frame_Framework_Architecture_v10.md)
- [Runtime Division](documents/Frame_Runtime_Division.md)

## License

Frame is licensed under MIT OR Apache-2.0.

## Links

- [Clean Language](https://github.com/clean-language)
- [Clean Language Compiler](../clean-language-compiler)
- [Clean Manager](../clean-manager)
- [Clean Extension](../clean-extension)

---

**Built with ❤️ for the Clean Language community**
