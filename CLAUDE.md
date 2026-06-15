# CLAUDE.md - Frame Framework Development Guide

This file provides guidance when working with the Frame Framework codebase. The Frame Framework is the official full-stack framework for Clean Language, compiling entirely to WebAssembly (WASM).

**Read [KNOWLEDGE.md](./KNOWLEDGE.md) before modifying any framework code** — it documents known fragile areas, plugin expand function behavior, and plugin.toml contract rules.

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
14. **[15_frame_jobs.md](documents/specification/15_frame_jobs.md)** - Background job queue, scheduled tasks, retry policies
15. **[16_frame_locale.md](documents/specification/16_frame_locale.md)** - Internationalization — translations, plurals, locale-aware formatting
16. **[17_frame_mcp.md](documents/specification/17_frame_mcp.md)** - MCP server plugin — tools, resources, prompts over stdio and HTTP+SSE
17. **[18_frame_client.md](documents/specification/18_frame_client.md)** - Client-side communication: frame.client plugin (api.*, live.*, feed.*, `load:`/`form:`/`send:` blocks)

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
- **[platform-architecture/](../foundation/platform-architecture/)** - Execution layers, Host Bridge, memory model
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

See **[PROJECT_STRUCTURE.md](documents/PROJECT_STRUCTURE.md)** — canonical reference for folder layout, plugin ownership, and file extension conventions.

#### Key Rules

1. **Standard HTML for pages** — Use `.html` in `app/web/pages/`. Full editor support (Emmet, Prettier, syntax highlighting).

2. **Standard CSS for styles** — Use `.css` in `public/css/`. No inline `<style>` tags. No CSS in HTML files.

3. **Clean templating in HTML** — Use `{{ variable }}` for interpolation and `cl-*` attributes for directives:
   ```html
   <h1>Welcome, {{ user.name }}</h1>
   <ul cl-iterate="post in posts"><li>{{ post.title }}</li></ul>
   <div cl-if="user.isAdmin">Admin Panel</div>
   ```

4. **Folder Ownership** — Plugins must be declared in `main.cln` via the `target:` block. Files in plugin-owned folders are processed automatically (`implicit_import = true`) — no per-file imports needed. See [PROJECT_STRUCTURE.md](documents/PROJECT_STRUCTURE.md) for the ownership table.

5. **No CSS in HTML** — Inline styles are prohibited. All CSS in `public/css/`, linked via `<link>` tags.

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
├── main.cln                # Clean config (NO JS)
├── app/
│   ├── web/
│   │   └── pages/
│   │       └── index.html  # Static HTML, NO <script> tags
│   ├── data/
│   │   └── models/
│   │       └── User.cln    # Clean model (NO JS)
│   └── server/
│       └── api/
│           └── users.cln   # Clean endpoints (NO JS)
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
- Inline route modifiers in fixed order: `[roles]` guard, `cache()`, `middleware()`
- Global error handling: `server: handle:` block in the server config (catches errors from all endpoints)
- Per-statement error handling: `onError:` (core Clean Language syntax)
- `guard:`, `returns:`, `cache:`, `handle:` as sub-blocks are removed — parse error if used
- Implement standard response helpers: `json()`, `html()`, `redirect()`, `notFound()`, etc.
- Use Host Bridge for all I/O operations

#### Frame Data (`frame-data/`)
Reference: [04_frame_data.md](documents/specification/04_frame_data.md)
- Use `data` keyword for model definitions
- Support block-based queries: `find:`, `where:`, `order:`, `limit:`
- Implement many-to-many via explicit junction tables
- Auto-generate migrations from schema diffs
- Support transactions with `transaction:` blocks

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

#### Frame Canvas (`frame.canvas/`)
Reference: [12_frame_canvas.md](documents/specification/12_frame_canvas.md)
- `canvasScene:` is the root block; all sub-blocks (`assets:`, `tween`, `timeline`, `animState`, `particles`, `draw:`, event handlers, etc.) live inside it
- Canvas files go in `app/canvas/` (auto-imported when `frame.canvas` is declared in `main.cln`)
- All drawing, audio, animation, and input calls go through the Host Bridge — never write raw JS
- `draw:` is immediate-mode per-frame rendering; `onFrame:` is per-frame update logic; state lives in `state:`

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
│   ├── frame.client/        # Client-side communication (api, live, feed)
│   ├── frame.data/          # ORM & database plugin
│   ├── frame.server/        # HTTP server & routing plugin
│   └── frame.ui/            # UI components & SSR plugin
├── documents/
│   ├── specification/       # All specification files (01-14)
│   ├── API_REFERENCE.md     # Quick API reference
│   ├── GETTING_STARTED.md   # Quick start tutorial
│   ├── PLUGIN_GUIDE.md      # Plugin system guide
│   └── PROJECT_STRUCTURE.md # Folder conventions
├── examples/                # Example Frame applications
├── tests/                   # Test suites
├── scripts/                 # Build and test scripts
├── foundation/management/        # Internal development docs
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
- Host Bridge documentation: `foundation/platform-architecture/HOST_BRIDGE.md`

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

**CRITICAL: You are a Team Developer AI.** When you discover something in another component, choose the correct channel based on what you found:

| What you found | Channel | Why |
|---|---|---|
| A **bug** (crash, wrong output, spec violation, regression) | **`report_error` MCP tool** — MANDATORY | Fingerprint dedup, occurrence tracking, automatic user notification on fix, visible on errors.cleanlanguage.dev |
| A **design proposal, directive change, schema/API request, architectural ask** | Markdown file in `../foundation/management/cross-component-prompts/` | Requires discussion, not auto-fix |

**Never** write a markdown file for something that is a bug. Bug reports in markdown are invisible to the dashboard, don't notify users when fixed, and can't be queried via `list_component_bugs`.

### What You CAN Do

- Read files from other components to understand interfaces
- Call `report_error` for bugs found in other components
- Write markdown prompts for design/architecture discussions
- Update your component to work with existing interfaces

### What You MUST NOT Do

- Directly edit code in other components
- Make changes to other components' configuration files
- Write a markdown file for something that is a bug — use `report_error` instead

See `../foundation/management/USER_TYPES_AND_ERROR_REPORTING.md` for the full policy.

## Documentation Sync Protocol

Facts about the language live in `foundation/spec/` (at the project root). Facts about the platform live in `foundation/platform-architecture/`. Do not duplicate them here — link to them instead.

**When you make a change in this component, update the corresponding spec file in the same commit:**

| Change type | Update required |
|-------------|-----------------|
| New or changed plugin contract | `foundation/spec/plugins/plugin-contract.md` |
| New or changed host bridge function | `foundation/platform-architecture/HOST_BRIDGE.md` |
| New or changed execution layer | `foundation/platform-architecture/EXECUTION_LAYERS.md` |

The spec files are the single source of truth. Component documentation explains implementation — it does not redefine language rules.
