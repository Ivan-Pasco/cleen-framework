# Architecture Proposal: Clean UI + SSR Integration

**Date:** 2025-01-09
**Updated:** 2025-01-09
**Status:** APPROVED - Pure Plugin Approach (Option C)

---

## 1. Executive Summary

This proposal integrates Clean UI (declarative widget system) with the existing SSR HTML-first pages system while maintaining full backwards compatibility. The approach uses "interactive islands" where Clean UI screens can be embedded within HTML pages.

**Key Decision:** All UI parsing and processing happens in the frame.ui plugin, NOT in the compiler. This keeps the Clean Language compiler focused on core language features while plugins handle framework-specific DSL.

---

## 2. Design Principles

1. **Keep HTML-first SSR as default** - Marketing pages, content sites use HTML
2. **Clean UI for interactive interfaces** - Dashboards, forms, apps use `ui.*`
3. **Islands architecture** - Embed Clean UI widgets in HTML pages
4. **Progressive enhancement** - Pages work without JavaScript, enhance with hydration
5. **Reuse existing architecture** - No rewrites, extend current systems
6. **Backwards compatible** - All existing examples must continue to work

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SOURCE FILES                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  HTML Pages (.html.cln)          │  Clean UI Screens (.cln)                 │
│  ┌─────────────────────────┐     │  ┌─────────────────────────┐             │
│  │ <html>                  │     │  │ screen "Dashboard":     │             │
│  │   <app-header>          │     │  │   state:                │             │
│  │   <screen            │     │  │     count: integer = 0  │             │
│  │     screen="Counter"/>  │     │  │   ui.column:            │             │
│  │   <main>Content</main>  │     │  │     ui.text count       │             │
│  │ </html>                 │     │  │     ui.button "+"       │             │
│  └─────────────────────────┘     │  │       onClick:          │             │
│                                  │  │         count = count+1 │             │
│                                  │  └─────────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BUILD PIPELINE                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  1. Discovery (existing)                                                     │
│     └─ Scan app/ui/pages, components, screens                               │
│  2. Plugin Expansion (existing + enhanced)                                   │
│     └─ frame.ui plugin processes ui.* blocks                                │
│  3. Codegen (existing + enhanced)                                            │
│     └─ Generate SSR render functions + hydration data                       │
│  4. Compile to WASM (existing)                                               │
│     └─ cleanc produces app.wasm                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RUNTIME                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  SERVER (clean-server)              │  CLIENT (browser)                      │
│  ┌────────────────────────────┐     │  ┌────────────────────────────┐       │
│  │ Request → WASM Handler     │     │  │ clean-runtime.js           │       │
│  │ → SSR Render               │     │  │ ├─ Hydrate islands         │       │
│  │ → HTML + Hydration JSON    │─────┼──│ ├─ Attach event handlers   │       │
│  │ → Response                 │     │  │ ├─ Manage state updates    │       │
│  └────────────────────────────┘     │  │ └─ Canvas rendering        │       │
│                                     │  └────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. UI Processing Architecture (Pure Plugin)

### 4.1 No Compiler Changes

The compiler does NOT have special grammar for UI. Instead:

1. `screen Name:` blocks are captured as `framework_block`
2. The frame.ui plugin receives the raw content string
3. Plugin parses the content using string operations
4. Plugin generates standard Clean code

This keeps the compiler focused on core language features.

### 4.2 Plugin Parsing

The frame.ui plugin parses screen content using the same approach as HTML parsing:

```clean
functions:
    // Entry point for screen blocks
    string expand_block(string block_name, string attributes, string body)
        if block_name == "screen"
            return expand_screen(attributes, body)
        if block_name == "component"
            return expand_component(attributes, body)
        return body

    // Parse screen definition
    string expand_screen(string name, string content)
        // Parse state block
        string state_vars = parse_state_block(content)

        // Parse render block
        string render_tree = parse_render_block(content)

        // Generate Clean functions
        return generate_screen_code(name, state_vars, render_tree)
```

### 4.3 Generated Output

The plugin generates standard Clean functions:

```clean
functions:
    string __screen_Counter_render_ssr(integer count)
        string html = ""
        html = html + "<div class=\"ui-column\">"
        html = html + "<span>" + count.toString() + "</span>"
        html = html + "<button data-event-id=\"evt-0\">+</button>"
        html = html + "</div>"
        return html

    string __screen_Counter_hydration()
        return "{\"state\":{\"count\":0},\"events\":[...]}"
```

---

## 5. SSR Pages Integration (Islands)

### 5.1 Island Tag Convention

HTML pages can embed Clean UI screens using `<screen>`:

```html
<!-- app/ui/pages/dashboard.html.cln -->
<!DOCTYPE html>
<html>
<head><title>Dashboard</title></head>
<body>
    <app-header></app-header>

    <!-- Clean UI island -->
    <screen name="Counter" props='{"initial": 5}'></screen>

    <main>
        <h1>Static content</h1>
    </main>

    <!-- Another island -->
    <screen name="TodoList"></screen>

    <app-footer></app-footer>
</body>
</html>
```

### 5.2 Build Transform

The codegen transforms islands to:

```clean
// Generated code
string __route_handler_0()
    string html = "<!DOCTYPE html>..."
    html = html + __component_Header_render()

    // Island: Counter
    html = html + "<div data-screen=\"Counter\" data-props='{\"initial\":5}'>"
    html = html + __screen_Counter_render_ssr(5)  // SSR content
    html = html + "</div>"
    html = html + "<script type=\"application/json\" data-hydration=\"Counter\">"
    html = html + __screen_Counter_hydration_data(5)
    html = html + "</script>"

    html = html + "<main><h1>Static content</h1></main>"
    // ... more islands
    return html
```

### 5.3 Output HTML

```html
<div data-screen="Counter" data-props='{"initial":5}'>
    <!-- SSR content -->
    <div class="ui-column" style="gap:12px;padding:20px;">
        <span class="ui-text">5</span>
        <button class="ui-button">+</button>
    </div>
</div>
<script type="application/json" data-hydration="Counter">
{"state":{"count":5},"events":[{"id":"btn-0","type":"onClick","handler":"_evt_0"}]}
</script>
```

---

## 6. Server-Side Rendering Path

### 6.1 UI Widget → HTML

Each widget renders to semantic HTML:

| Widget | HTML Output |
|--------|-------------|
| `ui.column` | `<div class="ui-column" style="display:flex;flex-direction:column;gap:{gap}px">` |
| `ui.row` | `<div class="ui-row" style="display:flex;flex-direction:row;gap:{gap}px">` |
| `ui.text` | `<span class="ui-text">{content}</span>` |
| `ui.button` | `<button class="ui-button" data-event-id="{id}">{label}</button>` |
| `ui.textField` | `<input type="text" class="ui-textField" data-event-id="{id}" value="{value}">` |
| `ui.region target="canvas"` | `<div class="ui-canvas-region" data-canvas-scene="{id}"><canvas></canvas></div>` |

### 6.2 Accessibility

DOM-rendered widgets use semantic HTML:
- `ui.button` → `<button>` (keyboard accessible)
- `ui.textField` → `<input>` with `<label>`
- `ui.checkbox` → `<input type="checkbox">`
- Focus management preserved

Canvas regions are visual-only (no accessibility for drawings).

---

## 7. Client-Side Hydration Runtime

### 7.1 Runtime Components

Create `clean-runtime.js` (~5KB minified):

```javascript
// clean-runtime.js
const CleanUI = {
    // Find and hydrate all islands
    hydrate() {
        document.querySelectorAll('[data-screen]').forEach(island => {
            const screenName = island.dataset.uiIsland;
            const hydrationScript = document.querySelector(
                `script[data-hydration="${screenName}"]`
            );
            if (hydrationScript) {
                const data = JSON.parse(hydrationScript.textContent);
                this.hydrateIsland(island, data);
            }
        });
    },

    // Hydrate single island
    hydrateIsland(element, data) {
        // Create reactive state proxy
        const state = this.createReactiveState(data.state, element);

        // Attach event handlers
        data.events.forEach(evt => {
            const el = element.querySelector(`[data-event-id="${evt.id}"]`);
            if (el) {
                el.addEventListener(evt.type.replace('on', '').toLowerCase(), (e) => {
                    this.executeHandler(evt.handler, state, e);
                });
            }
        });

        // Initialize canvas scenes
        element.querySelectorAll('[data-canvas-scene]').forEach(canvas => {
            this.initCanvasScene(canvas, data.canvasScenes[canvas.dataset.canvasScene]);
        });
    },

    // Reactive state with re-render on change
    createReactiveState(initial, element) {
        return new Proxy(initial, {
            set: (target, prop, value) => {
                target[prop] = value;
                this.updateElement(element, target);
                return true;
            }
        });
    },

    // Canvas scene rendering
    initCanvasScene(container, scene) {
        const canvas = container.querySelector('canvas');
        const ctx = canvas.getContext('2d');
        // Execute draw commands from scene.draw
        // Set up animation frame if scene.animated
    }
};

// Auto-hydrate on DOMContentLoaded
document.addEventListener('DOMContentLoaded', () => CleanUI.hydrate());
```

