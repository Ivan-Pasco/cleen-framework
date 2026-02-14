# Frame UI Runtime

This directory contains the browser runtime for Frame UI applications.

## Files

- `shell.html` - HTML template that hosts the application
- `loader.js` - WASM loader with full browser API bridge (framework code)

## How It Works

The loader uses a single, unified approach: WASM `_start()` registers event handlers via `_ui_on_event`, and handlers update the DOM via bridge functions. No full-page re-rendering — all updates are targeted.

```html
<script src="loader.js" data-wasm="app.wasm"></script>
```

1. Loads and instantiates the WASM module with all bridge function imports
2. Calls `_start()` — the WASM module registers event handlers via `_ui_on_event`
3. Events are dispatched via document-level delegation using CSS selectors
4. Handlers make targeted DOM updates via bridge functions (`_ui_update_element`, `_ui_toggle_class`, etc.)

## Bridge Functions

All bridge functions use WASM pointer+length format for strings.

### Event Registration

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_on_event` | selector, event_type, handler_idx | integer | Register event handler via document-level delegation |

### Event Context (available during event handler execution)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_event_attr` | attr_name | string | Get attribute from current event target |
| `_ui_event_value` | — | string | Get value (inputs) or textContent (others) of event target |
| `_ui_event_closest_attr` | selector, attr_name | string | Find closest ancestor matching selector, get its attribute |
| `_ui_event_type` | — | string | Get event type (click, input, etc.) |

### DOM Manipulation (single element)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_update_element` | selector, content | integer | Set innerHTML of first matching element |
| `_ui_update_attr` | selector, attr, value | integer | Set attribute on first matching element |
| `_ui_get_text` | selector | string | Get textContent of first matching element |
| `_ui_get_attr` | selector, attr | string | Get attribute of first matching element |
| `_ui_toggle_class` | selector, class | integer | Toggle CSS class |
| `_ui_add_class` | selector, class | integer | Add CSS class |
| `_ui_remove_class` | selector, class | integer | Remove CSS class |
| `_ui_set_style` | selector, property, value | integer | Set inline style |
| `_ui_update_element_self` | content | integer | Update current event target's textContent |

### DOM Batch (querySelectorAll)

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_query_set_style` | selector, property, value | integer | Set style on ALL matching elements |
| `_ui_query_set_attr` | selector, attr, value | integer | Set attribute on ALL matching elements |
| `_ui_query_add_class` | selector, class | integer | Add class to ALL matching elements |
| `_ui_query_remove_class` | selector, class | integer | Remove class from ALL matching elements |
| `_ui_filter_by_attr` | selector, attr, value | integer | Show elements matching attr value, hide others (`*` for all) |
| `_ui_filter_by_text` | selector, name_attr, desc_attr, query | integer | Filter by text search across name/description attributes |

### Browser APIs

| Function | Params | Returns | Description |
|----------|--------|---------|-------------|
| `_ui_clipboard_write` | text | integer | Copy text to clipboard |
| `_ui_location_href` | url | integer | Navigate to URL |
| `_ui_location_query` | param | string | Get URL query parameter |
| `_ui_location_path` | — | string | Get current pathname |
| `_ui_observe_visible` | selector, class | integer | Add class when elements scroll into view (one-shot) |
| `_ui_set_timeout` | handler_idx, delay_ms | integer | Call `handle_event_N` after delay |

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
    s = _ui_on_event(".copy-button", "click", 0)
    s = _ui_on_event(".search-input", "input", 1)
    s = _ui_observe_visible(".animate-on-scroll", "animated")

functions:
    handle_event_0()
        string code = _ui_event_closest_attr(".code-window", "data-code")
        integer s = _ui_clipboard_write(code)
        s = _ui_update_element_self("Copied!")
        s = _ui_set_timeout(2, 2000)

    handle_event_1()
        string query = _ui_event_value()
        integer s = _ui_filter_by_text(".card", "data-name", "data-description", query)

    handle_event_2()
        integer s = _ui_update_element_self("Copy")
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
