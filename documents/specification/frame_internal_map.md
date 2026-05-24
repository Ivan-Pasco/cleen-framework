# Frame Internal Map

This file maps every specification document and internal module relationship in the Frame framework.
It helps development tools locate relevant files, understand module boundaries, and perform contextual reasoning when generating or validating code.

---

## 1. Document Index

| # | File | Description |
|---|------|-------------|
| 01 | [01_frame_overview.md](01_frame_overview.md) | High-level overview, architecture, and philosophy |
| 02 | [02_frame_cli.md](02_frame_cli.md) | CLI commands and workflow |
| 03 | [03_frame_server.md](03_frame_server.md) | Server runtime, HTTP routing, `endpoints:` blocks, SSR pipeline |
| 04 | [04_frame_data.md](04_frame_data.md) | ORM system, block-based queries, migrations, many-to-many |
| 05 | [05_frame_ui.md](05_frame_ui.md) | UI rendering (SSR + CSR), components, hydration, theming |
| 06 | [06_frame_auth.md](06_frame_auth.md) | Authentication (sessions, JWT), authorization (roles, permissions) |
| 07 | [07_frame_plugins.md](07_frame_plugins.md) | Plugin system, lifecycle hooks, extensibility |
| 08 | [08_frame_platforms.md](08_frame_platforms.md) | Multi-platform packaging (Web, PWA, Mobile, Desktop, Server, CLI) |
| 09 | [09_frame_dev_guidelines.md](09_frame_dev_guidelines.md) | Developer conventions, naming, workflow, CI/CD |
| 10 | [10_compiler_plugins.md](10_compiler_plugins.md) | Compiler plugin architecture (Clean Language plugins) |
| 11 | [11_database_plugins.md](11_database_plugins.md) | Database plugin architecture, C-ABI interface, runtime drivers |
| 12 | [12_frame_canvas.md](12_frame_canvas.md) | Canvas rendering, animation, drawing primitives, bridge functions |
| 13 | [13_frame_future_evolution.md](13_frame_future_evolution.md) | Roadmap, research directions, versioning policy |
| 14 | [14_frame_ui_client_communication.md](14_frame_ui_client_communication.md) | Client-side communication via frame.client plugin (api.*, live.*, feed.* namespaces) |
| -- | [frame_bridge_contracts.md](frame_bridge_contracts.md) | Host Bridge JSON contracts, WASM imports, platform availability |

---

## 2. Module Relationships

Each Frame subsystem has a clear boundary and relies on the Host Bridge as the shared runtime interface.

```
┌────────────────────────────────────────────┐
│              Frame Framework                │
├────────────────────────────────────────────┤
│  CLI              (02) → Orchestrates build│
│  Server           (03) ──▶ Host Bridge     │
│  Data / ORM       (04) ──▶ DB + Env        │
│  UI               (05) ──▶ HTTP + FS       │
│  Client Comm      (14) ──▶ api/live/feed   │
│  Auth             (06) ──▶ Crypto + Env    │
│  Plugins          (07) ──▶ All Layers      │
│  Platforms        (08) ──▶ Host Wrappers   │
│  Compiler Plugins (10) ──▶ Compiler        │
│  Database Plugins (11) ──▶ Data Layer      │
│  Canvas           (12) ──▶ Bridge (canvas) │
│  Guidelines       (09)                     │
│  Future           (13)                     │
└────────────────────────────────────────────┘
```

### Dependency Summary

| Module | Depends On | Provides |
|--------|-----------|----------|
| CLI | Compiler, Config | Build, serve, test commands |
| Server | WASM runtime, Bridge | Routing, SSR, API execution |
| Client | Browser runtime | api.*, live.*, feed.* communication |
| Data | DB Bridge | ORM, schema migration, validation |
| UI | Server, Bridge | SSR/CSR, components, hydration |
| Auth | Crypto, Env, Data | Sessions, JWT, role management |
| Plugins | All | Extensibility layer |
| Platforms | CLI, Server | Target packaging, runtime adapters |
| Compiler Plugins | Compiler | Block expansion, keyword handling |
| Database Plugins | Data, Bridge | Driver implementations, connection pooling |
| Canvas | Bridge | 2D rendering, animation, input handling |

---

## 3. Repository Paths

Standard paths used across all Frame-based projects. Each path is owned by the plugin whose `plugin.toml` declares it under `[paths] owns`.

