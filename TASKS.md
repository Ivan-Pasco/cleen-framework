# Frame Framework — Development Tasks

This file tracks **pending** work for the Frame Framework. Completed work is summarized at the bottom for context.

**Priority Levels:**
- 🔴 **CRITICAL**: Core functionality, blocking other work
- 🟡 **MEDIUM-HIGH**: Important features with significant impact
- 🟢 **LOW**: Nice-to-have improvements and optimizations

**Status:**
- ⬜ Not Started · 🔄 In Progress · ❌ Blocked · 🔀 Moved (to another component/repo)

**Architecture Note:** Plugins provide DSL expansion (compile Clean syntax into bridge calls); runtime bridge implementations live in `clean-server` (separate repo). Tasks tagged `[plugin]` are in scope here; `[runtime]` items are tracked in clean-server.

---

## Pending — In Scope (clean-framework)

### Cross-Component — Compiler v0.32.0 Release Coordination 🟡 MEDIUM-HIGH

- ⬜ **Raise frame.mcp pin to "0.32.0" once compiler v0.32.0 ships** — `plugins/frame.mcp/plugin.toml` currently pins `min_compiler_version = "0.30.357"` as a temporary fix. The honest target is `0.32.0` per the version-track correction documented in `../foundation/management/cross-component-prompts/compiler-v0.32-version-track-correction.md`. Required steps once compiler ships v0.32.0:
  1. Lift `min_compiler_version` in `frame.mcp/plugin.toml` from `"0.30.357"` to `"0.32.0"`.
  2. Audit the other plugins (`frame.client`, `frame.ui`, `frame.canvas`) — at least 10 framework patterns use `events:` blocks (introduced post-v0.31.0). Consider declaring `min_compiler_version = "0.32.0"` on those plugins too so consumers get a clear error if they downgrade.
  3. Rebuild all plugins against `cln 0.32.0` via the existing build loop.
  4. Bump framework `Cargo.toml` from `2.12.69` to `2.13.0` (minor — new compiler-feature dependency).
  5. Run COMITA. Tag should be `v2.13.0`, not a patch.

### Browser Host Bridge Parity 🟡 MEDIUM-HIGH

- ⬜ **60 missing stdlib bridge functions in browser host JS** — `check_host_parity.py --host browser --strict` reports 60 functions declared `hosts = ["all"]` in `foundation/platform-architecture/function-registry.toml` that no browser-side JS runtime implements. Three runtime files share the gap: `plugins/frame.canvas/runtime/loader.js`, `plugins/frame.ui/runtime/loader.js`, `plugins/frame.client/runtime/bridge.js`. Families:
  - `array_*` (13): push, pop, get, set, slice, map, filter, find, reduce, sort, reverse, concat, contains
  - `list.*` (10): add, allocate, clear, contains, get, isEmpty, push, remove, set
  - `math_*` (16): abs_i32, max_i32, min_i32, fmod, hypot, expm1, log1p, is_finite, is_infinite, is_nan, ln10, ln2, log10e, log2e, sqrt1_2, sqrt2, log
  - `string_*` (17): length, contains, equals, equals_ignore_case, starts_with, ends_with, char_at, char_code_at, from_char_code, index_of, last_index_of, join, pad_start, pad_end, replace_first, reverse, is_empty, is_blank
  - Misc (4): float_to_string_fixed, http_get_response_header, print_debug, print_error

  Pragmatic fix: add all 60 to `frame.canvas/runtime/loader.js` in `createStdlibImports()`. Most are one-liners (`array_push: (h, v) => …`, `math_log: Math.log`).

- ⬜ **(Follow-up)** Extract a shared `clean-stdlib.js` consumed by all three browser runtimes — eliminates the 3-way duplication of the existing 87 stdlib functions in `frame.canvas/runtime/loader.js`. Larger refactor; do only when one of the three runtimes diverges in a way that bites.

### Test Infrastructure 🟡 MEDIUM-HIGH

- ✅ ~~**Plugin tests cannot resolve plugin functions**~~ — Fixed 2026-06-04: `scripts/test-plugins.sh` now concatenates `src/main.cln` + a generated `assert()` helper + the test's `functions:` block before invoking `cln check`. Compilation-pass parity is the guard; 3/4 plugin test suites (auth, data, server) compile cleanly.
- ⬜ **`cln test` runtime stub missing** — `cln test` fails at load time with `unknown import: env::_state_reset_all`. The runner doesn't provide host stubs for plugin-emitted imports. Until this is resolved upstream, `test-plugins.sh` uses `cln check` (compilation only, not execution). Cross-component: clean-language-compiler or clean-server needs to ship a runner with these stubs.
- ✅ ~~**frame.ui missing `validate_block`**~~ — Fixed 2026-06-04. `plugins/frame.ui/src/main.cln` now exports `validate_block` covering `component` (name + client mode + render()), `page` (path), and `layout` (name) per `05_frame_ui.md`. Declared in `plugins/frame.ui/plugin.toml` under `[exports]` as `validate = "validate_block"`.

