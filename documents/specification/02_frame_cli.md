# Frame CLI Specification (02)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/docs/specification/02_frame_cli.md`

---

## 1. Introduction

The **Frame CLI** is the command-line interface used to create, build, serve, and deploy Frame projects. It acts as the entry point for developers, the compiler, and automation systems.

Frame CLI bridges the developer experience between Clean Language source code and the runtime WASM environment.

---

## 2. Goals

| Goal | Description |
|------|--------------|
| **Unified workflow** | Provide a single tool to create, serve, and build apps. |
| **Automation-ready** | CLI commands can be scripted or run in CI/CD pipelines. |
| **Developer clarity** | Short, clear commands with readable output. |
| **Consistency** | Commands share structure and argument patterns. |
| **Extensibility** | Plugins can add new commands dynamically. |

---

## 3. Command Overview

| Command | Description |
|----------|--------------|
| `frame new <name>` | Create a new Frame project using the standard structure. |
| `frame serve` | Run the development server with live reload and auto compile. |
| `frame build` | Compile both backend and frontend into optimized WASM bundles. |
| `frame db:plan` | Display SQL migration plan based on schema changes. |
| `frame db:migrate` | Apply pending migrations to the configured database. |
| `frame db:seed` | Run seeding scripts located in `db/seed.cln`. |
| `frame api:spec` | Generate an OpenAPI specification for your API endpoints. |
| `frame api:sdk` | Create type-safe client SDKs for Clean, TypeScript, or Swift. |
| `frame pwa:init` | Generate manifest, service worker, and icons for PWA support. |
| `frame mobile:init` | Scaffold a Capacitor mobile wrapper and default plugins. |
| `frame desktop:init` | Scaffold a Tauri desktop wrapper and host adapters. |
| `frame server:init` | Generate Dockerfile, health endpoints, and env templates. |

---

## 4. Command Structure

Every Frame command follows this structure:

```
frame <namespace>:<action> [options]
```

Examples:
```bash
frame new myapp
frame db:migrate
frame api:spec --format=json
```

Namespaces include: `db`, `api`, `mobile`, `desktop`, `pwa`, and `server`.

Each action maps to a specific **Clean compiler task** or **Host Bridge adapter**.

---

## 5. Global Options

| Flag | Description |
|------|--------------|
| `--help` | Show usage details for a command. |
| `--version` | Display current CLI version and compiler version. |
| `--verbose` | Enable detailed build or runtime logs. |
| `--target=<env>` | Specify build target (`web`, `pwa`, `mobile`, `desktop`, `server`, `cli`). |
| `--clean` | Force full rebuild and cache clear. |
| `--watch` | Rebuild automatically on file changes (default for `serve`). |

---

## 6. Internal Architecture

### CLI Layers
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

### Execution Flow
1. CLI parses the input command.  
2. Maps it to a handler defined in the **Frame CLI Registry**.  
3. The handler executes compiler tasks, migrations, or file generation.  
4. Results are logged via `host:log` for consistent host output.

---

## 7. Project Initialization

```bash
frame new myapp
```
Creates the following structure:
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

Optional flags:
```bash
--template=api     # start with API-only backend
--template=full    # full-stack app (default)
--template=ui      # UI-only app
```

---

## 8. Build System

### Command
```bash
frame build --target=server
```

The build process has 4 stages:
1. **Parse:** Read all `.cln` files and configs.  
2. **Compile:** Generate `.wasm` files.  
3. **Link:** Combine runtime modules and Host Bridge definitions.  
4. **Emit:** Produce deployable `/dist` output.

Outputs:
```
dist/
├── server.wasm
├── ui.wasm
├── config.json
└── manifest.islands.json
```

---

## 9. Development Server

### Command
```bash
frame serve
```

Starts a live-reloading local server that:
- Compiles `.cln` files on save.
- Serves WASM via Node or Deno.
- Watches `/public` for assets.
- Logs compiler messages to the console.

Server runs by default at:
```
http://localhost:8080
```

---

## 10. Plugin Commands

The CLI can load new commands dynamically from Frame plugins.

Plugin folder example:
```
plugins/
└── charts/
    ├── plugin.cln
    └── cli/add_chart.cln
```

If a plugin defines `cli/add_chart.cln`, it registers automatically:
```bash
frame charts:add_chart
```

---

## 11. Error Handling

Errors are reported in a standard JSON structure, compatible with both CLI and API outputs:
```json
{
  "ok": false,
  "err": { "code": "BUILD_FAIL", "message": "Syntax error in schema.cln" }
}
```

CLI logs follow these levels:
```
INFO → SUCCESS → WARNING → ERROR → FATAL
```

Use `--verbose` to see detailed stack traces or compiler output.

---

## 12. AI Integration Notes

The CLI specification is designed to be **AI-readable and automatable**:
- Each command is self-contained with predictable syntax.
- Output is deterministic (JSON where possible).  
- This allows Claude Code or other AI agents to execute, inspect, and reason about builds automatically.

When integrating AI agents:
- Always run commands in `--verbose` for context-rich output.
- Prefer machine-readable flags (`--json`, `--silent=false`).
- Store all build results in `/dist` for inspection.

---

## 13. Future Additions

| Feature | Description |
|----------|--------------|
| `frame test` | Built-in test runner for unit and integration tests. |
| `frame deploy` | One-step deployment to popular platforms. |
| `frame plugin:init` | Create new plugin scaffolds. |
| `frame agent` | AI helper interface for build and debugging automation. |

---

**End of Document 02**