### 7.2 Hydration Strategies

Support multiple strategies via `client` attribute:

| Strategy | Behavior |
|----------|----------|
| `client="off"` | No hydration, pure SSR (default) |
| `client="on"` | Hydrate immediately on load |
| `client="visible"` | Hydrate when scrolled into view (IntersectionObserver) |
| `client="idle"` | Hydrate during browser idle time (requestIdleCallback) |
| `client="only"` | Client-only, no SSR (skeleton placeholder) |

---

## 8. Render Target System

### 8.1 Target Selection

```clean
app "MyApp":
    ui.target "hybrid"  // or "dom" or "canvas"
```

| Target | UI Widgets | Canvas Scenes |
|--------|------------|---------------|
| `dom` | HTML elements | Not available |
| `canvas` | Canvas-drawn | Canvas-drawn |
| `hybrid` | HTML elements | Canvas in `ui.region target="canvas"` |

### 8.2 Hybrid Mode (Recommended)

- Interactive widgets (buttons, inputs) → DOM for accessibility
- Visual-only content → Canvas for performance
- Canvas scenes only in `ui.region target="canvas"` blocks

---

## 9. Plugin Boundaries

### 9.1 Core Clean UI Plugin (`frame.ui`)

Defines and implements:
- All `ui.*` layout primitives
- All `ui.*` core widgets
- SSR rendering
- Hydration data generation
- DOM event handling

### 9.2 Domain Plugins (Marketplace)

Namespaced extensions:
- `charts.barChart`, `charts.lineChart` - charting widgets
- `maps.mapView` - mapping widget
- `admin.dataGrid` - data table with sorting/filtering
- `forms.datePicker`, `forms.fileUpload` - form widgets

Plugins provide:
- SSR render function
- Hydration metadata
- Client-side behavior (bundled JS if needed)

---

## 10. Impact Analysis

### 10.1 Compiler Changes

| File | Change | Risk |
|------|--------|------|
| `grammar.pest` | Remove `screen_block`, `ui_block` rules; update `framework_block` exclusions | Low - removal |
| `ast/mod.rs` | Keep existing types (may remove later) | None |
| `parser/token_parser.rs` | Remove `parse_screen_block()`, `parse_ui_block()` etc. | Low - removal |
| `plugins/expander.rs` | No change - plugins handle expansion | None |

**Note:** We are REMOVING compiler UI support, not adding. Screen blocks will be handled as `framework_block` by the plugin.

### 10.2 Framework Changes

| File | Change | Risk |
|------|--------|------|
| `codegen.rs` | Add island detection, hydration output | Medium |
| `discovery.rs` | Scan for `.cln` screens | Low |
| `frame.ui plugin` | Implement full ui.* expansion | High (new code) |

### 10.3 Runtime Changes

| Component | Change | Risk |
|-----------|--------|------|
| `clean-server` | No change (serves HTML) | None |
| `host-bridge` | Add canvas functions (future) | Low |
| `clean-runtime.js` | New file | New code |

### 10.4 Backwards Compatibility

- All existing HTML pages continue to work unchanged
- All existing components continue to work unchanged
- No changes to API routes or database code
- New features are opt-in

---

## 11. Migration Plan

### Phase 1: Foundation (No breaking changes)
1. Add `ui.*` grammar support to compiler
2. Implement frame.ui plugin for SSR rendering
3. Add hydration data generation

### Phase 2: Client Runtime (Optional feature)
1. Create `clean-runtime.js`
2. Add `<screen>` support to codegen
3. Implement basic hydration

### Phase 3: Canvas (Future)
1. Add canvas host bridge functions
2. Implement canvas rendering in runtime
3. Add `ui.canvasScene` support

---

## 12. Decision Points (Requiring Approval)

1. **Remove duplicate codegen?** - Should we remove the Rust HTML processing in `codegen.rs` and use `frame.ui` plugin exclusively?
   - **Recommendation:** Keep both for now. Rust codegen for HTML pages, plugin for ui.* screens.

2. **Client runtime bundling?** - How should `clean-runtime.js` be delivered?
   - **Recommendation:** Auto-inject `<script>` when page has islands.

3. **State reactivity model?** - Proxy-based (simple) or signals (efficient)?
   - **Recommendation:** Start with Proxy, optimize later if needed.

---

## READY FOR APPROVAL

This proposal maintains full backwards compatibility while enabling Clean UI integration. All existing functionality continues to work. New features are additive and opt-in.

**Next steps after approval:**
1. Create `tasks.md` with detailed implementation tasks
2. Begin implementation starting with grammar additions
3. Incremental PRs with tests for each feature

---

**End of Proposal**
