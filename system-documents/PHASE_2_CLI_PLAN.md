# Phase 2: Frame CLI Implementation Plan

**Branch:** `feature/phase-2-cli`
**Start Date:** 2025-01-15
**Target Completion:** 2-3 weeks
**Quality Standard:** 100% test coverage

---

## Overview

Phase 2 focuses on implementing the Frame CLI, which serves as the primary entry point for developers using the Frame Framework. The CLI will handle project creation, building, serving, and database management.

**Reference Specification:** `/documents/specification/02_frame_cli.md`

---

## Goals

1. **Unified workflow** - Single tool for all Frame operations
2. **Automation-ready** - CI/CD compatible
3. **Developer clarity** - Clear, concise commands
4. **Consistency** - Shared argument patterns
5. **Extensibility** - Plugin support for additional commands

---

## Architecture

### CLI Structure

```
frame-cli/
├── src/
│   ├── main.rs                 # Entry point, argument parsing
│   ├── lib.rs                  # Public API
│   ├── commands/
│   │   ├── mod.rs              # Command registry
│   │   ├── new.rs              # Project creation
│   │   ├── build.rs            # Build command
│   │   ├── serve.rs            # Development server
│   │   ├── db.rs               # Database commands (plan, migrate, seed)
│   │   ├── api.rs              # API tools (spec, sdk)
│   │   └── platform.rs         # Platform commands (pwa, mobile, desktop, server)
│   ├── compiler/
│   │   ├── mod.rs              # Clean compiler integration
│   │   ├── invoke.rs           # Subprocess management
│   │   └── errors.rs           # Error parsing
│   ├── templates/
│   │   ├── mod.rs              # Template engine
│   │   ├── project.rs          # Project templates
│   │   └── files.rs            # File generation
│   ├── watcher/
│   │   ├── mod.rs              # File watcher
│   │   └── events.rs           # Change events
│   └── server/
│       ├── mod.rs              # Dev server
│       ├── livereload.rs       # WebSocket live reload
│       └── static_files.rs     # Static file serving
├── templates/
│   ├── full-stack/             # Full-stack template
│   ├── api-only/               # API-only template
│   └── ui-only/                # UI-only template
└── tests/
    ├── integration/
    │   ├── new_test.rs
    │   ├── build_test.rs
    │   ├── serve_test.rs
    │   └── db_test.rs
    └── unit/
        ├── compiler_test.rs
        ├── templates_test.rs
        └── watcher_test.rs
```

---

## Commands to Implement

### 1. `frame new <name>` - Project Creation

**Priority:** HIGH
**Complexity:** MEDIUM
**Estimated Time:** 2-3 days

**Features:**
- Create project directory structure
- Generate config files (app.cln, database.cln, ui.cln)
- Generate initial schema (db/schema.cln)
- Generate package.frame.toml
- Create .gitignore and README.md
- Support multiple templates (--template flag)

**Templates:**
- `full` - Full-stack app (default)
- `api` - API-only backend
- `ui` - UI-only frontend

**Directory Structure:**
```
myapp/
├── app/
│   ├── pages/
│   │   └── index.cln
│   ├── api/
│   │   └── hello.cln
│   └── components/
│       └── layout.cln
├── db/
│   └── schema.cln
├── config/
│   ├── app.cln
│   ├── database.cln
│   └── ui.cln
├── public/
│   ├── css/
│   ├── js/
│   └── icons/
├── .frame/                     # Build cache
├── .gitignore
├── README.md
└── package.frame.toml
```

**Tests:**
- [ ] Create project with default template
- [ ] Create project with api template
- [ ] Create project with ui template
- [ ] Reject invalid project names
- [ ] Handle existing directory
- [ ] Verify all files created
- [ ] Verify file permissions
- [ ] Verify file contents

---

### 2. `frame build` - Production Build

**Priority:** HIGH
**Complexity:** HIGH
**Estimated Time:** 4-5 days

**Features:**
- Invoke Clean compiler for .cln → .wasm compilation
- Bundle multiple WASM modules
- Copy static assets to dist/
- Generate build manifest
- Support multiple targets (--target flag)
- Incremental builds
- Error parsing and reporting

**Build Targets:**
- `web` - Static web build
- `pwa` - Progressive Web App
- `mobile` - Mobile (Capacitor)
- `desktop` - Desktop (Tauri)
- `server` - Server deployment
- `cli` - CLI application

**Build Output:**
```
dist/
├── server.wasm
├── ui.wasm
├── static/
│   ├── css/
│   ├── js/
│   └── icons/
├── config.json
└── manifest.json
```

**Tests:**
- [ ] Build simple project
- [ ] Build multi-module project
- [ ] Build with static assets
- [ ] Build for all targets
- [ ] Incremental builds (only changed files)
- [ ] Handle compilation errors
- [ ] Handle missing files
- [ ] Handle invalid .cln syntax
- [ ] Clean compiler not found
- [ ] Output directory creation

---

### 3. `frame serve` - Development Server

