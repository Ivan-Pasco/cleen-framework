# Next Session Prompt — Data Persistence Migration Continuation

**Date written:** 2026-07-17 (updated)
**Written by:** Session that landed Sub-cycle 1 partial + Phase 4 MCP updates
**Purpose:** Give a fresh session enough context to continue the migration without repeating discovery work
**Supersedes:** the prior version of this same file (see git log for the original 2026-07-17 handoff)

---

## Situation at handoff

The frame.data v2 migration is mid-execution. **Option A was formally adopted on 2026-07-17** by the plugin owner: the v3.x sub-cycle plan (v1.2 semantics on typed emission) is retired; migration proceeds against the v2 spec directly (entity/data pairing, `.data` accessor, `Database` service, sub-block `data <T>:` form).

**What's now committed and safe (as of this handoff):**

- Phase 1 (plugin spec finalization) — complete. `foundation/spec/plugins/frame-data.ebnf` v2.0.0 and `frame-data-semantics.md` v2.0.0 are committed.
- Framework user-facing docs and 3 book chapters (ch12/ch13/ch14) rewritten for v2 with "🚧 IN ACTIVE DEVELOPMENT" banners.
- Cross-component prompts marked with Option A adoption.
- Plugin source header (`plugins/frame.data/src/main.cln` lines 1-38) updated to retire the v3.x sub-cycle plan.
- Migration plan and status doc (`SPEC_DATA_PERSISTENCE_MODEL.md` §15) fully current.
- **New (2026-07-17, this session):** Sub-cycle 1 partial landed (`ccd5c2d` in `clean-framework`) — see "What was done this session" below.
- **New (2026-07-17, this session):** Phase 4 MCP updates landed (`c53e6ec2` in `clean-language-compiler`) — see "What was done this session" below.

**What's NOT done and open:**

- **Sub-cycle 1 (v2) remainder** — `.data` accessor generation, entity/data pairing verification (DAT-P001..P005), type resolution from the paired entity (currently stubbed as `string`), `indexes:`/`relations:`/`queries:` sub-block semantic consumers, Am 10 spec construction changes, and rejection diagnostic for bare-field `data:` (v1). Estimated ~700-900 LOC remaining out of the ~1400 LOC Sub-cycle 1 target.
- **Sub-cycles 2 (v2), 3 (v2), and 4 (v2)** — read verbs with DAT-Q020, Database service + pairing + invariants, DSL parsers + migrate + smoke test. Per-sub-cycle LOC targets sum to ~3300 additional LOC.
- **~230 v1 references across ~25 book chapters** — mechanical rewrites (not pedagogical). Attempted this session but skipped due to parallel-session activity on 5/7 target chapters (see "What was NOT done and why").
- **AI-driven verification of migrated apps** — still blocks on a working v2 plugin (needs Sub-cycles 2-4).

---

## What was done this session (2026-07-17)

### Sub-cycle 1 (v2) partial — commit `ccd5c2d` in `clean-framework`

Source-only step. `plugin.wasm` is unchanged; the release is on the plugin owner's `comita`. `cln check` reports 133 functions, 0 errors on the modified source.

Added to `plugins/frame.data/src/main.cln`:

- **New helpers** (~200 LOC total) inserted after `count_fields` in the DSL string-helper block:
  - `is_v2_sub_block_form(body)` — form detection based on line-start of `fields:`, `indexes:`, `relations:`, `queries:`, or `table `.
  - `is_v2_sub_block_header(trimmed_line)` — extract-boundary detection.
  - `extract_v2_sub_block(body, sub_block_name)` — reads a named sub-block's raw content, stops at the next sub-block header or end-of-body. Preserves original indentation.
  - `extract_v2_table_line(body)` — reads the unquoted table name from `table "..."`.
  - `extract_v2_field_names(fields_body)` — enumerates leading identifiers per line as a comma-separated list.
- **`expand_data_model`** (main.cln:1839 area) now detects the v2 form and synthesizes a v1-shape body from the `fields:` sub-block. Types default to `string` pending Sub-cycle 3 entity/data pairing (DAT-P003 + DAT-M016). This produces a class shell that compiles cleanly but is **not** yet functionally complete.
- **`validate_data_model`** (main.cln:3553 area) enforces DAT-M015 (`fields:` required) on v2 bodies. Bare-field v1 bodies still accepted as a transitional convenience; Sub-cycle 3 will add the deprecation rejection once pairing is in place.
- **Dispatcher** (main.cln:196 area) rejects `Model.insert`, `Model.insert_id`, `Model.update`, `Model.upsert`, `Model.delete` block forms with `FRAME-DATA-E021`. Each rejection names the intended `Database.*` replacement per DAT-S001/DAT-S003/DAT-S004/DAT-Q014/DAT-Q015 with AI-actionable text.

