# CLAUDE.md - Frame Framework Development Guide

This file provides guidance when working with the Frame Framework codebase. The Frame Framework is the official full-stack framework for Clean Language, compiling entirely to WebAssembly (WASM).

## Project Overview

Frame is a modern, full-stack web framework that unifies frontend, backend, and data layers into a single, type-safe programming model. It embodies the Clean Language philosophy: simple, declarative, and transparent code.

### Core Principles

- **Type-Safe Full Stack**: End-to-end type safety from database to UI
- **WebAssembly Native**: Compiles to WASM for predictable performance
- **Secure by Default**: Sandboxed execution with clear Host Bridge boundaries
- **Universal Runtime**: One codebase runs on web, mobile, desktop, and server
- **Zero Boilerplate**: Minimal, declarative syntax
- **SSR + Islands**: Server-side rendering with selective client hydration

## Documentation Structure

### Core Specifications

All specifications are located in `/documents/specification/`:

1. **[01_frame_overview.md](documents/specification/01_frame_overview.md)** - High-level architecture, philosophy, and core concepts
2. **[02_frame_cli.md](documents/specification/02_frame_cli.md)** - CLI commands (v1 deprecated, v2 migration guide)
3. **[03_frame_server.md](documents/specification/03_frame_server.md)** - Server runtime, HTTP APIs using `endpoints:` blocks, and Host Bridge
4. **[04_frame_data.md](documents/specification/04_frame_data.md)** - ORM system with block-based queries, migrations, and many-to-many relationships
5. **[05_frame_ui.md](documents/specification/05_frame_ui.md)** - UI components, SSR/CSR, hydration strategies, and theming
6. **[06_frame_auth.md](documents/specification/06_frame_auth.md)** - Authentication (sessions, JWT), authorization (roles, permissions)
7. **[07_frame_plugins.md](documents/specification/07_frame_plugins.md)** - Plugin system, lifecycle hooks, and extensibility
8. **[08_frame_platforms.md](documents/specification/08_frame_platforms.md)** - Multi-platform deployment (Web, PWA, Mobile, Desktop, Server, CLI)
9. **[09_frame_dev_guidelines.md](documents/specification/09_frame_dev_guidelines.md)** - Coding standards, naming conventions, and best practices
10. **[10_compiler_plugins.md](documents/specification/10_compiler_plugins.md)** - Compiler plugin architecture (Clean Language plugins)
11. **[11_database_plugins.md](documents/specification/11_database_plugins.md)** - Database plugin architecture, C-ABI interface, runtime drivers
12. **[12_frame_canvas.md](documents/specification/12_frame_canvas.md)** - Canvas rendering, animation, drawing primitives, bridge functions
13. **[13_frame_future_evolution.md](documents/specification/13_frame_future_evolution.md)** - Roadmap, research directions, versioning policy

### Reference Documents

- **[frame_bridge_contracts.md](documents/specification/frame_bridge_contracts.md)** - Host Bridge JSON contracts for system integration
- **[frame_internal_map.md](documents/specification/frame_internal_map.md)** - Document index and module relationship map

### Guides

- **[GETTING_STARTED.md](documents/GETTING_STARTED.md)** - Quick start tutorial for new users
- **[PROJECT_STRUCTURE.md](documents/PROJECT_STRUCTURE.md)** - Folder conventions and file discovery
- **[API_REFERENCE.md](documents/API_REFERENCE.md)** - API quick reference cheat sheet
- **[PLUGIN_GUIDE.md](documents/PLUGIN_GUIDE.md)** - Plugin system from the user's perspective

### Parent-Level Documentation

For project-wide docs (architecture, contributing, platform), see the parent folder:
- **[platform-architecture/](../platform-architecture/)** - Execution layers, Host Bridge, memory model
- **[README.md](../README.md)** - Project overview
- **[CLAUDE.md](../CLAUDE.md)** - Cross-component work policy

## Language Specification

**CRITICAL**: All Frame code MUST comply with the [Clean Language Specification](/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/documentation/Clean_Language_Specification.md).

Key Clean Language rules for Frame:

- **Functions blocks required**: All functions except `start()` must be in `functions:` blocks
- **Method calls require parentheses**: `value.toString()` not `value.toString`
- **Use lowercase namespaces**: `math.sqrt()` not `Math.sqrt()`
- **Generic type is `any`**: No explicit type parameters, use `any` for generics
- **Indentation uses tabs only**: No spaces for indentation
- **Block-based syntax**: `endpoints:`, `data:`, `functions:`, `where:`, `order:`

## Development Rules

### Code Quality Standards