**Priority:** HIGH
**Complexity:** HIGH
**Estimated Time:** 4-5 days

**Features:**
- Start HTTP server on localhost:8080
- Auto-compile on file changes
- Live reload via WebSocket
- Serve static files from public/
- Error overlay in browser
- Request logging

**Live Reload:**
- Watch .cln files
- Watch public/ directory
- Detect changes and trigger recompilation
- Send reload signal to browser via WebSocket
- Maintain state when possible

**Tests:**
- [ ] Start server on default port
- [ ] Start server on custom port
- [ ] Serve static files
- [ ] Auto-recompile on .cln change
- [ ] Auto-reload on static file change
- [ ] WebSocket connection
- [ ] Error overlay display
- [ ] Request logging
- [ ] Graceful shutdown

---

### 4. `frame db:*` - Database Commands

**Priority:** MEDIUM
**Complexity:** MEDIUM
**Estimated Time:** 3-4 days

#### `db:plan`

Display pending migrations based on schema changes.

**Features:**
- Compare current schema.cln with database state
- Generate migration plan (SQL operations)
- Show up/down migrations
- Dry-run mode (no changes)

#### `db:migrate`

Apply pending migrations to database.

**Features:**
- Connect to database (via config/database.cln)
- Read migration history
- Apply pending migrations
- Track applied migrations
- Rollback on error

#### `db:seed`

Run database seeding scripts.

**Features:**
- Execute db/seed.cln
- Insert test/development data
- Handle foreign key dependencies
- Rollback on error

**Tests:**
- [ ] db:plan shows pending migrations
- [ ] db:plan with no changes
- [ ] db:migrate applies migrations
- [ ] db:migrate skips applied migrations
- [ ] db:migrate rollback on error
- [ ] db:seed inserts data
- [ ] db:seed handles FK constraints
- [ ] Database connection errors
- [ ] Invalid schema.cln
- [ ] Missing config/database.cln

---

### 5. `frame api:*` - API Tools

**Priority:** LOW
**Complexity:** MEDIUM
**Estimated Time:** 3-4 days

#### `api:spec`

Generate OpenAPI 3.1 specification from endpoints.

**Features:**
- Discover all endpoints in app/api/
- Extract types from endpoint definitions
- Generate OpenAPI schema
- Output JSON or YAML
- Include request/response examples

#### `api:sdk`

Generate type-safe client SDKs.

**Features:**
- Generate TypeScript SDK
- Generate Swift SDK (future)
- Generate Kotlin SDK (future)
- Type mapping (Clean → target language)
- HTTP client generation
- Error handling

**Tests:**
- [ ] api:spec generates valid OpenAPI
- [ ] api:spec includes all endpoints
- [ ] api:spec includes all types
- [ ] api:sdk generates TypeScript
- [ ] Generated SDK compiles
- [ ] Missing endpoints directory

---

### 6. `frame <platform>:init` - Platform Commands

**Priority:** LOW
**Complexity:** LOW
**Estimated Time:** 2-3 days

#### `pwa:init`

**Features:**
- Generate manifest.json
- Generate service worker template
- Generate icons (multiple sizes)
- Update index.html

#### `mobile:init`

**Features:**
- Generate Capacitor config
- Scaffold iOS project
- Scaffold Android project
- Add default plugins

#### `desktop:init`

**Features:**
- Generate Tauri config
- Scaffold Rust backend
- Configure allowlist
- Add window configuration

#### `server:init`

**Features:**
- Generate Dockerfile
- Generate docker-compose.yml
- Add health endpoints
- Generate .env template

**Tests:**
- [ ] pwa:init generates all files
- [ ] mobile:init scaffolds projects
- [ ] desktop:init configures Tauri
- [ ] server:init generates Docker files
- [ ] Verify generated config validity

---

## Implementation Order

### Week 1: Core Infrastructure

**Days 1-2:** CLI Framework
- Set up clap argument parsing
- Global flags (--verbose, --version, --help, --target)
- Command registry
- Error handling and reporting
- JSON output mode

**Days 3-4:** `frame new` Command
- Template engine
- Project directory creation
- File generation from templates
- All three templates (full, api, ui)
- Complete tests

**Day 5:** `frame new` Polish
- Error handling
- Edge cases
- Documentation
- Integration tests

---

### Week 2: Build System

**Days 1-3:** `frame build` Command
- Clean compiler integration
- WASM compilation
- Module bundling
- Asset copying
- Build manifest generation

**Days 4-5:** `frame serve` Command
- HTTP server
- Static file serving
- Basic request handling
- File watcher implementation
- Auto-recompile

---

### Week 3: Development Tools

**Days 1-2:** `frame serve` Polish
- Live reload (WebSocket)
- Error overlay
- Request logging
- Tests

**Days 3-5:** Database Commands
- `db:plan` implementation
- `db:migrate` implementation
- `db:seed` implementation
- Database connection handling
- Migration tracking
- Tests

---

### Week 4 (if needed): API & Platform Tools

