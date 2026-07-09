# bug-repro-db-in-browser

Minimal reproduction for two compiler bugs reported against the browser target of `frame.data`:

- **BRIDGE-HOST-MISMATCH** — fingerprint `ac38e3d2b5034f1a…`
- **CODEGEN-PLUGIN-EMITTED-HELPER-BRIDGE-DROP** — fingerprint `c2c6e33d…`

## Setup

```
target: browser
plugins: [frame.data, frame.ui]
```

`frame.data`'s `plugin.toml` declares `_db_query` with `hosts = ["server"]`. This example calls `Todo.exists:` from a browser-side click handler — code that is compiled into `frontend.wasm`.

## Expected behavior

Two acceptable outcomes:

1. **Hard compile-time error at the call site** (preferred DX):
   ```
   frame.data's `Todo.exists:` cannot be used from browser target —
   `_db_query` is server-only.
   ```

2. **At minimum:** `frontend.wasm` must NOT list `_db_query` as an import.

## Actual behavior (once helper-emit works)

The bug reports state that `frontend.wasm` contains a `_db_query` import even though the target is `browser`.

## How to inspect

```bash
# Build (with a compiler that emits plugin helpers)
cln build --target=browser

# Inspect imports
wasm-tools objdump build/frontend.wasm | grep -A1 import
# or
wasm-tools print build/frontend.wasm | grep '(import'
```

Look for a line like:

```
(import "env" "_db_query" (func ...))
```

If present, the bug is confirmed.

## Notes for the compiler team

- Two files touch the bug surface: `main.cln` (contains the inline `data Todo` model AND the browser handler calling `Todo.exists:`) and `index.html` (loader).
- The model is inlined intentionally — `cln build main.cln` does not auto-discover files under `app/data/models/` without a `package.clean.toml` or explicit `use` directive.
- No server endpoints are declared — so any `_db_query` import in the output is unambiguous evidence of the leak.
- The `frame.ui` plugin is included only to provide the DOM-side handler wiring; it is not part of the bug.
