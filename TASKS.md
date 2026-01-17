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

---

## Phase 1: Foundation & Core Infrastructure

### 1.1 Host Bridge Implementation (CRITICAL 🔴)

Reference: [frame_bridge_contracts.md](documents/specification/frame_bridge_contracts.md)

#### HTTP Bridge (`bridge:http`)
- ⬜ Implement `bridge:http.request` for outbound HTTP requests
- ⬜ Implement `bridge:http.respond` for SSR responses
- ⬜ Implement `bridge:http.redirect` for redirects
- ⬜ Add request parsing (headers, body, query params)
- ⬜ Add response serialization (JSON, HTML)
- ⬜ Write comprehensive tests for HTTP bridge
- ⬜ Add error handling with standard envelope

#### Database Bridge (`bridge:db`)
- ⬜ Implement `bridge:db.query` with parameterized queries
- ⬜ Implement `bridge:db.tx` for transactions
- ⬜ Implement `bridge:db.config` for connection management
- ⬜ Add PostgreSQL driver support
- ⬜ Add MySQL driver support
- ⬜ Add SQLite driver support
- ⬜ Implement connection pooling
- ⬜ Write comprehensive tests for DB bridge
- ⬜ Add query timeout handling

#### Environment Bridge (`bridge:env`)
- ⬜ Implement `bridge:env.get` for reading environment variables
- ⬜ Implement `bridge:env.list` for listing all variables
- ⬜ Add secure secret handling
- ⬜ Write tests for env bridge

#### Time Bridge (`bridge:time`)
- ⬜ Implement `bridge:time.now` for current time
- ⬜ Implement `bridge:time.sleep` for delays
- ⬜ Add timezone support
- ⬜ Write tests for time bridge

#### Crypto Bridge (`bridge:crypto`)
- ⬜ Implement `bridge:crypto.random` for random bytes
- ⬜ Implement `bridge:crypto.hash` for hashing
- ⬜ Implement `bridge:crypto.verify` for password verification
- ⬜ Implement `bridge:crypto.sign` for JWT signing
- ⬜ Add bcrypt/argon2 support
- ⬜ Add SHA256/512 support
- ⬜ Write comprehensive tests for crypto bridge

#### Log Bridge (`bridge:log`)
- ⬜ Implement `bridge:log.info` for info logs
- ⬜ Implement `bridge:log.warn` for warnings
- ⬜ Implement `bridge:log.error` for errors
- ⬜ Add structured logging support
- ⬜ Add log level filtering
- ⬜ Write tests for log bridge

#### Filesystem Bridge (`bridge:fs`)
- ⬜ Implement `bridge:fs.read` for reading files
- ⬜ Implement `bridge:fs.write` for writing files
- ⬜ Implement `bridge:fs.list` for directory listing
- ⬜ Add permission checks
- ⬜ Add sandboxing for file access
- ⬜ Write tests for fs bridge

#### System Bridge (`bridge:sys`)
- ⬜ Implement `bridge:sys.exit` for process exit
- ⬜ Implement `bridge:sys.platform` for platform info
- ⬜ Write tests for sys bridge

---

## Phase 2: Frame CLI (CRITICAL 🔴)

Reference: [02_frame_cli.md](documents/specification/02_frame_cli.md)

### 2.1 Core Commands

#### `frame new` - Project Scaffolding
- ⬜ Implement project template generation
- ⬜ Create default folder structure (app/, db/, config/, public/)
- ⬜ Generate starter files (schema.cln, app.cln, etc.)
- ⬜ Support --template flag (api, full, ui)
- ⬜ Write tests for project scaffolding

#### `frame serve` - Development Server
- ⬜ Implement file watcher for auto-reload
- ⬜ Integrate with Clean Language compiler
- ⬜ Set up WASM runtime loader
- ⬜ Implement hot module replacement
- ⬜ Add logging and error display
- ⬜ Write tests for serve command

#### `frame build` - Production Build
- ⬜ Implement compilation pipeline
- ⬜ Add WASM optimization passes
- ⬜ Generate dist/ output folder
- ⬜ Support --target flag (web, pwa, mobile, desktop, server, cli)
- ⬜ Implement bundle minification
- ⬜ Write tests for build command

### 2.2 Database Commands

