# Frame Multi-Platform Architecture

## Overview

This document captures the architectural design decisions made for evolving the Frame
framework from a web-only framework to a multi-platform framework supporting web,
desktop, mobile, canvas, and terminal targets — while keeping full backwards
compatibility with existing web applications.

**Core principle:** The same logic, data, auth, and state code runs on every target.
Only the rendering layer changes per platform.

---

## Folder Structure

```
my-app/
  main.cln                  ← package declaration and target config

  src/
    data/                   ← frame.data: models, migrations, seeds
      models/
      migrations/
      seeds/

    logic/                  ← pure business logic functions, no renderer code

    state/                  ← reactive state files, companion to each screen

    server/                 ← frame.server: API endpoints and middleware only
      api/
      middleware/

    ui/                     ← abstract component library (frame.ui), any target

    web/                    ← frame.web: browser rendering
      routes.cln            ← special cases only (guards, redirects, rewrites, errors)
      components/           ← web-specific reusable components
      pages/                ← auto-routed by file path

    desktop/                ← frame.desktop: native desktop rendering
      components/
      screens/

    mobile/                 ← frame.mobile: native mobile rendering
      components/
      screens/

    canvas/                 ← frame.canvas: draw calls, scenes, audio
      components/
      scenes/               ← reusable scenes, embeddable in any target

    term/                   ← frame.term: terminal/TUI rendering
      components/
      views/

  assets/
    images/
    fonts/
```

---

## Layer Ordering

Each layer depends only on layers above it:

| Order | Folder    | Plugin         | Depends on              |
|-------|-----------|----------------|-------------------------|
| 1     | data/     | frame.data     | nothing                 |
| 2     | logic/    | —              | data/                   |
| 3     | state/    | —              | logic/, data/           |
| 4     | server/   | frame.server   | logic/, data/           |
| 5     | ui/       | frame.ui       | nothing                 |
| 6     | web/      | frame.web      | logic/, state/, ui/     |
| 7     | desktop/  | frame.desktop  | logic/, state/, ui/     |
| 8     | mobile/   | frame.mobile   | logic/, state/, ui/     |
| 9     | canvas/   | frame.canvas   | logic/, state/          |
| 10    | term/     | frame.term     | logic/, state/          |

---

## Plugins

| Plugin          | Role                                                         | Status   |
|-----------------|--------------------------------------------------------------|----------|
| frame.data      | Database models, queries, migrations, seeds                  | Exists   |
| frame.auth      | Authentication, JWT, sessions, roles, CSRF                   | Exists   |
| frame.server    | HTTP server, middleware, explicit API endpoints               | Exists   |
| frame.ui        | Abstract component library and navigation, any target        | Evolving |
| frame.web       | HTML renderer, auto-routing from pages/, SSR                 | Evolving |
| frame.canvas    | Draw calls, scenes, audio, animations, embeddable anywhere   | Exists   |
| frame.desktop   | Native desktop renderer via Tauri host bridge                | Proposed |
| frame.mobile    | Native mobile renderer bridge                                | Proposed |
| frame.term      | Terminal/TUI renderer                                        | Proposed |

---

## Key Architectural Decisions

### 1. main.cln as Package Entry Point

Package configuration moves from `package.clean.toml` into `main.cln` using a
`package:` block. This keeps one language for everything. The compiler parses
`package:` blocks declaratively (no logic allowed) before full compilation.

```clean
package: MyApp
    version: "1.0.0"

    target: web
        plugins: [frame.web, frame.data, frame.auth, frame.ui]
        entry: src/web/pages/home.cln

    target: desktop
        plugins: [frame.desktop, frame.canvas, frame.data, frame.auth, frame.ui]
        entry: src/desktop/screens/home.cln
```

### 2. State Files as Screen Companions

State files in `state/` are the companions to view files. They replace the role
that HTML template files previously played for data binding.

```
state/home.cln   ←──── companion to ────►  web/pages/home.cln
                 ←──── companion to ────►  desktop/screens/home.cln
                 ←──── companion to ────►  mobile/screens/home.cln
```

The same state file drives all three renders. Logic is written once. Only the
component composition in each target file differs.

State uses the existing `state:` block syntax:

```clean
state:
    string userName = ""
    boolean loading = true

    computed:
        string displayName
            return userName ?? "Guest"
```

### 3. File-Based Routing with routes.cln for Special Cases

Pages in `src/web/pages/` are auto-routed by file path. No routing code needed
for standard pages:

```
src/web/pages/home.cln           →   /
src/web/pages/profile.cln        →   /profile
src/web/pages/profile/[id].cln   →   /profile/:id
src/web/pages/settings.cln       →   /settings
```

Special cases that cannot be expressed as files go in `src/web/routes.cln`:

