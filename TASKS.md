# Frame Framework - Development Tasks

This file tracks all development tasks for the Frame Framework project. Tasks are organized by module and priority.

**Priority Levels:**
- 🔴 **CRITICAL**: Core functionality, blocking other work
- 🟡 **MEDIUM-HIGH**: Important features with significant impact
- 🟢 **LOW**: Nice-to-have improvements and optimizations

**Status:**
- ⬜ Not Started
- 🔄 In Progress
- ✅ Completed
- ❌ Blocked
- 🔀 Moved (to another component/repo)

**Architecture Note:** The Frame Framework uses a plugin-based architecture. Plugins provide the DSL expansion layer (compiling Clean syntax into bridge function calls). The runtime bridge implementations live in `clean-server` (separate repository). Tasks below are tagged `[plugin]` or `[runtime]` to clarify ownership.

---

## Phase 1: Foundation & Core Infrastructure

### 1.1 Host Bridge Implementation (CRITICAL 🔴)

Reference: [frame_bridge_contracts.md](documents/specification/frame_bridge_contracts.md)

> **Note:** Bridge functions are *declared* in plugin.toml files and *implemented* in `clean-server`. The plugin declarations are complete. Runtime implementations are tracked in the clean-server repo.

#### HTTP Bridge (`bridge:http`) — 🔀 Tracked in clean-server
- 🔀 All HTTP bridge functions declared in `frame.server/plugin.toml`
- 🔀 Runtime implementation in `clean-server`

#### Database Bridge (`bridge:db`) — 🔀 Tracked in clean-server
- 🔀 All DB bridge functions declared in `frame.data/plugin.toml`
- 🔀 Runtime implementation in `clean-server`

#### Environment Bridge (`bridge:env`) — 🔀 Tracked in clean-server
- 🔀 Bridge function declared in `frame.auth/plugin.toml` (`_env_get`)
- 🔀 Runtime implementation in `clean-server`

#### Time Bridge (`bridge:time`) — 🔀 Tracked in clean-server
- 🔀 Bridge function declared in `frame.auth/plugin.toml` (`_time_now`)
- 🔀 Runtime implementation in `clean-server`

#### Crypto Bridge (`bridge:crypto`) — 🔀 Tracked in clean-server
- 🔀 All crypto bridge functions declared in `frame.auth/plugin.toml`
- 🔀 Runtime implementation in `clean-server`

#### Log Bridge (`bridge:log`) — 🔀 Tracked in clean-server
- 🔀 Runtime implementation in `clean-server`

#### Filesystem Bridge (`bridge:fs`) — 🔀 Tracked in clean-server
- 🔀 Runtime implementation in `clean-server`

#### System Bridge (`bridge:sys`) — 🔀 Tracked in clean-server
- 🔀 Runtime implementation in `clean-server`

---

## Phase 2: Frame CLI (CRITICAL 🔴) — 🔀 Managed by `cleen`

Reference: [02_frame_cli.md](documents/specification/02_frame_cli.md)

> **Note:** The CLI was replaced by `cleen` (the Clean Language package manager). CLI commands like `new`, `serve`, `build`, `db:*`, `api:*` are now `cleen` commands. Tracked in the clean-manager repo.

- 🔀 `cleen new` - Project scaffolding → clean-manager
- 🔀 `cleen serve` - Development server → clean-server
- 🔀 `cleen build` - Production build → clean-manager
- 🔀 `cleen db:*` - Database commands → clean-server
- 🔀 `cleen api:*` - API commands → clean-manager

---

## Phase 3: Frame Server — Plugin DSL ✅ | Runtime 🔀

Reference: [03_frame_server.md](documents/specification/03_frame_server.md)

### 3.1 WASM Runtime — 🔀 Tracked in clean-server

### 3.2 HTTP Router — 🔀 Tracked in clean-server

### 3.3 Endpoints System [plugin] ✅

- ✅ Parse `endpoints:` blocks from Clean code
- ✅ Support METHOD /path: syntax (GET, POST, PUT, DELETE, PATCH)
- ✅ Implement `guard:` sub-block for auth
- ✅ Implement `handle:` sub-block for logic
- ✅ Implement `returns:` sub-block for OpenAPI annotation
- ✅ Implement `cache:` sub-block for HTTP caching (maxAge, noCache)
- ✅ Write tests for endpoints system

