# Current State Report - Clean Framework UI Integration

**Date:** 2025-01-09
**Purpose:** Analysis of existing implementation before Clean UI integration

---

## 1. Repository Structure

### Key Directories Scanned

| Directory | Purpose | Status |
|-----------|---------|--------|
| `clean-language-compiler/src/` | Compiler (parser, AST, codegen, plugins) | Active |
| `clean-manager/src/core/` | Build pipeline (discovery, codegen, server) | Active |
| `clean-server/src/` | HTTP runtime (Axum, Wasmtime, host bridge) | Active |
| `clean-framework/plugins/` | Framework plugins (frame.ui, frame.data, etc.) | Partial |
| `clean-ui/` | Clean UI specifications | Spec only |
| `clean-canvas/` | Clean Canvas specifications | Spec only |

---

## 2. What Is Implemented

### 2.1 SSR Pages System (Working)

**Location:** `clean-manager/src/core/codegen.rs`

- HTML pages in `app/ui/pages/*.html`
- Components in `app/ui/components/*.cln`
- File-based routing (automatic discovery)
- Component tag expansion (`<app-header>` → function call)
- Route parameter extraction (`[slug].html` → `_req_param("slug")`)
- Variable interpolation (`{{slug}}` → string concatenation)

**Evidence:**
```rust
// clean-manager/src/core/codegen.rs:446
fn convert_html_to_clean(html: &str, components: &[Component]) -> Result<String>
```

### 2.2 Compiler Plugin System (Working)

**Location:** `clean-language-compiler/src/plugins/`

- `FrameworkPlugin` trait with `expand()` method
- WASM plugin loading via `WasmPluginLoader`
- `PluginRegistry` for registration
- `PluginExpander` for AST transformation
- Grammar support for `framework_block` statements

**Evidence:**
```pest
// grammar.pest:163
framework_block = { !keyword ~ identifier ~ ":" ~ framework_block_content }
```

### 2.3 Server Runtime (Working)

**Location:** `clean-server/src/`

- Axum HTTP server
- Wasmtime WASM execution
- Route matching via `matchit`
- Host bridge functions:
  - `_http_route`, `_http_route_protected`
  - `_req_param`, `_req_query`, `_req_body`, `_req_header`
  - `_req_method`, `_req_path`
  - `_auth_get_session`, `_auth_require_auth`, `_auth_require_role`
  - `_db_query`, `_db_execute` (via host-bridge crate)

### 2.4 Framework Plugins (WASM files exist)

| Plugin | WASM | Source | Status |
|--------|------|--------|--------|
| `frame.ui` | 7.5 KB | 852 lines | **NOT USED** - codegen bypasses it |
| `frame.data` | ~2 KB | 82 lines | Exists but limited |
| `frame.web` | exists | exists | Exists |
| `frame.auth` | exists | exists | Exists |

---

## 3. What Is Stubbed / Partial

### 3.1 frame.ui Plugin

**Location:** `clean-framework/plugins/frame.ui/`

The plugin exists and is well-written (852 lines of Clean code), but **it is completely bypassed**. The `clean-manager/src/core/codegen.rs` does HTML→Clean transformation directly in Rust instead of invoking the plugin WASM.

**Evidence:**
- `codegen.rs` has `convert_html_to_clean()` function written in Rust
- No code in `clean-manager` calls `frame.ui` plugin functions
- Plugin WASM is 7.5 KB but never loaded for HTML processing

### 3.2 frame.data ORM

**Location:** `clean-framework/plugins/frame.data/src/main.cln`

Basic expansion exists for `model:`, `query:`, `transaction:` blocks, but:
- No `find:`, `where:`, `order:`, `limit:` block-based query builder
- No relation support (hasMany, belongsTo)
- Just generates simple `_db_query()` calls

### 3.3 endpoints: Block Syntax

The grammar supports `framework_block` which would handle `endpoints:`, but:
- No working `frame.web` plugin that expands `endpoints:` to route registrations
- Current SSR pages don't use `endpoints:` blocks
- API routes are simple `.cln` files returning strings

---

## 4. What Is Spec-Only (Not Implemented)

### 4.1 Clean UI Widget System

**Specs:** `clean-ui/clean_ui_syntax_reference_v_1.md`, `clean_ui_elements_specification_v_1.md`

