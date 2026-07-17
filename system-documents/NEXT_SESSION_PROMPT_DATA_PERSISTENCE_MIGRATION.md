# Next Session Prompt — Data Persistence Migration Continuation

**Date written:** 2026-07-17
**Written by:** Previous session working on the frame.data v2 migration
**Purpose:** Give a fresh session enough context to continue the migration without repeating discovery work

---

## Situation at handoff

The frame.data v2 migration is in mid-execution. **Option A was formally adopted on 2026-07-17** by the plugin owner: the v3.x sub-cycle plan (v1.2 semantics on typed emission) is retired; migration proceeds against the v2 spec directly (entity/data pairing, `.data` accessor, `Database` service, sub-block `data <T>:` form).

**What's already committed and safe:**

- Phase 1 (plugin spec finalization) — complete. `foundation/spec/plugins/frame-data.ebnf` v2.0.0 and `frame-data-semantics.md` v2.0.0 are committed.
- Framework user-facing docs and 3 book chapters (ch12/ch13/ch14) rewritten for v2 with "🚧 IN ACTIVE DEVELOPMENT" banners.
- Cross-component prompts marked with Option A adoption.
- Plugin source header (`plugins/frame.data/src/main.cln` lines 1-38) updated to retire the v3.x sub-cycle plan.
- Migration plan and status doc (`SPEC_DATA_PERSISTENCE_MODEL.md` §15) fully current.

**What's NOT done and open:**

- **Sub-cycle 1 (v2) of the plugin rewrite** — ~1400 LOC of Clean plugin code implementing data-model expansion + `.data` accessor + preamble + dispatcher.
- **Phase 4 MCP responses in `clean-language-compiler/src/mcp/server.rs`** — was attempted in an earlier session, hard-reset by workspace churn. May survive a retry now that the compiler team is doing normal committed work.
- **~230 v1 references across ~25 book chapters** — mechanical rewrites (not pedagogical). Deferrable.

---

## Read these first (10 minutes, mandatory)

Do not skip these — the prior session's context is dense and repeating it wastes real time:

1. **`clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md`** — the committed design spec. §15 has the migration status, Option A adoption record, and per-phase state.

2. **`clean-framework/system-documents/DRAFT_MIGRATION_PLAN_DATA_PERSISTENCE_MODEL.md`** — the 4-phase plan. Phase 1 done, Phase 2 active, Phase 3 partial with banners, Phase 4 unblocked.

3. **`foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md`** — the per-sub-cycle LOC re-projection for Option A execution. §3 has the four sub-cycle scopes with LOC targets (Sub-cycle 1: ~1400, Sub-cycle 2: ~1050, Sub-cycle 3: ~1050, Sub-cycle 4: ~1200; total ~4700).

4. **`foundation/spec/plugins/frame-data.ebnf`** and **`foundation/spec/plugins/frame-data-semantics.md`** — the v2 grammar and semantics (both v2.0.0). These describe what the plugin must implement.

5. **`clean-framework/plugins/frame.data/src/main.cln` header (lines 1-45)** — the current plugin status. Confirms Option A. Points at the execution plan.

---

## Three tracks available — pick based on your context

You do not need to pick all three. Each is independently valuable. **My honest recommendation** based on the previous session's experience: **Track B (book rewrites) is the safest to complete end-to-end in a single session.** Track A (plugin) is the highest-value but is real engineering work that may span sessions. Track C (MCP) is low-effort but was hard-reset once — verify commit stability early.

### Track A — Sub-cycle 1 (v2) plugin execution

**What:** Implement Sub-cycle 1 of the v2 plugin: expand `data <T>:` blocks with new sub-block form (`table`, `fields:`, `indexes:`, `relations:`, `queries:`), generate `.data` accessor methods on the entity class, ship the preamble helpers + dispatcher + Am 10 spec construction.

**Scope:** ~1400 LOC of Clean-language plugin code. Per re-baking proposal §3.1.

**Target file:** `clean-framework/plugins/frame.data/src/main.cln` (current: 3722 lines, mostly Sub-cycle 1 skeleton from the retired v3.x plan — most of it is reusable infrastructure, some needs rewriting for v2 sub-blocks).

**Reference material:**
- `foundation/spec/plugins/frame-data.ebnf` v2.0.0 §2 (data block sub-block form).
- `foundation/spec/plugins/frame-data.ebnf` v2.0.0 §3 (`.data` accessor).
- `foundation/spec/plugins/frame-data-semantics.md` v2.0.0 DAT-M014..M020, DAT-A001..A005, DAT-P001..P005.
- `foundation/spec/plugins/plugin-contract.md` — plugin contract v3 (`expand_block_typed`, typed emission).
- `plugins/frame.data/src/main.cln` current dispatcher at lines ~108-135 — retain the structure, update `expand_data_model` for new sub-block form.

**Honest risks:**