### 3.4 Request/Response Helpers [plugin]

- ✅ Implement `req.params` for path parameters
- ✅ Implement `req.query` for query parameters
- ✅ Implement `req.json()` for JSON parsing
- ✅ Implement `json()` response helper (`jsonResponse`)
- ✅ Implement `html()` response helper (`htmlResponse`)
- ✅ Implement `redirect()` response helper (`redirectTo`)
- ✅ Implement `notFound()` response helper
- ✅ Implement `badRequest()` response helper
- ✅ Implement `unauthorized()` response helper
- ✅ Implement `forbidden()` response helper
- ✅ Implement `httpHeader()` response helper
- ✅ Bridge declarations: `_req_header`, `_req_headers`, `_req_body`, `_req_body_field`, `_req_form`, `_req_ip`

### 3.5 SSR Pipeline [plugin]

- ✅ Implement HTML page processing in frame.ui plugin
- ✅ Add HTML escaping by default
- ✅ Generate HTML from Clean components
- ⬜ Create islands manifest for hydration [runtime]

---

## Phase 4: Frame Data (ORM) — Plugin DSL ✅ | Runtime 🔀

Reference: [04_frame_data.md](documents/specification/04_frame_data.md)

### 4.1 Model Definitions [plugin] ✅

- ✅ Parse `data` keyword for models
- ✅ Support field types (integer, string, boolean, datetime, number)
- ✅ Implement field constraints (pk, auto, unique, default)
- ✅ Generate CREATE TABLE SQL with proper type mapping
- ✅ Write tests for model parsing

### 4.2 Relationships [plugin]

- ✅ Implement `link:` sub-block for JOIN queries
- ✅ Parse JOIN conditions with proper table aliasing
- ✅ Implement onDelete cascades in schema generation
- ✅ Write tests for relationships

### 4.3 Query Builder [plugin] ✅

- ✅ Implement `Model.find:` for SELECT queries
- ✅ Implement `where:` sub-block for filtering
- ✅ Implement `order:` sub-block for sorting
- ✅ Implement `limit:` for pagination
- ✅ Implement `Model.first:` for single records
- ✅ Implement `Model.count:` for counting
- ✅ Implement `link:` for many-to-many joins
- ✅ Write tests for query builder

### 4.4 Mutations [plugin] ✅

- ✅ Implement `Model.insert:` for INSERT (with parameterized queries)
- ✅ Implement `Model.update:` with `set:` sub-block
- ✅ Implement `Model.delete:` for DELETE
- ✅ SQL injection prevention via parameterized values
- ✅ Write tests for mutations

### 4.5 Transactions [plugin] ✅

- ✅ Implement `Data.tx:` for transactions
- ✅ Add automatic rollback on error (try/catch with _db_rollback)
- ✅ Write tests for transactions

### 4.6 Raw Queries [plugin] ✅

- ✅ Implement `db.query()` for untyped SQL
- ✅ Implement `db.queryAs(Type)` for typed SQL
- ✅ Support parameterized queries
- ✅ Write tests for raw queries

### 4.7 Migrations [plugin] ✅

- ✅ Generate `CREATE TABLE IF NOT EXISTS` via `Model.migrate()` method
- ✅ Implement schema diff via `_db_migration_diff()` bridge call (auto-diff at startup)
- ✅ Generate SQL migrations with `up:` / `down:` blocks in `migrate` block syntax
- ✅ Migration version tracking via `_db_register_migration()` bridge call
- ✅ Migration rollback via `_db_rollback_migration()` bridge call
- ✅ Migration status via `_db_migration_status()` bridge call
- ✅ Validation: requires migration name and `up:` block

### 4.8 Validation [plugin] ✅

- ✅ Add field validation rules (min, max, email, range)
- ✅ Generate `validate(field_name, value)` method per model
- ⬜ Implement runtime validation via bridge (server-side enforcement)

### 4.9 Configuration [plugin] ✅

- ✅ Implement `data:` config block (engine, host, port, database, pool)
- ✅ Support `env()` references for secrets
- ✅ Generate `_db_configure()` bridge call

---

## Phase 5: Frame UI — Plugin DSL ✅ | Runtime 🔀

Reference: [05_frame_ui.md](documents/specification/05_frame_ui.md)

### 5.1 Component System [plugin] ✅