Defines:
- `ui.column`, `ui.row`, `ui.stack`, `ui.region`, `ui.spacer`
- `ui.text`, `ui.button`, `ui.textField`, `ui.checkbox`, `ui.select`
- `ui.canvasScene` for canvas integration
- `screen`, `app`, `state:` blocks
- Events: `onClick:`, `onChange (value):`, `onFocus:`, `onBlur:`
- Render targets: `dom`, `canvas`, `hybrid`

**Implementation Status:** NONE

No compiler support for `ui.*` syntax. The grammar doesn't parse these constructs. No runtime for client-side hydration or canvas rendering.

### 4.2 Clean Canvas System

**Spec:** `clean-canvas/clean_canvas_syntax_reference_v_1.md`

Defines:
- `canvas.clear`, `canvas.circle`, `canvas.rect`, `canvas.line`
- `canvas.text`, `canvas.image`
- `canvas.translate`, `canvas.rotate`, `canvas.scale`
- `canvas.save`, `canvas.restore`
- `onFrame (dt):`, `onPointerDown (x, y):`, `onKeyDown (key):`

**Implementation Status:** NONE

No host bridge functions for canvas operations. No client-side runtime.

### 4.3 Client-Side Hydration

**Spec:** `05_frame_ui.md` mentions `client="on|off|visible|idle|only"`

**Implementation Status:** NONE

- No hydration payload generation
- No client-side JavaScript runtime
- No event attachment after SSR
- Pages are fully static HTML after server render

---

## 5. What Is Dead / Unreferenced

### 5.1 frame.ui Plugin WASM

The `plugins/frame.ui/plugin.wasm` file (7.5 KB) is:
- Built and present
- Has comprehensive HTML processing logic
- Never invoked by the build system

The Rust codegen in `clean-manager` duplicates this functionality.

### 5.2 Spec Features in 05_frame_ui.md

The specification describes features that don't exist:
- `<script type="text/clean">` data blocks (partially implemented, but commented out)
- `client="visible"` lazy hydration
- Layout wrapping system
- Auth protection via `<page auth="required">`

---

## 6. Validated Findings

### A) frame.ui plugin WASM not used

**Confirmed:** `clean-manager/src/core/codegen.rs` has its own `convert_html_to_clean()` function. No code path loads or calls `frame.ui` plugin WASM for HTML processing.

### B) Clean UI spec (ui.column, ui.button, etc.) has no implementation

**Confirmed:** Grep for `ui\.column|ui\.row|ui\.text|ui\.button|screen|ui\.region` in compiler returns NO files. The grammar doesn't parse these constructs.

### C) Client-side hydration / interactivity status

**Confirmed:** No client-side JavaScript runtime exists. No hydration markers generated. No event attachment code. Pages are pure static HTML.

### D) endpoints: block syntax support in compiler

**Confirmed:** Grammar supports `framework_block` generically. A plugin COULD handle `endpoints:`, but current `frame.web` plugin doesn't fully implement this. SSR pages use file-based routing instead.

### E) ORM query builder syntax

**Confirmed:** `frame.data` plugin exists but only supports basic `model:`, `query:`, `transaction:` expansion. No `Article.find:`, `where:`, `order:` block-based query builder syntax.

---

## 7. Highest-Risk Areas

1. **Duplicate codegen paths** - HTML processing exists in both Rust (`codegen.rs`) and Clean (`frame.ui/src/main.cln`). Must decide which to use and remove the other.

2. **No client runtime** - Clean UI requires a client-side runtime for state, events, and canvas. This doesn't exist at all.

3. **Grammar gaps** - `ui.*` syntax needs grammar additions, not just plugins. Plugins can't add new syntax shapes.

4. **State management** - Clean UI's `state:` blocks and reactive updates require compiler support and runtime reactivity system.

5. **Canvas host bridge** - Canvas operations need new WASM imports (`canvas_clear`, `canvas_circle`, etc.) in the host bridge.

---

## 8. Summary

| Category | Count | Notes |
|----------|-------|-------|
| **Implemented** | 3 | SSR pages, plugin system, server runtime |
| **Stubbed** | 3 | frame.ui (bypassed), frame.data (basic), endpoints (grammar only) |
| **Spec-only** | 3 | Clean UI widgets, Canvas system, Hydration |
| **Dead code** | 2 | frame.ui WASM, spec features in 05_frame_ui.md |

**Bottom line:** The SSR HTML-first pages system works well. Clean UI / Canvas is entirely spec-only with zero implementation. Client-side interactivity doesn't exist.
