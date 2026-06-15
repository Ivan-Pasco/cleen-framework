# Frame Developer Guidelines (09)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0
**Location:** `/documents/specification/09_frame_dev_guidelines.md`

---

## 1. Goals

- **Consistency first:** Same naming, formatting, and file structure across all modules.
- **Clarity over cleverness:** Prefer readable code to smart one-liners.
- **Single mental model:** Clean syntax and types flow across UI, Server, and Data.

---

## 2. Naming & Formatting

| Item | Convention | Example |
|------|-----------|---------|
| Classes / Components | PascalCase | `UserBadge`, `OrderService` |
| Variables / functions | camelCase | `userId`, `loadOrders()` |
| Files (non-Clean) | kebab-case | `user-service.toml` |
| Files (Clean components) | PascalCase.cln | `UserCard.cln` |
| Tabs vs spaces | **Tabs only** | — |
| Line length | Soft 100–120 chars | — |
| Block style | Heads end with `:`, indent consistently | — |
| Property access | No `self`/`this` — Clean is context-aware | — |

**Example (Clean Language):**

```clean
data User
    integer id : pk, auto
    string email : unique
    boolean active = true

functions:
    boolean canPublish(User u)
        return u.role == "admin"
```

---

## 3. Project Structure

For the canonical folder layout, see [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md).

**Quick reference — plugin folder ownership:**

| Plugin | Owned paths |
|--------|-------------|
| `frame.ui` | `app/ui/web/pages/`, `app/ui/web/components/`, `app/ui/web/layouts/` |
| `frame.server` | `app/server/`, `app/server/api/`, `app/server/middleware/` |
| `frame.data` | `app/data/`, `app/data/models/`, `app/data/`, `app/data/migrations/`, `app/data/` |
| `frame.auth` | `app/auth/` |
| `frame.canvas` | `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` |
| `frame.client` | *(no owned folders — available everywhere once declared in `main.cln`)* |

---

## 4. Clean Language Conventions

- Prefer declarative blocks: `where:`, `order:`, `limit:`, `guard:`, etc.
- Avoid duplication; rely on type inference where obvious.
- Keep host calls behind the Host Bridge — never embed raw host APIs.
- Use explicit returns and typed function signatures.
- Escape HTML by default; only use `rawHtml()` for trusted content.

---

## 5. UI Guidelines

- Default to SSR; opt-in to client hydration with `client="on|visible|idle|only"`.
- Keep components small, focused, and typed.
- Accessibility: semantic tags, `aria-*` attributes, focus order, keyboard support.
- Theming: centralize colors and spacing in design-token blocks → CSS variables.

---

## 6. Data (ORM) Guidelines

- One model per file; place field constraints (`min`, `max`, `unique`, `default`) next to the field.
- Use relation fields instead of manual foreign keys.
- Wrap multi-step writes in `transaction:` blocks.
- Keep migrations clean; review the diff before committing.

---

## 7. Server Guidelines

- Write pure functions where possible; avoid global mutable state.
- Keep endpoints thin; move business rules into services.
- Log structured objects via the host bridge log functions.

---

## 8. Testing

| Level | Scope | Location |
|-------|-------|----------|
| Unit | Components (SSR snapshot), services, policies | `/tests/unit/` |
| Integration | API routes, DB transactions, auth flows | `/tests/integration/` |
| E2E | Headless browser for hydration and user actions | `/tests/e2e/` |

---

## 9. Security

- HTTPS everywhere; `SameSite` and `HttpOnly` cookies.
- Short-lived JWTs with refresh; rotate secrets via environment variables.
- Least-privilege Host Bridge allowlists (FS, HTTP, ENV).
- Validate inputs at both compile-time (types) and runtime (model constraints).

---

## 10. Performance

- Prefer SSR for first paint; hydrate only interactive parts.
- Use pagination and field projections in queries.
- Cache static assets; use HTTP caching headers for GET routes.
- Measure with timing bridge functions.

---

## 11. Versioning & Releases

- Semver for packages and plugins.
- Tag releases and record changes in `CHANGELOG.md`.
- Lock compiler version via `.tool-versions` or `/.frame/pin`.

---

## 12. Git Workflow

- Branches: `main` (stable), `develop` (integration), `feat/*`, `fix/*`.
- Conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `perf:`.
- PR checklist: tests pass, docs updated, lint clean.

---

## 13. CI/CD

```bash
cleen build --target=server    # + additional platform targets
cleen test                      # lint + test suite
```

- Cache `/dist` between runs.
- Security job: dependency audit, secrets scan.

---

## 14. Editor Configuration

`.editorconfig`:

```ini
root = true

[*]
indent_style = tab
indent_size = 4
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
```

---

## 15. AI Development Notes

- Deterministic, typed code helps AI agents reason about correctness.
- Prefer `--json` outputs and normalized contracts for machine-readable results.
- Bridge schemas are documented in `foundation/spec/plugins/plugin-contract.md` and `foundation/platform-architecture/HOST_BRIDGE.md`.

---

> **See also:** [13_frame_future_evolution.md](13_frame_future_evolution.md) for roadmap and versioning policy.
> **Architecture boundaries:** [ARCHITECTURE_BOUNDARIES.md](../../../../foundation/management/ARCHITECTURE_BOUNDARIES.md)
