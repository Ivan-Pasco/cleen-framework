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
2. **[02_frame_cli.md](documents/specification/02_frame_cli.md)** - CLI commands, build system, and development workflow
3. **[03_frame_server.md](documents/specification/03_frame_server.md)** - Server runtime, HTTP APIs using `endpoints:` blocks, and Host Bridge
4. **[04_frame_data.md](documents/specification/04_frame_data.md)** - ORM system with block-based queries, migrations, and many-to-many relationships
5. **[05_frame_ui.md](documents/specification/05_frame_ui.md)** - UI components, SSR/CSR, hydration strategies, and theming
6. **[06_frame_auth.md](documents/specification/06_frame_auth.md)** - Authentication (sessions, JWT), authorization (roles, permissions)
7. **[07_frame_plugins.md](documents/specification/07_frame_plugins.md)** - Plugin system, lifecycle hooks, and extensibility
8. **[08_frame_platforms.md](documents/specification/08_frame_platforms.md)** - Multi-platform deployment (Web, PWA, Mobile, Desktop, Server, CLI)
9. **[09_frame_dev_guidelines.md](documents/specification/09_frame_dev_guidelines.md)** - Coding standards, naming conventions, and best practices
10. **[frame_bridge_contracts.md](documents/specification/frame_bridge_contracts.md)** - Host Bridge JSON contracts for system integration
11. **[frame_internal_map.md](documents/specification/frame_internal_map.md)** - Document index and module relationship map

### Reference Documentation

- **[ARCHITECTURE.md](documents/ARCHITECTURE.md)** - Deep technical dive into Frame's implementation
- **[README.md](documents/README.md)** - Quick start guide and overview
- **[CONTRIBUTING.md](documents/CONTRIBUTING.md)** - Contribution guidelines and workflow
- **[ROADMAP.md](documents/ROADMAP.md)** - Future features and release timeline

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
├── frame-cli/           # CLI implementation (Rust)
├── frame-server/        # Server runtime (Rust)
├── frame-data/          # ORM layer (Rust)
├── frame-ui/            # UI components (Rust)
├── frame-auth/          # Authentication (Rust)
├── frame-plugins/       # Plugin system (Rust)
├── host-bridge/         # Host Bridge implementation (Rust)
├── documents/
│   ├── specification/   # All specification files
│   ├── ARCHITECTURE.md
│   ├── README.md
│   ├── CONTRIBUTING.md
│   └── ROADMAP.md
├── examples/            # Example Frame applications
├── tests/               # Test suites
└── CLAUDE.md           # This file
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

Reference: [ARCHITECTURE.md](documents/ARCHITECTURE.md)

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
2. Review [ARCHITECTURE.md](documents/ARCHITECTURE.md) for technical details
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

Reference: [ROADMAP.md](documents/ROADMAP.md)

Track these metrics:
- Compilation time per 1000 LOC
- Test coverage percentage
- API latency (p50, p95, p99)
- Memory usage
- Throughput (req/sec)

---

**Remember**: Frame is not just a framework—it's a unified programming model. Every decision should align with the Clean Language philosophy of simplicity, type safety, and transparency.

For detailed implementation guidance, always refer to the specification documents linked above.