```
/app/web/pages/*.html              → SSR page templates (HTML with {{ }} and cl-* directives)  [frame.ui]
/app/web/pages/*.cln               → Companion data loaders (paired by filename: load/guard)   [frame.ui]
/app/web/components/*.cln          → Reusable UI components                                    [frame.ui]
/app/web/layouts/*.html            → Page layout wrappers                                      [frame.ui]
/app/server/*.cln             → Server entry points                                       [frame.server]
/app/server/api/*.cln         → HTTP API endpoints                                        [frame.server]
/app/logic/*.cln    → Business logic services                                   [frame.server]
/app/server/middleware/*.cln  → Request middleware                                        [frame.server]
/app/data/*.cln                → Data models / ORM                                         [frame.data]
/app/data/models/*.cln         → Model definitions                                         [frame.data]
/app/data/*.cln        → Reusable query definitions                                [frame.data]
/app/data/migrations/*.cln     → Schema migration files                                    [frame.data]
/app/data/*.cln   → Repository layer                                          [frame.data]
/app/auth/*.cln                → Authentication and authorization configuration            [frame.auth]
/app/canvas/*.cln              → Canvas application entry points                           [frame.canvas]
/app/canvas/scenes/*.cln       → Scene definitions                                         [frame.canvas]
/app/canvas/sprites/*.cln      → Sprite sheet definitions                                  [frame.canvas]
/app/canvas/audio/*.cln        → Audio asset definitions                                   [frame.canvas]
/public/*                      → Static assets (CSS, images, served as-is)
/dist/*                        → Compiled WASM and bundles
/documents/specification/*     → Specification documents
/tests/*                       → Unit and integration tests
```

### Plugin Folder Ownership

| Plugin | Owned Paths |
|--------|-------------|
| `frame.ui` | `app/web/pages/`, `app/web/components/`, `app/web/layouts/` |
| `frame.server` | `app/server/`, `app/server/api/`, `app/logic/`, `app/server/middleware/` |
| `frame.data` | `app/data/`, `app/data/models/`, `app/data/`, `app/data/migrations/`, `app/data/` |
| `frame.auth` | `app/auth/` |
| `frame.canvas` | `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` |
| `frame.client` | *(no owned folders — used in component handlers and backend code)* |

### Plugin Block Handles

Each plugin responds to specific top-level block keywords in `.cln` files:

| Plugin | Block Keywords | Notes |
|--------|----------------|-------|
| `frame.server` | `endpoints` | |
| `frame.data` | `data`, `migrate` | |
| `frame.auth` | `auth`, `protected`, `login`, `roles` | |
| `frame.ui` | `component`, `screen`, `page`, `html`, `styles`, `ui` | |
| `frame.canvas` | `canvasScene`, `draw`, `onFrame`, `onPointerDown`, `onPointerMove`, `onKeyDown` | |
| `frame.client` | *(none — bridge-function-only plugin)* | Provides `api.*`, `live.*`, `feed.*` namespaces |

---

## 4. Host Bridge Reference

| Namespace | Reference | Used By |
|-----------|-----------|---------|
| `host:http` | frame_bridge_contracts.md | Server, UI |
| `host:db` | frame_bridge_contracts.md | Data |
| `host:crypto` | frame_bridge_contracts.md | Auth |
| `host:env` | frame_bridge_contracts.md | Auth, Server |
| `host:time` | frame_bridge_contracts.md | Server, Data |
| `host:log` | frame_bridge_contracts.md | Server, Plugins |
| `host:fs` | frame_bridge_contracts.md | Desktop, CLI |
| `host:sys` | frame_bridge_contracts.md | CLI, Platform |

---

## 5. AI Reading Order

When AI agents process the repository:

1. Start with **frame_internal_map.md** (this file) to locate specs.
2. Load module-specific docs (03_server, 04_data, etc.) as needed.
3. Parse bridge references from **frame_bridge_contracts.md**.
4. Cross-verify relationships between Host Bridge namespaces and Clean modules.

### Task-to-Document Mapping

| Task | Read These Documents |
|------|---------------------|
| Generate a new UI component | 05_frame_ui.md |
| Add client-side communication (api/live/feed) | 14_frame_ui_client_communication.md |
| Create an API endpoint | 03_frame_server.md + 04_frame_data.md |
| Extend ORM or migrations | 04_frame_data.md + frame_bridge_contracts.md |
| Add new plugin hooks | 07_frame_plugins.md + 10_compiler_plugins.md |
| Add canvas/graphics | 12_frame_canvas.md |
| Configure authentication | 06_frame_auth.md |
| Package for deployment | 08_frame_platforms.md |
| Add database driver | 11_database_plugins.md |

---

## 6. Version Compatibility

| Component | Version Source | Notes |
|-----------|---------------|-------|
| Frame Core | project.toml | Declares compiler version |
| Plugins | plugin.toml | Must declare compatible `min_compiler_version` |
| Bridge Schema | frame_bridge_contracts.md | Standard JSON envelope format |

---

## 7. Maintenance Policy

- This file must always reference the latest spec files in `/documents/specification/`.
- New modules must be added here before a release tag.
- Any document removed or renamed must be reflected here immediately.