**What is NOT in this partial** (all deferred to later Sub-cycle 1 slices or Sub-cycles 2+):

- `.data` accessor generation on the entity class (DAT-A001..A005).
- Entity/data-block pairing verification (DAT-P001..P005).
- Type resolution from the paired entity (DAT-M016 — currently stubbed as `string`).
- `indexes:`/`relations:`/`queries:` sub-block semantic consumers (parser recognizes them as boundaries; nothing consumes their contents).
- Am 10 spec construction changes.
- Rejection diagnostic for bare-field `data:` declarations (v1 form).

### Phase 4 MCP updates — commit `c53e6ec2` in `clean-language-compiler`

`cargo check --lib` passes. Commit survived — the reset risk that hit the earlier session did not recur.

Updated `src/mcp/server.rs`:

- **`tool_get_quick_reference` §Rule A** (~line 3371 pre-edit) rewritten from ORM DSL (`Model.exists:`/`update:`/`delete:`/`count:`) to the v2 surface: `Entity.data.<method>` for reads; `Database.save`/`delete`/`deleteOrFail`/`saveAll`/`deleteAll` for writes; `db.query`/`queryAs` escape hatch inside `app/data/reports/`. Includes explicit v1→v2 migration recipes for insert, update, and delete (WRONG kept as rejection-context reminder). Calls out `FRAME-DATA-E021` as the compile-time error users will hit.
- **`tool_get_app_structure`** (~line 3587 pre-edit) rewritten end-to-end for v2:
  - §1 folder layout adds `app/entity/` and `app/data/reports/`; documents entity↔data pairing (DAT-P001..P005) and the `frame.data:` config-lives-in-main.cln convention (DAT-I004).
  - §2 Law 1 updated to reflect entity→types edges and data/models,data/reports→entity edges.
  - §3 decision tree extended with steps for entity classes, data blocks, report classes, and `frame.data:` config in main.cln.
  - §6 Forbidden Patterns table adds `E-STRUCT-012..015` and `FRAME-DATA-E021` with semantic-rules cross-references.
  - §7 Growth Path adds `app/entity/` to the starter tree and `app/data/reports/` to the growth triggers.
  - §8 Known Limitations rewritten for v2: entity class is the type; `.data` and `Database` are compile-time namespaces; removed v1 forms.
  - §9 example rewritten end-to-end: `app/entity/user.cln` (invariants + methods) + `app/data/models/user.cln` (sub-block form with `fields:`/`indexes:`/`queries:`) + `app/logic/auth.cln` using `Database.save` and `User.data.byEmail`.
- Tip text at the bottom of `tool_get_app_structure` updated to reflect entity-vs-data placement.

Other MCP tools (`tool_get_specification`, `tool_get_plugin_examples`, `tool_list_error_codes`, `tool_get_feature_spec`) were **not modified** — a grep for `Model.insert|Model.update|Model.delete|Model.upsert|Model.insert_id|db.insert|db.update|db.delete|data: [A-Z]` after the edits shows only the intentional Rule A rejection-context lines and one incidental reference in §8 that's now `data T:` (correct v2 shape).

### What was NOT done and why

- **Track B — book chapter mechanical rewrites — skipped.** 5 of the 7 highest-priority target chapters (`ch25.md`, `ch29.md`, `ch32.md`, `ch33.md`, `ch34.md`) had uncommitted edits in the workspace root from a parallel session. The prior handoff prompt says "if the workspace shows evidence of active parallel activity, stop and re-evaluate." Attempting to rewrite them would have risked stepping on the other session's work at commit time or overwriting in-progress edits.
  - Only 2 target chapters were clean: `ch26.md` and `app-b.md` (~14 v1 references total). A future session with a clean workspace should knock those out in <30 minutes plus finish the rest of the ~230 v1 references across ~25 chapters.

---

## Read these first (10 minutes, mandatory)

Do not skip these — the prior sessions' context is dense and repeating it wastes real time:

