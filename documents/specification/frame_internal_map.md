This file maps every specification document and internal module relationship in the Frame framework.
It helps AI development tools (like Claude Code) locate relevant files, understand module boundaries, and perform contextual reasoning when generating or validating code.

1. Document Index
Section	File	Description
01	/docs/specification/01_frame_overview.md	High-level overview, architecture, and philosophy.
02	/docs/specification/02_frame_cli.md	CLI commands, arguments, and build/deploy flow.
03	/docs/specification/03_frame_server.md	Server runtime, Host Bridge, WASM loader, SSR pipeline.
04	/docs/specification/04_frame_data.md	ORM system, queries, migrations, and schema validation.
05	/docs/specification/05_frame_ui.md	UI rendering (SSR + CSR), components, theming, hydration.
06	/docs/specification/06_frame_auth.md	Authentication system, sessions, JWT, and role guards.
07	/docs/specification/07_frame_plugins.md	Plugin API, lifecycle hooks, and extension model.
08	/docs/specification/08_frame_platforms.md	Multi-platform packaging (Web, PWA, Mobile, Desktop, Server, CLI).
09	/docs/specification/09_frame_dev_guidelines.md	Developer conventions, naming, workflow, and CI/CD.
10	/docs/specification/10_frame_future_evolution.md	Roadmap, research directions, and versioning policy.
2. Module Relationships

Each Frame subsystem has a clear boundary and relies on the Host Bridge as the shared runtime interface.
Below is a conceptual dependency map:

┌──────────────────────────────────────┐
│            Frame Framework            │
├──────────────────────────────────────┤
│  CLI        (02)                     │
│  Server     (03) ───▶ Host Bridge    │
│  Data       (04) ───▶ DB + Env       │
│  UI         (05) ───▶ HTTP + FS      │
│  Auth       (06) ───▶ Crypto + Env   │
│  Plugins    (07) ───▶ All Layers     │
│  Platforms  (08) ───▶ Host Wrappers  │
│  Guidelines (09)                     │
│  Future     (10)                     │
└──────────────────────────────────────┘

Summary of Dependencies
Module	Depends On	Provides
CLI	Compiler, Config	Build, serve, test commands
Server	WASM runtime, Bridge	Routing, SSR, API execution
Data	DB Bridge	ORM, schema migration, validation
UI	Server, Bridge	SSR/CSR, components, hydration
Auth	Crypto, Env, Data	Sessions, JWT, role management
Plugins	All	Extensibility layer
Platforms	CLI, Server	Target packaging, runtime adapters
3. Repository Paths

These are the standard paths used across all Frame-based projects:

/app/pages/*.html        → SSR page templates (HTML with {{ }} and cl-* directives)
/app/pages/*.cln         → Companion data loaders (paired by filename: load/guard)
/app/components/*.cln    → Reusable UI components
/app/layouts/*.html      → Page layout wrappers
/app/api/*.cln           → Server endpoints
/app/data/*.cln          → Data models / ORM
/app/auth/*.cln          → Auth configuration
/app/canvas/*.cln        → Canvas applications
/public/*                → Static assets (CSS, images)
/dist/*                  → Compiled WASM and bundles
/docs/specification/*    → Developer and AI documentation
/docs/specification/ai_context/* → AI context data
/tests/*                 → Unit and integration tests

4. AI Context Overview
Context Files
File	Description
frame_symbols.md	Stable tokens, syntax rules, and Clean keywords.
frame_bridge_contracts.md	JSON schema of all Host Bridge operations.
frame_internal_map.md	This file (index and module relationships).
AI Reading Order

When AI agents process the repository:

Start with frame_internal_map.md (to locate specs).

Load module-specific docs (03_server, 04_data, etc.) as needed.

Parse syntax and bridge references from frame_symbols.md and frame_bridge_contracts.md.

Cross-verify relationships between Host Bridge namespaces and Clean modules.

5. Module Integration Example

When generating a project scaffold or analyzing dependencies:

FrameApp
 ├── CLI (02)  → orchestrates build
 ├── Server (03) → executes WASM backend
 ├── Data (04) → ORM layer
 ├── UI (05)   → renders HTML + hydration
 ├── Auth (06) → protects routes
 ├── Plugins (07) → inject hooks
 ├── Platforms (08) → outputs builds
 └── Config (shared) → .cln files for each subsystem


AI tools can safely:

Trace calls from Data.tx to the host:db contract.

Identify which modules expose functions: or render() methods.

Suggest fixes or improvements by analyzing .cln files using this structure.

6. Version Compatibility
Component	Version Source	Notes
Frame Core	frame.json or frame.lock	Declares CLI/compiler version
Plugins	plugin.cln	Must declare compatible requires: version
Bridge Schema	frame_bridge_contracts.md	Backward-compatible within a major version
AI Context Files	/docs/specification/ai_context/*	Version-matched with compiler
7. Host Bridge Reference Links
Namespace	Reference	Used In
host:http	frame_bridge_contracts.md	Server, UI
host:db	frame_bridge_contracts.md	Data
host:crypto	frame_bridge_contracts.md	Auth
host:env	frame_bridge_contracts.md	Auth, Server
host:time	frame_bridge_contracts.md	Server, Data
host:log	frame_bridge_contracts.md	Server, Plugins
host:fs	frame_bridge_contracts.md	Desktop, CLI
host:sys	frame_bridge_contracts.md	CLI, Platform
8. AI Reasoning Guidance

When an AI agent is asked to:

Generate a new component, it should read from 05_frame_ui.md + frame_symbols.md.

Extend ORM or migrations, it should read from 04_frame_data.md + frame_bridge_contracts.md.

Add new plugin hooks, it should use 07_frame_plugins.md and this internal map.

Package for deployment, combine 08_frame_platforms.md + 02_frame_cli.md.

9. Example AI Workflow

Developer asks:
“Generate a new API endpoint that fetches users from the DB.”

AI checks:

Internal map → finds /docs/specification/03_frame_server.md for routes.

Reads /docs/specification/04_frame_data.md for ORM usage.

Confirms DB bridge format via /docs/specification/ai_context/frame_bridge_contracts.md.

Output is guaranteed consistent with the Frame specification.

10. Maintenance Policy

This file must always reference the latest spec files in /docs/specification/.

Version updates or new modules (e.g. 11_frame_testing.md) should be added here.

Any document removed or renamed must be reflected here before a release tag.