**Days 1-2:** API Tools
- `api:spec` implementation
- OpenAPI generation
- Tests

**Days 3-5:** Platform Tools
- `pwa:init`
- `mobile:init`
- `desktop:init`
- `server:init`
- Tests

---

## Dependencies

### Rust Crates

```toml
[dependencies]
clap = { version = "4.0", features = ["derive", "cargo"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"                    # Web server for `serve`
tower = "0.4"                    # Middleware
tower-http = "0.5"               # Static files, CORS
notify = "6.0"                   # File watcher
tungstenite = "0.21"             # WebSocket (live reload)
tera = "1.19"                    # Template engine
walkdir = "2.4"                  # Directory traversal
console = "0.15"                 # Terminal colors/formatting
indicatif = "0.17"               # Progress bars
dialoguer = "0.11"               # Interactive prompts
sqlx = { version = "0.7", features = ["postgres", "mysql", "sqlite", "runtime-tokio"] }
```

### External Tools

- **Clean Language Compiler** (cln) - Must be in PATH
- **wasm-opt** (optional) - For WASM optimization
- **Docker** (optional) - For server:init testing

---

## Testing Strategy

### Unit Tests

- All modules in src/ have corresponding tests
- Test all edge cases (invalid input, missing files, etc.)
- Mock external dependencies (compiler, file system)

### Integration Tests

- Test full command workflows
- Use temporary directories
- Test with real Clean compiler (if available)
- Test error scenarios

### Coverage Target

- **100% code coverage** for all modules
- Use `cargo-tarpaulin` or `cargo-llvm-cov`
- Fail CI if coverage drops below 95%

---

## Error Handling

### Error Codes

```rust
pub enum CliError {
    CompilerNotFound,           // COMPILER_NOT_FOUND
    CompilationFailed,          // COMPILATION_FAILED
    InvalidProjectName,         // INVALID_PROJECT_NAME
    ProjectAlreadyExists,       // PROJECT_EXISTS
    MissingConfig,              // CONFIG_NOT_FOUND
    InvalidConfig,              // CONFIG_INVALID
    DatabaseConnectionFailed,   // DB_CONNECTION_FAILED
    MigrationFailed,            // MIGRATION_FAILED
    TemplateNotFound,           // TEMPLATE_NOT_FOUND
    IoError(std::io::Error),    // IO_ERROR
}
```

### Error Format

```json
{
  "ok": false,
  "err": {
    "code": "COMPILATION_FAILED",
    "message": "Syntax error in app/pages/index.cln:12",
    "details": {
      "file": "app/pages/index.cln",
      "line": 12,
      "column": 5,
      "hint": "Expected ';' after statement"
    }
  }
}
```

---

## Success Criteria

### Functionality

- [ ] All commands implemented and tested
- [ ] 100% test coverage across all modules
- [ ] Integration with Clean compiler
- [ ] Error handling for all scenarios
- [ ] JSON output mode working

### Performance

- [ ] `frame new` completes in <2s
- [ ] `frame build` compiles 1000 LOC in <5s
- [ ] `frame serve` hot reload in <100ms
- [ ] File watcher latency <50ms

### Quality

- [ ] All tests passing
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] README examples work

---

## Documentation

### CLI Help Text

```bash
$ frame --help
Frame - Full-Stack Framework for Clean Language

USAGE:
    frame <COMMAND>

COMMANDS:
    new         Create a new Frame project
    serve       Start development server with live reload
    build       Build project for deployment
    db:plan     Show pending database migrations
    db:migrate  Apply pending migrations
    db:seed     Run database seeding scripts
    api:spec    Generate OpenAPI specification
    api:sdk     Generate client SDK
    pwa:init    Initialize PWA support
    mobile:init Initialize mobile project
    desktop:init Initialize desktop project
    server:init Initialize server deployment
    help        Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
        --verbose    Enable verbose output
```

### README Examples

```bash
# Create new project
frame new myapp
cd myapp

# Start development server
frame serve

# Build for production
frame build --target=server

# Database commands
frame db:plan
frame db:migrate
frame db:seed
```

---

## Risk Mitigation

### Compiler Integration

**Risk:** Clean compiler not stable yet

**Mitigation:**
- Mock compiler interface for testing
- Provide clear error messages
- Document compiler version requirements

### File Watcher Reliability

**Risk:** File watcher may miss changes

**Mitigation:**
- Use robust library (notify)
- Add manual reload command
- Test on all platforms

### WebSocket Live Reload

**Risk:** Connection drops, state loss

**Mitigation:**
- Automatic reconnection
- Fallback to manual reload
- State preservation when possible

---

## Next Steps After Phase 2

Once Frame CLI is complete:

1. **Phase 2.1: Frame Server** - WASM runtime, routing, request handling
2. **Phase 2.2: Frame Data** - ORM, query builder, migrations
3. **Phase 2.3: Frame UI** - Component system, SSR, hydration

---

**Status:** Ready to begin
**Branch:** feature/phase-2-cli
**First Task:** Set up clap CLI framework