#### `frame db:plan` - Migration Planning
- ⬜ Implement schema diff detection
- ⬜ Generate SQL migration preview
- ⬜ Display migration plan in console
- ⬜ Write tests for db:plan

#### `frame db:migrate` - Run Migrations
- ⬜ Apply pending migrations
- ⬜ Track migration history
- ⬜ Support rollback functionality
- ⬜ Write tests for db:migrate

#### `frame db:seed` - Database Seeding
- ⬜ Load and execute seed.cln files
- ⬜ Support multiple seed files
- ⬜ Write tests for db:seed

#### `frame db:reset` - Reset Database
- ⬜ Drop all tables
- ⬜ Recreate schema
- ⬜ Re-run migrations
- ⬜ Write tests for db:reset

### 2.3 API Commands

#### `frame api:spec` - OpenAPI Generation
- ⬜ Parse endpoints: blocks
- ⬜ Generate OpenAPI 3.1 specification
- ⬜ Support --format flag (json, yaml)
- ⬜ Write tests for api:spec

#### `frame api:sdk` - SDK Generation
- ⬜ Generate Clean Language client
- ⬜ Generate TypeScript client
- ⬜ Generate Swift client
- ⬜ Generate Kotlin client
- ⬜ Write tests for api:sdk

### 2.4 Platform Commands

#### `frame pwa:init` - PWA Initialization
- ⬜ Generate manifest.json
- ⬜ Create service worker template
- ⬜ Generate app icons
- ⬜ Write tests for pwa:init

#### `frame mobile:init` - Mobile Initialization
- ⬜ Set up Capacitor project
- ⬜ Configure iOS project
- ⬜ Configure Android project
- ⬜ Write tests for mobile:init

#### `frame desktop:init` - Desktop Initialization
- ⬜ Set up Tauri project
- ⬜ Configure Windows build
- ⬜ Configure Linux build
- ⬜ Configure macOS build
- ⬜ Write tests for desktop:init

#### `frame server:init` - Server Initialization
- ⬜ Generate Dockerfile
- ⬜ Create health check endpoints
- ⬜ Generate environment template
- ⬜ Write tests for server:init

---

## Phase 3: Frame Server (CRITICAL 🔴)

Reference: [03_frame_server.md](documents/specification/03_frame_server.md)

### 3.1 WASM Runtime

- ⬜ Implement WASM module loading
- ⬜ Add module caching
- ⬜ Implement execution sandboxing
- ⬜ Add per-request isolation
- ⬜ Write tests for WASM runtime

### 3.2 HTTP Router

- ⬜ Implement file-based routing
- ⬜ Add dynamic route parameters (/:id)
- ⬜ Parse query parameters
- ⬜ Parse request body (JSON, form data)
- ⬜ Implement route resolution
- ⬜ Write tests for router

### 3.3 Endpoints System

- ⬜ Parse `endpoints:` blocks from Clean code
- ⬜ Support METHOD /path: syntax
- ⬜ Implement `guard:` sub-block for auth
- ⬜ Implement `returns:` sub-block for types
- ⬜ Implement `cache:` sub-block for caching
- ⬜ Implement `handle:` sub-block for logic
- ⬜ Write tests for endpoints system

### 3.4 Request/Response Helpers

- ⬜ Implement `req.params` for path parameters
- ⬜ Implement `req.query` for query parameters
- ⬜ Implement `req.headers` for headers
- ⬜ Implement `req.body` for request body
- ⬜ Implement `req.json()` for JSON parsing
- ⬜ Implement `req.form` for form data
- ⬜ Implement `json()` response helper
- ⬜ Implement `html()` response helper
- ⬜ Implement `redirect()` response helper
- ⬜ Implement `notFound()` response helper
- ⬜ Implement `badRequest()` response helper
- ⬜ Implement `unauthorized()` response helper
- ⬜ Write tests for request/response helpers

### 3.5 SSR Pipeline

- ⬜ Implement server-side rendering engine
- ⬜ Add component tree rendering
- ⬜ Generate HTML from Clean components
- ⬜ Create islands manifest for hydration
- ⬜ Add HTML escaping by default
- ⬜ Write tests for SSR pipeline

---

## Phase 4: Frame Data (ORM) (CRITICAL 🔴)

