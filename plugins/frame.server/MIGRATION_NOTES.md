# frame.server v3 Typed Emission Migration Notes

**Date:** 2026-07-01
**Target:** Plugin Contracts v3 — atomic migration from v1 string emission to v3 typed AST emission
**Compiler required:** 0.30.412 (ships `_emit_stmt_from_source` at bridges.rs:786–856)
**Spec:** foundation/spec/plugins/contracts/typed-emission.md (Amendment 1 + Amendment 2)
**Pilot reference:** plugins/frame.locale commit d252787 (proves the recipe)

---

## 1. Why This Migration

Five open dashboard bugs (5e144689da64, 835d361c9515, cdb735a9453a, 475477313256, 7f8d244b7a46)
all trace to `expand_endpoints` building a Clean source string via repeated `string.concat`.
The resulting buffer crosses 64 KiB WASM page boundaries, producing null-byte corruption,
OOB pointer arithmetic, and allocator exhaustion (O(N²) triangular-sum concat loop).

Typed emission removes the bug class structurally: the plugin never produces a result
string. The compiler builds AST nodes directly from typed emission calls.

---

## 2. Atomic Migration Constraint

Per Amendment 1: a single plugin.wasm cannot host v1 exports and v3 imports simultaneously.
The v1 dispatch linker refuses instantiation with "unknown import: env::_stmt_call".
This migration is therefore all-or-nothing in one release.

**v1 exports to DELETE:**
- `expand_block` (replaced by `expand_block_typed`)
- `emit_server_helpers` (replaced by `emit_module_helpers_typed`)
- `emit_server_init` (replaced by `emit_server_init_typed`)
- `emit_per_request` (replaced by `emit_per_request_typed`)

**v3 exports to ADD:**
- `expand_block_typed`
- `emit_module_helpers_typed`
- `emit_server_init_typed`
- `emit_per_request_typed`

Validate, get_keywords remain unchanged (they are not lifecycle slots and have no v1/v3 split).

---

## 3. Handler Body Strategy — §3.11 Pass-Through

frame.server's core challenge: user-authored handler bodies.

The spec §3.11 `_emit_stmt_from_source` bridge is designed exactly for this:
- The plugin extracts the raw handler body substring from `body_lp` using existing
  `extract_handler_body` (which does re-indentation). 
- The extracted fragment is the user's code, verbatim (no plugin-built strings mixed in).
- Call `_emit_stmt_from_source(ctx, extracted_body, origin_offset)` → stmt_handle.
- Pass stmt_handle to `_emit_function` as the body.

**CRITICAL**: The extracted body from `extract_handler_body` is re-indented to `\t\t` base.
We need to call `_emit_stmt_from_source` on this re-indented body. The parser is
indentation-aware; the fragment must be parseable as a standalone statement block.
We wrap it in a synthetic statement block context by passing the extracted lines directly.

For `origin_offset`: since `extract_handler_body` slices from `remaining` (which itself
was sliced from `body`), exact byte offsets are not trivially available. We pass `0`
(acceptable per spec §3.11 — diagnostics will position at block start rather than exact
line, but this is better than the zero diagnostic quality of v1).

---

## 4. V1 Code Path Map → V3 Bridge Sequence

### 4.1 `expand_endpoints` → `expand_block_typed` for "endpoints" block

| v1 action | v3 equivalent |
|-----------|--------------|
| `result = "functions:\n"` | (no string; emit functions directly) |
| `result = result + "\tstring " + handler_name + "()\n"` | `_emit_function(ctx, handler_name, "[]", void_t, body_stmt, 2)` |
| `route_calls = route_calls + "\t_http_route(...)` | `_emit_route(ctx, method, path, handler_fn, "{}")` |
| `result = result + "start:\n" + route_calls` | (route_calls go to server_init via `_emit_route` sugar) |
| For LIVE: `result = result + "\tvoid " + connect_name + "()\n"` | `_emit_function(ctx, connect_name, "[]", void_t, connect_body_stmt, 2)` |
| For LIVE: `route_calls += "\t_http_ws_route(...)` | `_emit_statement_into_start(ctx, ws_route_stmt)` |