1. **NO PLACEHOLDER IMPLEMENTATIONS**: Never create functions that return dummy values. All code must be fully functional.
2. **NO FALLBACK IMPLEMENTATIONS**: Avoid temporary simplified implementations.
3. **WORKING CODE ONLY**: All code must be production-ready and functional.
4. **REFERENCE SPECIFICATIONS**: Always consult the relevant specification document before implementing.

### Folder Conventions (CRITICAL)

**Files are ONLY recognized and processed by plugins based on their folder location.**

A file outside its expected folder will NOT be compiled by the appropriate plugin. This is the core architectural principle.

#### Standard Project Structure

```
app/
├── pages/              # frame.ui → SSR pages (standard HTML)
│   ├── index.html
│   ├── about.html
│   └── blog/
│       └── [slug].html
│
├── components/         # frame.ui → Reusable UI components
│   ├── Header.cln
│   └── Footer.cln
│
├── backend/            # frame.httpserver → HTTP server
│   ├── api/            # API endpoints
│   │   ├── users.cln
│   │   └── posts.cln
│   ├── services/       # Business logic
│   └── middleware/      # Middleware
│
├── data/               # frame.data → Data models/ORM
│   ├── models/
│   │   ├── User.cln
│   │   └── Post.cln
│   ├── queries/
│   ├── migrations/
│   └── repositories/
│
├── auth/               # frame.auth → Auth configuration
│   └── auth.cln
│
├── canvas/             # frame.canvas → Canvas applications
│   ├── scenes/
│   ├── sprites/
│   └── audio/
│
└── public/             # Static assets (served as-is)
    ├── css/
    │   └── style.css
    ├── js/
    └── images/
```

#### File Extensions by Content Type

| Content Type | Extension | Folder | Editor Support |
|-------------|-----------|--------|----------------|
| SSR Pages | `.html` | `pages/` | Full (any HTML editor) |
| Stylesheets | `.css` | `public/css/` | Full (any CSS editor) |
| Components | `.cln` | `components/` | Clean Language |
| API Endpoints | `.cln` | `backend/api/` | Clean Language |
| Data Models | `.cln` | `data/models/` | Clean Language |
| Auth Config | `.cln` | `auth/` | Clean Language |
| Canvas Apps | `.cln` | `canvas/` | Clean Language |

#### Key Rules

1. **Standard HTML for pages** - Use `.html` extension for SSR pages. This ensures full editor support (syntax highlighting, Emmet, autocomplete, Prettier).

2. **Standard CSS for styles** - Use `.css` extension in `public/css/`. No inline `<style>` tags in HTML pages. Styles must be in separate CSS files.

3. **Clean templating in HTML** - Use `{{ variable }}` for interpolation and `cl-*` attributes for directives:
   ```html
   <h1>Welcome, {{ user.name }}</h1>
   <ul cl-iterate="post in posts">
       <li>{{ post.title }}</li>
   </ul>
   <div cl-if="user.isAdmin">Admin Panel</div>
   ```

4. **Folder Ownership (Plugin Declaration Required)** - Plugins must always be declared in `app.cln` via the `plugins:` block. Each plugin declares folder ownership in `plugin.toml`. When `implicit_import = true`, files in owned folders don't need explicit import statements — the declared plugin processes them automatically:
   - `backend/api/users.cln` → processed by `frame.httpserver`
   - `data/models/User.cln` → processed by `frame.data`
   - `auth/auth.cln` → processed by `frame.auth`
   - `canvas/scenes/game.cln` → processed by `frame.canvas`
   - `components/Button.cln` → processed by `frame.ui`

   **Folder Ownership Rules (per plugin.toml `[paths]` section):**
   | Path Pattern | Owning Plugin |
   |--------------|---------------|
   | `app/backend/`, `app/backend/api/`, `app/backend/services/`, `app/backend/middleware/` | `frame.httpserver` |
   | `app/data/`, `app/data/models/`, `app/data/queries/`, `app/data/migrations/` | `frame.data` |
   | `app/auth/` | `frame.auth` |
   | `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` | `frame.canvas` |
   | `app/pages/`, `app/components/`, `app/layouts/` | `frame.ui` |

5. **No CSS in HTML** - Inline styles are prohibited. All CSS must be in `public/css/` and linked via `<link>` tags.

### Examples and Demos Policy

## ABSOLUTE PROHIBITION - READ CAREFULLY

**NEVER, UNDER ANY CIRCUMSTANCES, CREATE JAVASCRIPT CODE TO SIMULATE CLEAN LANGUAGE BEHAVIOR.**

This is a HARD RULE with NO EXCEPTIONS. Violations include:

- Writing `<script>` tags in HTML files that simulate what Clean would do
- Creating "demo" or "preview" JavaScript that mimics plugin functionality
- Adding JavaScript "for now" or "temporarily" until the compiler works
- Using localStorage, fetch simulations, or any JS to fake database/auth/canvas behavior
- Creating "working mockups" or "visual previews" with JavaScript logic

### What To Do Instead

1. **If the compiler/server can't run the code yet**:
   - Create ONLY the `.cln` source files with correct syntax
   - Create ONLY static `.html` templates WITHOUT any `<script>` tags
   - Document what the example WILL do when compiled, don't simulate it

2. **If asked to create a "working demo"**:
   - REFUSE if it requires JavaScript simulation
   - Explain that demos must compile and run on actual Clean infrastructure
   - Offer to implement the missing compiler/server functionality instead

3. **If asked to "make it work in the browser"**:
   - REFUSE if it means adding JavaScript
   - Static HTML/CSS only - no dynamic behavior without Clean compilation

### Valid Example Structure

```
example/
├── app.cln                 # Clean config (NO JS)
├── app/
│   ├── pages/
│   │   └── index.html      # Static HTML, NO <script> tags
│   ├── data/
│   │   └── models/
│   │       └── User.cln    # Clean model (NO JS)
│   └── backend/
│       └── api/
│       └── users.cln       # Clean endpoints (NO JS)
└── public/
    └── css/
        └── style.css       # CSS only (NO JS)
```

### Invalid - NEVER DO THIS

```html
<!-- WRONG - This violates the directive -->
<script>
  // Simulating Clean behavior with JavaScript
  const users = JSON.parse(localStorage.getItem('users'));
  function loadUsers() { ... }
</script>
```

### Why This Rule Exists

1. JavaScript simulations LIE about framework capabilities
2. They create FALSE CONFIDENCE in untested code
3. They DIVERGE from actual behavior over time
4. They WASTE TIME building throwaway code
5. They DELAY actual compiler/server implementation
6. They CONFUSE users about what Clean Language actually does

### Enforcement

Before creating ANY file, ask yourself:
- Does this file contain `<script>` tags? → **STOP, DON'T CREATE IT**
- Does this file contain JavaScript logic? → **STOP, DON'T CREATE IT**
- Am I "simulating" or "mocking" Clean behavior? → **STOP, DON'T CREATE IT**
- Could this be misunderstood as "working" when it's not? → **STOP, DON'T CREATE IT**

**If in doubt, create .cln files only and leave HTML as pure static templates.**

### Module-Specific Rules

#### Frame CLI (`frame-cli/`)
Reference: [02_frame_cli.md](documents/specification/02_frame_cli.md)
- Implement all commands as specified (`new`, `serve`, `build`, `db:*`, `api:*`, `mobile:init`, etc.)
- Use JSON output format for machine-readable results
- Provide clear error messages with error codes
- Support `--verbose`, `--target`, and other global flags

#### Frame Server (`frame-server/`)
Reference: [03_frame_server.md](documents/specification/03_frame_server.md)
- Use `endpoints:` blocks for all HTTP APIs
- Support declarative sub-blocks: `guard:`, `returns:`, `cache:`, `handle:`
- Implement standard response helpers: `json()`, `html()`, `redirect()`, `notFound()`, etc.
- Use Host Bridge for all I/O operations

#### Frame Data (`frame-data/`)
Reference: [04_frame_data.md](documents/specification/04_frame_data.md)
- Use `data` keyword for model definitions
- Support block-based queries: `find:`, `where:`, `order:`, `limit:`
- Implement many-to-many via explicit junction tables
- Auto-generate migrations from schema diffs
- Support transactions with `Data.tx:` blocks

#### Frame UI (`frame-ui/`)
Reference: [05_frame_ui.md](documents/specification/05_frame_ui.md)
- Use `component` keyword for UI components
- Default to SSR, opt-in to client hydration
- Support hydration strategies: `client="off|on|visible|idle|only"`
- Escape HTML by default, only use `rawHtml()` for trusted content

#### Frame Auth (`frame-auth/`)
Reference: [06_frame_auth.md](documents/specification/06_frame_auth.md)
- Support both session-based and JWT authentication
- Use HTTP-only, SameSite cookies for sessions
- Implement role-based access control
- Provide CSRF protection by default