- **The plugin owner may have their own Sub-cycle 1 execution plan/timing.** Before starting, check whether the plugin owner (author of `reports/2026-07-03-session-4-frame-data-deferred-v*.md`) has published a Sub-cycle 1 timeline or claimed the work. If yes, don't overwrite it.
- **Workspace churn:** `clean-language-compiler` had 30+ commits in the last week and previously hard-reset uncommitted work. `clean-framework` seems more stable but has plugin binary rebuilds happening. Commit often — every logical chunk, not once at the end.
- **~1400 LOC is not a single-session task** for careful plugin engineering. If you start Track A, plan to hand off mid-Sub-cycle if you run out of time. Commit whatever's done and write a handoff note describing where you stopped.

**Exit criteria for a complete Sub-cycle 1:**

- `expand_data_model` handles the new sub-block form (`table`, `fields:`, `indexes:`, `relations:`, `queries:`).
- `.data` accessor is generated as static methods on the paired entity class.
- Pairing verification runs at plugin-compile time (E-STRUCT-013 for `data:` block without paired entity).
- Field alignment check (E-STRUCT-014).
- Plugin builds (`bash plugins/frame.data/build.sh`) and produces valid WASM.
- At least one integration test passes: `data User:` with `fields:` + `queries:` block gets processed and `User.data.findByX()` calls compile.

### Track B — Book chapter mechanical rewrites (~230 v1 references, ~25 files)

**What:** Update code examples across the remaining book chapters from v1.2 block-form syntax to v2 patterns. This is pedagogically low-risk work — chapters teach other topics (RBAC, CSRF, testing, WebSockets, patterns, reference app) with v1 code examples that need surgical replacement.

**Scope:** ~2-3 hours of mechanical edits. Every hit is a code example inside a chapter that teaches something unrelated to mutations.

**Files (per prior-session survey):**

| File | v1 refs | Priority |
|---|---:|---|
| `book-2-frame/chapters/ch25.md` (RBAC) | 9 | High |
| `book-2-frame/chapters/ch26.md` (CSRF) | 8 | High |
| `book-2-frame/chapters/ch34.md` (Reference App) | 8 | High |
| `book-2-frame/chapters/ch33.md` (Common Patterns) | 7 | High |
| `book-2-frame/chapters/ch32.md` (Testing) | 7 | Medium |
| `book-2-frame/chapters/app-b.md` (Field Types Ref) | 6 | Medium |
| `book-2-frame/chapters/ch29.md` (Real-Time) | 5 | Medium |
| Other book-2 chapters | 4 or fewer | Low |
| `book-5-production/chapters/*` (~13 files) | 4 or fewer | Low |

**Conversion patterns (memorize these before starting):**

- `X.insert:` block → construct entity + `Database.save(entity)`
- `X.update:` with `where:` + `set:` → load via `.data.findOrFailById(...)` or scoped query, mutate fields, `Database.save(entity)`
- `X.delete:` with `where:` → load entity, `Database.delete(entity)` (lenient) or `Database.deleteOrFail(entity)` (strict)
- `X.upsert:` → construct entity with primary key set + `Database.save(entity)` (idempotent by PK)
- `X.insertMany:` → build `list<X>` of entities, `Database.saveAll(list)`

**Add a "🚧 IN ACTIVE DEVELOPMENT" banner** to each rewritten chapter matching the language used in ch12/ch13/ch14. See ch12.md's banner (near line 5) for the canonical wording.

**Warning about other people editing books:** ~40 other book files show as modified in the workspace root — someone else is working on the books. **Only commit files you personally edit.** Use `git add <specific-files>`, never `git add -A` or `git add books`.

**Exit criteria:** each rewritten chapter has zero non-narrative v1 references (v1 syntax may appear in narrative "before" examples explaining what changed), passes a manual scan for internal consistency, has a banner.

### Track C — Phase 4 MCP responses in `clean-language-compiler/src/mcp/server.rs`

**What:** Update the MCP tool responses to teach v2 syntax to AI instances. Three functions to edit at approximately these line numbers (verify before editing — file may have grown):

- `tool_get_app_structure` (~line 3587): full rewrite for v2 (entity/, data/models/ with sub-blocks, `.data` accessor, `Database` service, DbC placement rules, updated forbidden patterns).
- `tool_get_quick_reference` (~line 2790): targeted edits to Rule A section (~line 584-620) and Database Queries section (~line 682-745) — the earlier session already wrote the target content, but the changes were hard-reset.
- Others (`tool_get_specification`, `tool_get_plugin_examples`, `tool_list_error_codes`, `tool_get_feature_spec`) — verified not needing changes in the earlier session, but verify again.

**Reference material for the rewrites:**

- The earlier session's target content was substantially preserved in the git history of `system-documents/SPEC_DATA_PERSISTENCE_MODEL.md` and could be reconstructed. But the cleanest approach is to write fresh from the committed v2 spec, not from history.
- `cross-component-prompts/compiler-data-persistence-migration-phase-4-mcp.md` has the detailed prompt for this work.

**Honest risks:**

- **The earlier attempt was reverted by `git reset --hard origin/main`** — a workspace process (likely an AI agent using worktrees) reset uncommitted work. Before starting, check if the compiler repo has stabilized on committed work (recent commits should look like normal engineering work, not repeated resets).
- **Commit within minutes of writing.** Don't accumulate uncommitted changes. Every function rewrite should be its own commit that survives immediately.

