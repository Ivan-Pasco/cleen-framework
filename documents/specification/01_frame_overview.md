# Frame Framework Overview (01)

**Project:** Frame – Full‑Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/docs/specification/01_frame_overview.md`  

---

## 1. Introduction

Frame is the official full‑stack framework for **Clean Language**, created to unify frontend, backend, and data logic under one compiler, one runtime, and one mental model.

The framework compiles entirely to **WebAssembly (WASM)**, which means that applications built with Frame can run seamlessly on any host—Node.js, Rust, Deno, Tauri, or future WASI environments.

Frame embodies the **Clean Language philosophy**: simple, declarative, and transparent code that’s easy to reason about and verify.

---

## 2. Core Philosophy

| Principle | Description |
|------------|-------------|
| **Simplicity** | Minimal syntax, no decorators, no boilerplate. One way to do things clearly. |
| **Type Safety** | Every variable, property, and API is typed from source to output. |
| **Transparency** | No hidden magic—what you read is what runs. |
| **Performance** | WASM runtime ensures predictable speed across environments. |
| **Portability** | One binary can run on many hosts with the same behavior. |
| **Security** | Sandboxed execution, clear Host Bridge boundaries, no unsafe access. |

---

## 3. Architecture Overview

Frame is divided into modular layers that communicate through **Clean Language definitions** and the **Host Bridge**.

```
┌────────────────────────────┐
│       Frame CLI            │  → project management, build, serve
├────────────────────────────┤
│      Frame Server          │  → WASM execution, routing, SSR
├────────────────────────────┤
│      Frame Data (ORM)      │  → data modeling, migrations, type-safe queries
├────────────────────────────┤
│      Frame UI              │  → HTML-based UI, SSR + CSR components
├────────────────────────────┤
│      Frame Auth            │  → sessions, JWT, roles
├────────────────────────────┤
│      Frame Plugins         │  → extensibility hooks, CLI, UI, or runtime modules
└────────────────────────────┘
             ↓
       Host Bridge (Node, Rust, Deno, etc.)
```

Each layer is written mainly in Clean and compiled to WebAssembly. The **Host Bridge** handles system operations such as I/O, HTTP, database access, and file system.

---

## 4. Clean-to-WASM Pipeline

1. **Source:** developer writes Clean code (`.cln`).  
2. **Compile:** Clean compiler transforms to **CIR (Clean Intermediate Representation)**.  
3. **Emit:** CIR is compiled to **WASM modules**.
4. **Link:** The Frame CLI bundles WASM with configuration and host bindings.
5. **Run:** Host executes the WASM binary using the Frame runtime.

```
.cln → CIR → .wasm → runtime → output
```

---

## 5. Project Structure

```
myapp/
├── app/
│   ├── api/            # backend routes
│   ├── pages/          # frontend pages
│   └── components/     # reusable UI widgets
├── db/
│   ├── schema.cln      # ORM model definitions
│   └── migrations/     # generated SQL diffs
├── config/
│   ├── app.cln         # app configuration
│   ├── ui.cln          # theming
│   ├── data.cln        # db connections
│   └── auth.cln        # auth settings
├── public/             # static assets (CSS, JS, icons)
└── dist/               # compiled WASM + host bundle
```

The **Frame CLI** commands automatically detect this structure, ensuring consistent builds.

---

## 6. Lifecycle Summary

| Stage | Responsibility |
|--------|----------------|
| **Development** | `frame serve` compiles and reloads on changes. |
| **Build** | `frame build` generates WASM and host bundles. |
| **Deploy** | Deploy on Node, Rust, or WASI-compatible hosts. |
| **Run** | The Host Bridge loads the WASM, starts the HTTP server, and serves UI and API. |

---

## 7. Integration with Clean Language

Frame is not a separate framework—it is an **extension of Clean Language**.  
All logic, UI, and data share the same type system, compiler, and syntax.  
This guarantees **end‑to‑end type consistency** and removes the friction of multi‑language stacks.

Frame defines additional namespaces for its modules:

| Namespace | Purpose |
|------------|----------|
| `frame.cli` | Build and serve commands. |
| `frame.server` | HTTP handlers and runtime APIs. |
| `frame.data` | ORM and migrations. |
| `frame.ui` | Components and layout elements. |
| `frame.auth` | Sessions and role handling. |
| `frame.plugin` | Extension API. |

---

## 8. Host Bridge Summary

The **Host Bridge** connects the WASM runtime with host capabilities:

| Bridge Namespace | Responsibility |
|------------------|----------------|
| `host:http` | Request handling, JSON responses. |
| `host:db` | SQL drivers, pooled connections. |
| `host:env` | Environment variables, secrets. |
| `host:log` | Logging from WASM to host console. |
| `host:crypto` | Randomness, hashing. |
| `host:fs` | File system operations (optional). |

---

## 9. Frame for AI Development

This document and the full specification are designed to support **AI‑assisted development** using Claude Code or other models.  
To help the AI reason effectively:

- Each module has its own specification file.
- Functions and classes include **typed, unambiguous syntax**.
- The Host Bridge contracts are described as **deterministic schemas**.
- The documentation uses **consistent formatting and names** so the AI can link concepts easily.

When prompting the AI, always include the relevant document(s) from `/docs/specification/` to provide the full context.

---

## 10. Next Documents

| Next File | Description |
|------------|-------------|
| `02_frame_cli.md` | Details the CLI tool, commands, flags, and how they map to build processes. |
| `03_frame_server.md` | Explains runtime flow, WASM execution, and the Host Bridge internals. |
| `04_frame_data.md` | Documents the ORM, migrations, and database drivers. |

---

**End of Document 01**