Reference: [04_frame_data.md](documents/specification/04_frame_data.md)

### 4.1 Model Definitions

- ⬜ Parse `data` keyword for models
- ⬜ Support field types (integer, string, boolean, datetime, etc.)
- ⬜ Implement field constraints (pk, auto, unique, default)
- ⬜ Add validation rules (min, max, regex, email)
- ⬜ Write tests for model parsing

### 4.2 Relationships

- ⬜ Implement one-to-many relationships
- ⬜ Implement many-to-many via junction tables
- ⬜ Add foreign key constraints
- ⬜ Support onDelete cascades
- ⬜ Write tests for relationships

### 4.3 Query Builder

- ⬜ Implement `Model.find:` for SELECT queries
- ⬜ Implement `where:` sub-block for filtering
- ⬜ Implement `order:` sub-block for sorting
- ⬜ Implement `limit:` for pagination
- ⬜ Implement `Model.first:` for single records
- ⬜ Implement `Model.count:` for counting
- ⬜ Implement `link:` for many-to-many joins
- ⬜ Write tests for query builder

### 4.4 Mutations

- ⬜ Implement `Model.insert:` for INSERT
- ⬜ Implement `Model.update:` for UPDATE
- ⬜ Implement `Model.delete:` for DELETE
- ⬜ Add set: sub-block for updates
- ⬜ Write tests for mutations

### 4.5 Transactions

- ⬜ Implement `Data.tx:` for transactions
- ⬜ Add automatic rollback on error
- ⬜ Support nested transactions
- ⬜ Write tests for transactions

### 4.6 Raw Queries

- ⬜ Implement `db.queryAs()` for typed SQL
- ⬜ Implement `db.query()` for untyped SQL
- ⬜ Support parameterized queries
- ⬜ Write tests for raw queries

### 4.7 Migrations

- ⬜ Implement schema diff algorithm
- ⬜ Generate SQL migrations (up/down)
- ⬜ Add migration version tracking
- ⬜ Support migration rollback
- ⬜ Generate migration files in db/migrations/
- ⬜ Write tests for migrations

---

## Phase 5: Frame UI (MEDIUM-HIGH 🟡)

Reference: [05_frame_ui.md](documents/specification/05_frame_ui.md)

### 5.1 Component System

- ⬜ Parse `component` keyword
- ⬜ Support `props:` block for component props
- ⬜ Implement `render()` function requirement
- ⬜ Add type validation for props
- ⬜ Write tests for component parsing

### 5.2 Server-Side Rendering

- ⬜ Implement HTML generation from components
- ⬜ Add automatic HTML escaping
- ⬜ Support `rawHtml()` for trusted content
- ⬜ Implement string interpolation in templates
- ⬜ Write tests for SSR

### 5.3 Client Hydration

- ⬜ Implement `client` attribute parsing (off|on|visible|idle|only)
- ⬜ Generate islands manifest
- ⬜ Create browser loader (loader.js)
- ⬜ Implement hydration strategies
- ⬜ Add component lifecycle hooks (onMount, onVisible, onIdle)
- ⬜ Write tests for hydration

### 5.4 Event Handling

- ⬜ Parse event attributes (onClick, onInput, onChange, etc.)
- ⬜ Generate event handler bindings
- ⬜ Implement server-side action handling
- ⬜ Implement client-side event handling
- ⬜ Write tests for event handling

### 5.5 Theming System

- ⬜ Parse config/ui.cln for theme config
- ⬜ Generate CSS variables from theme
- ⬜ Support CSS framework integration
- ⬜ Write tests for theming

### 5.6 Custom Tags

- ⬜ Implement tag registration in config/tags.cln
- ⬜ Map tag names to component files
- ⬜ Support custom element rendering
- ⬜ Write tests for custom tags

---

## Phase 6: Frame Auth (MEDIUM-HIGH 🟡)

Reference: [06_frame_auth.md](documents/specification/06_frame_auth.md)

### 6.1 Session Authentication

- ⬜ Implement session creation
- ⬜ Add session storage (in-memory, Redis, database)
- ⬜ Implement HTTP-only cookie management
- ⬜ Add SameSite and Secure flags
- ⬜ Implement session timeout
- ⬜ Add CSRF token generation and validation
- ⬜ Write tests for session auth