Guard code, cache code, middleware code, and the auto-wrap logic become typed stmt
sequences prepended to the handler body block.

**Key insight for auto-wrap**: Instead of string-rewriting `return X` → `return __route_default_body(X)`,
in v3 we still call `extract_handler_body` and then `_emit_stmt_from_source`. The
`handler_sets_response` check still guards whether to wrap. For the wrap case we still
do the string replacement on the extracted body **before** passing to `_emit_stmt_from_source`.
This is legitimate: the extracted body is user-authored, and the wrap replaces identifiers
within it, not mixed plugin-string construction. Alternative: accept this is a string
operation on user-authored content and apply it pre-`_emit_stmt_from_source`.

**Decision**: For auto-wrap and guard/cache/middleware injection, we build a compound
stmt_block:
1. Guard stmts (if roles present): built via `_stmt_call(_auth_require_role, ...)` + `_stmt_if` for the check
2. Cache stmt (if cache present): `_stmt_call(_http_set_cache, [int_lit])` or `_stmt_call(_http_no_cache, [])`
3. Middleware stmts (if mw present): `_stmt_call(mw_name, [])` + assign + `_stmt_if`
4. Handler body stmt: `_emit_stmt_from_source(ctx, handler_code, 0)` (where handler_code
   has the auto-wrap already applied if needed)
5. `_stmt_block(ctx, "[guard_handles..., cache_handle, mw_handles..., body_handle]")` → compound_block

### 4.2 `emit_server_helpers` → `emit_module_helpers_typed`

The response helper functions (jsonResponse, badRequest, etc.) currently emitted via
`emit_response_helpers()` as a string must be emitted via `_emit_function` calls.

**Challenge**: emit_response_helpers emits ~15 wrapper functions, each with one or two
body statements. These function bodies ARE plugin-authored (not user code), so we cannot
use `_emit_stmt_from_source` — that bridge is for user-authored content only.

**Solution**: Each helper function body is a single statement (one `_http_respond(...)` call
or similar). Build each body via `_stmt_call` + `_stmt_return` + `_stmt_block`.

**Alternatively** (simpler): The response helpers are plugin-authored Clean fragments that
the compiler needs to parse. These are NOT user-authored content — they are fixed, known,
safe strings. BUT using `_emit_stmt_from_source` with plugin-authored strings is explicitly
prohibited by spec §3.11.

**Correct approach**: Use the typed statement constructors for every helper function:
- `_stmt_return(ctx, _expr_call(ctx, "_http_respond", "[status_expr, ct_expr, body_expr]"))` → return_stmt
- `_stmt_block(ctx, "[return_stmt]")` → body_block
- `_emit_function(ctx, "jsonResponse", '[{"name":"data","type_handle":str_t}]', str_t, body_block, 2)`

This is verbose but correct and complete. The typed-emission.md §11 estimates this as
~90 LOC for emit_module_helpers_typed.

### 4.3 `emit_server_init` → `emit_server_init_typed`

Current v1 slot returns empty string. In v3: emit nothing. Return 0.

Route registrations happen via `_emit_route` from within `expand_block_typed`, which
per spec §3.6 handles both function emission and server_init registration in one call.

### 4.4 `emit_per_request` → `emit_per_request_typed`

Current v1 slot returns empty string. In v3: emit nothing. Return 0.

### 4.5 LIVE/WebSocket handlers

Same `_emit_stmt_from_source` pattern for ws_connect_body, ws_message_body, ws_close_body.
Route registration uses `_emit_statement_into_start` with a `_stmt_call` to `_http_ws_route`.

### 4.6 `expand_server` (server: block)

This block doesn't handle user handler bodies — it generates plugin-authored start: statements.
ALL code here is plugin-authored (port, static dir, mail config, cors config, etc.).
Every statement → typed constructors:
- `_stmt_call(ctx, "_email_configure", "[host_expr, port_expr, ...]")` → stmt
- `_emit_statement_into_start(ctx, stmt)` 

The global error handler function body IS user-authored (captured from `handle:` clause).
→ `_emit_stmt_from_source(ctx, handle_body, 0)` for the body, then `_emit_function`.

### 4.7 `expand_routes` (routes: block)