#### Host Bridge (`host-bridge/`)
Reference: [frame_bridge_contracts.md](documents/specification/frame_bridge_contracts.md)
- Use standard envelope: `{"fn": "bridge:namespace.function", "args": {...}}`
- Return `{"ok": true, "data": {...}}` or `{"ok": false, "err": {...}}`
- Implement all namespaces: `http`, `db`, `env`, `time`, `crypto`, `log`, `fs`, `sys`
- Use standard error codes: `DB_ERROR`, `AUTH_ERROR`, `NETWORK_FAIL`, etc.

### Testing Strategy

Reference: [09_frame_dev_guidelines.md](documents/specification/09_frame_dev_guidelines.md)

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test module interactions
3. **E2E Tests**: Test full application flows

Test location: `/tests/<module>/`

**Testing Philosophy**:
- 100% test coverage for critical paths
- Test both success and error cases
- Use mock Host Bridge for testing WASM modules
- Snapshot testing for UI components

### Error Handling

All errors must follow the standard envelope format:

```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable description",
    "details": {}
  }
}
```

Standard error codes:
- `BUILD_FAIL` - Compilation errors
- `DB_ERROR` - Database operations
- `AUTH_ERROR` - Authentication failures
- `NETWORK_FAIL` - Network/HTTP errors
- `VALIDATION_ERROR` - Data validation
- `NOT_FOUND` - Resource not found
- `PERMISSION_DENIED` - Authorization failures

### Naming Conventions

**Clean Language**:
- Classes/Components: `PascalCase` (e.g., `UserProfile`, `BlogPost`)
- Functions/Variables: `camelCase` (e.g., `getUserById`, `userCount`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- Files: `PascalCase.cln` for components, `kebab-case` for others

**Rust**:
- Types: `PascalCase` (e.g., `UserProfile`)
- Functions: `snake_case` (e.g., `get_user_count`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)

### Commit Standards

Follow Conventional Commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Examples**:
- `feat(orm): add many-to-many relationship support`
- `fix(compiler): resolve type inference for nested generics`
- `docs(api): update endpoint documentation`

## Project Structure

```
clean-framework/
├── plugins/                 # Core framework plugins (Clean Language)
│   ├── frame.auth/          # Authentication & authorization plugin
│   ├── frame.canvas/        # Canvas rendering & game dev plugin
│   ├── frame.data/          # ORM & database plugin
│   ├── frame.httpserver/    # HTTP server & routing plugin
│   └── frame.ui/            # UI components & SSR plugin
├── documents/
│   ├── specification/       # All specification files (01-13)
│   ├── API_REFERENCE.md     # Quick API reference
│   ├── GETTING_STARTED.md   # Quick start tutorial
│   ├── PLUGIN_GUIDE.md      # Plugin system guide
│   └── PROJECT_STRUCTURE.md # Folder conventions
├── examples/                # Example Frame applications
├── tests/                   # Test suites
├── scripts/                 # Build and test scripts
├── system-documents/        # Internal development docs
├── CLAUDE.md                # This file
├── README.md                # Project overview
└── TASKS.md                 # Development task tracker
```

## Development Workflow

1. **Read relevant specification**: Always start by reading the spec document
2. **Create tasks**: Add tasks to TASKS.md before implementation
3. **Write tests first**: TDD approach for new features
4. **Implement**: Follow specifications exactly
5. **Test**: Run all tests and verify
6. **Document**: Update docs if needed
7. **Commit**: Use conventional commit format

## Host Bridge Development

The Host Bridge is the ONLY interface between WASM and system resources.

**Key principles**:
- All bridge calls use JSON envelopes
- Responses use standard `ok`/`err` format
- Bridge functions are stateless
- Security enforced through allowlists

**Namespaces to implement**:
1. `bridge:http` - HTTP requests and responses
2. `bridge:db` - Database operations
3. `bridge:env` - Environment variables
4. `bridge:time` - Time operations
5. `bridge:crypto` - Cryptographic operations
6. `bridge:log` - Structured logging
7. `bridge:fs` - Filesystem (desktop/CLI only)
8. `bridge:sys` - System information

## WAT Spec Compliance (Host Function Signatures)

**IMPORTANT: Host function signatures are enforced by a machine-checkable WAT contract.**

The clean-server enforces all host function signatures via `clean-server/host-bridge/tests/spec_compliance.wat`. This file declares every host function import with its exact WASM signature. Any WASM module the framework generates or interacts with must match these signatures exactly.

### Key Conventions

1. **ALL string input parameters** use raw `(ptr: i32, len: i32)` pairs (NOT length-prefixed single pointers)
2. **Return strings** use length-prefixed format: `[4-byte LE length][UTF-8 data]`
3. **Integer values** use `i64` (not `i32`) for: `print_integer`, `int_to_string`, `string_to_int`
4. When the framework generates or references host function calls, signatures must match the WAT spec

### Authoritative References

- WAT spec guard: `clean-server/host-bridge/tests/spec_compliance.wat`
- Host Bridge documentation: `platform-architecture/HOST_BRIDGE.md`

## Security Requirements

1. **WASM Sandboxing**: All application code runs in isolated WASM context
2. **Host Bridge Allowlisting**: Explicit permissions for system access
3. **HTTPS Only**: Enforce HTTPS in production
4. **Secure Cookies**: HTTP-only, SameSite, Secure flags
5. **CSRF Protection**: Built-in for POST/PUT/PATCH/DELETE
6. **SQL Injection Prevention**: Parameterized queries only
7. **XSS Prevention**: Automatic HTML escaping
8. **Input Validation**: Both compile-time and runtime

## Performance Targets

**Compilation**:
- < 1s per 1000 LOC
- < 100ms incremental rebuild
- < 500MB memory usage

**Runtime**:
- < 50ms first request (SSR page)
- < 10ms p95 API latency (simple endpoint)
- < 100ms p95 API latency (with database)
- > 10k req/sec throughput (simple endpoints)

## AI Development Notes

This framework is designed for AI-assisted development:

- **Deterministic compilation**: Same input → same output
- **Typed interfaces**: All contracts are explicitly typed
- **Clear module boundaries**: Each module has single responsibility
- **JSON-based communication**: Machine-readable formats
- **Comprehensive specs**: Every feature is documented

When prompting AI assistants:
1. Always include relevant specification document(s)
2. Reference the Clean Language Specification
3. Use `--json` flags for machine-readable output
4. Provide full context from multiple spec files if needed

## Common Commands

```bash
# Build the compiler and CLI
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Create a new Frame project
./target/release/frame-cli new myapp

# Start development server
./target/release/frame-cli serve

# Build for production
./target/release/frame-cli build --target=server

# Run database migrations
./target/release/frame-cli db:migrate
```

## Getting Help

1. Check the relevant specification document first
2. Review the specification files in `documents/specification/`
3. Look at examples in `/examples/`
4. Search existing issues on GitHub
5. Ask in GitHub Discussions

## Quality Assurance

**Before committing**:
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] TASKS.md is updated (tasks completed/added)
- [ ] Follows Clean Language Specification
- [ ] Follows Frame specifications

