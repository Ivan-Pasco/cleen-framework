# Frame CLI Specification (02)

**Project:** Frame – Full-Stack Framework for Clean Language
**Location:** `/docs/specification/02_frame_cli.md`

---

## 1. Introduction

Frame development uses two CLI tools from the Clean Language ecosystem:

| Tool | Purpose |
|------|---------|
| `cln` | Clean Language compiler – compilation and plugin loading |
| `cleen` | Clean package manager – plugin and project management |

There is no standalone `frame` CLI. All framework functionality is accessed through `cln` and `cleen`.

---

## 2. Goals

| Goal | Description |
|------|-------------|
| **Unified workflow** | `cln` handles compilation; `cleen` handles project and plugin management |
| **Automation-ready** | All commands can be scripted or run in CI/CD pipelines |
| **Developer clarity** | Short, clear commands with readable output |
| **Consistency** | Commands share structure and argument patterns |
| **Extensibility** | Plugins extend functionality via WASM |

---

## 3. Project Management

### Creating a New Project

```bash
cleen project create myapp --plugins=frame.data,frame.httpserver,frame.ui,frame.auth
cd myapp
```

#### Plugin-Based Folder Scaffolding

When creating a project, the CLI reads each plugin's `plugin.toml` and auto-creates folders based on the `[paths]` configuration:

```toml
# Example from plugin.toml
[paths]
owns = ["app/backend"]
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
  [frame.data] Creating app/data/
  [frame.httpserver] Creating app/backend/
  [frame.ui] Creating app/pages/
  [frame.ui] Creating app/components/
  [frame.ui] Creating app/layouts/
  [frame.auth] Creating app/auth/
  [core] Creating app/public/css/

  Creating files...
  [core] Creating app.cln
  [core] Creating project.toml
  [frame.auth] Creating app/auth/auth.cln
  [frame.ui] Creating app/pages/index.html
  [frame.httpserver] Creating app/backend/health.cln

Project created successfully!

myapp/
├── app.cln                          # Main entry point
├── project.toml                     # Project manifest
└── app/
    ├── pages/
    │   └── index.html               # Home page
    ├── components/
    ├── layouts/
    ├── backend/
    │   └── health.cln               # Health check endpoint
    ├── data/
    ├── auth/
    │   └── auth.cln                 # Auth configuration
    └── public/
        └── css/
```

#### Starter File Templates

Plugins can provide starter file templates in their `[templates]` section:

```toml
# plugin.toml
[templates]
files = [
  { path = "app/auth/auth.cln", template = "auth_config.cln" },
  { path = "app/backend/health.cln", template = "health_endpoint.cln" },
]
```

The CLI reads templates from `~/.cleen/plugins/<name>/<version>/templates/` and copies them to the project.

#### Adding Plugins to Existing Project

```bash
$ cleen plugin add frame.data

Adding frame.data@2.0.0...
  Creating app/data/

Plugin installed. Declare frame.data in your app.cln plugins: block. Files in app/data/ can then use frame.data without individual import statements (implicit import).
```

#### Implicit Plugin Import

When a plugin is declared in `app.cln`, files in its owned folders do not need their own import statement. This is controlled by `implicit_import = true` in plugin.toml:

```toml
# plugin.toml
[paths]
owns = ["app/data"]
implicit_import = true
```

```clean
// app/data/User.cln
// No import statement needed in this file - frame.data is declared in app.cln
// and implicit_import = true means files in owned folders skip individual imports

data User
    integer id : pk, auto
    string email : unique
```

#### Shared Folder (Cross-Cutting)

The `app/shared/` folder is not owned by any plugin — it's for code that can be safely used by both UI and backend:

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

---

## 4. Development Workflow

### Compile with Plugins

```bash
cln compile app.cln -o app.wasm --plugins
```

### Run with Host Bridge

```bash
./host-bridge run app.wasm
```

### Production Build

```bash
cln compile app.cln -o app.wasm --plugins -O3
```

---

## 5. Plugin Management

```bash
# Install plugins
cleen plugin add frame.httpserver
cleen plugin add frame.data@1.0.0

# List installed plugins
cleen plugin list

# Create new plugin
cleen plugin create my-plugin

# Build plugin
cd my-plugin
cleen plugin build

# Install local plugin
cleen plugin add ./
```

---

## 6. Global Options

| Flag | Description |
|------|-------------|
| `--help` | Show usage details for a command |
| `--version` | Display current CLI version and compiler version |
| `--verbose` | Enable detailed build or runtime logs |
| `--target=<env>` | Specify build target (`web`, `pwa`, `mobile`, `desktop`, `server`, `cli`) |
| `--clean` | Force full rebuild and cache clear |
| `--watch` | Rebuild automatically on file changes |

---

**End of Document 02**