### Plugin DSL Gaps

#### Phase 4.8: Runtime validation [plugin] 🟢 LOW
- ⬜ Declare validation bridge functions in `frame.data/plugin.toml` and generate calls from `validate(field, value)` per model. (Runtime side: `_db_validate_field` impl in clean-server.)

#### Phase 5.5: Client hydration [plugin] 🟡 MEDIUM-HIGH
- ✅ ~~Emit hydration manifest for the runtime~~ — Fixed 2026-06-04. `frame.ui/src/main.cln` `assemble()` now scans HTML source files for `<screen name="X" client="Y">` tags and injects a virtual `__hydration_manifest.cln` containing a function `_ui_hydration_manifest()` that returns the JSON `{"screens":{"Name":"mode",...}}`. Screens without a `client` attribute default to `off`. Duplicate screen names are deduplicated (last wins).
  - ⬜ **Spec divergence to resolve**: `documents/specification/05_frame_ui.md §14.1` describes the manifest as `{"components":{name:{"bundle":..., "strategy":...}}, "pages":{...}}`. The implementation emits `{"screens":{name:"mode"}}` because: (a) Frame compiles to a single WASM, no per-component JS bundles; (b) screens (not components) are the actual hydration units. Developer decision needed: update spec to reflect WASM architecture, or extend implementation to bundle-split. Tracked under Principle 25 (spec change requires approval).
- ⬜ Full hydration strategy impl (IntersectionObserver, requestIdleCallback) is `[runtime]`.

