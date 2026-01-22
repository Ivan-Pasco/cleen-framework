# Clean UI Screens Example

This example demonstrates the **islands architecture** in Clean Framework, where interactive UI components (screens) are embedded within HTML pages.

## Overview

The islands architecture allows you to:

1. **Define screens** - Reusable, stateful UI components with Clean syntax
2. **Embed in HTML** - Use `<screen>` tags in HTML pages
3. **Choose hydration** - SSR-only, immediate, on-visible, or on-idle hydration
4. **Pass props** - Override initial state via HTML attributes

## Project Structure

```
screens/
├── config.cln                    # Framework configuration
├── app/
│   └── ui/
│       ├── pages/
│       │   └── index.html    # HTML page with embedded screens
│       └── screens/
│           ├── Counter.cln       # Counter screen definition
│           └── TodoList.cln      # TodoList screen definition
└── README.md
```

## Screen Syntax

Screens are defined using Clean Language with a declarative syntax:

```clean
screen Counter:
    state:
        count: integer = 0
        label: string = "Count"

    render:
        ui.column gap 16 padding 20:
            ui.text label + ": " + count.toString()
            ui.row gap 8:
                ui.button "-"
                    onClick:
                        count = count - 1
                ui.button "+"
                    onClick:
                        count = count + 1
```

### State Block

Define reactive state variables with types and defaults:

```clean
state:
    count: integer = 0
    name: string = ""
    active: boolean = true
```

### Render Block

Build the UI using `ui.*` widgets:

- `ui.column` - Vertical flex container
- `ui.row` - Horizontal flex container
- `ui.text` - Text display (supports expressions)
- `ui.button` - Clickable button with events
- `ui.textField` - Text input with binding

## Embedding Screens

Use the `<screen>` tag in HTML pages:

```html
<!-- Basic usage -->
<screen name="Counter" client="on"></screen>

<!-- With prop overrides -->
<screen name="Counter" client="on" count="10" label="My Counter"></screen>

<!-- SSR only (no client interactivity) -->
<screen name="Counter" client="off"></screen>

<!-- Lazy hydration (on scroll into view) -->
<screen name="Counter" client="visible"></screen>

<!-- Idle hydration (when browser is idle) -->
<screen name="Counter" client="idle"></screen>
```

### Hydration Modes

| Mode | Description |
|------|-------------|
| `off` | Server-side render only, no client JavaScript |
| `on` | Immediate hydration on page load |
| `visible` | Hydrate when element scrolls into viewport |
| `idle` | Hydrate when browser is idle |

## Running the Example

```bash
# Build the example
frame build

# Start the development server
frame serve

# Open in browser
open http://localhost:3000
```

## Generated Output

When the page is rendered, each `<screen>` tag generates:

1. **Island wrapper** - `<div data-screen="Name" data-client="mode">`
2. **SSR content** - Pre-rendered HTML from the screen's render function
3. **Hydration data** - JSON with state and event bindings (if client != "off")
4. **Runtime script** - Automatically injected when any screen needs hydration

## Key Features Demonstrated

1. **Multiple instances** - Same screen can be used multiple times with different props
2. **State isolation** - Each screen instance has its own state
3. **Progressive enhancement** - SSR content works without JavaScript
4. **Selective hydration** - Only interactive screens load client code
5. **Lazy loading** - `visible` and `idle` modes defer hydration