The redirect directives generate purely plugin-authored bridge calls:
- `_stmt_call(ctx, "_http_redirect_route", '[from_expr, to_expr, status_expr]')` → stmt
- `_emit_statement_into_start(ctx, stmt)` 

Guard: `_build_state_get`/`_build_state_set` remain — these are [bridge] functions,
not typed-emission bridges. They continue to be called as regular external functions.

---

## 5. Bridges Required

From typed-emission.md §3 + bridges.rs analysis:

```clean
external:
    // Statement constructors
    integer _stmt_call(integer ctx, string callee, string args_json)
    integer _stmt_assign(integer ctx, string target, integer expr_handle)
    integer _stmt_if(integer ctx, integer cond_handle, integer then_block_handle, integer else_block_handle)
    integer _stmt_return(integer ctx, integer expr_handle)
    integer _stmt_block(integer ctx, string stmts_json)
    // Expression constructors
    integer _expr_string_lit(integer ctx, string value)
    integer _expr_int_lit(integer ctx, integer value_lo, integer value_hi)
    integer _expr_bool_lit(integer ctx, integer value)
    integer _expr_ident(integer ctx, string name)
    integer _expr_call(integer ctx, string callee, string args_json)
    integer _expr_binop(integer ctx, integer op_code, integer lhs_handle, integer rhs_handle)
    // Type constructors
    integer _type_string(integer ctx)
    integer _type_integer(integer ctx)
    integer _type_boolean(integer ctx)
    integer _type_void(integer ctx)
    // Declaration emitters
    integer _emit_function(integer ctx, string name, string params_json, integer return_type, integer body_stmt, integer flags)
    integer _emit_route(integer ctx, string method, string path, integer handler_fn_handle, string attrs_json)
    integer _emit_statement_into_start(integer ctx, integer stmt_handle)
    integer _emit_error(integer ctx, integer severity, string code, string message, string span)
    // Pass-through bridge (§3.11)
    integer _emit_stmt_from_source(integer ctx, string source, integer origin_offset)
```

Total: 20 bridges.

---

## 6. plugin.toml Changes

```toml
[compatibility]
expansion_version = "3.0.0"
emission_ops_hash = "60d06836a7068ee66a330e8c26ca59bc47cc44991b4f31d56e95e06c707eeaf0"
min_compiler_version = "0.30.412"

[blocks]
endpoints  = { expand = "expand_block_typed", version = 3 }
server     = { expand = "expand_block_typed", version = 3 }
routes     = { expand = "expand_block_typed", version = 3 }
middleware = { expand = "expand_block_typed", version = 3 }

[lifecycle]
module_helpers_are_roots = true
module_helpers = "emit_module_helpers_typed"
server_init    = "emit_server_init_typed"
per_request    = "emit_per_request_typed"
```

Note: `[exports]` table must NOT list `expand` (v1 entry) any more. The v3 path uses
`[blocks]` entries and typed lifecycle slots.

---

## 7. Sub-cycle Plan

1. **Prep**: This document (MIGRATION_NOTES.md). No WASM changes.
2. **module_helpers slot**: `emit_module_helpers_typed` — 15 helper functions via `_emit_function`.
3. **expand_block_typed core**: endpoints block handling — the main migration.
4. **server + routes blocks**: `expand_server_typed`, `expand_routes_typed` via typed stmts.
5. **Lifecycle cleanup**: `emit_server_init_typed`, `emit_per_request_typed` (both empty/trivial).
6. **Prune v1 exports**: Delete `expand_block`, `emit_server_helpers`, `emit_server_init`, `emit_per_request`.
7. **Build + verify + COMITA tag**.

---

## 8. Bugs Expected to Close

| Bug fingerprint | Expected outcome |
|----------------|-----------------|
| 5e144689da64 | CLOSED — no page-boundary buffer; typed emission |
| 835d361c9515 | CLOSED — no result string; no null-byte insertion point |
| cdb735a9453a | CLOSED — no string concat for re-parse; eliminates SYN001 source |
| 475477313256 | INVESTIGATE — may have secondary cause in codegen unrelated to plugin |
| 7f8d244b7a46 | CLOSED — no triangular-sum concat; eliminates PLUGIN_TRAP on large endpoints |
