# Frame Framework

**The Official Full-Stack Framework for Clean Language**

Frame is a modern, full-stack web framework that unifies frontend, backend, and data layers into a single, type-safe programming model. Built on Clean Language and compiled entirely to WebAssembly (WASM), Frame applications run seamlessly across Node.js, Rust, Deno, Tauri, and future WASI environments.

## Overview

Frame embodies the **Clean Language philosophy**: simple, declarative, and transparent code that's easy to reason about and verify. One language, one type system, one compiler—from UI to database.

### Key Features

- **🎯 Type-Safe Full Stack**: End-to-end type safety from database to UI
- **⚡ WebAssembly Native**: Compiles to WASM for predictable performance across platforms
- **🔒 Secure by Default**: Sandboxed execution with clear Host Bridge boundaries
- **🌐 Universal Runtime**: One codebase runs on web, mobile, desktop, and server
- **📦 Zero Boilerplate**: Minimal, declarative syntax—one clear way to do things
- **🚀 SSR + Islands**: Server-side rendering with selective client hydration

## Quick Start

### Installation

```bash
# Install the Frame CLI
npm install -g @clean/frame-cli

# Or using cargo
cargo install frame-cli
```

### Create Your First App

```bash
# Create a new Frame project
frame new myapp
cd myapp

# Start the development server
frame serve
```

Your app is now running at `http://localhost:8080`!

## Project Structure

```
myapp/
├── app/
│   ├── api/              # Backend API routes
│   ├── pages/            # Frontend pages (SSR by default)
│   └── components/       # Reusable UI components
├── db/
│   ├── schema.cln        # Data models and ORM definitions
│   └── migrations/       # Auto-generated SQL migrations
├── config/
│   ├── app.cln           # Application configuration
│   ├── ui.cln            # UI theming and settings
│   ├── data.cln          # Database connections
│   └── auth.cln          # Authentication settings
├── public/               # Static assets (CSS, JS, images)
└── dist/                 # Compiled WASM bundles
```

## Example: Hello World

### 1. Create a Page

**`app/pages/index.cln`**
```clean
component HomePage
    functions:
        Widget render()
            return (
                <div class="container">
                    <h1>Hello, Frame!</h1>
                    <p>Welcome to the Clean Language full-stack framework.</p>
                </div>
            )
```

### 2. Create an API Endpoint

**`app/api/users.cln`**
```clean
endpoints:
    list<User> get(integer? page = 1)
        return User.find:
            where:
                active == true
            order:
                createdAt desc
            limit: 20
```

### 3. Define a Data Model

**`db/schema.cln`**
```clean
data User
    integer id : pk, auto
    string name
    string email : unique
    boolean active = true
    datetime createdAt : default=now
```

### 4. Run Migrations

```bash
frame db:migrate
```

That's it! You have a fully functional full-stack application with type-safe API endpoints, server-rendered UI, and database integration.

## Core Components

### Frame CLI

Command-line interface for creating, building, and deploying Frame applications.

```bash
frame new <name>        # Create new project
frame serve             # Development server with hot reload
frame build             # Compile to WASM bundles
frame db:migrate        # Run database migrations
frame test              # Run tests
```

### Frame Server

WASM runtime that executes Clean code and manages HTTP requests, routing, and server-side rendering.

- File-based routing
- Automatic API endpoint generation
- Server-side rendering (SSR)
- Host Bridge for system integration

### Frame Data (ORM)

Declarative, block-based ORM with compile-time validation and automatic migrations.

```clean
User.find:
    where:
        age >= 18
        active == true
    order:
        name asc
    limit: 100
```

### Frame UI

HTML-first component system with SSR by default and optional client-side hydration.

```clean
component UserCard
    props:
        name: string
        role: string

    functions:
        Widget render()
            return (
                <div class="user-card">
                    <h3>{name}</h3>
                    <span class="role">{role}</span>
                </div>
            )
```

### Frame Auth

Built-in authentication with sessions, JWT, and role-based access control.

```clean
functions:
    Response postLogin(LoginForm form)
        User? u = User.first:
            where:
                email == form.email

        if u == null or not checkPassword(form.password, u.hash)
            return error(401, "Invalid credentials")

        Session s = auth.session.create(u.id)
        return auth.session.setCookie(s, redirect("/dashboard"))
```

## Platform Support

Frame applications can be deployed to multiple platforms from a single codebase:

| Platform | Target | Host Environment |
|----------|--------|------------------|
| **Web** | `--target=web` | Static hosting (Netlify, Vercel) |
| **PWA** | `--target=pwa` | Progressive Web App with offline support |
| **Mobile** | `--target=mobile` | iOS/Android via Capacitor |
| **Desktop** | `--target=desktop` | Windows/Linux/macOS via Tauri |
| **Server** | `--target=server` | Node.js, Rust, Deno |
| **CLI** | `--target=cli` | Command-line tools and daemons |

## Build and Deploy

### Build for Production

```bash
# Build for web
frame build --target=web

# Build for server
frame build --target=server

# Build for mobile
frame build --target=mobile
```

### Deploy

```bash
# Deploy to your preferred platform
# The compiled WASM bundles are in /dist

# Example: Deploy to Node.js
cd dist/server
node index.js
```

## Documentation

- **[Architecture Guide](./ARCHITECTURE.md)** - Deep dive into Frame's technical architecture
- **[Functional Specification](./FUNCTIONAL_SPEC.md)** - Complete feature specifications
- **[Knowledge Base](./KNOWLEDGE_BASE.md)** - Technical reference and best practices
- **[Contributing Guide](./CONTRIBUTING.md)** - How to contribute to Frame
- **[API Reference](./API_REFERENCE.md)** - Complete API documentation
- **[Roadmap](./ROADMAP.md)** - Future features and timeline

## Philosophy

Frame follows Clean Language principles:

- **Simplicity**: Minimal syntax, no decorators, no boilerplate
- **Type Safety**: Every variable, property, and API is typed from source to output
- **Transparency**: No hidden magic—what you read is what runs
- **Performance**: WASM runtime ensures predictable speed across environments
- **Portability**: One binary runs on many hosts with the same behavior
- **Security**: Sandboxed execution, clear Host Bridge boundaries

## Community

- **Website**: [cleanframework.dev](https://cleanframework.dev) (coming soon)
- **Documentation**: [docs.cleanframework.dev](https://docs.cleanframework.dev) (coming soon)
- **GitHub**: [github.com/clean-lang/frame](https://github.com/clean-lang/frame)
- **Discord**: Join our community server (coming soon)

## License

Frame is open source software licensed under the Apache 2.0 license.

## Getting Help

- Check the [documentation](./KNOWLEDGE_BASE.md)
- Browse [examples](./examples/)
- Ask questions in GitHub Discussions
- Report bugs via GitHub Issues

---

**Ready to build?** Run `frame new myapp` and start coding!
