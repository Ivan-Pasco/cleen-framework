# frame.ui Plugin

UI components plugin for Clean Language. Provides DSL blocks for building reactive UI components with SSR and hydration support.

## Blocks

### component

Defines a reusable UI component with props, state, and render method.

**Attributes:**
- `name` (required) - Component class name (PascalCase)
- `client` (optional, default: "off") - Hydration mode:
  - `off` - No client-side JavaScript
  - `on` - Hydrate immediately
  - `visible` - Hydrate when visible
  - `idle` - Hydrate when browser is idle
  - `only` - Client-side only (no SSR)

**Example:**
```clean
import:
    frame.ui

component: name="Counter" client="on"
    props:
        integer initialValue = 0

    state:
        integer count = 0

    render()
        return "<div class=\"counter\">
            <span>" + count().toString() + "</span>
            <button onclick=\"increment()\">+</button>
        </div>"

    increment()
        setCount(count() + 1)
```

### layout

Defines a layout wrapper with slot support.

**Attributes:**
- `name` (required) - Layout class name

**Example:**
```clean
layout: name="MainLayout"
    return "<!DOCTYPE html>
    <html>
    <head>
        <title>My App</title>
    </head>
    <body>
        <nav>...</nav>
        <main>" + _slot() + "</main>
        <footer>...</footer>
    </body>
    </html>"
```

### page

Defines a page component with routing metadata.

**Attributes:**
- `name` (required) - Page class name
- `path` (required) - URL path (must start with /)
- `layout` (optional) - Layout component to use
- `title` (optional) - Page title

**Example:**
```clean
page: name="HomePage" path="/" layout="MainLayout" title="Home"
    return "<h1>Welcome</h1>
    <p>This is the home page.</p>"
```

## Hydration Modes

| Mode | SSR | Client Hydration | Use Case |
|------|-----|------------------|----------|
| `off` | Yes | No | Static content |
| `on` | Yes | Immediate | Interactive content |
| `visible` | Yes | On visibility | Below-fold content |
| `idle` | Yes | When idle | Low-priority interactivity |
| `only` | No | Immediate | Client-only widgets |

## Generated Code

Components generate a class with:
- Props as read-only fields
- State with getter/setter methods
- `render()` - Returns HTML string
- `renderSSR()` - Returns HTML with hydration attributes

## Building

```bash
./build.sh
# or
cln compile src/main.cln -o plugin.wasm
```

## Installation

```bash
cleen plugin install ./
```