### 6.2 JWT Authentication

- ⬜ Implement JWT signing (HS256, RS256)
- ⬜ Implement JWT verification
- ⬜ Add token expiration handling
- ⬜ Implement refresh token flow
- ⬜ Write tests for JWT auth

### 6.3 Roles & Permissions

- ⬜ Parse config/roles.cln
- ⬜ Implement role-based access control
- ⬜ Add `auth.can()` permission check
- ⬜ Implement `guard:` integration with endpoints
- ⬜ Add policy function support
- ⬜ Write tests for RBAC

### 6.4 Password Hashing

- ⬜ Implement bcrypt hashing
- ⬜ Implement argon2 hashing
- ⬜ Add password verification
- ⬜ Write tests for password hashing

### 6.5 Multi-Tenant Support

- ⬜ Add tenantId to session claims
- ⬜ Implement tenant isolation in queries
- ⬜ Add tenant-based routing
- ⬜ Write tests for multi-tenancy

---

## Phase 7: Frame Plugins (LOW 🟢)

Reference: [07_frame_plugins.md](documents/specification/07_frame_plugins.md)

### 7.1 Plugin System

- ⬜ Implement plugin discovery
- ⬜ Parse plugin.cln manifest
- ⬜ Add plugin lifecycle management
- ⬜ Implement plugin sandboxing
- ⬜ Write tests for plugin system

### 7.2 Plugin Hooks

- ⬜ Implement UI hooks (registerTags)
- ⬜ Implement CLI hooks (registerCLI)
- ⬜ Implement server hooks (registerServer)
- ⬜ Implement data hooks (registerData)
- ⬜ Implement build hooks (registerBuild)
- ⬜ Write tests for plugin hooks

### 7.3 Plugin Permissions

- ⬜ Parse permissions from plugin.cln
- ⬜ Implement allowlist enforcement
- ⬜ Add permission prompt for elevated access
- ⬜ Write tests for permissions

---

## Phase 8: Platform Support (MEDIUM-HIGH 🟡)

Reference: [08_frame_platforms.md](documents/specification/08_frame_platforms.md)

### 8.1 Web Platform

- ⬜ Implement static file serving
- ⬜ Add asset bundling
- ⬜ Generate deployment artifacts
- ⬜ Write tests for web platform

### 8.2 PWA Platform

- ⬜ Generate manifest.json
- ⬜ Create service worker
- ⬜ Add offline caching strategy
- ⬜ Generate app icons
- ⬜ Write tests for PWA platform

### 8.3 Mobile Platform (Capacitor)

- ⬜ Set up Capacitor integration
- ⬜ Implement iOS build
- ⬜ Implement Android build
- ⬜ Add native plugin bridge
- ⬜ Write tests for mobile platform

### 8.4 Desktop Platform (Tauri)

- ⬜ Set up Tauri integration
- ⬜ Implement Windows build
- ⬜ Implement Linux build
- ⬜ Implement macOS build
- ⬜ Add desktop-specific bridges
- ⬜ Write tests for desktop platform

### 8.5 Server Platform

- ⬜ Create Node.js runtime adapter
- ⬜ Create Rust runtime adapter
- ⬜ Create Deno runtime adapter
- ⬜ Add Docker support
- ⬜ Implement health check endpoints
- ⬜ Write tests for server platform

---

## Phase 9: Testing Infrastructure (CRITICAL 🔴)

### 9.1 Test Framework

- ⬜ Set up test runner architecture
- ⬜ Implement `tests:` block parser
- ⬜ Add assertion library
- ⬜ Implement test execution engine
- ⬜ Add test result reporting
- ⬜ Implement `frame test` command
- ⬜ Write tests for test framework

### 9.2 Unit Testing

- ⬜ Create unit test utilities
- ⬜ Add mock Host Bridge for testing
- ⬜ Implement component testing helpers
- ⬜ Write tests for all modules

### 9.3 Integration Testing

- ⬜ Create integration test harness
- ⬜ Add database test utilities
- ⬜ Implement HTTP test client
- ⬜ Write end-to-end API tests

### 9.4 Code Coverage

- ⬜ Integrate coverage reporting
- ⬜ Add coverage thresholds
- ⬜ Generate coverage reports
- ⬜ Aim for 80%+ coverage on critical paths