1. **`clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md`** — the committed design spec. §15 has the migration status (updated 2026-07-17 with this session's landings), Option A adoption record, and per-phase state.

2. **`clean-framework/system-documents/DRAFT_MIGRATION_PLAN_DATA_PERSISTENCE_MODEL.md`** — the 4-phase plan.

3. **`foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md`** — the per-sub-cycle LOC re-projection for Option A execution. §3 has the four sub-cycle scopes with LOC targets (Sub-cycle 1: ~1400, Sub-cycle 2: ~1050, Sub-cycle 3: ~1050, Sub-cycle 4: ~1200; total ~4700).

4. **`foundation/spec/plugins/frame-data.ebnf`** and **`foundation/spec/plugins/frame-data-semantics.md`** — the v2 grammar and semantics (both v2.0.0). These describe what the plugin must implement.

5. **`clean-framework/plugins/frame.data/src/main.cln`** — the current plugin source. Read: header (lines 1-45), the new v2 helpers block (grep for `is_v2_sub_block_form`), the updated dispatcher (lines ~180-230), and the updated `expand_data_model` (line ~1839 area).

6. **`clean-language-compiler/src/mcp/server.rs`** — the updated MCP responses. Read `tool_get_app_structure` (~line 3587) and `tool_get_quick_reference` §Rule A (~line 3371, use `grep -n "Rule A" src/mcp/server.rs` if line numbers have shifted).

---

## Three tracks available — pick based on your context

You do not need to pick all three. Each is independently valuable.

### Track A — Sub-cycle 1 (v2) completion

**What:** Land the remaining ~700-900 LOC of Sub-cycle 1: `.data` accessor generation on the entity class, entity/data-block pairing verification (DAT-P001..P005 — reads the paired `app/entity/<basename>.cln`), field type resolution from the paired entity (replaces the current `string` stub), `indexes:`/`relations:`/`queries:` sub-block consumers, Am 10 spec construction, and rejection diagnostic for bare-field `data:` (v1 form).

**Where:** `clean-framework/plugins/frame.data/src/main.cln`. Build on top of the helpers landed in commit `ccd5c2d`.

**Reference material:**
- `foundation/spec/plugins/frame-data-semantics.md` DAT-A001..A005 (accessor), DAT-P001..P005 (pairing), DAT-M016 (type inheritance), DAT-M019 (indexes semantics), DAT-M020 (relations).
- `foundation/spec/plugins/plugin-contract.md` — plugin contract v3 (`expand_block_typed`, typed emission, entity file reads via compiler-provided read bridges).
- Recent commit `ccd5c2d` — the sub-block parser + partial dispatcher rewrite that this track builds on.

**Honest risks:**

- **Pairing verification requires the compiler to expose an entity-file-read bridge.** Check whether this exists in the current typed-emission surface. If not, this track blocks on a compiler-side prerequisite that must be reported via `report_error` (`component=compiler`) before Sub-cycle 1 can complete.
- **Committed sub-block parser handles boundary detection but does not yet parse field constraints inside `fields:` sub-block.** Full Sub-cycle 1 needs a constraint parser (`primary`, `generated`, `required`, `unique`, `default:`, `as`) to feed the class spec correctly.
- **Sub-cycle 1 completion may still require multiple sessions.** Keep commits small — every logical piece its own commit — as this session did.

### Track B — Book chapter mechanical rewrites (~230 v1 references, ~25 files)

**What:** Same scope as prior handoff. See prior handoff's Track B for the file list and conversion patterns.

**Guardrail:** **Check `git status "books and content/"` first.** If more than one or two target chapters have uncommitted work from another session, stop and either coordinate with the other author or skip the track. Do not overwrite parallel work.

**If clean:** the mechanical conversion patterns are:
- `X.insert:` block → construct entity + `Database.save(entity)`
- `X.update:` with `where:` + `set:` → load via `.data.findOrFailById(...)` or scoped query, mutate fields, `Database.save(entity)`
- `X.delete:` with `where:` → load entity, `Database.delete(entity)` (lenient) or `Database.deleteOrFail(entity)` (strict)
- `X.upsert:` → construct entity with primary key set + `Database.save(entity)` (idempotent by PK); or `first:` + `save` for match-based upsert
- `X.insertMany:` → build `list<X>` of entities, `Database.saveAll(list)`

Add the "🚧 IN ACTIVE DEVELOPMENT" banner to each rewritten chapter per ch12.md's template. Only commit files you personally edit — use explicit `git add <path>`, never `git add -A`.

### Track C — Additional MCP tool updates in `clean-language-compiler/src/mcp/server.rs`

**What:** The two highest-value MCP tools (`tool_get_app_structure`, `tool_get_quick_reference` §Rule A) are done. Remaining low-priority passes:
- Verify `tool_get_specification`, `tool_get_plugin_examples`, `tool_list_error_codes`, `tool_get_feature_spec` have no v1-flavored content leftover. Prior session verified them; the grep I ran this session confirms no stale references. But the plugin_examples tool may include v1 sample code snippets — worth spot-checking.

**Effort:** 20-40 minutes if anything is found; 5 minutes if the grep confirms nothing.

---

## Guardrails (learned from prior sessions — do not violate)

1. **Cross-component boundaries are real.** You are working in either `clean-framework/`, workspace root, or `clean-language-compiler/`. Only edit files in the component you're actively working in for each track.
   - Track A: `clean-framework/`.
   - Track B: workspace root (`books and content/` lives there).
   - Track C: `clean-language-compiler/`.

2. **Never `git add -A` or `git add .`** — the workspace has substantial uncommitted work from other people. Always stage explicit files.

3. **Commit early, commit often.** This session committed each track independently as a defense against reset scenarios; both survived. Follow the same pattern.

4. **Do not touch `.claude/settings.json`, `.claude/scheduled_tasks.lock`, or `.claude/fragile-paths.txt`** — those are per-session infrastructure files that survive across git operations.

5. **Do not modify the frame.data plugin binary or manifest without owner permission.** `plugin.toml` version bumps and `plugin.wasm` rebuilds belong to the plugin owner's `comita` workflow. Track A modifies `src/main.cln` (source code) only.

6. **`comita` is not the same as `git commit`.** `comita` is the full release workflow (version tag, push, wait for CI, install via cleen). For migration work-in-progress, just use `git commit`. The plugin owner runs `comita` when a release is ready.

7. **If the workspace shows evidence of active parallel activity, stop and re-evaluate.** This session skipped Track B for exactly this reason. `git status` on the target folder is the first thing to run.

---

## What "done" looks like at the end of this session

Being honest: the migration will not fully complete in one session. What you can genuinely finish depends on which track(s) you pick:

- **Track A remainder (Sub-cycle 1 completion, ~700-900 LOC):** likely multi-session unless focused. Aim for one coherent slice per session — e.g., "pairing verification only" or ".data accessor generation only" — with a handoff note where you stop.
- **Track B alone (2-3 hours, assumes clean workspace):** all remaining book chapters have v2 code with banners. Real Phase 3 completion.
- **Track C alone (20-40 min):** low-priority MCP audit. Small.

**Recommended for a fresh session with 2-4 hours budget and clean workspace:** Track B end-to-end (real, finishable, high-signal). Then a Sub-cycle 1 slice from Track A.

**Recommended for a fresh session with 6+ hours budget:** Track A pairing verification + `.data` accessor generation. If that runs out of time, hand off cleanly.

---

## After this session

Whatever you finish, before ending:

1. **Verify commits landed** — `git log --oneline -5` in each affected repo. Confirm the commits you made are there.
2. **Update `SPEC_DATA_PERSISTENCE_MODEL.md` §15** if Phase 2 or Phase 4 status changed materially.
3. **Rewrite this file** with the new state — either overwrite in place (as this session did) or write a fresh dated companion.

---

## Contact points for questions (files, not people)

- **Migration design questions:** `SPEC_DATA_PERSISTENCE_MODEL.md` §11 has all resolved design decisions with rationale.
- **Sub-cycle LOC targets:** `foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md` §3.
- **v1 → v2 mechanical conversion patterns:** `documents/specification/04_frame_data.md` §14 "Migration from v1" has explicit before/after code examples.
- **Plugin contract (typed emission, `expand_block_typed`, etc.):** `foundation/spec/plugins/plugin-contract.md`.
- **Grammar productions the plugin must accept:** `foundation/spec/plugins/frame-data.ebnf` v2.0.0.
- **Semantic rules the plugin must enforce:** `foundation/spec/plugins/frame-data-semantics.md` v2.0.0.
- **What this session landed:** commits `ccd5c2d` (clean-framework) and `c53e6ec2` (clean-language-compiler).

---

## Latest commits protecting migration work (as of this handoff)

Reference points if you need to verify nothing has been lost:

| Commit | Repo | Description |
|---|---|---|
| `5ff3057` | workspace root | Phase 1 spec (frame-data.ebnf v2.0.0, semantics v2.0.0, cross-component prompts) |
| `1d5d608` | clean-framework | Phase 1 framework docs + system-documents |
| `c3d3230` | workspace root | Book ch12 mutations rewrite |
| `78e9a99` | workspace root | Book ch13 transactions rewrite |
| `2d07545` | workspace root | Book ch14 seed examples |
| `8fe2ac7` | clean-framework | 2026-07-17 plugin header retirement (Option A adopted) |
| `2f78640` | clean-framework | 2026-07-17 framework doc banners + §15 update |
| `0627157` | workspace root | 2026-07-17 book banners + prompt collision-resolved markers |
| `ccd5c2d` | clean-framework | **2026-07-17 Sub-cycle 1 (v2) partial — sub-block parser + expand_data_model routing + v2 dispatcher rejections (this session)** |
| `c53e6ec2` | clean-language-compiler | **2026-07-17 Phase 4 MCP updates — tool_get_app_structure and tool_get_quick_reference §Rule A rewritten for v2 (this session)** |

If any of these commits are missing from `git log` in their respective repo, something significant happened — investigate before proceeding.

---

**End of prompt. Begin by reading the "Read these first" section, then the "What was done this session" section for the most recent state.**