- ✅ Parse `component` keyword
- ✅ Implement `html:` block for component rendering
- ✅ Implement `screen` blocks
- ✅ Support component registry via registry_json
- ✅ Write tests for component parsing

### 5.2 Server-Side Rendering [plugin] ✅

- ✅ Implement HTML generation from components
- ✅ Add automatic HTML escaping
- ✅ Support `{{ }}` interpolation in templates
- ✅ Support `{{{ }}}` for raw HTML
- ✅ Write tests for SSR

### 5.3 HTML-First Pages [plugin] ✅

- ✅ Process HTML pages with `process_html()` entry point
- ✅ Extract page metadata (layout, auth, roles)
- ✅ Support companion `.cln` files (load/guard)
- ✅ Transform HTML body to render functions
- ✅ Generate page classes with proper routing

### 5.4 Directives [plugin] ✅

- ✅ Implement `cl-if` directive
- ✅ Implement `cl-else` directive
- ✅ Implement `cl-iterate` directive
- ✅ Implement `cl-bind` directive
- ✅ Implement `cl-client` directive
- ✅ Implement `cl-show` directive (CSS display:none toggle)
- ✅ Implement `cl-validate` directive (email, required, min, max validation)
- ✅ Implement `cl-slot` directive (named content projection)

### 5.5 Client Hydration [plugin]

- ✅ Parse `client` attribute (off|on|visible|idle|only)
- ✅ Detect when client runtime is needed
- ✅ Generate runtime injection code
- ⬜ Full hydration strategy implementation (intersection observer, requestIdleCallback) [runtime]

### 5.6 Event Handling [plugin] ✅

- ✅ Parse event attributes (onclick, oninput, onsubmit, onchange, onfocus, onblur, onkeydown, onkeyup, onmouseenter, onmouseleave)
- ✅ Transform to `data-on-*` attributes for SSR hydration
- ✅ Implement event modifiers (.prevent, .stop, .once, .enter, .escape)

### 5.7 Theming System [plugin] ✅

- ✅ Parse `ui:` config block with `theme:` sub-block
- ✅ Parse `colors:` and `spacing:` theme sections
- ✅ Generate CSS variables (`:root { --color-*, --spacing-* }`)
- ✅ Inject via `_ui_inject_head_css()` bridge call

### 5.8 Form Support [plugin] ✅

- ✅ Two-way data binding via `cl-bind`
- ✅ Form validation via `cl-validate`
- ✅ CSRF token auto-injection (hidden `_csrf` field in POST/PUT/PATCH/DELETE forms)

### 5.9 Layout System [plugin] ✅

- ✅ Parse layout attribute from pages
- ✅ Implement layout wrapping via `_ui_load_layout()` bridge call
- ✅ Default `<slot></slot>` replacement with page content
- ✅ Named slot support (`<slot name="X">...</slot>`)

---

## Phase 6: Frame Auth — Plugin DSL ✅ | Runtime 🔀

Reference: [06_frame_auth.md](documents/specification/06_frame_auth.md)

### 6.1 Session Authentication [plugin] ✅

- ✅ Implement session creation helper
- ✅ Implement HTTP-only cookie management
- ✅ Implement SameSite and Secure flag configuration
- ✅ Implement session timeout configuration
- ✅ Implement CSRF token generation and validation
- ✅ Bridge declarations for session storage, retrieval, deletion
- ✅ Write tests for session auth

### 6.2 JWT Authentication [plugin] ✅

- ✅ Implement JWT signing (configurable algorithm)
- ✅ Implement JWT verification
- ✅ Add token expiration handling (TTL configuration)
- ✅ Implement refresh token flow helper
- ✅ Bridge declarations for JWT operations
- ✅ Write tests for JWT auth

### 6.3 Roles & Permissions [plugin] ✅

- ✅ Parse `roles:` config block
- ✅ Implement role-based access control
- ✅ Add `can()` permission check helper
- ✅ Add `hasRole()` role check helper
- ✅ Implement `protected:` block integration
- ✅ Bridge declarations for role/permission checking
- ✅ Write tests for RBAC

### 6.4 Password Hashing [plugin] ✅

- ✅ Implement Argon2id hashing helper
- ✅ Implement password verification helper
- ✅ Bridge declarations for crypto operations
- ✅ Write tests for password hashing

### 6.5 Multi-Tenant Support [plugin] ✅

