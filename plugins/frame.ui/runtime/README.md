# Frame UI Runtime

This directory contains the browser runtime for Frame UI applications.

## Files

- `shell.html` - HTML template that hosts the application
- `loader.js` - WASM loader with full browser API bridge (framework code)

## How It Works

The loader uses a single, unified approach: WASM `_start()` registers event handlers via `_ui_onEvent`, and handlers update the DOM via bridge functions. No full-page re-rendering — all updates are targeted.

```html
<script src="loader.js" data-wasm="app.wasm"></script>
```

1. Loads and instantiates the WASM module with all bridge function imports
2. Calls `_start()` — the WASM module registers event handlers via `_ui_onEvent`
3. Events are dispatched via document-level delegation using CSS selectors
4. Handlers make targeted DOM updates via bridge functions (`_ui_updateElement`, `_ui_toggleClass`, etc.)

## Bridge Functions

All bridge functions use WASM pointer+length format for strings.

### Event Registration

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_onEvent` | selector, event_type, handler_idx | integer | Register event handler via document-level delegation |

### Event Context (available during event handler execution)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_eventAttr` | attr_name | string | Get attribute from current event target |
| `_ui_eventValue` | — | string | Get value (inputs) or textContent (others) of event target |
| `_ui_eventClosestAttr` | selector, attr_name | string | Find closest ancestor matching selector, get its attribute |
| `_ui_eventType` | — | string | Get event type (click, input, etc.) |

### DOM Manipulation (single element)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_updateElement` | selector, content | integer | Set innerHTML of first matching element |
| `_ui_updateAttr` | selector, attr, value | integer | Set attribute on first matching element |
| `_ui_getText` | selector | string | Get textContent of first matching element |
| `_ui_getAttr` | selector, attr | string | Get attribute of first matching element |
| `_ui_toggleClass` | selector, class | integer | Toggle CSS class |
| `_ui_addClass` | selector, class | integer | Add CSS class |
| `_ui_removeClass` | selector, class | integer | Remove CSS class |
| `_ui_setStyle` | selector, property, value | integer | Set inline style |
| `_ui_updateElementSelf` | content | integer | Update current event target's textContent |

### DOM Batch (querySelectorAll)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_querySetStyle` | selector, property, value | integer | Set style on ALL matching elements |
| `_ui_querySetAttr` | selector, attr, value | integer | Set attribute on ALL matching elements |
| `_ui_queryAddClass` | selector, class | integer | Add class to ALL matching elements |
| `_ui_queryRemoveClass` | selector, class | integer | Remove class from ALL matching elements |
| `_ui_filterByAttr` | selector, attr, value | integer | Show elements matching attr value, hide others (`*` for all) |
| `_ui_filterByText` | selector, name_attr, desc_attr, query | integer | Filter by text search across name/description attributes |

### Browser APIs

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_clipboardWrite` | text | integer | Copy text to clipboard |
| `_ui_locationHref` | url | integer | Navigate to URL |
| `_ui_locationQuery` | param | string | Get URL query parameter |
| `_ui_locationPath` | — | string | Get current pathname |
| `_ui_observeVisible` | selector, class | integer | Add class when elements scroll into view (one-shot) |
| `_ui_setTimeout` | handler_idx, delay_ms | integer | Call `handle_event_N` after delay |

## Usage

### Compile

```bash
cln compile app/client/main.cln -o dist/app.wasm --plugins
```

### Deploy

Copy `loader.js` to your dist folder and add the script tag to your HTML:

```html
<script src="loader.js" data-wasm="app.wasm"></script>
```

### Example Clean Language Client Code

```clean
plugins:
    frame.ui

start:
    integer s = 0
    s = _ui_onEvent(".copy-button", "click", 0)
    s = _ui_onEvent(".search-input", "input", 1)
    s = _ui_observeVisible(".animate-on-scroll", "animated")

functions:
    handle_event_0()
        string code = _ui_eventClosestAttr(".code-window", "data-code")
        integer s = _ui_clipboardWrite(code)
        s = _ui_updateElementSelf("Copied!")
        s = _ui_setTimeout(2, 2000)

    handle_event_1()
        string query = _ui_eventValue()
        integer s = _ui_filterByText(".card", "data-name", "data-description", query)

    handle_event_2()
        integer s = _ui_updateElementSelf("Copy")
```

## Fullstack Apps

Fullstack apps produce two WASM files:

```
dist/
├── index.html      (shell + loader)
├── loader.js       (runtime)
├── frontend.wasm   (browser - UI)
└── backend.wasm    (server - API endpoints)
```

Run the backend with clean-server:
```bash
clean-server backend.wasm -p 3000
```