```clean
routes:
    redirect: "/old-about" → "/about"
    redirect: "/blog/:slug" → "/posts/:slug"

    guard: "/admin/*" [admin]
    guard: "/dashboard/*" [user]

    error: 404 → pages/not-found.cln
    error: 500 → pages/error.cln

    rewrite: "/:username" → pages/profile/[id].cln
        resolve: username → User.first({ handle: username })
```

Rule: if a page maps cleanly to a URL, use `pages/`. If it has special behavior
(guards on many routes, redirects, rewrites, ambiguous URLs, error pages), use
`routes.cln`.

### 4. HTML Pages Unchanged

Existing web pages using `html:` blocks and SSR continue to work exactly as today.
The `src/web/pages/` folder is the new home for what was previously in `app/pages/`.
No syntax changes. No migration of html: blocks.

### 5. Abstract Component Layer (frame.ui)

`ui/` contains platform-agnostic component definitions. Each renderer plugin
implements them for its platform.

```clean
// ui/button.ui — defined once
component: Button
    props: label string, onClick fn
    layout:
        Box
            padding: 12
            Text
                content: label
```

| Renderer     | Button becomes        |
|--------------|-----------------------|
| frame.web    | `<button>` HTML       |
| frame.canvas | rect + text draw call |
| frame.desktop| native button widget  |
| frame.mobile | native touch button   |

Direct specialized components in `web/components/`, `desktop/components/` etc.
are always available when the abstract layer is not needed.

### 6. Two Modes for Components

**Direct mode** — renderer-specific, same as today:
```clean
// web/components/avatar.web.cln
component: Avatar
    html:
        <div class="avatar">
            <img src="{props.user.avatar}" />
            <span>{props.user.name}</span>
        </div>
```

**Abstract mode** — works on any target:
```clean
// ui/avatar.ui
component: Avatar
    props: user User
    layout:
        Row
            gap: 12
            Image
                src: props.user.avatar
                size: 48
            Text
                content: props.user.name
```

### 7. Canvas as Embeddable Component

`canvas/scenes/` are reusable across targets. A scene can run as a full-screen
canvas target OR be embedded inside a web page, desktop screen, or mobile screen
as a component.

Web page embedding example:
```clean
// web/pages/dashboard.cln
page: Dashboard
    use: state/dashboard.cln

    html:
        <div class="dashboard">
            <h1>Dashboard</h1>
            <canvas cln-scene="ChartScene" cln-data="{state.metrics}" />
        </div>
```

The same `canvas/scenes/chart.cln` scene runs standalone or embedded.

### 8. server/ is for APIs, Not Page Routing

The `server/` folder contains only explicit API endpoints and middleware.
Page routing is handled automatically. This separates two concerns that were
previously mixed in `endpoints:` blocks.

```
server/api/users.cln      ← GET /api/users, POST /api/users
server/api/posts.cln      ← GET /api/posts
server/middleware/cors.cln ← applied globally via routes.cln
```

---

## Migration from Current Structure

This is a folder reorganization. No syntax changes required.

| Current location        | New location               |
|-------------------------|----------------------------|
| `app/pages/home.cln`    | `src/web/pages/home.cln`   |
| `app/pages/helpers.cln` | `src/logic/helpers.cln`    |
| `app/models/`           | `src/data/models/`         |
| `app/` (endpoints)      | `src/server/api/`          |
| `package.clean.toml`    | `main.cln` (package block) |

**Effort estimate:** 2-4 hours for an existing app. Compiler changes needed:
- Support `main.cln` package declaration
- Auto-routing from `src/web/pages/`
- `routes.cln` special-case routing file

---

## Implementation Phases

### Phase 1 — Structural Reorganization (Web Only)
*No new features, no new plugins, full backwards compatibility*

- [ ] Support `main.cln` as package declaration entry point
- [ ] Auto-routing from `src/web/pages/` by file path
- [ ] `routes.cln` for guards, redirects, rewrites, error pages
- [ ] Migrate existing app folder convention to `src/` structure
- [ ] `state/` as first-class folder (state: block already works)
- [ ] `server/api/` for explicit API endpoints only

### Phase 2 — Abstract Component Layer
*frame.ui evolves from web-only to platform-agnostic*

- [ ] Define abstract component protocol in `foundation/spec/`
- [ ] `ui/` folder as abstract component library
- [ ] Component lifecycle (mount, update, unmount) in protocol
- [ ] Reconciliation protocol for each renderer
- [ ] `nav:` block for cross-platform navigation
- [ ] Refactor frame.ui to implement the abstract protocol

### Phase 3 — Canvas as Universal Renderer
*frame.canvas becomes embeddable anywhere*

- [ ] Canvas scene embedding via `cln-scene` attribute in HTML
- [ ] Canvas scene embedding in desktop/mobile screens
- [ ] Immediate mode support for game/audio loops
- [ ] `canvas/components/` for canvas-specific UI elements

### Phase 4 — New Targets
*One at a time, each as a new plugin*

