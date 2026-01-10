# Frame CLI Specification (02)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.0 (DEPRECATED)
**Location:** `/docs/specification/02_frame_cli.md`

---

## DEPRECATION NOTICE (v2)

> **This document describes the v1 Frame CLI which has been deprecated in v2.**
>
> In Frame 2.0, the `frame` CLI is replaced by two separate tools:
>
> | v1 Tool | v2 Replacement | Purpose |
> |---------|----------------|---------|
> | `frame new` | `cleen project create` | Project scaffolding |
> | `frame serve` | `cln compile --watch` + host-bridge | Development server |
> | `frame build` | `cln compile --plugins` | Production build |
> | `frame db:*` | Host Bridge runtime | Database operations |
> | `frame plugin:*` | `cleen plugin *` | Plugin management |
>
> **For v2 documentation, see:**
> - [10_compiler_plugins.md](./10_compiler_plugins.md) - Plugin architecture
> - [01_frame_overview.md](./01_frame_overview.md) - Framework overview

---

## Historical Reference (v1)

The following documentation is preserved for reference purposes only. It describes the original Frame CLI architecture that has been superseded by the Clean Language compiler (`cln`) and package manager (`cleen`).

---

## 1. Introduction (v1)

The **Frame CLI** was the command-line interface used to create, build, serve, and deploy Frame projects. It acted as the entry point for developers, the compiler, and automation systems.

Frame CLI bridged the developer experience between Clean Language source code and the runtime WASM environment.

**Note:** This functionality is now handled by:
- `cln` (Clean Language compiler) - Compilation and plugin loading
- `cleen` (Clean package manager) - Plugin and project management
- Host Bridge runtime - Development server and database operations

---

## 2. Goals (v1)

| Goal | Description | v2 Status |
|------|--------------|-----------|
| **Unified workflow** | Provide a single tool to create, serve, and build apps. | Split into `cln` + `cleen` |
| **Automation-ready** | CLI commands can be scripted or run in CI/CD pipelines. | Maintained via `cln` |
| **Developer clarity** | Short, clear commands with readable output. | Maintained |
| **Consistency** | Commands share structure and argument patterns. | Maintained |
| **Extensibility** | Plugins can add new commands dynamically. | Replaced by WASM plugins |

---

## 3. Command Migration Guide

### v1 to v2 Command Mapping

| v1 Command | v2 Equivalent | Notes |
|------------|---------------|-------|
| `frame new <name>` | `cleen project create <name>` | Project scaffolding |
| `frame serve` | `./host-bridge run app.wasm` | With file watching |
| `frame build` | `cln compile app.cln -o app.wasm --plugins` | Production build |
| `frame db:plan` | (Host Bridge runtime) | At runtime |
| `frame db:migrate` | (Host Bridge runtime) | At runtime |
| `frame db:seed` | (Host Bridge runtime) | At runtime |
| `frame api:spec` | (Future: cleen generate openapi) | Planned |
| `frame api:sdk` | (Future: cleen generate sdk) | Planned |
| `frame pwa:init` | (Future: cleen project add pwa) | Planned |
| `frame mobile:init` | (Future: cleen project add mobile) | Planned |
| `frame desktop:init` | (Future: cleen project add desktop) | Planned |

---

## 4. v2 Development Workflow

### Creating a New Project

```bash
# v2 workflow
cleen plugin install frame.web frame.data frame.auth
cleen project create myapp
cd myapp
```

### Development

```bash
# Compile with plugins
cln compile app.cln -o app.wasm --plugins

# Run with host bridge
./host-bridge run app.wasm
```

### Production Build

```bash
# Optimized compilation
cln compile app.cln -o app.wasm --plugins -O3
```

---

## 5. Plugin Management (v2)

Plugin management has moved from `frame plugin:*` to `cleen plugin *`:

```bash
# Install plugins
cleen plugin install frame.web
cleen plugin install frame.data@1.0.0

# List installed plugins
cleen plugin list

# Create new plugin
cleen plugin create my-plugin

# Build plugin
cd my-plugin
cleen plugin build

# Install local plugin
cleen plugin install ./
```

---

## 6. Historical Documentation (v1)

The remainder of this document preserves the original v1 specification for historical reference.

### v1 Command Structure

```
frame <namespace>:<action> [options]
```

Examples:
```bash
frame new myapp
frame db:migrate
frame api:spec --format=json
```

Namespaces included: `db`, `api`, `mobile`, `desktop`, `pwa`, and `server`.

### v1 Global Options

| Flag | Description |
|------|--------------|
| `--help` | Show usage details for a command. |
| `--version` | Display current CLI version and compiler version. |
| `--verbose` | Enable detailed build or runtime logs. |
| `--target=<env>` | Specify build target (`web`, `pwa`, `mobile`, `desktop`, `server`, `cli`). |
| `--clean` | Force full rebuild and cache clear. |
| `--watch` | Rebuild automatically on file changes (default for `serve`). |

### v1 Internal Architecture

```
┌────────────────────────┐
│ User Command (frame)   │
├────────────────────────┤
│ Command Parser          │ → interprets command + options
├────────────────────────┤
│ Clean Compiler Adapter  │ → triggers Clean compilation to WASM
├────────────────────────┤
│ Host Bridge Invoker     │ → connects to host for serve/build tasks
└────────────────────────┘
```

### v1 Project Initialization

```bash
frame new myapp
```

Created the following structure:
```
myapp/
├── app/
│   ├── pages/
│   ├── api/
│   └── components/
├── db/schema.cln
├── config/app.cln
├── public/
└── .frame/              # internal build data
```

---

## 7. Migration Notes

When migrating from Frame v1 to v2:

1. **Remove `frame` CLI dependency** - No longer needed
2. **Install plugins separately** - Use `cleen plugin install`
3. **Update build scripts** - Replace `frame build` with `cln compile --plugins`
4. **Update dev scripts** - Use host-bridge directly
5. **Update CI/CD pipelines** - Reference new commands

---

**End of Document 02 (Deprecated in v2)**