#### Phase 7.2: Plugin lifecycle hooks 🟢 LOW
- ⬜ Define additional lifecycle hooks beyond `expand`/`validate`/`get_keywords` (e.g. `registerCLI`, `registerServer`). Requires spec change in `foundation/spec/plugins/plugin-contract.md`.
- ⬜ Plugin inter-communication contract (one plugin's expand output consumed by another).

#### Phase 7.3: Plugin permissions 🟢 LOW
- ⬜ Parse `[permissions]` table from `plugin.toml`.
- ⬜ Enforce allowlist at host bridge call sites (declaration side here; enforcement is `[runtime]`).

### Frame UI — Code Quality 🟢 LOW

- ⬜ **Re-enable trim on screen block names** — `plugins/frame.ui/src/main.cln:62-64` assigns `screen_name = temp_name` instead of `temp_name.trim()` because of a historical compiler issue. Verify whether compiler 0.30.229+ handles `.trim()` correctly on the substring result, then remove the workaround. Needs a runnable test before changing.
- ⬜ **Long functions in `src/main.cln`** — four functions exceed 380 lines and should be split into named helpers without behavior change. Approx start lines: 676 (~437 lines), 1230 (~463), 3164 (~617), 3319 (384, `render_ui_widget`). Blocked on runnable plugin tests so behavior parity can be verified.

### Frame Auth — Patterns 🟢 LOW

- ⬜ **Password-reset email hook** — `plugins/frame.auth/patterns/password-reset.cln:89` has `// TODO: send email with reset link containing raw_token`. Pattern (user template), so this is an integration point, not a missing impl. Reword as `// User integration: send email with reset link containing raw_token` once an email/messaging plugin contract is defined (requires `bridge:email` declaration first).

### Frame Canvas — Runtime Examples 🟢 LOW

- ⬜ **Canvas runtime examples have duplicated inline WASM loaders** — Five HTML demos under `plugins/frame.canvas/runtime/examples/` carry a full ~350–450-line inline WASM loader instead of using the shared `canvas-bridge.js` + `loader.js`:
  - `simple-demo.html` (385 lines — static draw scene, no event handlers)
  - `index.html` (365 lines — bouncing-balls, custom pointer event wiring)
  - `geometry.html` (390 lines — custom pointer event wiring)
  - `dashboard.html` (395 lines — custom pointer event wiring)
  - `gallery.html` (451 lines — custom pointer event wiring)

  Refactor blocked on two open questions: (a) the shared `loader.js`/`canvas-bridge.js` do not expose a `memory_runtime` module, yet the example .wasm files import `memory_runtime.mem_alloc` etc. — needs runtime verification that the shared-loader examples (`animated-ball.html`, `bouncing-balls.html`, `paddle-game.html`, `solar-system.html`) actually run; (b) the four examples with pointer handlers register them inline rather than via `_canvas_on_pointer_down()`, so the corresponding `.cln` files would need updating to register handlers per the bridge contract. Total duplicated code: ~2000 lines. Safe-to-migrate without code changes: `simple-demo.html` only.

### Phase 9.3: Testing Infrastructure 🟡 MEDIUM-HIGH

- ⬜ Implement `tests:` block parser in Clean Language *(cross-component: compiler)*
- ⬜ Implement `cleen test` command *(cross-component: clean-manager)*
- ⬜ Code coverage reporting
- ⬜ Coverage thresholds enforcement

### Phase 11: Performance & Optimization 🟢 LOW

- ⬜ Incremental compilation *(cross-component: compiler)*
- ⬜ Dead code elimination *(cross-component: compiler)*
- ⬜ WASM SIMD support *(cross-component: compiler)*
- ⬜ Connection pooling optimizations *[runtime]*
- ⬜ Response compression *[runtime]*
- ⬜ Code splitting
- ⬜ Lazy loading for components

### Phase 12: Developer Experience 🟡 MEDIUM-HIGH

- ⬜ Implement REPL *(cross-component: clean-manager or compiler)*
- ⬜ Hot module replacement (HMR) *[runtime + plugin]*

---

## Pending — Multi-Platform Architecture (forward-looking)

Reference: [system-documents/MULTI_PLATFORM_ARCHITECTURE.md](system-documents/MULTI_PLATFORM_ARCHITECTURE.md)

### Phase 1 — Structural Reorganization (Web Only) 🔴 CRITICAL
*No new features, no new plugins, full backwards compatibility.*

- ⬜ Support `main.cln` as package declaration entry point (`package:` block, declarative only)
- ⬜ Auto-routing from `src/web/pages/` by file path (e.g. `profile/[id].cln` → `/profile/:id`)
- ⬜ `routes.cln` for guards, redirects, rewrites, and error pages
- ⬜ Migrate existing `app/` folder convention to `src/` structure
- ⬜ `state/` as first-class folder alongside `logic/`, `data/`, `server/`
- ⬜ `server/api/` for explicit API endpoints only (page routing is automatic)

### Phase 2 — Abstract Component Layer (frame.ui) 🟡 MEDIUM-HIGH
*Must be fully specced in `foundation/spec/` and approved before any code is written.*

- ⬜ Define primitive component set in spec: `Box`, `Text`, `Image`, `Button`, `Row`, `Column`, `Input`, `Scroll`
- ⬜ Define `layout:` block syntax for composing primitives
- ⬜ Define component lifecycle protocol: mount, update, unmount
- ⬜ Define reconciliation protocol
- ⬜ Define `nav:` block for cross-platform navigation
- ⬜ Each renderer plugin declares which primitives it implements
- ⬜ Refactor frame.ui plugin to implement the abstract protocol (web renderer)

### Phase 3 — Canvas as Universal Renderer (frame.canvas) 🟡 MEDIUM-HIGH

- ⬜ Scene embedding in HTML via `cln-scene` directive on `<canvas>` element
- ⬜ Scene embedding in desktop/mobile screens via `layout:` canvas component
- ⬜ Add immediate mode (`onFrame:` loop) for games and audio alongside existing retained mode
- ⬜ `canvas/components/` for canvas-specific UI elements

### Phase 4 — frame.term (Terminal/TUI) 🟢 LOW

- ⬜ Select Rust TUI backend (crossterm or ratatui)
- ⬜ Host bridge for terminal I/O
- ⬜ Map frame.ui primitives to ANSI / box-drawing
- ⬜ Keyboard and mouse input through host bridge
- ⬜ `term/views/` auto-discovery

### Phase 5 — frame.desktop (Native Desktop) 🟢 LOW

- ⬜ Tauri host bridge: window, file system, system tray, native dialogs
- ⬜ Window lifecycle: open, close, minimize, resize
- ⬜ Menu bar and context menus
- ⬜ OS integration: notifications, file picker, clipboard
- ⬜ WebView rendering strategy first (frame.ui as HTML inside Tauri WebView)
- ⬜ Canvas rendering strategy later (frame.canvas inside Tauri — pixel-perfect)
- ⬜ `desktop/screens/` auto-discovery

### Phase 6 — frame.mobile (Native Mobile) 🟢 LOW

- ⬜ Capacitor shell: wrap web output in native iOS/Android shell
- ⬜ Capacitor plugin bridge: camera, GPS, push notifications, biometrics
- ⬜ `mobile/screens/` auto-discovery
- ⬜ Custom native bridge (future): WASM ↔ iOS UIKit / Android Views directly

---

## Out of Scope Here — Cross-Component Tracking

These items affect Frame but are owned by other repos. Listed for awareness only.

### 🔀 clean-server (runtime bridges)
- HTTP / DB / env / time / crypto / log / fs / sys bridge implementations
- Connection pooling, response compression
- Islands manifest serving, hydration runtime injection
- `bridge:email` namespace (blocks the password-reset pattern)
- Plugin permission allowlist enforcement

### 🔀 clean-manager / cleen
- `cleen new`, `serve`, `build`, `db:*`, `api:*` commands
- `cleen test` command
- REPL

### 🔀 clean-language-compiler
- `tests:` block parser
- Incremental compilation, dead-code elimination, WASM SIMD
- **Compiler bug**: nested if-else generates `unreachable` (workaround: invert + separate `if`)
- **Compiler bug**: `!=` operator in plugin output (workaround: `if not (a == b)` — frame.server:406)
- **Compiler bug**: `trim()` reliability (workaround: `strip_spaces()` in frame.ui:108-128)

---

## Completed Phases (summary)

| Phase | Area | Status |
|---|---|---|
| 1.1 | Host Bridge declarations (HTTP/DB/env/time/crypto/log/fs/sys) | ✅ Plugin-side complete; runtime in clean-server |
| 3.3–3.4 | endpoints, METHOD /path, guard/handle/returns/cache, request/response helpers, JSON/HTML/redirect/notFound/badRequest/unauthorized/forbidden/httpHeader | ✅ |
| 3.5 | SSR HTML processing, escaping, component HTML generation | ✅ (islands manifest pending) |
| 4.1–4.7 | Models, relationships, query builder, mutations, transactions, raw queries, migrations | ✅ |
| 4.8 | Field validation rules (min/max/email/range), `validate()` method | ✅ (runtime hook pending) |
| 4.9 | `data:` config, `env()` refs, `_db_configure()` | ✅ |
| 5.1–5.4 | Components, SSR, HTML-first pages, directives (`cl-if`/`cl-else`/`cl-iterate`/`cl-bind`/`cl-client`/`cl-show`/`cl-validate`/`cl-slot`) | ✅ |
| 5.5 | `client` attribute parsing | ✅ (full strategy pending in runtime) |
| 5.6–5.9 | Events + modifiers, theming, forms + CSRF, layouts + named slots | ✅ |
| 6.1–6.5 | Sessions, JWT, RBAC, password hashing, multi-tenant | ✅ |
| 7.1 | Plugin discovery, manifest parsing, lifecycle, implicit imports | ✅ |
| 9.1–9.2 | Test runner script, test fixtures, per-plugin test files | ✅ |
| 10.1–10.3 | Specs 01–14, bridge contracts, API reference, getting started, examples, edge-case docs | ✅ |

### Resolved Bug Fixes
- **Method call as statement** — fixed in compiler v0.30.98 (parser cursor restore in `statements.rs`)
- **HTML attribute quoting in frame.ui** — fixed 2026-04-03 (single-quoted HTML attrs, `&quot;` for JSON, runtime `string.fromCharCode(34)` for double quotes)
- **Plugin test file syntax** — fixed 2026-06-04 (functions wrapped in `functions:` block, `string`/`void` type declarations added per spec)

---

## Plugin Implementation Status

| Plugin | Status | Notes |
|---|---|---|
| frame.data    | ✅ Complete | validation, config, FK cascades, tenant isolation, migrations |
| frame.server  | ✅ Complete | returns + cache blocks |
| frame.auth    | ✅ Complete | multi-tenant support |
| frame.ui      | ✅ Complete | directives, events, theming, layouts, CSRF |
| frame.canvas  | ✅ Complete | input handling |
| frame.jobs    | ✅ Complete | background jobs DSL |
| frame.locale  | ✅ Complete | locale-aware formatting |
| frame.mcp     | ✅ Complete | MCP tool/resource handler generation |
| frame.client  | ✅ Complete | client-side api / live / feed |

All 9 plugins build cleanly with compiler 0.30.229.

---

**Last Updated**: 2026-06-04
