# Frame UI Specification (05)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/docs/specification/05_frame_ui.md`

---

## 1. Introduction

**Frame UI** is an HTML‑first, component‑based UI layer written in **Clean Language**. It renders pages on the server by default (SSR) and can optionally render or hydrate components on the client using a tiny browser bridge and an optional `ui.wasm` bundle.

Goals:
- Keep UI **simple, declarative, and readable**.
- Share **one type system** across UI, API, and Data.
- Offer **SSR by default**, with **client rendering on demand**.
- Make integration with **any CSS** straightforward.

---

## 2. Architecture Overview

```
Clean Components (.cln)
        │ compile
        ▼
 ui.wasm (optional, client-only logic)
 server.wasm (SSR renderer)
        │
        ├─ SSR HTML → browser
        └─ islands manifest → loader.js (hydrates selected nodes)
```

- **Server Render (default):** Clean components render to HTML in `server.wasm`.
- **Hydration (optional):** The browser reads a small **islands manifest** and hydrates only marked components.
- **Client Render (optional):** The browser can load `ui.wasm` to build or re‑build parts of the DOM without contacting the server.

---

## 3. Component Syntax (Clean)

A component is a Clean module with a `render()` function that returns HTML.

```clean
component UserBadge
    props:
        name: string
        role?: string

    functions:
        Widget render()
            return (
                <span class="user-badge">
                    {name}
                    if role != null:
                        <ui-tag>{role}</ui-tag>
                </span>
            )
```

**Key points**
- **Props** are typed and validated at compile time.
- No `self/this` required; Clean provides context implicitly.
- `Widget` represents HTML output (string/virtual tree compiled to SSR/CSR code).

### Registering a Tag
```clean
# config/tags.cln
_tags:
    "app-user-badge" = "app/components/UserBadge.cln"
```
Usage in HTML/markup:
```html
<app-user-badge name="Alice" role="Admin"></app-user-badge>
```

---

## 4. Events & Actions

Events are declared as attributes and compiled to typed handlers.

```html
<ui-button kind="primary" onClick="saveUser"></ui-button>
```

```clean
functions:
    void saveUser()
        # business logic here
```

- On the server, actions can trigger full requests.
- On the client (hydrated/component rendered), actions run via the browser bridge.

**Standard event attributes**: `onClick`, `onInput`, `onChange`, `onSubmit`, `onKeyDown`, `onKeyUp`.

---

## 5. Rendering Modes (Simple Rule)

Use a single attribute on any component tag to control hydration:

- `client="off"` → **SSR only** (default if omitted)
- `client="on"` → SSR + hydrate **after load**
- `client="visible"` → SSR + hydrate **when visible**
- `client="idle"` → SSR + hydrate **when idle**
- `client="only"` → **Client‑only** (no SSR)

Examples:
```html
<app-user-card user-id="42"></app-user-card>                 <!-- SSR only -->
<app-counter start="5" client="on"></app-counter>           <!-- hydrate on load -->
<app-chart dataset="q4" client="visible"></app-chart>       <!-- hydrate when visible -->
<app-live-feed topic="ai" client="only"></app-live-feed>    <!-- client only -->
```

### Pro controls (inside components)
```clean
component Counter
    hydrate: on        # off | on | idle | visible | only (default = off)

    functions:
        onMount()
        onVisible()
        onIdle()
```
Page attribute `client="..."` overrides the component `hydrate:` if both are present.

---

## 6. Server‑Side Rendering (SSR)

- All pages are SSR by default for **fast first paint** and **SEO**.
- SSR output is safe‑escaped by default (prevents XSS). Interpolation `{...}` escapes HTML.
- Use `rawHtml(...)` consciously for trusted HTML only.

### SSR Pipeline
1. Router resolves page → component tree.
2. `server.wasm` renders HTML from Clean components.
3. Islands manifest is generated from `client` attributes.
4. Browser receives HTML + `loader.js` + optional `ui.wasm`.

---

## 7. Client‑Side Rendering (Optional & Friendly)

You can run **Clean UI components in the browser** with `ui.wasm` for dynamic or offline views.

**Minimal HTML boot**
```html
<div id="app"></div>
<script type="module">
  const bridge = {
    dom: {
      create: t => document.createElement(t),
      setAttr: (el, k, v) => el.setAttribute(k, v),
      append: (p, c) => p.appendChild(c),
      text: (el, t) => { el.textContent = t; }
    }
  };
  const { instance } = await WebAssembly.instantiateStreaming(fetch('/ui.wasm'), { bridge });
  instance.exports.boot('app', {});
</script>
```

**Clean side**
```clean
module UIClient
    functions:
        boot(mountId: string, state: map<string, any>)
            # build DOM via bridge.dom.*
```

> Tip: SSR + hydrate is the default. Use `ui.wasm` for offline‑first pages or highly dynamic widgets.

---

## 8. Islands Manifest & Loader

When the SSR engine finds `client="..."`, it emits a small page‑level manifest:

```
dist/manifest.islands.json
{
  "app-chart": { "bundle": "/ui/app-chart.js", "strategy": "visible" },
  "app-counter": { "bundle": "/ui/app-counter.js", "strategy": "idle" }
}
```

A tiny `loader.js` hydrates only those nodes per strategy (load/visible/idle).

---

## 9. Theming & CSS

- Use standard CSS in `/public/ui.css` (or any file under `/public`).
- Variables in `/config/ui.cln` compile to CSS variables automatically.

**`/config/ui.cln`**
```clean
ui:
    theme:
        color:
            primary = "#2563eb"
            text = "#0f172a"
```

**`/public/ui.css`**
```css
:root {
  --color-primary: #2563eb;
  --color-text: #0f172a;
}
.user-badge { font-weight: 600; color: var(--color-text); }
```

You may use any CSS framework (Tailwind, Bootstrap) without special adapters.

---

## 10. Accessibility (A11y)

- Prefer semantic HTML elements (`<button>`, `<nav>`, `<header>`).
- Always set `aria-label` or text content for interactive elements.
- Maintain focus order; use `tabindex` wisely.
- Provide text alternatives for icons/images.

---

## 11. Security

- Interpolations `{...}` are **HTML‑escaped** by default.
- Avoid `rawHtml` except for trusted, sanitized content.
- Client bridges are minimal and never expose `eval` or unrestricted FS.
- Enforce CSP headers at the host (see Server spec).

---

## 12. Testing

- **SSR snapshots:** render components in a test host and compare HTML.
- **Hydration tests:** run the loader in a headless browser and assert DOM.
- **Bridge mocks:** use a mock bridge to simulate events and state changes.

---

## 13. AI Development Notes

For Claude Code and other AI tools:
- Components are registered in `config/tags.cln` with **deterministic names**.
- Rendering modes are explicit (`client` attr, or `hydrate:` in component).
- Bridge contracts are small, typed, and consistent across platforms.
- Use this file with `03_frame_server.md` to reason about SSR/CSR interactions.

---

## 14. File Locations

- Components: `/app/components/*.cln`
- Pages: `/app/pages/*.cln`
- Tags registry: `/config/tags.cln`
- UI config: `/config/ui.cln`
- Public CSS/JS: `/public/*`
- Build output: `/dist/ui.wasm`, `/dist/manifest.islands.json`, `/public/loader.js`

---

**End of Document 05**

