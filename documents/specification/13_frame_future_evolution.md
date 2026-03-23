# Frame Future Evolution

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0
**Location:** `/docs/specification/13_frame_future_evolution.md`

---

## 1. Vision

Portable, secure, elegant. All logic in Clean, running everywhere via WASM. The host only provides a small typed bridge.

---

## 2. Roadmap (12–18 months)

- **Pure WASI Runtime** — Wasmtime/wasmCloud; wasi:http, timers, sockets.
- **Distributed Runtime** — Multi-node jobs, scheduler, durable queue.
- **Streaming & Incremental Rendering** — Chunked SSR/JSON streams.
- **Graph Layer** — Auto GraphQL (or Clean graph) from ORM.
- **Visual Builder** — Drag-and-drop over Clean; persists `.cln`.
- **Plugin Marketplace** — Signed plugins, permission manifests, CI tests.
- **Observability Pack** — Spans via `host:log`; OTLP export.
- **Security Hardening** — Key rotation helpers, mTLS adapters, SRI.
- **DX** — `frame test`, `frame deploy`, incremental builds, hot-reload.

---

## 3. Research

- Deterministic scheduling across hosts.
- Zero-copy host-WASM data (shared memory proposals).
- IndexedDB/OPFS-backed SQLite parity across browsers.

---

## 4. Compatibility

- Semver for compiler/CLI; deprecations one minor ahead.
- Bridges follow semver within a major release.
- Migration helpers auto-upgrade manifests/configs.

---

## 5. Success Metrics

| Metric | Target |
|--------|--------|
| SSR TTFP | ≤ 150ms (baseline hardware) |
| Hydration | < 20ms for simple islands |
| p95 CRUD latency | < 100ms (local region) |
| Compile times | Improve release-over-release |

---

## 6. Post-v1.0 Enhancements

The following features are planned for post-v1.0 releases. They improve performance, developer experience, and advanced capabilities but are NOT required for a working Frame application.

### 6.1 Compiler Optimizations

**Incremental Compilation**
Currently the compiler recompiles everything from scratch. Incremental compilation tracks which source files changed, caches parsed ASTs and type information, and only reanalyzes affected modules. Target: < 100ms single-file rebuild. Requires a build cache (`.clean-cache/` directory) in the compiler.

**Dead Code Elimination**
Detect and remove functions, variables, and imports that are never used. Reduces WASM binary size. If a model defines 10 methods but only 3 are called, the unused 7 should not appear in the output WASM. Standard optimization pass after semantic analysis.

**WASM SIMD Support**
WASM SIMD extensions for parallel numeric operations (`v128` types, `f32x4` operations). Benefits canvas/game plugins with heavy math — vector operations, matrix transforms, physics calculations. The compiler's codegen emits SIMD instructions when it detects parallelizable numeric patterns.

**Code Splitting**
Instead of one large `.wasm` file per application, generate multiple smaller WASM modules — one per page or component group. Enables faster initial page loads, parallel module loading, and better cache efficiency (unchanged modules don't need re-downloading).

### 6.2 Runtime Optimizations

**Connection Pooling**
The frame.data plugin generates `_db_configure()` calls with `pool_max` and `pool_idle_timeout` parameters. The server maintains a pool of open database connections, reuses them across requests, and closes idle ones after the configured timeout. Critical for production performance under concurrent load.

**Response Compression**
Server-side gzip/brotli compression for HTTP responses. Compress text/html, application/json, and text/css responses above a size threshold. Standard HTTP server optimization, no plugin changes needed.

**Lazy Loading for Components**
Split components into separate WASM modules loaded on demand. The compiler generates separate entry points per component; the server serves them with appropriate `<script async>` or `<script defer>` attributes based on the component's hydration mode.

### 6.3 Developer Experience

**Hot Module Replacement (HMR)**
During development, when a `.cln` or `.html` file is saved:
1. File watcher detects the change
2. Only the changed module is recompiled
3. Update pushed to browser via WebSocket
4. Browser replaces old component without full page reload

Requires dev server mode in clean-server with file watching and WebSocket support.

**REPL (Read-Eval-Print Loop)**
Interactive Clean Language shell:
```
$ cleen repl
clean> string name = "Alice"
clean> print("Hello " + name)
Hello Alice
```
Requires a minimal compiler mode for single expressions, an in-memory WASM runtime with persistent state, and input handling with history.

**Code Coverage Reporting**
During test execution, track which source lines are exercised. Generate coverage reports (terminal summary, HTML, LCOV for CI). Requires the compiler to instrument WASM with coverage counters, and `cleen test` to collect and report results.

**Coverage Thresholds**
Configuration in `package.clean.toml`:
```toml
[test]
coverage_threshold = 80
```
`cleen test --coverage` fails with non-zero exit code if below threshold. Used in CI pipelines.

### 6.4 Advanced Features

**Runtime Validation via Bridge**
Server-side enforcement of frame.data validation rules. The plugin generates `validate()` methods that run in WASM, but the server should also validate at the bridge boundary to catch invalid data from external API calls or direct database access that bypasses the ORM.

**Client Hydration Strategies**
The frame.ui plugin parses `client="visible"` and `client="idle"` attributes. The server-generated `loader.js` must implement:
- `visible`: IntersectionObserver — load WASM when element scrolls into view
- `idle`: requestIdleCallback — load WASM when browser is idle

Currently `client="on"` (immediate hydration) is the only functional mode.

**Plugin Inter-Communication**
Allow plugins to declare dependencies and call each other's exported functions. For example, frame.ui could call frame.auth's `csrf_generate()` directly. The compiler resolves plugin dependencies and links exports at compile time.

---

## 7. Contributing

- RFCs for proposals; small patches with tests.
- Private channel for security reports.

---

**End of Frame Future Evolution Specification**
