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
cleen project create myapp --plugins=frame.data,frame.httpserver,frame.ui,frame.auth
cd myapp
```

#### Plugin-Based Folder Scaffolding

When creating a project with `cleen project create`, the CLI reads each plugin's `plugin.toml` and auto-creates folders based on the `[paths]` configuration:

```toml
# Example from plugin.toml
[paths]
owns = ["app/backend", "app/backend/api", "app/backend/services"]
auto_create = true
```

**Scaffolding Process:**

1. CLI parses `--plugins` flag (or reads from `project.toml`)
2. For each plugin, reads `plugin.toml` from the plugin registry
3. If `auto_create = true`, creates all folders listed in `owns`
4. Creates starter files from plugin templates
5. Generates `app.cln` with plugins and imports

**Example Output:**

```bash
$ cleen project create myapp --plugins=frame.data,frame.httpserver,frame.ui,frame.auth

Creating project 'myapp'...

  Creating folders...
  [frame.data] Creating app/data/models/
  [frame.data] Creating app/data/queries/
  [frame.data] Creating app/data/migrations/
  [frame.data] Creating app/data/repositories/
  [frame.httpserver] Creating app/backend/api/
  [frame.httpserver] Creating app/backend/services/
  [frame.httpserver] Creating app/backend/middleware/
  [frame.ui] Creating app/ui/pages/
  [frame.ui] Creating app/ui/components/
  [frame.ui] Creating app/ui/layouts/
  [frame.ui] Creating app/ui/styles/
  [frame.auth] Creating app/config/

  Creating files...
  [core] Creating app.cln
  [core] Creating project.toml
  [frame.auth] Creating app/config/auth.cln
  [frame.auth] Creating app/config/project.cln
  [frame.ui] Creating app/ui/pages/index.html
  [frame.ui] Creating app/ui/styles/theme.cln
  [frame.httpserver] Creating app/backend/api/health.cln

Project created successfully!

myapp/
├── app.cln                          # Main entry point
├── project.toml                     # Project manifest
└── app/
    ├── config/
    │   ├── project.cln              # Project settings
    │   └── auth.cln                 # Auth configuration
    ├── data/
    │   ├── models/
    │   ├── queries/
    │   ├── migrations/
    │   └── repositories/
    ├── backend/
    │   ├── api/
    │   │   └── health.cln           # Health check endpoint
    │   ├── services/
    │   └── middleware/
    └── ui/
        ├── pages/
        │   └── index.html       # Home page
        ├── components/
        ├── layouts/
        └── styles/
            └── theme.cln            # Default theme
```

#### Starter File Templates

Plugins can provide starter file templates in their `[templates]` section:

```toml
# plugin.toml
[templates]
files = [
  { path = "app/config/auth.cln", template = "auth_config.cln" },
  { path = "app/backend/api/health.cln", template = "health_endpoint.cln" },
]
```

The CLI reads templates from `~/.cleen/plugins/<name>/<version>/templates/` and copies them to the project.

#### Adding Plugins to Existing Project

When adding a plugin to an existing project, folders are created on-demand:

```bash
$ cleen plugin add frame.data

Installing frame.data@2.0.0...
  Creating app/data/
  Creating app/data/models/
  Creating app/data/queries/
  Creating app/data/migrations/
  Creating app/data/repositories/

Plugin installed. Add to your app.cln:
  plugins:
      frame.data
```

#### Implicit Plugin Import

Files in plugin-owned folders can optionally skip the `plugins:` declaration if `implicit_import = true` in plugin.toml:

```toml
# plugin.toml
[paths]
owns = ["app/data/models"]
implicit_import = true
```

```clean
// app/data/models/User.cln
// No need for 'plugins: frame.data' - it's implicit

data User
    integer id : pk, auto
    string email : unique
```

#### Shared Folder (Cross-Cutting)

The `app/shared/` folder is not owned by any plugin - it's for code that can be safely used by both UI and backend:

```bash
$ cleen project create myapp --with-shared

# Creates additional folders:
myapp/
└── app/
    └── shared/
        ├── types/       # DTOs, shared structs, contracts
        ├── validation/  # Shared validation rules
        └── utils/       # Pure helpers (no IO, no DB, no secrets)
```

**Rules for shared code:**
- No plugin-specific imports
- No I/O operations (database, file system, network)
- No secrets or environment variables
- Pure functions only

```clean
// app/shared/types/CreateUserDTO.cln
// Safe to import from both UI and backend

type CreateUserDTO
    string name
    string email
    string password
```

```clean
// app/shared/validation/email.cln
// Validation rules usable everywhere

functions:
    boolean isValidEmail(string email)
        if email.length() < 3
            return false
        if email.indexOf("@") < 0
            return false
        return true
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
