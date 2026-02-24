Frame Developer Guidelines (09)

Project: Frame – Full-Stack Framework for Clean Language
Version: 1.0

1) Goals

Consistency first: same naming, formatting, and file structure across all modules.

Clarity over cleverness: prefer readable code to smart one-liners.

Single mental model: Clean syntax and types flow across UI, Server, and Data.

2) Naming & Formatting

Classes / Components: PascalCase (e.g., UserBadge, OrderService)

Variables / functions: camelCase (e.g., userId, loadOrders())

Files: kebab-case for non-Clean files; PascalCase.cln for components.

Tabs vs spaces: Tabs.

Line length: soft 100–120 chars.

Block style: heads end with : and indent blocks consistently.

Property access: no self/this — rely on Clean’s context-awareness.

Example (Clean)

class User
    integer id : pk, auto
    string email : unique
    boolean active = true

functions:
    boolean canPublish(User u)
        return u.role == "admin"

3) Project Structure

Frame uses a clean architecture-based folder structure:

```
myapp/
├── app.cln                 # Main entry point
├── project.toml            # Project configuration
│
└── app/
    ├── pages/              # SSR pages + companion loaders (frame.ui)
    │   ├── index.html
    │   ├── index.cln       # Companion: load(), guard()
    │   └── blog/
    │       ├── [slug].html
    │       └── [slug].cln
    │
    ├── components/         # Reusable UI components (frame.ui)
    │   ├── Header.cln
    │   └── Footer.cln
    │
    ├── layouts/            # Page layout wrappers (frame.ui)
    │   └── main.html
    │
    ├── api/                # HTTP endpoints (frame.httpserver)
    │   └── users.cln
    │
    ├── data/               # Data models / ORM (frame.data)
    │   ├── User.cln
    │   └── migrations/
    │
    ├── auth/               # Auth configuration (frame.auth)
    │   └── config.cln
    │
    └── public/             # Static assets
        └── css/
            └── main.css

├── dist/                   # Compiled WASM output
└── docs/specification/     # Documentation
```

Plugin Folder Ownership:
- frame.ui → app/pages/, app/components/, app/layouts/
- frame.httpserver → app/api/
- frame.data → app/data/
- frame.auth → app/auth/
- frame.canvas → app/canvas/

4) Clean Language Conventions

Prefer declarative blocks: where:, order:, limit: etc.

Avoid duplication: rely on type inference where obvious.

Keep host calls behind the Host Bridge; never embed raw host APIs.

Use explicit returns and typed function signatures.

Escape HTML by default; only use rawHtml() for trusted content.

5) UI Guidelines

Default to SSR, opt-in to client with client="on|visible|idle|only".

Keep components small, focused, and typed.

Accessibility: semantic tags, aria-*, focus order, keyboard support.

Theming: centralize colors/spacing in /config/ui.cln → CSS variables.

6) Data (ORM) Guidelines

One model per file; keep field rules (min, max, unique, default) by the field.

Use relations (related) instead of manual FKs.

Wrap multi-step writes in Data.tx: blocks.

Keep migrations clean: run frame db:plan before committing.

7) Server Guidelines

Pure functions where possible; no global mutable state.

Keep endpoints thin; move business rules into services/modules.

Log structured objects via host:log.

8) Testing

Unit: components render (SSR snapshot), services, policies.

Integration: API routes, DB TX behavior, auth flows.

E2E: headless browser for hydration and actions.

Location: /tests/<area>/...

Run (future): frame test.

9) Security

HTTPS everywhere; SameSite & HttpOnly cookies.

Short-lived JWTs with refresh; rotate secrets.

Least-privilege Host Bridge allowlists (FS, HTTP, ENV).

Validate inputs (compile-time + runtime).

10) Performance

Prefer SSR for first paint; hydrate only what’s interactive.

Use pagination and select projections.

Cache static assets; consider HTTP caching for GET routes.

Measure with timing logs (host:time.now).

11) Versioning & Releases

Semver for packages and plugins.

Tag releases and record changes in CHANGELOG.md.

Lock compiler/CLI version via .tool-versions or /.frame/pin.

12) Git Workflow

main (stable), develop (integration), feat/*, fix/*.

Conventional commits: feat:, fix:, docs:, chore:, refactor:, perf:.

PR checklist: tests, docs, lint.

13) CI/CD

Build: frame build --target=server (+ platform targets).

Lint & test; cache /dist.

Security job: dependency audit, secrets scan.

14) Editor & Tooling

.editorconfig

root = true

[*]
indent_style = tab
indent_size = 4
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

15) AI Development Notes

Deterministic, typed code helps agents.

Prefer --json outputs & normalized contracts.

Put bridge schemas in /docs/specification/ai_context/.

10_frame_future_evolution.md

(Place at /docs/specification/10_frame_future_evolution.md)

Frame Future Evolution (10)

Project: Frame – Full-Stack Framework for Clean Language
Version: 1.0

1) Vision

Portable, secure, elegant. All logic in Clean, running everywhere via WASM. The host only provides a small typed bridge.

2) Roadmap (12–18 months)

Pure WASI Runtime — Wasmtime/wasmCloud; wasi:http, timers, sockets.

Distributed Runtime — multi-node jobs, scheduler, durable queue.

Streaming & Incremental Rendering — chunked SSR/JSON streams.

Graph Layer — auto GraphQL (or Clean graph) from ORM.

Visual Builder — drag-and-drop over Clean; persists .cln.

Plugin Marketplace — signed plugins, permission manifests, CI tests.

Observability Pack — spans via host:log; OTLP export.

Security Hardening — key rotation helpers, mTLS adapters, SRI.

DX — frame test, frame deploy, incremental builds, hot-reload.

3) Research

Deterministic scheduling across hosts.

Zero-copy host↔WASM data (shared memory proposals).

IndexedDB/OPFS-backed SQLite parity across browsers.

4) Compatibility

Semver for compiler/CLI; deprecations one minor ahead.

Bridges backward-compatible within a major.

Migration helpers auto-upgrade manifests/configs.

5) Success Metrics

SSR TTFP ≤ 150ms (baseline hardware).

Hydration < 20ms for simple islands.

p95 CRUD latency < 100ms (local region).

Compile times improve release-over-release.

6) Contributing

RFCs for proposals; small patches with tests.

Private channel for security reports.