- ✅ Add `tenant:` config sub-block in auth config
- ✅ Implement `tenant_getId()` helper (reads tenantId from session claims)
- ✅ Implement `tenant_require()` helper (requires valid tenant context)
- ✅ Implement `tenant_matches(tenantId)` helper (compares tenant IDs)
- ✅ Implement tenant isolation in queries (auto-filter by tenant_id in find/first/count/update/delete, auto-inject in insert)

---

## Phase 7: Frame Plugins (LOW 🟢) — ✅ Architecture Complete

Reference: [07_frame_plugins.md](documents/specification/07_frame_plugins.md)

### 7.1 Plugin System ✅

- ✅ Plugin discovery via plugin.toml
- ✅ Plugin manifest parsing (name, version, exports, bridge, handles)
- ✅ Plugin lifecycle (expand, validate, get_keywords)
- ✅ Implicit import via folder ownership
- ✅ Bridge function declarations
- ✅ 5 production plugins: frame.data, frame.server, frame.auth, frame.ui, frame.canvas

### 7.2 Plugin Hooks

- ⬜ Implement additional lifecycle hooks (registerCLI, registerServer, etc.)
- ⬜ Plugin inter-communication

### 7.3 Plugin Permissions

- ⬜ Parse permissions from plugin.toml
- ⬜ Implement allowlist enforcement

---

## Phase 8: Platform Support (MEDIUM-HIGH 🟡) — 🔀 Tracked in clean-manager/clean-server

Reference: [08_frame_platforms.md](documents/specification/08_frame_platforms.md)

- 🔀 Platform-specific build targets → clean-manager
- 🔀 PWA, Mobile (Capacitor), Desktop (Tauri) → future phases

---

## Phase 9: Testing Infrastructure (CRITICAL 🔴) — ✅ Foundation Complete

### 9.1 Test Framework ✅

- ✅ Test runner architecture (scripts/run-framework-tests.sh)
- ✅ 105+ test files across unit, integration, and e2e categories
- ✅ 87 compiled WASM test outputs
- ✅ Test fixtures and utilities

### 9.2 Test Coverage

**Tests by plugin:**
- ✅ frame.ui: 8 unit test files (component, hydration, events, state, etc.)
- ✅ frame.auth: 6 unit test files (JWT, sessions, passwords, roles, etc.)
- ✅ frame.data: 6 unit test files (models, fields, methods, queries, etc.)
- ✅ frame.server: 9 unit test files (endpoints, guards, paths, validation, etc.)
- ✅ Bridge: 6 unit test files (HTTP, DB, env, crypto, time, log)
- ✅ Integration: 3 test files (cross-plugin scenarios)
- ✅ E2E: 4 test files (full application flows)

### 9.3 Remaining Test Work

- ⬜ Implement `tests:` block parser in Clean Language
- ⬜ Implement `cleen test` command
- ⬜ Code coverage reporting
- ⬜ Coverage thresholds

---

## Phase 10: Documentation & Examples (MEDIUM-HIGH 🟡) — ✅ Comprehensive

### 10.1 Documentation ✅

- ✅ 13 numbered specification documents (01-13)
- ✅ Bridge contracts reference (frame_bridge_contracts.md)
- ✅ Internal module map (frame_internal_map.md)
- ✅ API reference cheat sheet (API_REFERENCE.md)
- ✅ Getting started guide (GETTING_STARTED.md)
- ✅ Project structure guide (PROJECT_STRUCTURE.md)
- ✅ Plugin guide (PLUGIN_GUIDE.md)
- ✅ Development guidelines (09_frame_dev_guidelines.md)

### 10.2 Examples ✅

- ✅ 20 example applications in /examples/
- ✅ Covers: REST API, blog, todo, full-stack, registration, browser counter, canvas, etc.

### 10.3 Remaining Documentation Work

- ✅ Expand canvas input handling examples (12_frame_canvas.md — sections 7.3, 7.4)
- ✅ Add edge case documentation (04_frame_data.md — section 16: migrations, transactions, tenants, slots)
- ✅ Extend platform availability matrix (frame_bridge_contracts.md — 11 new bridge functions)

---

## Phase 11: Performance & Optimization (LOW 🟢)

- ⬜ Incremental compilation
- ⬜ Dead code elimination
- ⬜ WASM SIMD support
- ⬜ Connection pooling optimizations (runtime)
- ⬜ Response compression (runtime)
- ⬜ Code splitting
- ⬜ Lazy loading for components