- [ ] frame.desktop (Tauri host bridge)
- [ ] frame.mobile (native bridge)
- [ ] frame.term (TUI renderer)

---

## New Platform Plans

### frame.ui — Abstract Component Layer

Foundational. Everything else depends on it. The current frame.ui is HTML-specific
and needs to evolve into a protocol that any renderer can implement.

**What needs to be defined:**
- Primitive components: `Box`, `Text`, `Image`, `Button`, `Row`, `Column`, `Input`, `Scroll`
- `layout:` block syntax for composing them
- Component lifecycle: mount, update, unmount
- Reconciliation protocol — how renderers know what changed and update efficiently
- `nav:` block for cross-platform navigation
- Each renderer plugin declares which primitives it implements

**Risk:** This is the biggest design risk. Getting it wrong breaks all targets.
Must be fully specced in `foundation/spec/` and approved before any code is written.

---

### frame.canvas — Embeddable

Lower risk — the plugin already exists. Two additions needed:

- Scenes embeddable in HTML via a `cln-scene` directive on a `<canvas>` element
- Scenes embeddable in desktop/mobile screens via a canvas component in `layout:`
- Retained mode (current scene graph) stays unchanged
- Add immediate mode (`onFrame:` loop) for games and audio

The host bridge already handles draw calls. The main work is the embedding mechanism
— how a parent renderer hands a bounded area to the canvas renderer.

---

### frame.term — Terminal/TUI

Simplest new target. Build this before desktop and mobile — strict terminal
constraints will expose any gaps in the frame.ui abstract protocol quickly.

**What it needs:**
- A Rust TUI host (crossterm or ratatui as the backend)
- Maps frame.ui primitives to ANSI/box-drawing output:
  - `Box` → border characters
  - `Text` → styled terminal text
  - `Row` / `Column` → terminal layout grid
- Keyboard and mouse input through the host bridge
- No network, GPU, or file system required for basic use

If frame.ui primitives work correctly here, they work anywhere.

---

### frame.desktop — Native Desktop

Tauri is the natural host — Rust-based, aligns with the compiler, mature, and
already present in the ecosystem.

**What it needs:**
- Tauri host bridge implementation: window, file system, system tray, native dialogs
- frame.ui primitives rendered inside Tauri
- Window lifecycle: open, close, minimize, resize
- Menu bar and context menus
- OS integration: notifications, file picker, clipboard

**Two rendering strategies — choose one to start:**

| Strategy | Approach | Effort | Result |
|---|---|---|---|
| WebView-based | Render frame.ui as HTML inside Tauri WebView | Low | Ships sooner, less native feel |
| Canvas-based | Render everything via frame.canvas inside Tauri | High | Fully custom, pixel-perfect |

WebView approach ships first. Canvas-based is the long-term right answer.

---

### frame.mobile — Native Mobile

Most complex target. Two realistic paths:

| Path | Approach | Effort | Result |
|---|---|---|---|
| Capacitor shell | Wrap web output in native shell | Low | Camera, GPS, push notifications via Capacitor plugins. Less native feel. |
| Custom native bridge | WASM talks directly to iOS UIKit / Android Views | High | Full native feel, much more work |

Build Capacitor first, custom bridge later. The frame.ui abstract layer is what
makes the custom bridge viable eventually — the same component definitions render
native on mobile once the bridge is there.

---

### Build Order

| Order | Target | Depends on | Reason |
|---|---|---|---|
| 1 | frame.ui spec | nothing | Everything else depends on this |
| 2 | frame.canvas embeddable | frame.ui | Contained, validates embedding concept |
| 3 | frame.term | frame.ui | Validates abstract protocol end-to-end |
| 4 | frame.desktop | frame.ui, frame.canvas | WebView approach first |
| 5 | frame.mobile | frame.ui | Capacitor approach first |

---

## What Does Not Change

- All `html:` block syntax
- All `frame.data` queries and ORM methods
- All `frame.auth` authentication
- All `frame.canvas` scenes
- All existing `endpoints:` routing (still works, server/ is additive)
- All `state:` block syntax
- Compiler, language spec, type system

---

## Research Validation

The architecture converges with patterns from major frameworks:

| Pattern                          | Validated by                        |
|----------------------------------|-------------------------------------|
| Logic/view separation            | Flutter, React, Compose, SwiftUI    |
| Platform abstraction layer       | Qt QPA, React Native renderer       |
| Unidirectional state flow        | Elm TEA, Redux, MVI                 |
| File-based routing + escape hatch| Next.js, SvelteKit, Remix, Nuxt     |
| Canvas as universal renderer     | Flutter/Skia, Impeller              |
| State as companion to views      | SwiftUI @State, Compose ViewModel   |

WASM compilation gives Clean Language one advantage none of these have: the same
binary runs identically on web, desktop, and server. Logic files are not just
shared source — they compile to the same WASM regardless of target.
