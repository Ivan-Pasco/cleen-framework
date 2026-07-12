# Migration Plan — Data Persistence Model (Plugin-First)

**Status:** In progress. Revised 2026-07-10; plan approved and Phase 1 completed same day. Phases 2, 3, 4 partial or blocked as detailed below.
**Scope:** Full migration of Frame's data persistence model. Touches the `frame.data` plugin, its plugin-specific spec files, framework documentation, books, tests, examples, and MCP server responses. No compiler grammar/parser/semantic/codegen changes.
**Audience:** Framework owner, framework team, MCP maintainers, doc writers. Not user-facing.
**Read first:** `SPEC_DATA_PERSISTENCE_MODEL.md` — this plan assumes that spec is the target.
**Authoritative summary:** `/Users/earcandy/.claude/plans/staged-splashing-starlight.md` — approved plan file. This document expands on that plan with per-phase detail.

**Phase status as of 2026-07-11:**

- **Phase 1 (Plugin spec finalization):** ✅ Complete. `foundation/spec/plugins/frame-data.ebnf` and `frame-data-semantics.md` both at v2.0.0. Draft spec renamed to `SPEC_DATA_PERSISTENCE_MODEL.md`; all eight open questions resolved. Developer approval recorded 2026-07-10.
- **Phase 2 (frame.data plugin rewrite):** ⏸️ Blocked. Scope collision with in-flight v3 sub-cycle plan discovered. See `foundation/management/cross-component-prompts/framework-data-persistence-migration-phase-2-scope-collision.md` and the re-baking proposal at `foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md`. Awaiting sub-cycle plan owner's Option A / Option B decision.
- **Phase 3 (docs and books rewrite):** ✅ Complete for first-tier docs (`04_frame_data.md`, `PROJECT_STRUCTURE.md`, `GETTING_STARTED.md`, `API_REFERENCE.md`, `09_frame_dev_guidelines.md`, `01_frame_overview.md`, `03_frame_server.md`, `06_frame_auth.md`). ⏸️ Books deferred to Phase 4 AI-driven migration mechanism (~273 references across book-2-frame and book-5-production).
- **Phase 4 (MCP updates + AI-driven verification):** Partial. ✅ MCP response updates complete (`get_app_structure` fully rewritten, `get_quick_reference` targeted edits, other tools verified as needing no changes). ⏸️ AI-driven migration verification blocked on Phase 2 (needs working v2 plugin to migrate example apps against).

This document is the honest work plan for executing the migration. It orders the work into phases, names each affected component, defines exit criteria, and calls out the real risks.

Important upfront:

- **This is a breaking change.** No backwards compatibility. Every existing Frame application using `Model.insert:`, `Model.update:`, `Model.delete:`, or bare-field `data:` declarations breaks until migrated.
- **Migration mechanism is AI-with-MCP.** Frame is an AI-friendly framework. Existing apps are migrated by AI instances using updated MCP responses (Phase 4) and updated docs (Phase 3), plus the plugin's clear rejection diagnostics for removed syntax (Phase 2). No dedicated migration CLI is built.
- **Cross-component coordination is required.** The framework, plugin specs, and MCP server live in separate components (per the project's `CLAUDE.md` boundaries). No single AI instance can execute this plan end-to-end; work must be handed off between components using the `foundation/management/cross-component-prompts/` protocol.
- **Estimated effort is 2–4 weeks** across all components, assuming design accepted without major revision. Substantially shorter than earlier estimates because no compiler work is needed and no migration tooling is built.

---

## 1. Scope and affected components

### 1.1 Architectural finding that shapes this plan

Read-only exploration of the compiler/plugin boundary confirmed:

- **The Clean Language compiler treats `data:` block contents as opaque text.** Per `foundation/spec/grammar.ebnf` §6.13, `framework_block_content = { ANY - NEWLINE }` — raw characters, not structured sub-blocks. Per `foundation/spec/plugins/plugin-contract.md` §5.1, the compiler passes the raw body to the plugin's `expand_block_typed` as a length-prefixed JSON string. **The compiler never parses the contents of a `data:` block.** Plugins can add, remove, or restructure sub-blocks inside `data:` freely without any compiler grammar changes.
- **Plugins add "new syntax" by generating standard Clean code.** The plugin's `expand` function returns a JSON envelope containing Clean source that the compiler re-parses through the full pipeline. The `.data` accessor and the `Database` service are implemented by having the plugin generate the appropriate classes and methods; the compiler treats them as ordinary Clean code.
- **`frame.data`'s existing architecture already fits this migration.** The plugin uses `expand_block_typed` (`plugins/frame.data/src/main.cln` lines 108–135) as a dispatcher that reads block bodies and routes to sub-expanders. It emits typed AST via bridge functions from `plugin-contract.md`. The migration extends this existing pattern — it does not require a new plugin capability.
- **Plugin-specific spec files exist and are the right home for grammar changes.** `foundation/spec/plugins/frame-data.ebnf` and `foundation/spec/plugins/frame-data-semantics.md` describe the plugin's DSL and semantics separately from the core language spec. Grammar changes for this migration land there.

**Consequence:** the migration is entirely plugin-side plus documentation and MCP-response updates. No core language, no compiler parser, no compiler semantic analysis, no compiler codegen changes.

### 1.2 Components touched by this migration

| Component | Files affected | Nature of change |
|---|---|---|
| **Plugin spec** | `foundation/spec/plugins/frame-data.ebnf`, `foundation/spec/plugins/frame-data-semantics.md` | Rewrite plugin syntax spec: new `data:` sub-blocks (`table`, `fields:`, `indexes:`, `relations:`, `queries:`), `.data` accessor, `Database` service surface, removal of block-form mutations. Requires developer approval per Principle 25. |
| **`frame.data` plugin** | `clean-framework/plugins/frame.data/src/main.cln` (~4,422 lines), `plugin.toml`, `tests/` | Substantial rewrite of `expand_block_typed` and its sub-expanders. Remove `*.insert:` / `*.update:` / `*.delete:` handlers. Add `.data.*` accessor generation. Add `Database` service class generation. Add entity/data-block pairing verification. Add invariant enforcement in generated write paths. |
| **Framework spec docs** | `clean-framework/documents/specification/04_frame_data.md`, `09_frame_dev_guidelines.md`, `PROJECT_STRUCTURE.md`, `GETTING_STARTED.md`, `API_REFERENCE.md`, `README.md` | Rewrite `04_frame_data.md` end-to-end. Update related docs referencing the data model or folder layout. |
| **Books** | `books-and-content/book-1-starter/` and any other Frame books | Rewrite chapters touching data. Update every code example. |
| **Framework tests** | `clean-framework/tests/` (~4 files exercise `data:` or mutations) | Migrate to new syntax; add new-feature coverage. |
| **Framework examples** | `clean-framework/examples/` | Migrate to new syntax via AI-with-MCP flow (also proves the migration mechanism). |
| **Compiler-side test fixtures** | `clean-language-compiler/tests/cln/` (~6 files exercise `data:` or mutation DSL) | Migrate to new syntax. The compiler grammar itself does not change, so no grammar-regression tests are added — just update the fixtures to compile under the new plugin. |
| **MCP server** | `clean-language-compiler/src/mcp/server.rs` (functions at lines ~2497, 2790, 3587) | Update `get_specification`, `get_quick_reference`, `get_app_structure` responses to reflect new patterns. Rust source edit in the compiler component but content-only (not a language change). |
| **Cross-component prompts** | `foundation/management/cross-component-prompts/` | Write 4 new prompts and reconcile 2 existing overlapping proposals. |

### 1.3 Components deliberately NOT affected

For clarity, these are unaffected by this migration:

- **Core language spec** — `foundation/spec/grammar.ebnf`, `semantic-rules.md`, `type-system.md`, `ast.md`, `stdlib-reference.md`. No changes.
- **Compiler internals** — `clean-language-compiler/src/parser/`, `src/semantic/`, `src/codegen/`, `src/ast/`. No changes.
- **Other framework plugins** — `frame.auth`, `frame.ui`, `frame.server`, `frame.canvas`, `frame.jobs`, `frame.locale`, `frame.mcp`, `frame.client`. Only `frame.data` is affected.
- **Host bridge implementations** — `clean-server`, `clean-node-server`, `clean-framework` browser runtime. The bridge functions (`bridge:db.*`) that `frame.data` uses do not change; only the plugin's generated code changes.
- **`clean-manager` (`cleen`)** — no migration CLI is built. Registry and installer are unaffected.
- **`clean-ui`, `clean-canvas`, other language sibling components** — separate concerns.
- **VS Code extension** — thin LSP client. All language intelligence comes from the language server (bundled with the compiler); syntax highlighting via semantic tokens. Since the compiler grammar doesn't change, extension needs no update.

### 1.4 Migration mechanism

Existing Frame apps are migrated by AI instances (Claude Code or equivalent) using:

- **Updated MCP responses** (Phase 4) — provide the new patterns and structural guidance.
- **Updated docs and books** (Phase 3) — reference material the AI reads to understand the new model.
- **Plugin rejection diagnostics** (Phase 2) — when the AI tries to compile old-style code, the plugin's clear error message names the replacement pattern (e.g., `"Model.insert: is no longer supported — use Database.save(entity) — see 04_frame_data.md §7"`).

No dedicated `cleen migrate` command is built. This is a deliberate choice reflecting Frame's positioning as an AI-first framework, and it drops one whole phase from the plan.

**Consequence:** Phase 4 (MCP responses) is not just AI guidance — it is the primary migration surface. Its quality determines whether the migration succeeds in practice. Phase 4's exit criteria explicitly include AI-driven migration verification (see §2.4).

---

## 2. Migration phases

Four phases. Phases 1 and 2 sequential. Phases 3 and 4 parallelize once Phase 2 completes.

### Phase 1 — Plugin spec finalization

**Goal:** Turn the draft spec into a committed plugin spec. Resolve all open questions. Reconcile with existing overlapping proposals.

**Entry criteria:**
- `SPEC_DATA_PERSISTENCE_MODEL.md` reviewed by framework owner.
- No blocking design questions remain.

**Work:**

1. **Resolve the eight open questions in §11 of the draft spec:**
   - §11.1 Sentinel value for unpersisted `id`s.
   - §11.2 Connection routing.
   - §11.3 Semantics of `Database.delete` on missing row.
   - §11.4 Nested transactions (resolved — inherits current spec's compile-time error).
   - §11.5 Cross-entity joins in the query DSL.
   - §11.6 Domain-only fields (computed properties).
   - §11.7 Named-argument construction.
   - §11.8 Where cross-entity queries live.

2. **Rewrite `foundation/spec/plugins/frame-data.ebnf`:**
   - New productions for `data:` block sub-blocks: `table`, `fields:` (with column-name overrides and constraints), `indexes:`, `relations:`, `queries:`.
   - Remove productions for bare-field `data:` declaration form.
   - Remove productions for block-form mutations (`*.insert:`, `*.update:`, `*.delete:`).
   - New productions for `.data` accessor path.
   - Retain productions for query DSL (`find`, `first`, `findOrFail`, `count`, `exists` with sub-blocks).
   - Retain productions for `transaction:` block.

3. **Rewrite `foundation/spec/plugins/frame-data-semantics.md`:**
   - Semantics for entity/data-block pairing (compile-time check, plugin-owned).
   - Semantics for `.data` accessor resolution and dispatch.
   - Semantics for `Database.save`/`Database.delete` (invariant enforcement, `id` handling per §11.1).
   - Removal of semantics for block-form mutations.
   - Retained semantics for the query DSL (unchanged).

4. **Get developer approval** per Principle 25 (spec changes require explicit approval). No spec file lands without approval.

5. **Reconcile with existing overlapping proposals in `foundation/management/cross-component-prompts/`:**
   - `compiler-mcp-app-structure-add-entity-folder-and-dbc.md` — its `app/entity/` guidance folds into this migration's Phase 4 (MCP updates). Either supersede or merge, but don't leave two competing sources.
   - `framework-frame-data-examples-driver-and-entity-persistence.md` — retain Feature 2 (examples driver); retire Feature 1 (Stage B auto-persistence), which is superseded by the `.data` accessor + `Database` service design.

**Exit criteria:**
- Committed plugin spec files (`frame-data.ebnf`, `frame-data-semantics.md`) with developer approval documented.
- All eight open questions resolved and folded into the spec body.
- The two existing overlapping proposals reconciled (updated or retired).

**Deliverables:**
- Committed `foundation/spec/plugins/frame-data.ebnf`.
- Committed `foundation/spec/plugins/frame-data-semantics.md`.
- Updated `SPEC_DATA_PERSISTENCE_MODEL.md` promoted from draft to committed (removed `DRAFT_` prefix).
- Cross-component prompt for the framework team (`framework-data-persistence-migration-phase-1-spec.md`) if the spec work is handed off to a session working in a different repo.

**Risks:**
- Open questions may take multiple rounds of discussion before resolution. Budget time.
- Developer may reject parts of the design. Budget for revision.

---

### Phase 2 — `frame.data` plugin rewrite

**Goal:** `frame.data` plugin implements the new model end-to-end. Old syntax rejected with clear diagnostics. All plugin tests green.

**Entry criteria:**
- Phase 1 complete. Plugin spec committed.
- Cross-component prompt for framework team merged into `foundation/management/cross-component-prompts/`.

**Work (in `clean-framework/plugins/frame.data/`):**

1. **First task — small proof-of-concept.** Before the full rewrite, implement the `.data` accessor for one entity end-to-end (register + login flow). This validates the plan's assumption that plugin-generated code is sufficient — no compiler changes needed. If the POC surfaces unexpected compiler-side requirements, escalate before committing to the full rewrite.

2. **Rewrite `src/main.cln`:**
   - **`expand_block_typed` dispatcher** — retain the existing dispatch structure at lines 108–135. Update `expand_data_model` (the sub-expander for `data:` blocks) to parse the new sub-block form.
   - **New sub-block parsing:**
     - `table "<name>"` — capture table name; use for DDL and query generation.
     - `fields:` — capture per-field constraints (`primary`, `generated`, `required`, `unique`, `default:`) and column-name overrides (`as "column_name"`). Field types are read from the paired domain class.
     - `indexes:` — capture and emit DDL for each index. Support single-field and tuple-form indexes; suffix `unique` for uniqueness.
     - `relations:` — capture `<name>: <cardinality> <TargetClass> on <fk_column>` declarations. Emit foreign-key DDL and auto-reverse properties.
     - `queries:` — capture query methods (name, params, return type, body). Emit implementations wrapping the retained DSL (`User.find:`, etc.).
   - **Reject bare-field `data:` declaration form** at the top of `expand_data_model` with a clear diagnostic pointing to spec §5.1.
   - **`.data` accessor generation** — for each `data <T>:` block, emit a static-method surface on `T` such that `T.data.<queryName>(...)` dispatches to the corresponding `queries:` method. The compiler treats the emitted methods as ordinary Clean methods.
   - **`Database` service generation** — emit a top-level `Database` class (or namespace, depending on plugin capability) providing:
     - `Database.save(entity)` — inspects the entity type, determines the paired data block, generates INSERT or UPDATE SQL, dispatches through the host bridge (`_db_execute`). Populates DB-generated `id` on INSERT per §11.1 resolution.
     - `Database.delete(entity)` — determines primary key, generates DELETE SQL, dispatches through the host bridge.
   - **Pairing verification** — at expand time:
     - Verify every `data <T>:` block has a matching `class <T>` in `app/entity/<name>.cln`.
     - Verify every field in `fields:` exists on the paired domain class.
     - Verify every non-optional field on the domain class is either represented in `data:` or explicitly marked domain-only (per §11.6).
     - Emit clear diagnostics for all failure modes.
   - **Invariant enforcement** — in generated `Database.save` code, invoke the entity's `always:` block before persisting. If any invariant fails, raise `RUN005` and do not persist.

3. **Update `plugin.toml`:**
   - **`[handles].expressions`:**
     - Remove `*.insert:`, `*.update:`, `*.delete:` handlers.
     - Retain `*.find:`, `*.first:`, `*.findOrFail:`, `*.count:`, `*.exists:` handlers.
     - Add handlers for `.data` accessor dispatch if needed (may not be, if generated code handles it via ordinary method resolution).
   - **`[bridge]`** — add or update bridge declarations for any new capabilities (e.g., if `Database.save` needs a new bridge function; likely not, as it uses existing `_db_execute`).
   - **`[handles].rejections`** (new subsection if it doesn't exist, or via handler code) — declare rejection responses for `*.insert:`, `*.update:`, `*.delete:` with the exact diagnostic text: `"Model.insert: is no longer supported — use Database.save(entity) — see 04_frame_data.md §7"` (and equivalents for update/delete).

4. **Update `plugins/frame.data/tests/`:**
   - **Update existing tests** to new syntax. Verify each passes — this is the regression baseline confirming the new plugin preserves the behavior of the old.
   - **Add tests for new sub-blocks** — each of `table`, `fields:`, `indexes:`, `relations:`, `queries:` gets positive and negative unit tests.
   - **Add tests for `.data` accessor** — dispatch resolution, error on missing method, error on missing paired data block.
   - **Add tests for `Database.save` / `Database.delete`** — against a real Postgres backend, against a real SQLite backend, and against the examples driver.
   - **Add tests for pairing verification** — one test per failure mode (data without entity, entity without required data field, field name mismatch, type mismatch, etc.).
   - **Add tests for invariant enforcement** — save that violates `always:` must raise `RUN005` without persisting.
   - **Add cross-driver DSL parity tests** — same test code, three drivers, same results.

5. **Retain Postgres/SQLite/examples drivers** with unchanged DSL semantics. Confirm the retained query DSL (`find`, `first`, `findOrFail`, `count`, `exists`) works identically under the new plugin.

**Exit criteria:**
- Proof-of-concept for `.data` accessor works end-to-end (register + login flow using new API).
- `bash build.sh` in `plugins/frame.data/` produces valid WASM.
- All unit tests pass.
- All integration tests against real Postgres and SQLite pass.
- All examples-driver tests pass.
- End-to-end test: a Frame app using the new API (register → login → update → delete → transaction with multiple entities) works.
- Rejection diagnostics for removed syntax are clear enough that an AI could act on them without human help (verified in Phase 4).
- Plugin tagged release built and CI-verified per the `comita` workflow.

**Deliverables:**
- Rewritten `frame.data` plugin.
- Plugin test suite covering regression + new features.
- Plugin release tagged and CI-passed.
- Updated plugin README.

**Risks:**
- **Plugin architecture assumption is wrong.** If `.data` accessor requires compiler-side name resolution we missed, this plan understates scope. **Mitigation:** the POC in step 1 catches this early. If the POC fails, escalate — the migration becomes cross-boundary.
- **`.data` accessor dispatch is new plugin work.** No precedent in the existing plugin. Careful design and testing required.
- **DB-generated `id` handling** interacts with §11.1 resolution. Get it right in the spec before implementing.
- **Cross-file analysis** — pairing verification requires the plugin to read `app/entity/` files to verify against `app/data/models/` files. Confirm the plugin API supports this. If not, this becomes a compile-time constraint on the developer (add to §10.2 as a documented limitation) rather than a plugin check.

---

### Phase 3 — Framework docs and books rewrite (parallel with Phase 4)

**Goal:** All user-facing documentation reflects the new model. No stale references remain.

**Entry criteria:**
- Phase 2 complete. Plugin is ready to be documented as it now works.

**Work (in `clean-framework/documents/` and books):**

1. **Rewrite `04_frame_data.md` end-to-end:**
   - Replace all `data:` bare-field examples with the sub-block form.
   - Replace all `Model.insert:` / `Model.update:` / `Model.delete:` examples with `Database.save` / `Database.delete`.
   - Add sections for `app/entity/` folder, `.data` accessor, `Database` service, pairing verification.
   - Retain the query DSL section (§3) unchanged in verbs and sub-blocks; update surrounding narrative.
   - Retain the `transaction:` section (§4) unchanged.

2. **Update `09_frame_dev_guidelines.md`:**
   - Update naming conventions to reflect `app/entity/` folder.
   - Update guidance on where different kinds of code live (business methods on domain, queries on data block, orchestration in logic).
   - Update code style examples.

3. **Update `PROJECT_STRUCTURE.md`:**
   - Add `app/entity/` to the canonical folder layout.
   - Update `app/data/models/` description to reflect new sub-block form.
   - Add any new folders from §11.8 resolution (`app/data/reports/` if adopted).

4. **Update related spec docs that reference the data model:**
   - `01_frame_overview.md` — data model description.
   - `03_frame_server.md` — endpoint examples touching data.
   - `06_frame_auth.md` — user model examples.
   - Any other spec doc using `data:` blocks or mutations.

5. **Rewrite books:**
   - `books-and-content/book-1-starter/` — every chapter touching data.
   - Any other Frame books in the ecosystem.
   - Update every code example, every screenshot annotation, every tutorial narrative.

6. **Update the framework README and getting-started docs:**
   - `clean-framework/README.md`.
   - `clean-framework/documents/GETTING_STARTED.md`.
   - `clean-framework/documents/API_REFERENCE.md`.

**Exit criteria:**
- Every code example in every user-facing doc compiles under the new plugin (verified by copying examples to `.cln` test files and running the compiler).
- Every doc's narrative reflects the new model.
- No stale references to `Model.insert:`, bare-field `data:`, `UserQueries`, or other removed patterns.

**Deliverables:**
- Rewritten `04_frame_data.md`.
- Updated `09_frame_dev_guidelines.md`, `PROJECT_STRUCTURE.md`, and related spec files.
- Rewritten books.
- Updated getting-started, API reference, and README.

**Risks:**
- **Books are large.** Multiple chapters, many code examples. Budget substantial time.
- **Cross-references break.** Spec docs reference each other; updating one may require updating several.

---

### Phase 4 — MCP server updates + AI-driven migration verification (parallel with Phase 3)

**Goal:** MCP tools return guidance and examples consistent with the new spec. AI-driven migration mechanism is proven to work on real example apps.

**Entry criteria:**
- Phase 2 complete. Plugin implements the new model.
- Phase 3 in progress or complete. Docs available for MCP responses to reference.

**Work (in `clean-language-compiler/src/mcp/server.rs`):**

1. **Update `get_app_structure` response (~line 3587):**
   - Add `app/entity/` to the canonical folder tree.
   - Update `app/data/models/` description to reflect new sub-block form.
   - Add DbC placement guidance for `require`/`ensure`/`always:` on entity classes (folding in the `compiler-mcp-app-structure-add-entity-folder-and-dbc.md` proposal that was reconciled in Phase 1).
   - Add naming rules for entity classes.
   - Add decision-tree steps for "when to create an entity file."

2. **Update `get_specification` response (~line 2497):**
   - Return the updated `04_frame_data.md` content (or a summary linking to it).

3. **Update `get_quick_reference` response (~line 2790):**
   - Add `data <T>:` sub-block syntax.
   - Add `.data` accessor pattern.
   - Add `Database.save` / `Database.delete` patterns.
   - Remove references to `Model.insert:`, `Model.update:`, `Model.delete:`.

4. **Update `get_feature_spec` response:**
   - New feature spec entries for `data` persistence, entity/data pairing, `.data` accessor, `Database` service.

5. **Update `get_plugin_examples` response:**
   - Add examples showing the new pattern.

6. **Update `list_error_codes` response:**
   - Add new error codes for pairing failures and rejected old syntax.

**Migration verification** — this is the load-bearing check that the AI-with-MCP flow actually works:

7. **Migrate `clean-framework/examples/` apps via AI-with-MCP:**
   - For each app in `clean-framework/examples/` that uses `data:` blocks:
     - Give an AI instance (Claude Code or equivalent) the old app.
     - Ask it to migrate to the new model using only the MCP responses (as they will exist after this Phase's updates) and the updated docs (from Phase 3).
     - Provide no human hints, no direct pointers to the spec, no manual code writes.
     - Verify the migrated app compiles and runs end-to-end.
   - Do this for at least two representative examples.

8. **Iterate on MCP responses based on AI-migration results:**
   - If the AI gets stuck, look at what it asked or what code it produced.
   - Identify the missing or unclear guidance in the MCP responses.
   - Update the response and rerun the migration test.
   - Repeat until the AI can consistently produce working migrations.

9. **Publish a migration guide** combining spec §13 + notes from AI-driven migration testing. The guide is for humans who want to understand the migration; the AI does the actual work.

10. **Announce the breaking change:**
   - Version bump on `frame.data` (major version).
   - Release notes.
   - Blog post or changelog with the migration guide linked.

11. **Keep old `frame.data` plugin versions in the `cleen` registry** for pinned use during transition. Set an expected deprecation timeline (e.g., "old versions supported for 6 months post-launch").

**Exit criteria:**
- Every MCP tool response reflects the new spec.
- At least two `clean-framework/examples/` apps successfully migrated end-to-end by an AI instance using only MCP + docs, with no human help.
- MCP responses iterated as needed until AI migration is producible without failures.
- Migration guide published.
- Breaking-change announcement published.
- Old plugin versions preserved in registry.
- Compiler patch release with updated MCP responses per the `comita` workflow.

**Deliverables:**
- Updated `src/mcp/server.rs` responses.
- Compiler patch release.
- Migrated `clean-framework/examples/` apps.
- Migration guide document.
- Public breaking-change announcement.

**Risks:**
- **MCP guidance drift.** Stale MCP responses cause AI-generated code to break silently for weeks. **Mitigation:** ship Phase 4 alongside the `frame.data` release (Phase 2 exit), not later.
- **AI-driven migration fails on complex apps.** Real-world apps may have patterns the AI struggles with. **Mitigation:** the AI-migration verification in step 7 catches this before the announcement. If the AI can't produce working migrations, iterate MCP responses until it can. The AI-with-MCP flow is deliberate — Frame is an AI-first framework — but it must actually work.
- **Rejection diagnostics unclear.** If the plugin (from Phase 2) says "syntax not supported" without pointing at the new form, AI instances waste cycles guessing. **Mitigation:** Phase 2 explicitly requires rejection diagnostics to name the replacement pattern; Phase 4's AI verification exercises each diagnostic.
- **Multiple MCP tools contain overlapping guidance.** Keeping them consistent takes discipline. **Mitigation:** treat `get_app_structure` as the canonical source and cross-reference from the others.

---

## 3. Cross-component coordination

### 3.1 Component boundaries and hand-off protocol

Per `Clean Language/CLAUDE.md`, each component lives in its own folder and AI instances cannot edit code across component boundaries. This migration requires hand-off between:

- **`clean-framework`** — Phase 2 (`frame.data` plugin rewrite), Phase 3 (spec docs), and Phase 4 first steps (migrating example apps).
- **`foundation/spec/plugins/`** — Phase 1 plugin spec files. Shared foundation folder; requires developer approval per Principle 25.
- **`clean-language-compiler`** — Phase 4 MCP server response updates in `src/mcp/server.rs`.
- **Books repos** — Phase 3 book rewrites (if books live in a separate repo).

**Hand-off mechanism:** cross-component prompts in `foundation/management/cross-component-prompts/`, following the prefix routing table in the folder's README.

### 3.2 Cross-component prompts to write

The following prompts need to be written and placed in `foundation/management/cross-component-prompts/`:

1. **`framework-data-persistence-migration-phase-1-spec.md`** — Phase 1 plugin spec finalization. Instructs a session working in the framework/foundation to rewrite the plugin spec files and get developer approval.

2. **`framework-data-persistence-migration-phase-2-plugin.md`** — Phase 2 `frame.data` plugin rewrite. Instructs a session working in `clean-framework/plugins/frame.data/` to rewrite the plugin per the committed spec.

3. **`framework-data-persistence-migration-phase-3-docs.md`** — Phase 3 docs and books rewrite. Instructs a session working in `clean-framework/documents/` and books to rewrite user-facing content.

4. **`compiler-data-persistence-migration-phase-4-mcp.md`** — Phase 4 MCP server response updates and AI-driven migration verification. Instructs a session working in `clean-language-compiler/src/mcp/server.rs` to update responses and verify AI migration works.

### 3.3 Existing cross-component prompts that need reconciliation

Two existing proposals in `foundation/management/cross-component-prompts/` overlap with this migration:

- **`compiler-mcp-app-structure-add-entity-folder-and-dbc.md`** — proposes adding `app/entity/` and DbC guidance to MCP. This is a subset of what Phase 4 does. Reconcile by superseding this existing proposal; note the supersession in a header addition to that file so anyone finding it knows to look at this migration instead.

- **`framework-frame-data-examples-driver-and-entity-persistence.md`** — proposes the `examples` driver (Feature 2, still valid and complementary to this migration) and Stage B entity auto-persistence (Feature 1, superseded by the `.data` accessor + `Database` service design). Reconcile by:
  - Retaining Feature 2 (examples driver) as-is; it fits the new model unchanged.
  - Retiring Feature 1 (Stage B auto-persistence) by editing the proposal to note that this migration supersedes it.

**Both existing proposals should be reconciled before Phase 1 completes**, so that the framework team isn't chasing multiple competing designs.

---

## 4. Testing strategy

### 4.1 Test philosophy

Framework testing philosophy (per `clean-framework/CLAUDE.md`): real bridge, no mocks, real database. This migration follows that philosophy.

### 4.2 Test pyramid for this migration

**Plugin tests (`plugins/frame.data/tests/`):**
- Unit tests for each `data:` sub-block parser.
- Unit tests for pairing verification (positive and every negative failure mode).
- Unit tests for `.data` accessor generation.
- Unit tests for `Database.save` / `Database.delete` code emission.
- Integration tests against a real Postgres backend.
- Integration tests against a real SQLite backend.
- Integration tests against the examples driver.
- Cross-driver DSL parity: same test code, three drivers, same results.

**Framework end-to-end tests (`clean-framework/tests/`):**
- Full Frame app that exercises the new model end-to-end.
- Register → log in → update → delete via new API.
- Multi-entity `transaction:` block test.
- Invariant enforcement: a `Database.save` that would violate `always:` raises `RUN005` and does not persist.

**Compiler-side fixture tests (`clean-language-compiler/tests/cln/`):**
- Existing `.cln` fixtures that exercise `data:` blocks or mutation DSL: update to new syntax; each must compile and run under the new plugin.
- No new grammar-regression tests needed — the compiler grammar doesn't change.

**Migration verification tests (Phase 4):**
- AI-instance-driven migration of at least two `clean-framework/examples/` apps.
- Verify each migrated app compiles and runs end-to-end.
- Verify each rejection diagnostic from Phase 2 is actionable by an AI without human help.

### 4.3 Regression coverage

Before Phase 2 begins, the current test suite for the data model must be catalogued so nothing gets silently lost. Specifically:

1. **Inventory existing tests** exercising `data:` blocks or mutation DSL:
   - `clean-framework/tests/` — ~4 files (per exploration).
   - `clean-language-compiler/tests/cln/` — ~6 files (per exploration).
2. **For each test:** determine what behavior it verifies. Confirm the new model preserves that behavior (or explicitly decides not to, with justification).
3. **Rewrite each test to the new syntax** as part of Phase 2 and confirm it still passes.

### 4.4 Migration mechanism verification

This is Phase 4's exit criterion and deserves its own subsection because it's the load-bearing check:

- **Give an AI instance an existing Frame example app.**
- **Provide only the updated MCP responses and updated docs.**
- **No human hints, no manual code writes, no direct spec references.**
- **Ask the AI to migrate the app to the new model.**
- **Verify the migrated app compiles and runs end-to-end.**
- **Do this for at least two representative examples.**

If any of these fails, treat it as a Phase 4 blocker. Iterate MCP responses (and possibly docs) until AI-driven migration is reliably producible.

---

## 5. Risks and mitigations

### 5.1 Real risks, being honest

**Risk: Plugin architecture assumption is wrong.**
The plan is built on the finding that the compiler treats `data:` block contents as opaque and plugins can add new sub-blocks freely. If the exploration missed something — if `.data` accessor requires compiler-side name resolution, or some new sub-block trips a grammar constraint — this plan understates scope.
**Mitigation:** Phase 2's first task is a small proof-of-concept implementing `.data` accessor for one entity end-to-end. If it fails, we know we need compiler changes and can escalate before the full rewrite.

**Risk: Design revisions during Phase 1 cascade to all downstream phases.**
**Mitigation:** get design signoff explicitly before Phase 2 starts. Don't begin the plugin rewrite while the spec is still moving.

**Risk: Cross-component coordination stalls.**
Different components have different owners, priorities, and timelines. If one blocks, the whole migration blocks.
**Mitigation:** parallelize Phases 3 and 4. Communicate blockers early.

**Risk: AI-driven migration fails on complex apps.**
Real code has quirks, custom patterns, half-migrated states. The AI-with-MCP flow may hit surprises.
**Mitigation:** Phase 4's exit criteria explicitly test this on two representative example apps. If the AI can't produce working migrations, iterate MCP responses until it can. Frame is an AI-first framework by design; the mechanism must actually work.

**Risk: MCP guidance drift causes AI-generated code to break for weeks.**
AI instances read MCP responses. Stale responses produce broken code silently.
**Mitigation:** ship Phase 4 (MCP updates) at the same time as the plugin release (Phase 2 exit), not later.

**Risk: Books and docs lag the implementation.**
Users read docs; if docs are stale, users write broken code even after the plugin is updated.
**Mitigation:** treat Phase 3 as launch-critical. Docs must be updated before the version bump ships publicly.

**Risk: Rejection diagnostics unclear.**
If the plugin says "syntax not supported" without pointing at the new form, AI instances waste cycles guessing.
**Mitigation:** Phase 2 explicitly requires rejection diagnostics to name the replacement pattern. Phase 4's AI-migration verification exercises each diagnostic.

**Risk: Some existing Frame applications don't migrate.**
Not everyone will update immediately. Old versions of `frame.data` need to remain available.
**Mitigation:** don't delete old plugin versions from the `cleen` registry. Set a deprecation timeline (e.g., "6 months post-launch, then removed").

**Risk: Cross-file analysis for pairing verification may not fit the plugin API.**
Pairing verification requires the plugin to read `app/entity/` files to verify against `app/data/models/` files. If the plugin API doesn't support this, pairing check becomes a documented developer responsibility rather than a plugin-enforced constraint.
**Mitigation:** confirm plugin API supports cross-file reads during the Phase 2 POC. If not, fall back to documented developer discipline; add a "Known Limitations" note to the spec.

### 5.2 Not-really-risks

**"What if the design isn't approved?"** — this is the point of Phase 1's approval gate. If the design isn't approved, the migration doesn't start. This is the process working.

**"What if it takes 4 weeks?"** — that's the estimate. Slippage happens. As long as phases stay ordered (1→2 sequential; 3 and 4 after 2), slippage doesn't cause corruption; it just delays completion.

**"What if we ship without books?"** — don't. Phase 3 gates the public release.

---

## 6. Success criteria

The migration is complete when all of these are true:

- [ ] `foundation/spec/plugins/frame-data.ebnf` reflects the new model with developer approval.
- [ ] `foundation/spec/plugins/frame-data-semantics.md` reflects the new model with developer approval.
- [ ] `frame.data` plugin implements pairing verification, `.data` accessor, `Database` service.
- [ ] `frame.data` plugin rejects removed syntax (`Model.insert:`, `Model.update:`, `Model.delete:`, bare-field `data:`) with clear diagnostics naming the replacement pattern.
- [ ] All plugin tests pass (existing regression migrated + new-feature coverage added).
- [ ] All framework tests pass.
- [ ] All compiler-side fixture tests (updated to new syntax) pass.
- [ ] `04_frame_data.md` is rewritten and accurate.
- [ ] `09_frame_dev_guidelines.md`, `PROJECT_STRUCTURE.md`, `GETTING_STARTED.md`, `API_REFERENCE.md`, `README.md` are updated.
- [ ] Books are rewritten.
- [ ] `clean-framework/examples/` apps are migrated.
- [ ] MCP responses (`get_app_structure`, `get_specification`, `get_quick_reference`, others) reflect the new model.
- [ ] AI-driven migration verified: at least two `clean-framework/examples/` apps successfully migrated end-to-end by an AI instance using only MCP + docs.
- [ ] Migration guide published.
- [ ] Public breaking-change announcement published.
- [ ] Old plugin versions remain available for pinned use.
- [ ] Two existing overlapping proposals in `foundation/management/cross-component-prompts/` reconciled (superseded or retired).

---

## 7. Post-migration follow-up

After the migration ships, three follow-up items are worth tracking:

1. **Monitor error reports** for issues developers hit that the AI-migration flow didn't handle. If patterns emerge, iterate MCP responses or docs.
2. **Watch adoption rates** in the Clean Language ecosystem. If uptake is slow, understand why and communicate.
3. **Retire old plugin versions eventually.** Don't leave old `frame.data` available indefinitely — enforce the deprecation timeline set in Phase 4.

---

## 8. Status

This plan is a **draft**. It has been approved as a summary plan (see `/Users/earcandy/.claude/plans/staged-splashing-starlight.md`) but the detailed per-phase work has not been executed. Represents the honest work required as of 2026-07-10.

Before executing:

1. Framework owner reviews and approves the detailed plan.
2. Framework team confirms feasibility of Phase 2 work (particularly the POC).
3. MCP maintainers confirm Phase 4 work (particularly the AI-migration verification step).
4. Cross-component prompts written and placed.
5. Existing overlapping proposals reconciled.
6. Phase 1 begins.

Timeline estimates are rough calibration only. Actual timeline depends on team capacity, review rounds, and open-question resolution.