---

## Phase 12: Developer Experience (MEDIUM-HIGH 🟡)

- 🔀 VSCode extension → clean-extension repo (already exists)
- 🔀 Syntax highlighting → clean-extension
- 🔀 Auto-completion → clean-extension
- ⬜ Implement REPL
- ⬜ Hot module replacement (HMR)

---

## Known Clean Language Compiler Bugs

These bugs were discovered during Frame development and affect the compilation of Frame applications. Tracked here for reference; fixes belong in `clean-language-compiler`.

### Compiler Return Value Bug 🔴 CRITICAL

**Issue**: When a function call returns a value but the return value is not used, the compiler does not emit code to drop the value from the WASM stack. Causes WASM validation error.

**Workaround**: Always assign return values to a variable.

### Nested If-Else Generates Unreachable 🔴 CRITICAL

**Issue**: Void functions with nested if-else statements generate `unreachable` instructions after else blocks, causing runtime traps.

**Workaround**: Replace `else` with separate `if` with inverted condition.

### Not-Equal Operator in Plugin Output 🟡 MEDIUM

**Issue**: `!=` operator in generated code causes compilation issues.

**Workaround**: frame.server converts `!=` to `if not (a == b)` pattern (line 406 of main.cln).

### trim() Function Issues 🟡 MEDIUM

**Issue**: Built-in `trim()` function has reliability issues in certain contexts.

**Workaround**: frame.ui uses custom `strip_spaces()` reimplementation (lines 108-128 of main.cln).

---

## Plugin Implementation Summary

| Plugin | Version | LOC | WASM Size | Status |
|--------|---------|-----|-----------|--------|
| frame.data | 2.3.0 | 1,310 | 19KB | ✅ Complete (validation, config, FK cascades, tenant isolation, migrations) |
| frame.server | 2.1.0 | 759 | 39KB | ✅ Complete (returns, cache blocks) |
| frame.auth | 2.1.0 | 461 | 19KB | ✅ Complete (multi-tenant support) |
| frame.ui | 2.4.0 | 4,005 | 97KB | ✅ Complete (directives, events, theming, layouts, CSRF) |
| frame.canvas | 2.1.0 | 297 | 24KB | ✅ Complete (input handling) |
| **Total** | | **6,732** | **198KB** | |

---

## Bug Fixes

### frame.ui: HTML attribute quoting — ✅ Fixed 2026-04-03

- ✅ `extract_block_attributes()` converted single-quoted HTML attrs to escaped double quotes (`\"`), causing compiler tokenizer failure
- ✅ `extract_attributes()` had the same issue for SSR page rendering
- ✅ All hardcoded `\\\"` for HTML attributes in widget code generation (buttons, inputs, selects, images, links, dividers, spacers, cards, headings, canvas, etc.) converted to single quotes
- ✅ ARIA attribute generation, CSRF hidden fields, style injection, slot markers, loader script all fixed
- ✅ Plugin recompiled with compiler 0.31.0 (plugin.wasm 100,083 bytes)

**Root cause**: The compiler tokenizer does not support `\"` escape sequences in double-quoted strings in plugin output. The plugin was wrapping all HTML attribute values in escaped double quotes regardless of source quoting, and JSON serialization code was also using `\"`.

**Fix (Phase 1 — HTML attributes)**: All HTML attribute values in generated Clean code now use single quotes (`'`), which are valid HTML and don't conflict with Clean double-quoted strings.

**Fix (Phase 2 — JSON serialization)**: 
- `escape_string()` now uses `&quot;` HTML entity instead of `\"`, and collapses whitespace instead of `\n`/`\t` escapes
- Hydration JSON code uses `string __q = string.fromCharCode(34)` to produce double quote chars at runtime
- Event JSON uses `~` placeholder replaced with `__q` at runtime
- Validation rule JSON uses proper `"` chars (not `\"`)
- Plugin recompiled (plugin.wasm 100,424 bytes)

---

## Current Focus

**Phase 3 (ORM) - feature/phase-3-orm branch**

The plugins provide the complete DSL layer. Next steps:
1. Verify plugins compile correctly with latest compiler
2. Implement remaining plugin DSL features (returns:, cache:, directives, events, forms)
3. Bridge function runtime implementations → clean-server
4. Integration testing with clean-server

---

**Last Updated**: 2026-03-22