## Version Compatibility

Frame follows semantic versioning (semver):
- **Major** (2.0, 3.0): Breaking changes allowed
- **Minor** (1.1, 1.2): New features, backward compatible
- **Patch** (1.0.1, 1.0.2): Bug fixes only

Breaking changes require:
1. Advance notice (at least 2 minor versions)
2. Deprecation warnings
3. Migration documentation
4. Automated migration tools (when possible)

## Success Metrics

Track these metrics:
- Compilation time per 1000 LOC
- Test coverage percentage
- API latency (p50, p95, p99)
- Memory usage
- Throughput (req/sec)

---

**Remember**: Frame is not just a framework—it's a unified programming model. Every decision should align with the Clean Language philosophy of simplicity, type safety, and transparency.

For detailed implementation guidance, always refer to the specification documents linked above.

## Cross-Component Work Policy

**CRITICAL: AI Instance Separation of Concerns**

When working in this component and discovering errors, bugs, or required changes in **another component** (different folder in the Clean Language project), you must **NOT** directly fix or modify code in that other component.

Instead:

1. **Document the issue** by creating a prompt/task description
2. **Save the prompt** in a file that can be executed by the AI instance working in the correct folder
3. **Location for cross-component prompts**: Save prompts in `../system-documents/cross-component-prompts/` at the project root

### Prompt Format for Cross-Component Issues

```
Component: [target component name, e.g., clean-language-compiler]
Issue Type: [bug/feature/enhancement/compatibility]
Priority: [critical/high/medium/low]
Description: [Detailed description of the issue discovered]
Context: [Why this was discovered while working in the current component]
Suggested Fix: [If known, describe the potential solution]
Files Affected: [List of files in the target component that need changes]
```

### Why This Rule Exists

- Each component has its own context, dependencies, and testing requirements
- AI instances are optimized for their specific component's codebase
- Cross-component changes without proper context can introduce bugs
- This maintains clear boundaries and accountability
- Ensures changes are properly tested in the target component's environment

### What You CAN Do

- Read files from other components to understand interfaces
- Document compatibility issues found
- Create detailed prompts for the correct AI instance
- Update your component to work with existing interfaces

### What You MUST NOT Do

- Directly edit code in other components
- Make changes to other components' configuration files
- Modify shared resources without coordination
- Skip the prompt creation step for cross-component issues
