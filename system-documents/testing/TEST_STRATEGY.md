# Clean Framework Test Strategy

**Status**: Active — enforced by hooks and CI
**Owners**: Framework maintainers
**Related**: [tests/CONVENTIONS.md](../../tests/CONVENTIONS.md), [.github/workflows/test.yml](../../.github/workflows/test.yml)

## 1. Purpose

Define how the Clean Framework is tested end-to-end, and how the "no placeholder tests" rule is enforced automatically. This document is the source of truth for:

- What each layer of the test pyramid covers.
- Which hook fires which layer, and why.
- How placeholder tests are detected and blocked before they land.
- What every plugin owes the test suite.

If it isn't in this doc, it isn't the strategy. Update this doc when the strategy changes.

## 2. Non-negotiable rules

1. **Every test calls something real.** No string-matching against DSL syntax. No `.contains("createToken")` on a literal. See [CONVENTIONS.md §9](../../tests/CONVENTIONS.md).
2. **Every test emits `PASS:` or `FAIL:`.** The runner greps stdout. Silent success is failure.
3. **Every test runs inside `clean-server`** against the real host bridge. No mocks. SQLite `:memory:` is the isolated DB.
4. **A failing test is fixed in code, not in the test.** If the test contradicts spec, fix the test *to match spec*, then note why in a comment.
5. **Every test file is discoverable by the runner.** File extension `.cln`, in a folder the runner walks. No parallel test frameworks.
6. **Placeholders are blocked at commit time.** The `scripts/check-test-placeholders.py` guard runs pre-commit, pre-push, and in CI. It exits non-zero on any placeholder pattern.
7. **(Aspirational, currently blocked)** Every test SHOULD declare its plugin dependencies explicitly via a `plugins:` block. The MCP server, IDE extension, and external plugin authors all depend on this to operate correctly. Implicit plugin activation is a compiler bug (#73cfeba24d14). However — as of this writing, adding an explicit `plugins:` block to a test file that uses `frame.auth`, `frame.canvas`, `frame.client`, `frame.mcp`, `frame.server`, or `frame.data` triggers OTHER upstream bugs (#43ee45647d28, #7536dcc58643, #4c71acb99571, #3654d453ebda, plus additional per-plugin scaffolding gaps discovered during v2.12.160 remediation). Consequently, tests today use implicit activation and rely on the compiler to route block names to installed plugins. Once the upstream bugs land, this rule becomes MUST, and a mechanical grep-and-insert pass will add `plugins:` to every test file. That change is tracked in [PENDING.md](../../tests/PENDING.md).

   In the interim, tests SHOULD add `plugins:` where it works today (currently: files that use only `jobs:`, `locale:`, or standalone `html:` blocks without a server layer). Doing so keeps them forward-compatible.

## 3. Test layers

The suite is a pyramid. Each layer answers a different question and runs on a different cadence.

### Layer 1 — Runtime smoke (`tests/framework/runtime/`)
- **Question**: Does the toolchain work at all?
- **Contents**: A handful of files. Each ~5–20 lines. Compiles, prints `PASS:`, exits.
- **Runtime**: <5s total.
- **Runs on**: pre-commit, pre-push, CI, nightly.
- **Failure meaning**: Compiler or clean-server is broken. Stop everything.

### Layer 2 — Bridge contract tests (`tests/framework/unit/bridge/`)
- **Question**: Does each host bridge namespace answer with the correct shape?
- **Contents**: One file per bridge namespace (`http`, `db`, `env`, `crypto`, `time`, `log`, `email`, `job`, `i18n`, `mcp`, `ui`, `canvas`).
- **What each does**: Calls the real bridge from a `start:` block using dot notation, verifies return shape and error semantics with `assertions.cln` helpers, prints `PASS: bridge-<namespace>` or `FAIL:`.
- **Runtime**: <30s total.
- **Runs on**: pre-push, CI, nightly.
- **Failure meaning**: A bridge signature drifted, or clean-server's implementation regressed.

### Layer 3 — Plugin unit tests (`tests/framework/unit/plugins/<plugin>/`)
- **Question**: Does the plugin's DSL expand into real, callable bridge work?
- **Contents**: One `.cln` file per feature area (see per-plugin coverage matrix in §7).
- **What each does**: Declares the plugin's DSL block(s) as a user would, calls the resulting dot-notation API from `start:` or from an endpoint, asserts on real return values or observable side effects (DB row, HTTP response, in-memory state).
- **Runtime target**: <5min total across all plugins.
- **Runs on**: pre-push (fast subset), CI (full), nightly (full).
- **Failure meaning**: Plugin DSL, plugin WASM, or the compiler regressed.

### Layer 4 — Integration tests (`tests/framework/integration/`)
- **Question**: Do plugins compose correctly across the request lifecycle?
- **Contents**:
  - `web_auth/*` — auth + server (session, JWT, role gates on HTTP endpoints)
  - `web_data/*` — data + server (models exposed via endpoints, CRUD roundtrips)
  - `cross_plugin/*` — 3+ plugin interactions (auth + data + ui, jobs + email + data, etc.)
- **What each does**: Compiles a small app, boots clean-server, sends real HTTP requests via `http.get`/`http.post` from within the test (server-hosted client), asserts on response bodies and DB state.
- **Runtime target**: <5min total.
- **Runs on**: CI, nightly.
- **Failure meaning**: A regression in plugin composition, not any single plugin.

### Layer 5 — E2E application tests (`tests/framework/e2e/`)
- **Question**: Does a realistic app boot, serve traffic, and produce correct output end-to-end?
- **Contents**: A handful of scenario apps (todo, blog, auth-flow, api-server, live-app). Each is a mini-application (models + endpoints + optional UI + optional client blocks) plus a driver that exercises the full flow.
- **What each does**: Compiles the app, boots it, hits it with a sequence of HTTP requests (`http.get`, `http.post`), verifies DB state, session cookies, redirects, rendered HTML, JSON shapes.
- **Runtime target**: <10min total.
- **Runs on**: CI, nightly.
- **Failure meaning**: Framework-level regression — something a real user would hit on day one.

### Layer 6 — Plugin-internal expansion tests (`plugins/<plugin>/tests/`)
- **Question**: Does the plugin's `expand_block` / `validate_block` produce the code we expect?
- **Contents**: One `test_expand.cln` per plugin. Calls the plugin's own expand/validate helpers on fixture input, asserts on returned code strings.
- **Runtime target**: <30s total.
- **Runs on**: Every plugin build (`plugin.wasm` rebuild), CI, nightly.
- **Failure meaning**: The plugin's own expansion logic regressed. This is the only place where asserting on code strings is legitimate — because this layer's *unit under test is a code generator*.

### Layer 7 — Nightly heavy tests
- **Question**: Do we survive slower checks that aren't worth blocking every push?
- **Contents**: Postgres integration (parity vs SQLite), soak tests (server up for 10min under load), fuzz on user input paths, cross-platform WASM validation.
- **Runtime target**: <60min.
- **Runs on**: nightly (`.github/workflows/nightly-canaries.yml`).
- **Failure meaning**: Non-blocking for merges, but opens an issue automatically.

## 4. Hook / trigger mapping

Each layer fires on the earliest trigger that keeps that trigger fast. The pattern is: cheap checks early and often, expensive checks less often.

| Trigger | Layers | Time budget | Purpose |
|---|---|---|---|
| **pre-commit** (client-side git hook) | Placeholder guard + Runtime smoke + fast lint | <15s | Block obvious mistakes before they enter history. Never block the developer for more than a few seconds. |
| **pre-push** (client-side git hook) | Placeholder guard + Runtime smoke + Bridge contract + Fast plugin subset | <2min | Last local gate before code leaves the machine. Catches most regressions without waiting for CI. |
| **CI on PR** (`.github/workflows/test.yml`) | All of Layers 1–6 + placeholder guard | <15min | Authoritative gate for merges. |
| **CI on push to main** | Same as PR + smoke-deploy check | <15min | Redundant safety net after merge. |
| **Nightly** (`.github/workflows/nightly-canaries.yml`) | Layer 7 (Postgres, soak, fuzz) + full re-run of Layers 1–6 | <60min | Catches slow-burn regressions and flake. Opens an issue on failure. |
| **Every plugin build** | Layer 6 for that plugin only | <10s | Confirms the plugin's expansion output is valid before publishing the WASM. |

**Why this shape:**
- Layers 1–2 catch most compiler/bridge drift and cost almost nothing — cheap enough to run at commit time.
- Layers 3–5 catch plugin and integration bugs but cost real time — fine for CI, not pre-commit.
- Layer 6 is tied to the plugin lifecycle, not the framework lifecycle.
- Layer 7 buys parity/soak coverage without slowing developers down.

**What each hook actually invokes** — see `scripts/hooks/` (installed via `scripts/install-hooks.sh`) and `.github/workflows/`.

## 5. Placeholder detection

The single most important policy enforcement mechanism. Runs in three places:

1. **`scripts/check-test-placeholders.py`** — walks every `.cln` under `tests/` and `plugins/*/tests/`, reports offenses, exits non-zero on any.
2. **pre-commit + pre-push hooks** — call the script on the staged/committed files. Blocks the operation on failure.
3. **CI job** — runs the script on the full tree as its own required check.

### What counts as a placeholder

The script flags a test file as a placeholder if **any** of the following are true:

| Pattern | Why it's banned |
|---|---|
| Function body whose only assertion is `.contains(...)`, `.indexOf(...)`, `.startsWith(...)`, or `.endsWith(...)` against a **literal string** | String-matching pretend test (CONVENTIONS.md §9). |
| Function body that assigns a string literal that *looks like source code* (e.g. contains `(`, `)`, `:`, `=`) and only checks properties of that literal | The "here's what the code should look like" anti-pattern. |
| File that never calls any dot-notation namespace (`auth.`, `db.`, `http.`, `ui.`, `canvas.`, `api.`, `live.`, `feed.`, `queue.`, `t(`, `email.`, `crypto.`, `env.`, `time.`, `log.`, `mcp.`, `session.`, `jwt.`, `storage.`) | No real API surface exercised. |
| File with no `PASS:` line in any code path | CONVENTIONS.md §6 violation. |
| File containing `TODO`, `FIXME`, `xfail`, `pending`, `skip`, `not implemented`, `placeholder`, or `stub` in a comment | Documented incompleteness. |
| Function returning only `true`, `false`, `0`, or a literal without exercising anything | Trivial return. |
| File in a test directory that references `tests/framework/utils/bridge_mocks.cln` | Deprecated mock layer. |

### Exemptions

Exactly two exempt paths, both narrow:
- `tests/framework/utils/*.cln` — helper library, not tests.
- `plugins/*/tests/test_expand.cln` — the plugin's own expansion tests, per §3 Layer 6, are allowed to assert on generated code strings. They must still emit `PASS:`.

Exemptions live in the script itself. Adding a new exempt path requires editing the script *and* this document in the same commit.

### On finding a real upstream bug

If a test surfaces a genuine compiler/plugin/server bug that blocks writing it correctly, per [CONVENTIONS.md §7](../../tests/CONVENTIONS.md):

1. Call `report_error` via the MCP tool (component = whichever component owns the fix).
2. Leave the test *not* committed to main — do not check in a placeholder standing in for it.
3. Add a line in [`tests/PENDING.md`](../../tests/PENDING.md) with the error code and the test's intended purpose.
4. When the upstream fix ships, write the real test in the same PR that closes the error dashboard entry.

Under no circumstances does a placeholder ship "for now."

## 6. Plugin coverage matrix

Each plugin owes the test suite the following files. These names are stable — the placeholder guard uses this list to detect *missing* coverage as well as *fake* coverage.

### frame.auth (`tests/framework/unit/plugins/auth/`)
- `jwt-signverify-test.cln` — `auth.jwt.sign` → `auth.jwt.verify` roundtrip; expiry rejection.
- `session-lifecycle-test.cln` — `auth.setSession` → `auth.getCurrentUser` → `auth.clearSession` with cookie assertion.
- `password-hash-test.cln` — `auth.hashPassword` → `auth.verifyPassword` with correct + wrong password.
- `role-check-test.cln` — `roles:` block declaration + `auth.hasRole` + `auth.can` positive + negative.
- `csrf-token-test.cln` — `auth.csrf.generate` → `auth.csrf.validate` roundtrip and rejection of tampered token.
- `jwt-refresh-test.cln` — `auth.jwt.refresh` single-use rotation, replay rejection.
- `reset-token-test.cln` — `auth.createResetToken` → `auth.consumeResetToken`, expiry, single-use.

### frame.data (`tests/framework/unit/plugins/data/`)
- `model-crud-test.cln` — declare model, `Model.insert:`, `Model.first:`, `Model.update:`, `Model.delete:`, verify row counts.
- `query-where-order-limit-test.cln` — `Model.find:` with `where:`, `order:`, `limit:`, `offset:`.
- `transaction-rollback-test.cln` — `transaction:` block committing, and rolling back on `onError:`.
- `pagination-page-test.cln` — `Model.paginate:` returns correct `totalPages`, `hasNext`.
- `pagination-cursor-test.cln` — `Model.cursor:` returns correct `nextCursor`, `hasPrev`.
- `raw-query-test.cln` — `db.query:` and `db.queryAs` with parameterized SQL.
- `junction-link-test.cln` — many-to-many via junction model, `link:` clause, traversal.
- `joins-test.cln` — inner/left/right joins with `select:` projection.
- `upsert-insertid-test.cln` — `Model.upsert:` and `Model.insert_id:`.

### frame.server (`tests/framework/unit/plugins/server/`)
- `endpoint-routing-test.cln` — GET/POST/PUT/PATCH/DELETE on paths with params; `req.param`, `req.query`, `req.body`; `json()` and `status(code, body)` responses.
- `error-handler-test.cln` — `server: handle:` catches `NotFound`, `ValidationError`, `_` fallback.
- `sse-stream-test.cln` — `FEED "/…":` `feed.emit`, `feed.emitEvent`, `feed.close`, verify wire format.
- `websocket-test.cln` — `LIVE "/…":` `live.send`, `live.roomBroadcast`, `live.close`.
- `email-send-test.cln` — `email.send` and `email.sendFrom` against a mock SMTP (in-process capture — permitted because SMTP is external I/O, not a framework surface).
- `middleware-modifiers-test.cln` — `[role]`, `cache(...)`, `middleware(...)` ordering + effect.

### frame.ui (`tests/framework/unit/plugins/ui/`)
- `page-render-test.cln` — `page:` block with `{{ }}` interpolation and `cl-if`, `cl-iterate`, verify rendered HTML.
- `component-props-state-test.cln` — `component:` with `inputs:`, `state:`, `render:`, `events:`; server-render + state update.
- `event-modifiers-test.cln` — `.prevent`, `.stop`, `.once`, key filters produce expected wired-up handler code.
- `directive-cl-bind-test.cln` — `cl-bind` two-way binding requires hydration.
- `directive-cl-stream-test.cln` — `cl-stream` emits `data-cl-stream` and the stream URL.
- `html-block-interpolation-test.cln` — `html:` block inside a function body with escape + `rawHtml`.
- `form-helpers-test.cln` — `ui.inputValue`, `ui.formJson`, `ui.formData`, `ui.checked` roundtrip.

### frame.canvas (`tests/framework/unit/plugins/canvas/`)
- `scene-boot-test.cln` — `canvasScene:` compiles, `init:` fires, `draw:` runs one frame, `PASS:` emitted from `onFrame:` on frame 1.
- `draw-primitives-test.cln` — call every draw primitive; assert no runtime traps.
- `tween-lifecycle-test.cln` — `tween:` play → pause → resume → stop transitions.
- `animstate-transitions-test.cln` — `animState:` transitions correctly on input.
- `particles-emit-test.cln` — `emit particles` and `start`/`stop`.
- `input-events-test.cln` — synthetic pointer + key events dispatched to handlers.
- `collision-primitives-test.cln` — `collision.rects`, `collision.circles`, `collision.pointRect` on known cases.

Canvas is browser-first. On the server (no DOM), tests assert on compilation + expansion + generated handler wiring, not on rendered pixels. Pixel-level tests live in a browser harness (future — tracked in `tests/PENDING.md`).

### frame.client (`tests/framework/unit/plugins/client/`)
- `api-request-test.cln` — `api.get`, `api.post`, `api.put`, `api.patch`, `api.delete`; assert on `result.ok`, `result.status`, `result.body`.
- `api-auth-headers-test.cln` — `api.auth("bearer", token)` sets Authorization header on subsequent calls.
- `live-open-send-close-test.cln` — `live.open`, `live.send`, `live.close` state machine.
- `feed-open-on-close-test.cln` — `feed.open`, named `feed.on`, `feed.close`.
- `load-block-test.cln` — `load: varName from "..." at "..."` populates state.
- `form-block-test.cln` — `form: post "..."` submit → success → clear.
- `send-block-test.cln` — `send: name delete "..."` with `with:` variables.

Client is browser-first; server-run tests exercise the compiled DSL output and stub responses through `clean-server`'s test transport.

### frame.jobs (`tests/framework/unit/plugins/jobs/`)
- `enqueue-run-test.cln` — `queue.enqueue`, worker runs handler, `job.succeed` writes result.
- `retry-policy-test.cln` — `retry:` block with `maxAttempts` + `backoff: exponential`; forced failure retries N times.
- `scheduled-cron-test.cln` — `schedule.cron` registers, fires on schedule tick, `schedule.cancel` removes.
- `job-context-test.cln` — `job.id`, `job.args`, `job.attempt` inside handler.
- `queue-inspect-test.cln` — `queue.status`, `queue.result`, `queue.cancel`.

### frame.locale (`tests/framework/unit/plugins/locale/`)
- `translation-basic-test.cln` — `t("key")` looks up key; missing key falls back.
- `plural-forms-test.cln` — `tc("key", n)` picks correct CLDR form.
- `format-helpers-test.cln` — `locale.formatNumber`, `locale.formatCurrency`, `locale.formatDate`.

### frame.mcp (`tests/framework/unit/plugins/mcp/`)
- `mcp-server-boot-test.cln` — `mcp "name":` with a single tool boots and responds to `initialize`.
- `tool-dispatch-test.cln` — `tool "x":` with `params:` invoked via JSON-RPC returns the expected shape.
- `resource-fetch-test.cln` — `resource "scheme://…"` returns declared mimeType.
- `prompt-render-test.cln` — `prompt "x":` with `args:` renders template.

### Bridge contract (`tests/framework/unit/bridge/`)
One file per namespace: `http`, `db`, `env`, `crypto`, `time`, `log`, `email`, `job`, `i18n`, `mcp`, `ui`, `canvas`. Each file calls every documented function in its namespace at least once, asserts on shape.

### Integration (`tests/framework/integration/`)
- `web_auth/protected-routes-test.cln` — `[admin]` on endpoint → 401 without session, 200 with, 403 with wrong role.
- `web_auth/jwt-flow-test.cln` — login → refresh → protected call.
- `web_data/crud-endpoints-test.cln` — POST creates, GET reads, PATCH updates, DELETE removes; verify DB.
- `web_data/pagination-endpoint-test.cln` — endpoint returns paginated response.
- `cross_plugin/auth-data-ui-test.cln` — protected page renders with user data from DB.
- `cross_plugin/jobs-email-test.cln` — enqueue job that sends email; verify email captured.
- `cross_plugin/client-server-loop-test.cln` — server exposes endpoint, client-block calls it, response drives state.

### E2E (`tests/framework/e2e/`)
- `todo-app-test.cln` — full CRUD app end-to-end.
- `auth-flow-test.cln` — signup → login → protected → logout.
- `blog-app-test.cln` — posts + comments with roles.
- `api-server-test.cln` — REST API only, JSON only, error handlers.
- `client-app-test.cln` — SPA-shape: load, form, send, live, feed.

### Plugin-internal (`plugins/<plugin>/tests/`)
- One `test_expand.cln` per plugin. Asserts on generated code strings from `expand_block`.

## 7. Migration plan (current placeholder → real)

Ordering: bridge → auth → data → server → ui → canvas → client → jobs → locale → mcp → integration → e2e. Each plugin becomes a single PR that:

1. Rewrites every placeholder file in that plugin's folder to the file names above.
2. Deletes any old files that don't match the target list.
3. Updates the CI matrix to include the new folder.
4. Passes `check-test-placeholders.py` on the full tree.

While a plugin's rewrite is in progress, its old placeholder files remain — but the *guard* excludes only files listed in a temporary allowlist at `tests/PENDING.md` with the error code they're waiting on. Once the plugin's PR merges, its entries drop out of `PENDING.md`. `PENDING.md` must be empty by the end of the migration.

## 8. Maintenance & policy

### For maintainers reviewing PRs

- Every PR that touches a plugin must touch its unit test folder in the same commit. If it doesn't, the reviewer asks for the test or rejects the PR.
- Every PR that adds a bridge function must add a call to it in the bridge contract test. If it doesn't, `check-test-placeholders.py` may still pass, but the reviewer catches it via the coverage matrix in §6.
- Every PR that fixes a bug reported via `report_error` must include a test that would have caught the bug. This test is the definition-of-done, per CONVENTIONS.md §7.

### For agents (including Claude Code)

`CLAUDE.md` includes the following invariant, which agents must respect on every change:

> When editing plugin code, update the matching file(s) in `tests/framework/unit/plugins/<plugin>/` per `system-documents/testing/TEST_STRATEGY.md` §6. Do not add placeholder tests. If the fix cannot be verified without an upstream fix elsewhere, call `report_error` and add an entry to `tests/PENDING.md` — do not commit a fake test.

### For CI

- `.github/workflows/test.yml` runs `check-test-placeholders.py` as its first job (before compiling anything). Fail-fast.
- `.github/workflows/nightly-canaries.yml` runs the full suite + Postgres integration + soak.
- Both workflows fail loud on `FAIL:` in test output and on no-`PASS:`-line files.

### For hooks

- `scripts/install-hooks.sh` installs `pre-commit` and `pre-push` symlinks pointing to `scripts/hooks/`.
- The hooks fail closed: a hook that errors out for any reason exits non-zero. Do not add `|| true`.

## 9. Explicit non-goals

- **No coverage percentage target.** Coverage numbers are noise on a suite this size. The metric is *"every plugin's coverage matrix in §6 is green."*
- **No Rust or JS tests.** The framework is Clean Language; the tests must exercise the same execution path users hit.
- **No test-only DSL.** If a test needs a new API surface, add it to the plugin proper. Tests aren't a place to prototype hidden features.
- **No `skip` or `xfail` directives.** If a test can't run today, it doesn't exist yet — it lives in `tests/PENDING.md` until the upstream fix ships.
- **No re-testing the compiler.** Compiler correctness is the compiler's own test suite. Framework tests test the framework.

## 10. When this document is wrong

If the strategy in this document conflicts with reality (a plugin ships without matching tests, CI runs a layer this doc doesn't mention, a hook exists that isn't listed), fix the discrepancy in the same PR. Either the code or this document is wrong — the code and the doc must not diverge.

The last section of every PR description that touches tests or CI must answer: *"Does this change require an update to `TEST_STRATEGY.md`?"*