---

## Phase 10: Documentation & Examples (MEDIUM-HIGH 🟡)

### 10.1 Documentation

- ⬜ Create API reference documentation
- ⬜ Write tutorials and guides
- ⬜ Add inline code documentation
- ⬜ Create video tutorials (optional)

### 10.2 Examples

- ⬜ Create "Hello World" example
- ⬜ Create REST API example
- ⬜ Create full-stack CRUD app example
- ⬜ Create blog platform example
- ⬜ Create e-commerce example
- ⬜ Create authentication example
- ⬜ Create real-time app example

---

## Phase 11: Performance & Optimization (LOW 🟢)

### 11.1 Compiler Optimizations

- ⬜ Implement incremental compilation
- ⬜ Add dead code elimination
- ⬜ Implement tree shaking
- ⬜ Add WASM SIMD support
- ⬜ Enable parallel compilation

### 11.2 Runtime Optimizations

- ⬜ Implement WASM module caching
- ⬜ Add connection pooling optimizations
- ⬜ Implement query result caching
- ⬜ Add response compression
- ⬜ Optimize SSR rendering speed

### 11.3 Bundle Optimization

- ⬜ Implement code splitting
- ⬜ Add lazy loading for components
- ⬜ Minimize WASM bundle size
- ⬜ Optimize asset delivery

---

## Phase 12: Developer Experience (MEDIUM-HIGH 🟡)

### 12.1 CLI Improvements

- ⬜ Add interactive project setup wizard
- ⬜ Implement `frame generate` command
- ⬜ Add better error messages
- ⬜ Improve progress indicators

### 12.2 Debugging Tools

- ⬜ Add debugger integration
- ⬜ Implement REPL
- ⬜ Add hot module replacement (HMR)
- ⬜ Create performance profiler

### 12.3 Editor Integration

- ⬜ Create VSCode extension
- ⬜ Add syntax highlighting
- ⬜ Implement auto-completion
- ⬜ Add error diagnostics

---

## Success Criteria

Each phase is considered complete when:

1. ✅ All tasks in the phase are completed
2. ✅ Comprehensive tests are written and passing
3. ✅ Documentation is updated
4. ✅ Code follows Clean Language and Frame specifications
5. ✅ Performance targets are met (see CLAUDE.md)
6. ✅ Security requirements are satisfied

---

## Development Process

For each task:

1. **Read specification**: Consult the relevant spec document
2. **Plan**: Break down into smaller subtasks if needed
3. **Implement**: Write production-quality code
4. **Test**: Write comprehensive tests
5. **Document**: Update documentation
6. **Review**: Code review and feedback
7. **Mark complete**: Update this file with ✅

---

## Known Clean Language Compiler Bugs

These bugs were discovered during Frame development and affect the compilation of Frame applications.

### Compiler Return Value Bug 🔴 CRITICAL

**File**: Clean Language Compiler (`clean-language-compiler/src/codegen/`)

**Issue**: When a function call returns a value but the return value is not used (stored in a variable), the compiler does not emit code to drop the value from the WASM stack. This causes a WASM validation error: "type mismatch in return".

**Example problematic code**:
```clean
void draw()
    _canvas_clear(canvasId)  // Returns integer but not used
```

**Workaround**: Always assign return values to a variable:
```clean
void draw()
    integer _r = _canvas_clear(canvasId)  // Must capture return value
```

**Status**: ⬜ Not Started

---

### Nested If-Else Generates Unreachable 🔴 CRITICAL

**File**: Clean Language Compiler (`clean-language-compiler/src/codegen/`)

**Issue**: When a void function contains nested if-else statements, the compiler generates `unreachable` instructions after else blocks, causing runtime traps.

**Example problematic code**:
```clean
void updateGame()
    if ballY > 600.0
        lives = lives - 1
        if lives <= 0
            gameOver = true
        else
            resetBall()  // Compiler generates 'unreachable' after this
```

**Workaround**: Replace `else` with separate `if` statement:
```clean
void updateGame()
    if ballY > 600.0
        lives = lives - 1
        if lives <= 0
            gameOver = true
        if lives > 0
            resetBall()  // Works correctly
```

**Status**: ⬜ Not Started

---

## Plugin-Based Implementation Progress