**Exit criteria:**

- All three functions rewritten.
- All rewrites survive at least 15 minutes uncommitted before being committed (as a proxy for "this session's work is stable").
- Committed under a single commit with a message clearly describing what changed and why.
- No AI-driven migration verification yet (that requires a working v2 plugin from Track A).

---

## Guardrails (learned from prior sessions — do not violate)

1. **Cross-component boundaries are real.** You are working in `clean-framework/`, but the workspace root and `clean-language-compiler/` are separate git repos. Only edit files in the component you're actively working in for each track.
   - Track A: `clean-framework/`.
   - Track B: workspace root (`books and content/` lives there).
   - Track C: `clean-language-compiler/`.

2. **Never `git add -A` or `git add .`** — the workspace has substantial uncommitted work from other people (plugin binary rebuilds, compiler codegen fixes, plugin spec additions for other plugins, session reports). Always stage explicit files.

3. **Commit early, commit often.** The earlier session's Phase 4 MCP work was lost to a hard reset because it stayed uncommitted for too long. Every logical chunk should be its own commit.

4. **Do not touch `.claude/settings.json`, `.claude/scheduled_tasks.lock`, or `.claude/fragile-paths.txt`** — those are per-session infrastructure files that survive across git operations.

5. **Do not modify the frame.data plugin binary or manifest without owner permission.** `plugin.toml` version bumps and `plugin.wasm` rebuilds belong to the plugin owner's `comita` workflow. If you're doing Track A, you'll modify `src/main.cln` (source code) but not `plugin.wasm` (binary) or `plugin.toml` `version` field.

6. **`comita` is not the same as `git commit`.** `comita` is the full release workflow (version tag, push, wait for CI, install via cleen). For migration work-in-progress, just use `git commit`. The plugin owner runs `comita` when a release is ready.

7. **If the workspace shows evidence of active parallel activity (many worktree-agent-* branches, frequent commits by other authors, hard resets), stop and re-evaluate.** The prior session encountered exactly this.

---

## What "done" looks like at the end of this session

Being honest: the migration will not fully complete in one session. What you can genuinely finish depends on which track(s) you pick:

- **Track B alone (2-3 hours):** all book chapters have v2 code with banners. Real Phase 3 completion. Committable in one or two commits per book.
- **Track C alone (30-60 min):** MCP responses updated and committed. Phase 4 non-verification work done.
- **Track A partial (any time):** whatever Sub-cycle 1 work you complete, committed with a clear "stopped at [X], continue with [Y]" handoff note in a follow-up commit or a new next-session prompt.
- **Tracks B + C combined (~4 hours):** substantial Phase 3 and Phase 4 progress in one session, no plugin engineering.

**Recommended for a fresh session with 2-4 hours budget:** Track B + Track C. Real progress on two independent tracks, no engineering risk, banners can start coming down (partially) as Track A eventually completes.

**Recommended for a fresh session with 6+ hours budget and confidence in the workspace:** Track A first (Sub-cycle 1 partial), Track C, then Track B if time remains. Track A is the highest-value work and doing it early ensures the day's most-important commits are protected.

---

## After this session

Whatever you finish, before ending:

1. **Verify commits landed** — `git log --oneline -5` in each affected repo. Confirm the commits you made are there.
2. **Update `SPEC_DATA_PERSISTENCE_MODEL.md` §15** if Phase 2 or Phase 4 status changed materially (e.g., "Sub-cycle 1 partial — [what was done]" or "Phase 4 MCP updates complete and committed").
3. **Write a follow-up prompt** if there's meaningful continuation work — either update this file with new state and unfinished items, or write a fresh next-session prompt.

---

## Contact points for questions (files, not people)

- **Migration design questions:** `SPEC_DATA_PERSISTENCE_MODEL.md` §11 has all resolved design decisions with rationale.
- **Sub-cycle LOC targets:** `foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md` §3.
- **v1 → v2 mechanical conversion patterns:** `documents/specification/04_frame_data.md` §14 "Migration from v1" has explicit before/after code examples.
- **Plugin contract (typed emission, `expand_block_typed`, etc.):** `foundation/spec/plugins/plugin-contract.md`.
- **Grammar productions the plugin must accept:** `foundation/spec/plugins/frame-data.ebnf` v2.0.0.
- **Semantic rules the plugin must enforce:** `foundation/spec/plugins/frame-data-semantics.md` v2.0.0.

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
| `a4e0fe2` | clean-framework | 2026-07-16 aspirational warnings (superseded by 2f78640) |
| `643a042` | workspace root | 2026-07-16 book banners (superseded by 0627157) |
| `8fe2ac7` | clean-framework | 2026-07-17 plugin header retirement (Option A adopted) |
| `2f78640` | clean-framework | 2026-07-17 framework doc banners + §15 update |
| `0627157` | workspace root | 2026-07-17 book banners + prompt collision-resolved markers |

If any of these commits are missing from `git log` in their respective repo, something significant happened — investigate before proceeding.

---

**End of prompt. Begin by reading the "Read these first" section.**