The Frame Framework uses a plugin-based architecture where framework features are implemented as Clean Language plugins. These plugins are compiled along with user code and expand DSL blocks into standard Clean code at compile time.

### Completed Plugin Updates (2025-01-15)

#### ✅ frame.data (v2.0.0) - ORM Plugin
- Implemented `data` keyword for model definitions
- Added field constraint parsing (`:pk`, `:auto`, `:unique`, `:default=`)
- Implemented `where:`, `order:`, `limit:`, `link:` sub-blocks
- Added `Model.find:`, `Model.first:`, `Model.count:` query methods
- Added `Model.insert:`, `Model.update:`, `Model.delete:` mutation methods
- Implemented `Data.tx:` transaction blocks
- Added `db.query:` and `db.queryAs()` for raw SQL
- Full bridge declarations for database operations

#### ✅ frame.httpserver (v2.0.0) - HTTP Server Plugin
- Implemented `endpoints:` block parsing with GET/POST/PUT/DELETE/PATCH
- Added `guard:` sub-block for authorization
- Added `returns:` sub-block for OpenAPI hints
- Added `cache:` sub-block for caching
- Added `handle:` sub-block for handler logic
- Implemented response helpers: `json()`, `html()`, `redirect()`, `notFound()`, `badRequest()`, `unauthorized()`, `forbidden()`
- Implemented request helpers: `req.params.*`, `req.query.*`, `req.headers`, `req.json()`, `req.form`, `req.body`
- Full bridge declarations for HTTP operations

#### ✅ frame.auth (v2.0.0) - Authentication Plugin
- Implemented `auth:` config block with `session:`, `jwt:`, `roles:` sub-blocks
- Added session management: `session_create()`, `session_setCookie()`, `session_get()`, `session_destroy()`
- Added JWT operations: `jwt_sign()`, `jwt_verify()`, `jwt_refresh()`
- Implemented password hashing: `hashPassword()`, `verifyPassword()`
- Added role/permission checking: `can()`, `hasRole()`, `isAuthenticated()`
- Implemented CSRF token generation and validation
- Full bridge declarations for crypto, session, and JWT operations

#### ✅ frame.ui (v2.1.0) - UI Plugin
- Implemented HTML page processing with `component`, `screen`, `page` blocks
- Added directives: `if`, `else`, `each`, `bind`, `client`, `slot`, `show`, `validate`
- Implemented event handling: `onclick`, `oninput`, `onchange`, `onsubmit`, etc.
- Added event modifiers: `.prevent`, `.stop`, `.once`, `.enter`, `.escape`
- Implemented hydration modes: `off`, `on`, `visible`, `idle`, `only`
- Added slot support for component composition
- Full bridge declarations for UI operations

#### ✅ frame.canvas (v2.0.0) - Canvas/Game Plugin
Already comprehensive - includes:
- Canvas drawing (shapes, text, images, gradients, paths)
- Animation (requestAnimationFrame, delta time, FPS)
- Audio (sound effects, music, volume control)
- Sprites (sprite sheets, transformations)
- Input (mouse, keyboard, touch, gamepad)
- Collision detection (all shape combinations, raycast)
- Asset management (loading, caching, progress)
- Camera/viewport
- Easing functions (all common types)
- Scene management

---

## Current Focus

**Phase 1 - Foundation & Core Infrastructure**

The plugins provide the DSL layer. The Host Bridge runtime is implemented in `clean-server` (separate repository) and provides the actual bridge functions that the plugins declare.

**Plugin Architecture:**
- Plugins live in `/plugins/` directory
- Each plugin has `plugin.toml` (metadata, bridge declarations) and `src/main.cln` (expansion logic)
- Plugins declare bridge functions that the runtime must provide
- The compiler reads plugin.toml to know which blocks to handle and what imports to generate

**Next Steps:**
1. Verify plugins compile correctly with updated compiler
2. Implement bridge functions in clean-server runtime
3. Create example applications using the plugins
4. Write integration tests

**Estimated Timeline:**
- Plugin verification: 1-2 days
- Runtime implementation: 4-6 weeks
- Examples & tests: 2-3 weeks

**Total Estimated Time for v1.0**: 2-3 months (reduced due to plugin-based architecture)

---

**Last Updated**: 2025-01-15
