# Frame UI Specification (05)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.1
**Location:** `/documents/specification/05_frame_ui.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Introduction

**Frame UI** is an **HTML-first**, component-based UI layer for the Frame framework. Pages are written in **standard HTML** with custom tags defined by Clean Language components. This approach ensures:

- **Designer-friendly**: Edit pages in any HTML editor (VS Code, Sublime, Dreamweaver)
- **Standard tooling**: Full support for Emmet, Prettier, HTML validators
- **Zero learning curve**: Web developers already know HTML
- **Progressive enhancement**: Valid HTML even before compilation
- **SEO-ready**: SSR by default with clean, semantic markup

---

## 2. Architecture Overview

```
app/ui/web/pages/*.html      (HTML with Clean Language processing)
        │
        ▼ parse
app/ui/web/components/*.cln      (Clean components define custom tags)
        │
        ▼ compile
dist/app.wasm                (SSR renderer)
        │
        ▼ serve
HTML + loader.js             (Browser receives rendered page)
```

**Key principle**: Pages are HTML. Components are Clean. The compiler bridges them.

### 2.1 File Extensions

| Extension | Location | Purpose |
|-----------|----------|---------|
| `.html` | `app/ui/web/pages/` | All pages (both dynamic and static) — file-based routing |
| `.cln` | `app/ui/web/pages/` | Companion loaders for pages (`load()`, `guard()` functions) |
| `.cln` | `app/ui/web/components/` | Reusable UI components |

**All pages go in `app/ui/web/pages/`**, whether they use Clean directives or not. This gives every page consistent file-based routing (e.g., `about.html` → `/about`).

The `public/` folder (at the project root) is for **assets only** (CSS, images, fonts) — not for pages.

**Static page optimization**: Pages that contain no Clean directives (`{ }`, `cl-*` attributes, or `<app-*>` components) are detected at compile time and served as pre-rendered static HTML with no runtime template processing.

This allows:
- Standard HTML tooling support (editors, linters, formatters)
- IDEs recognize `.html` files for syntax highlighting
- Consistent URL routing for all pages regardless of dynamic content
- Zero overhead for purely static pages

### 2.2 Project Structure

> **Canonical reference:** [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) — complete folder reference.

Clean Framework uses automatic file discovery. Place files in the `app/` folder:

```
app/
  web/
    pages/                    # SSR page templates + companion loaders
      index.html              # → /
      index.cln               # Companion: load() and guard()
      about.html              # → /about
      blog/
        index.html            # → /blog
        index.cln
        [slug].html           # → /blog/:slug
        [slug].cln
    components/               # Custom HTML elements
      Header.cln              # → <app-header>
      UserCard.cln            # → <user-card>
    layouts/                  # Page layout wrappers
      main.html
  server/
    api/                      # JSON API endpoints (→ /api/*)
      users.cln
  data/                       # Data models / ORM
    User.cln
  logic/                      # Shared business logic (no plugin required)
    posts.cln
  auth/                       # Auth configuration
    auth.cln
  state/                      # Shared reactive state (multi-target)
    dashboard.cln
public/                       # Static assets (CSS, images, project root)
  css/
  images/
```

**Build command:**
```bash
cln compile main.cln -o app.wasm --plugins    # Compile with plugin expansion
cleen scan                                    # Preview discovered routes/components
```

---

## 3. Page Structure (HTML)

Pages are `.html` files in `app/ui/web/pages/`. File paths map to URL routes.

### 3.1 Basic Page

```html
<!-- app/ui/web/pages/index.html → / -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Welcome</title>
</head>
<body>
    <app-header></app-header>

    <main>
        <h1>Welcome to Frame</h1>
        <p>A modern full-stack framework.</p>
    </main>

    <app-footer></app-footer>
</body>
</html>
```

### 3.2 File-Based Routing

| File Path | URL Route |
|-----------|-----------|
| `app/ui/web/pages/index.html` | `/` |
| `app/ui/web/pages/about.html` | `/about` |
| `app/ui/web/pages/blog/index.html` | `/blog` |
| `app/ui/web/pages/blog/[slug].html` | `/blog/:slug` |
| `app/ui/web/pages/users/[id]/posts.html` | `/users/:id/posts` |

### 3.3 Page Metadata

Use standard HTML `<meta>` tags and a special `<page>` element for Frame-specific configuration:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Dashboard</title>
    <meta name="description" content="User dashboard">

    <!-- Frame page configuration -->
    <page layout="admin" auth="required"></page>
</head>
<body>
    <h1>Dashboard</h1>
    <user-stats></user-stats>
</body>
</html>
```

**Page attributes:**
- `layout` - Layout component to wrap page
- `auth` - Authentication requirement: `required`, `guest`, `none`
- `roles` - Required roles: `roles="admin,editor"`
- `cache` - Cache duration: `cache="1h"`, `cache="no-store"`

---

## 4. Custom Tags (Components)

Custom HTML tags are defined by Clean Language components. Tags follow the Web Components naming convention: **lowercase with hyphens, must contain a hyphen**.

### 4.1 Using Custom Tags

```html
<!-- Pass props as HTML attributes -->
<user-card user-id="123" show-avatar="true"></user-card>

<!-- Nested content becomes slot -->
<app-modal title="Confirm">
    <p>Are you sure you want to delete?</p>
</app-modal>

<!-- Multiple named slots -->
<app-layout>
    <slot name="header">
        <h1>Page Title</h1>
    </slot>
    <slot name="content">
        <p>Main content here</p>
    </slot>
</app-layout>
```

### 4.2 Component Definition (Clean)

Components are defined in `app/ui/web/components/*.cln` using `html:` blocks for templates:

```clean
// app/ui/web/components/UserCard.cln
component: tag="user-card"
    inputs:
        string userId
        boolean showAvatar = true

    html:
        <div class="user-card">
            <img src="{getAvatar()}" alt="">
            <h3>{userId}</h3>
        </div>

    functions:
        string getAvatar()
            return "/avatars/" + userId + ".png"
```

The plugin automatically generates a `render()` function from the `html:` block at compile time.

### 4.3 The `html:` Template Block

The `html:` block provides a clean, declarative way to write HTML templates inside components and functions. It uses `{expression}` interpolation and supports conditionals via `cl-if` attributes.

#### In Functions

Use `html:` inside any function that returns a string:

```clean
string build_nav(string lang, string nav_items_html)
    html:
        <nav class="navbar">
            <a href="/">{lang}</a>
            {!nav_items_html}
        </nav>
```

When `html:` appears without attributes, it generates `string __html = "" ... return __html`. The block must be the last statement in the function.

#### Named Variable Mode

When building multiple HTML fragments in one function, use `var=` to assign the block result to a named variable instead of returning immediately:

```clean
string build_page(string title, string items_html)
    html: var="header"
        <header>
            <h1>{title}</h1>
        </header>

    html: var="footer"
        <footer>
            <p>Copyright 2026</p>
        </footer>

    return header + items_html + footer
```

**Rules:**
- `html:` (no attributes) — generates `string __html = "" ... return __html`
- `html: var="name"` — generates `string name = "" ... name = name + ...` (no return)
- Named variable mode allows combining multiple HTML fragments with custom logic

#### Interpolation Syntax

| Syntax | Meaning | Generated Code |
|--------|---------|---------------|
| `{expression}` | Escaped interpolation (safe) | `_html_escape(expression)` |
| `{!expression}` | Raw interpolation (trusted HTML) | Direct output, no escaping |

**Safe by default**: `{expression}` always escapes HTML entities to prevent XSS. Use `{!expression}` only for trusted HTML fragments (e.g., pre-rendered component output).

#### Attribute Interpolation

Interpolation works inside HTML attribute values:

```clean
html:
    <div class="badge-{this.difficulty}" data-id="{this.id}">
        <span>{this.label}</span>
    </div>
```

#### Build-Time Transformation

The `html:` block is transformed at compile time with zero runtime cost. The input:

```clean
html:
    <h3 class="title">{this.title}</h3>
    <p>{this.description}</p>
```

Generates:

```clean
string __html = ""
__html = __html + "<h3 class=\"title\">" + _html_escape(this.title) + "</h3>"
__html = __html + "<p>" + _html_escape(this.description) + "</p>"
return __html
```

### 4.4 Component Registry

Components are auto-discovered from `app/ui/web/components/`. The tag name comes from the `tag=` attribute in the component definition, or is derived from the filename (PascalCase → kebab-case).

**Naming convention:**
- File: `UserCard.cln`
- Tag: `user-card`
- Class: `UserCard`

**Manual registry** (optional, in `/config/tags.cln`):

```clean
tags:
    "app-header": "app/ui/web/components/Header"
    "app-footer": "app/ui/web/components/Footer"
    "user-card": "app/ui/web/components/UserCard"
```

---

## 5. Data Binding

### 5.1 Interpolation Syntax

Use curly braces for dynamic values:

```html
<h1>Hello, {user.name}</h1>
<p>You have {notifications.count} notifications</p>
<span class="price">{formatCurrency(product.price)}</span>
```

**Rules:**
- `{expression}` - HTML-escaped by default (XSS-safe)
- `{!expression}` - Raw HTML (use only for trusted content)
- Expressions can include property access, function calls, and simple operations

### 5.2 Page Data (Companion File Pattern)

Pages get their server-side data from a **companion `.cln` file** paired by filename. This is the only mechanism for providing data to HTML templates. No `<script>` tags of any kind are allowed in page templates.

There are three layers involved in serving a page:

| Layer | Location | Responsibility |
|-------|----------|----------------|
| Business logic | `app/logic/` | Data-fetching queries, shared across all targets |
| Web adapter | `app/ui/web/pages/name.cln` | URL params, guard, calls `app/logic/` |
| Reactive state | `app/state/` | Client-side state, drives multiple targets |

**Page companion** (lives alongside the `.html` in `app/ui/web/pages/`):

The companion is a thin **web adapter**. It handles what is web-specific — URL params, request context, access guards — then delegates data-fetching to `app/logic/`. It does not query the database directly.

```
app/logic/
└── users.cln            # Shared: getById(), findActive(), etc.

app/ui/web/pages/
├── profile.html         # Template (pure HTML + { } + cl-*)
└── profile.cln          # Web adapter: params + guard, calls app/logic/users
```

**State companion** (lives in `app/state/`, drives the same view across multiple render targets):

```
app/state/dashboard.cln  ──→  app/ui/web/pages/dashboard.cln (web)
                          ──→  app/desktop/screens/dashboard.cln (future)
```

The state companion uses a `state:` block with optional `computed:` properties:

```clean
// app/state/dashboard.cln
state:
    string userName = ""
    boolean loading = true

    computed:
        string displayName
            return userName ?? "Guest"
```

**Companion file contract:**

The companion `.cln` file may export two functions:

| Function | Required | Purpose |
|----------|----------|---------|
| `load(Request request)` | Optional | Loads data for the page; return value becomes template variables |
| `guard(Request request)` | Optional | Runs before `load()`; return `redirect(url)` to block access, `null` to allow |

Both functions must be inside a `functions:` block. The `request` parameter gives access to path params, query strings, headers, and auth state.

**Two-tier access control:**

Think of it like a building. The front door checks if you're allowed in at all. Individual office doors check if you belong in that specific room.

- **Front door** (`routes.cln` guards) — "are you logged in?", "do you have the admin role?" — declared once, applies to all matching routes automatically.
- **Room door** (companion `guard()`) — "does this post actually belong to you?" — page-specific checks only.

Use `routes.cln` for authentication and broad role checks. Use the companion `guard()` only for resource-ownership checks that require loading data first.

**Example — Profile page (front door: routes.cln handles auth; room door: companion checks ownership):**

```clean
// app/logic/users.cln
functions:
	User getById(integer id)
		return User.first:
			where:
				id == id
```

```clean
// app/ui/web/routes.cln — front door, handles all /dashboard/* pages at once
routes:
    guard: "/dashboard/*" [user]
    guard: "/admin/*" [admin]
```

```clean
// app/ui/web/pages/profile.cln — room door, resource ownership only
import "app/logic/users"

functions:
	any guard(Request request)
		integer id = request.params.id
		User user = users.getById(id)
		if user.ownerId != request.auth.userId
			return redirect("/403")
		return null

	any load(Request request)
		integer id = request.params.id
		User user = users.getById(id)
		return { user: user }
```

```html
<!-- app/ui/web/pages/profile.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{user.name}'s Profile</title>
</head>
<body>
    <h1>{user.name}</h1>
    <p>Member since {formatDate(user.createdAt)}</p>
</body>
</html>
```

**Rules:**
- The `.html` file contains ZERO logic — no `<script>` tags, no data fetching
- The `.cln` companion is optional; pages without a companion receive no server data
- A companion with only `guard()` (no `load()`) is valid for auth-gated pages
- Data returned by `load()` is merged into the template's variable scope
- Path parameters (e.g., from `[id].html`) are available as `request.params.id`
- The compiler type-checks the companion's return type against template variable usage
- Data-fetching logic belongs in `app/logic/` so it can be reused by other targets and API endpoints

### 5.3 Conditional Rendering

All directives use the `cl-` prefix to clearly distinguish Clean Language directives from standard HTML attributes.

```html
<!-- cl-if/cl-else -->
<div cl-if="user.isAdmin">
    <admin-panel></admin-panel>
</div>
<div cl-else>
    <p>Access denied</p>
</div>

<!-- cl-show (renders but hides with CSS) -->
<div cl-show="isLoading">Loading...</div>
```

### 5.4 List Rendering

Uses `cl-iterate` to match the Clean Language `iterate` keyword:

```html
<ul>
    <li cl-iterate="post in posts">
        <a href="/blog/{post.slug}">{post.title}</a>
    </li>
</ul>

<!-- With index -->
<ol>
    <li cl-iterate="item, index in items">
        {index + 1}. {item.name}
    </li>
</ol>
```

### 5.5 Directive Reference

| Directive | Purpose | Example |
|-----------|---------|---------|
| `cl-if` | Conditional rendering | `<div cl-if="isVisible">` |
| `cl-else` | Alternative branch | `<div cl-else>` |
| `cl-iterate` | Iteration over collections | `<li cl-iterate="item in items">` |
| `cl-bind` | Two-way data binding | `<input cl-bind="username">` |
| `cl-show` | CSS visibility toggle | `<div cl-show="isLoading">` |
| `cl-client` | Hydration mode | `<div cl-client="on">` |
| `cl-slot` | Content projection | `<div cl-slot="header">` |
| `cl-validate` | Form validation | `<input cl-validate="email">` |

---

## 6. Event Handling

### 6.1 Event Attributes

Use standard HTML event attributes with Clean function names:

```html
<button onclick="saveUser">Save</button>
<input oninput="updateSearch" type="text">
<form onsubmit="handleSubmit">
    ...
</form>
```

### 6.2 Event Handlers (Clean)

Define handlers in the component or page:

```clean
// In component
component: tag="user-form"
    state:
        string name = ""

    functions:
        void saveUser()
            User.create(name: name)
            redirect("/users")

        void updateName(Event e)
            name = e.target.value
```

### 6.3 Event Modifiers

```html
<!-- Prevent default -->
<form onsubmit.prevent="handleSubmit">

<!-- Stop propagation -->
<button onclick.stop="handleClick">

<!-- Once only -->
<button onclick.once="initialize">

<!-- Key modifiers -->
<input onkeydown.enter="submitForm">
<input onkeydown.escape="cancelEdit">

<!-- Passive wheel (performance hint — preventDefault will never be called) -->
<div onwheel.passive="onZoom">

<!-- Pointer button filter -->
<div onpointerdown.left="startDraw">
```

### 6.4 Standard Events

#### Mouse events
| Event | Description |
|-------|-------------|
| `onclick` | Click/tap |
| `oninput` | Input value change |
| `onchange` | Input blur with change |
| `onsubmit` | Form submission |
| `onmouseover` | Mouse over element |
| `onmouseenter` | Mouse entered element (no bubbling) |
| `onmouseleave` | Mouse left element (no bubbling) |
| `onmousedown` | Mouse button pressed |
| `onmouseup` | Mouse button released |
| `onmousemove` | Mouse moved over element |

#### Keyboard events
| Event | Description |
|-------|-------------|
| `onkeydown` | Key pressed |
| `onkeyup` | Key released |
| `onkeypress` | Key press (character produced) |

#### Focus events
| Event | Description |
|-------|-------------|
| `onfocus` | Element focused |
| `onblur` | Element lost focus |

### 6.5 Drag Events

Drag events handle palette-to-canvas drag-and-drop and layer reordering. Use `draggable="true"` on the source element.

| Event | Fires on |
|-------|----------|
| `ondragstart` | Source element when drag begins |
| `ondragend` | Source element when drag ends (drop or cancel) |
| `ondragenter` | Drop target when dragged item enters |
| `ondragover` | Drop target while dragged item is over it (continuously) |
| `ondragleave` | Drop target when dragged item leaves |
| `ondrop` | Drop target when user releases the dragged item |

**Drag data transfer:**

```html
<div
    class="palette-item"
    draggable="true"
    id="nav-bar"
    ondragstart="startDrag">
    nav-bar
</div>

<div class="canvas-drop" ondrop="handleDrop" ondragover.prevent="">
</div>
```

```clean
functions:
    void startDrag()
        string id = ui.eventAttr("id")
        ui.setDragData("component-tag", id)

    void handleDrop()
        string tag = ui.getDragData("component-tag")
        // insert component at drop position
```

`ui.setDragData(key, value)` — store data on the drag operation  
`ui.getDragData(key) -> string` — retrieve in the drop handler

### 6.6 Pointer Events

Pointer events unify mouse, touch, and stylus input. Use them for drawing tools, resize handles, and precision hit-testing.

| Event | Description |
|-------|-------------|
| `onpointerdown` | Pointer button pressed / touch began |
| `onpointermove` | Pointer moved (no button required) |
| `onpointerup` | Pointer button released / touch ended |
| `onpointercancel` | Browser cancelled the pointer (e.g. scroll took over) |
| `onpointerenter` | Pointer entered element boundary |
| `onpointerleave` | Pointer left element boundary |

Access pointer data via `ui.eventDataJson()` inside the handler, which returns a JSON string with fields: `clientX`, `clientY`, `offsetX`, `offsetY`, `pointerId`, `pressure`, `pointerType`.

```html
<div
    class="canvas-surface"
    onpointerdown="onDown"
    onpointermove="onMove"
    onpointerup="onUp">
</div>
```

### 6.7 Wheel and Scroll Events

| Event | Description |
|-------|-------------|
| `onwheel` | User scrolled with a wheel device on this element |
| `onscroll` | Element's scrollable content was scrolled |

`ui.eventDataJson()` for `onwheel` returns: `deltaX`, `deltaY`, `deltaMode`, `clientX`, `clientY`, `ctrlKey`.  
Use `.passive` modifier on `onwheel` for smooth scroll performance.

```html
<div class="canvas-wrapper" onwheel.passive="onZoom"></div>
```

```clean
functions:
    void onZoom()
        string data = ui.eventDataJson()
        number deltaY = json.getNumber(data, "deltaY")
        boolean ctrl = json.getBool(data, "ctrlKey")
        if ctrl:
            zoom = math.clamp(zoom + (deltaY > 0 ? -0.1 : 0.1), 0.25, 4.0)
```

---

## 7. Hydration & Client Rendering

### 7.1 Default: SSR Only

By default, pages render on the server. No JavaScript required.

```html
<!-- This renders server-side, no client JS -->
<user-profile user-id="123"></user-profile>
```

### 7.2 Hydration Modes

Add the `client` attribute to enable client-side interactivity:

```html
<!-- Hydrate immediately after page load -->
<counter-widget start="0" client="on"></counter-widget>

<!-- Hydrate when element becomes visible -->
<data-chart dataset="sales" client="visible"></data-chart>

<!-- Hydrate when browser is idle -->
<notifications-panel client="idle"></notifications-panel>

<!-- Client-only (no SSR) -->
<live-chat room="support" client="only"></live-chat>
```

| Mode | SSR | Hydrate When |
|------|-----|--------------|
| `client="off"` (default) | Yes | Never |
| `client="on"` | Yes | Page load |
| `client="visible"` | Yes | Intersection Observer |
| `client="idle"` | Yes | requestIdleCallback |
| `client="only"` | No | Page load |

### 7.3 Component-Level Default

Set default hydration in the component definition:

```clean
component: tag="live-editor" client="on"
    // This component always hydrates by default
```

Page-level `client` attribute overrides component default.

---

## 8. Layouts

### 8.1 Layout Definition

```html
<!-- app/ui/web/layouts/main.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{title} - MyApp</title>
    <link rel="stylesheet" href="/css/main.css">
</head>
<body>
    <app-header></app-header>

    <main>
        <slot></slot>
    </main>

    <app-footer></app-footer>
</body>
</html>
```

### 8.2 Using Layouts

```html
<!-- app/ui/web/pages/about.html -->
<page layout="main"></page>

<h1>About Us</h1>
<p>We build great software.</p>
```

### 8.3 Named Slots

```html
<!-- app/ui/web/layouts/dashboard.html -->
<div class="dashboard">
    <aside>
        <slot name="sidebar"></slot>
    </aside>
    <main>
        <slot></slot>
    </main>
</div>
```

```html
<!-- app/ui/web/pages/dashboard.html -->
<page layout="dashboard"></page>

<slot name="sidebar">
    <nav-menu></nav-menu>
</slot>

<h1>Dashboard</h1>
<stats-overview></stats-overview>
```

---

## 9. Forms

### 9.1 Basic Forms

```html
<form onsubmit.prevent="createUser">
    <label>
        Name
        <input type="text" name="name" cl-bind="formData.name">
    </label>

    <label>
        Email
        <input type="email" name="email" cl-bind="formData.email">
    </label>

    <button type="submit">Create User</button>
</form>
```

### 9.2 Two-Way Binding

The `cl-bind` attribute creates two-way data binding. **`cl-bind` requires client hydration** — components using `cl-bind` must have `client="on"` (or another active hydration mode) or the binding will not function at runtime.

```html
<!-- This component must have client="on" for cl-bind to work -->
<search-widget client="on"></search-widget>
```

Inside a `client="on"` component:

```html
<input type="text" cl-bind="searchQuery">
<textarea cl-bind="content"></textarea>
<select cl-bind="selectedCountry">
    <option cl-iterate="country in countries" value="{country.code}">
        {country.name}
    </option>
</select>
```

### 9.3 Form Helpers (frame.ui bridge functions)

These bridge functions read DOM form state from within client-side handlers. They require `frame.client` to be declared alongside `frame.ui`.

| Clean Syntax | Description |
|-------------|-------------|
| `ui.inputValue(selector)` | Get current value of an input, textarea, or select |
| `ui.formJson(selector)` | Collect all named form inputs as a JSON string |
| `ui.formData(selector)` | Collect all named form inputs as URL-encoded string |
| `ui.checked(selector)` | Get checkbox checked state (1=checked, 0=unchecked) |
| `ui.setInput(selector, value)` | Set the value of an input, textarea, or select |

```clean
functions:
    submitForm()
        string body = ui.formJson("#my-form")
        integer s = api.post("/api/users", body, "onCreated")
```

See [18_frame_client.md](18_frame_client.md) for implementation details.

### 9.4 Form Validation

```html
<input
    type="email"
    cl-bind="email"
    required
    cl-validate="email"
    error-message="Please enter a valid email">

<span cl-if="errors.email" class="error">
    {errors.email}
</span>
```

---

## 10. Styling

### 10.1 Standard CSS

Use standard CSS files in `public/css/` (project root):

```html
<head>
    <link rel="stylesheet" href="/css/main.css">
</head>
```

### 10.2 Component CSS — Convention and Override

Each component automatically links a CSS file from `public/css/components/` matching its tag name. No configuration needed — the framework injects a `<link>` tag into the page `<head>` when the component renders.

```
public/css/components/
    user-card.css    ← linked automatically for tag="user-card"
```

`user-card.css` is plain CSS:

```css
.user-card {
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
}
```

`UserCard.cln` has no style block — CSS belongs in the CSS file:

```clean
component: tag="user-card"
    inputs:
        string name
        string avatar

    html:
        <div class="user-card">
            <img src="{inputs.avatar}" alt="{inputs.name}" />
            <span>{inputs.name}</span>
        </div>
```

The framework injects `<link rel="stylesheet" href="/css/components/user-card.css">` into the `<head>` automatically. The same href used by multiple instances of the same component on one page produces exactly one `<link>` tag.

To use a different CSS path, set the `css=` attribute on the component:

```clean
component: tag="user-card" css="/css/shared/cards.css"
    ...
```

CSS in `public/css/` is served directly by the HTTP server with no processing.

### 10.3 CSS Variables (Theming)

Define theme in `main.cln` using the `ui:` block (processed by `frame.ui`):

```clean
// main.cln
target:
    frame.ui
    frame.server

ui:
    theme:
        colors:
            primary: "#2563eb"
            secondary: "#64748b"
            background: "#ffffff"
            text: "#0f172a"
        spacing:
            sm: "0.5rem"
            md: "1rem"
            lg: "2rem"
```

Generated CSS:

```css
:root {
    --color-primary: #2563eb;
    --color-secondary: #64748b;
    --color-background: #ffffff;
    --color-text: #0f172a;
    --spacing-sm: 0.5rem;
    --spacing-md: 1rem;
    --spacing-lg: 2rem;
}
```

### 10.4 Framework Compatibility

Use any CSS framework. No adapters needed:

```html
<!-- Tailwind -->
<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
    Save
</button>

<!-- Bootstrap -->
<button class="btn btn-primary">Save</button>
```

---

## 11. Accessibility

### 11.1 Semantic HTML

Always use semantic elements:

```html
<!-- Good -->
<nav>
    <ul>
        <li><a href="/">Home</a></li>
    </ul>
</nav>

<main>
    <article>
        <h1>{post.title}</h1>
    </article>
</main>

<!-- Avoid -->
<div class="nav">
    <div class="nav-item" onclick="goHome">Home</div>
</div>
```

### 11.2 ARIA Attributes

```html
<button
    aria-label="Close dialog"
    aria-expanded="{isOpen}"
    onclick="toggleMenu">
    <icon-x></icon-x>
</button>

<div role="alert" aria-live="polite" cl-if="notification">
    {notification.message}
</div>
```

---

## 12. Security

### 12.1 XSS Prevention

- `{expression}` is HTML-escaped by default
- Use `{!expression}` only for trusted, sanitized HTML
- Never use raw HTML with user input

### 12.2 CSRF Protection

Forms automatically include CSRF tokens:

```html
<form method="POST" action="/api/users">
    <!-- CSRF token auto-injected -->
    <input type="text" name="name">
    <button type="submit">Create</button>
</form>
```

### 12.3 Content Security Policy

Configure CSP in `main.cln` using the `server:` block (processed by `frame.server`):

```clean
// main.cln
target:
    frame.ui
    frame.server

server:
    security:
        csp:
            default-src: "'self'"
            script-src: "'self' 'unsafe-inline'"
            style-src: "'self' 'unsafe-inline'"
```

---

## 13. File Structure

```
app/
├── web/
│   ├── pages/               # HTML pages + companion loaders
│   │   ├── index.html
│   │   ├── index.cln        # Companion: load(), guard()
│   │   ├── about.html
│   │   ├── blog/
│   │   │   ├── index.html
│   │   │   ├── index.cln
│   │   │   ├── [slug].html
│   │   │   └── [slug].cln
│   │   └── dashboard/
│   │       ├── index.html
│   │       └── index.cln
│   ├── components/          # Clean component definitions
│   │   ├── Header.cln
│   │   ├── Footer.cln
│   │   └── UserCard.cln
│   └── layouts/             # Layout templates
│       ├── main.html
│       └── admin.html
├── server/                  # Backend modules (frame.server)
│   └── api/                 # API endpoint modules
│       └── users.cln
├── logic/                   # Shared business logic (no plugin required)
│   └── posts.cln
├── data/                    # Data models (frame.data)
│   └── User.cln
├── auth/                    # Auth configuration (frame.auth)
│   └── auth.cln
└── state/                   # Shared reactive state (multi-target)
    └── dashboard.cln

public/                      # Static assets (project root)
├── css/
│   └── main.css
└── images/
    └── logo.svg

dist/                        # Build output
├── server.wasm
├── ui.wasm
└── manifest.islands.json
```

---

## 14. Build Output

### 14.1 Islands Manifest

```json
{
    "components": {
        "counter-widget": {
            "bundle": "/js/counter-widget.js",
            "strategy": "on"
        },
        "data-chart": {
            "bundle": "/js/data-chart.js",
            "strategy": "visible"
        }
    },
    "pages": {
        "/": {
            "components": ["app-header", "app-footer"]
        },
        "/dashboard": {
            "components": ["counter-widget", "data-chart"],
            "auth": "required"
        }
    }
}
```

### 14.2 Loader Script

The `loader.js` runtime is the single browser runtime for all Frame UI applications. It loads WASM, calls `_start()`, and provides bridge functions for event handling and DOM manipulation.

```html
<script src="loader.js" data-wasm="frontend.wasm"></script>
```

**Lifecycle:**

1. Loads and instantiates the WASM module with all bridge function imports
2. Calls `_start()` — the WASM module registers event handlers via `_ui_onEvent`
3. Events are dispatched via document-level delegation using CSS selectors
4. Handlers make targeted DOM updates via bridge functions (`_ui_updateElement`, `_ui_toggleClass`, etc.)

**Client-side code pattern:**

```clean
start:
    integer s = ui.onEvent(".btn-save", "click", "saveItem")
    s = ui.onEvent(".search-input", "input", "searchCards")

functions:
    saveItem()
        string value = ui.eventValue()
        integer s = ui.updateElement("#status", "Saved!")
        s = ui.setTimeout("clearStatus", 2000)

    searchCards()
        string query = ui.eventValue()
        integer s = ui.filterByText(".card", "data-name", "data-desc", query)

    clearStatus()
        integer s = ui.updateElement("#status", "")
```

Handler parameters are **string literals naming a top-level exported function**. The runtime dispatches via `instance.exports[handlerName]`, so the function must exist at module scope (the `functions:` block). Mis-typed names are reported to the browser console at dispatch time rather than silently dropped.

All bridge functions are declared in `plugin.toml` and implemented in `loader.js`. See the Bridge Contracts specification for the complete function reference.

For client-server communication (HTTP, WebSocket, SSE), see [18_frame_client.md](18_frame_client.md) -- the `frame.client` plugin.

---

## 15. Examples

### 15.1 Simple Page

```html
<!-- app/ui/web/pages/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Welcome</title>
</head>
<body>
    <app-header></app-header>

    <main class="container">
        <h1>Welcome to Frame</h1>
        <p>Build modern web apps with HTML.</p>

        <a href="/docs" class="btn btn-primary">Get Started</a>
    </main>

    <app-footer></app-footer>
</body>
</html>
```

### 15.2 Dynamic Page with Data

Blog detail page using the three-layer pattern — logic in `app/logic/`, companion as thin web adapter:

```clean
// app/logic/posts.cln
functions:
	Post getBySlug(string slug)
		return Post.first:
			where:
				slug == slug

	list<Comment> getComments(Post post)
		return post.comments:
			order:
				createdAt desc
```

```clean
// app/ui/web/pages/blog/[slug].cln
import "app/logic/posts"

functions:
	any load(Request request)
		string slug = request.params.slug
		Post post = posts.getBySlug(slug)
		list<Comment> comments = posts.getComments(post)
		return { post: post, comments: comments }
```

```html
<!-- app/ui/web/pages/blog/[slug].html -->
<page layout="main"></page>

<article class="blog-post">
    <header>
        <h1>{post.title}</h1>
        <p class="meta">
            By {post.author.name} on {formatDate(post.createdAt)}
        </p>
    </header>

    <div class="content">
        {!post.content}
    </div>

    <section class="comments">
        <h2>Comments ({comments.length})</h2>

        <div cl-iterate="comment in comments" class="comment">
            <comment-card comment-id="{comment.id}"></comment-card>
        </div>

        <comment-form post-id="{post.id}" client="on"></comment-form>
    </section>
</article>
```

### 15.3 Interactive Component

```clean
// app/ui/web/components/CommentForm.cln
component: tag="comment-form" client="on"
    inputs:
        string postId

    state:
        string content = ""
        boolean submitting = false

    html:
        <form class="comment-form">
            <textarea cl-bind="content" placeholder="Write a comment..."></textarea>
            <button type="submit" onclick="submitComment" cl-if="submitting == false">
                Post Comment
            </button>
            <button type="button" disabled cl-if="submitting">
                Posting...
            </button>
        </form>

    functions:
        void submitComment()
            submitting = true
            Comment.insert:
                postId = postId
                content = content
            content = ""
            submitting = false
```

---

## 16. DOM Query Functions

Read element positions and attributes from the live DOM inside client-side handlers. These functions are **browser-only** — calling them during SSR returns empty string / `[]` (the server registers no-op stubs so the WASM module still loads).

### 16.1 Bounding Box

```clean
BoundsResult bounds = ui.getBounds("#my-panel")
// bounds.x, bounds.y, bounds.width, bounds.height
// bounds.top, bounds.left, bounds.right, bounds.bottom

BoundsResult rel = ui.getOffsetBounds("#child-el")
// same fields but relative to the element's offset parent, not the viewport
```

### 16.2 Scroll Position

```clean
ScrollResult scroll = ui.getScroll("#content-area")
// scroll.scrollX, scroll.scrollY, scroll.scrollWidth, scroll.scrollHeight

ui.setScroll("#content-area", 0, 500)   // scroll to y=500
```

### 16.3 Query and Attributes

```clean
Array<string> items = ui.queryAll(".card")
// returns CSS selector paths: ["#root > .card:nth-child(1)", ...]

string href = ui.getAttr("a.active", "href")

string color = ui.getComputedStyle("#header", "background-color")
// returns "rgba(255, 255, 255, 1)" or similar
```

### 16.4 Usage: Selection Overlay

```clean
functions:
    void selectElement()
        string id = ui.eventAttr("id")
        BoundsResult b = ui.getBounds("#" + id)
        overlayTop    = b.top
        overlayLeft   = b.left
        overlayWidth  = b.width
        overlayHeight = b.height
```

---

## 17. iframe Communication Bridge

Send and receive `postMessage` between a parent page and a `<iframe>` it embeds. Used by the Designer to detect element clicks inside a compiled preview iframe.

### 17.1 `cl-preview` Directive

Mark an `<iframe>` as a Designer preview target. `frame.ui` injects a click-interceptor script into the iframe's `srcdoc` that posts `designer-select` messages to the parent.

```html
<iframe
    id="preview-frame"
    cl-preview
    srcdoc="{previewHtml}">
</iframe>
```

The injected script intercepts every click inside the iframe and posts:
```json
{
  "type": "designer-select",
  "selector": "#nav-bar",
  "tagName": "div",
  "bounds": { "x": 0, "y": 0, "width": 200, "height": 48, ... },
  "attrs": [{ "name": "class", "value": "nav-bar" }]
}
```

### 17.2 Receiving Messages

```clean
functions:
    setup():
        ui.iframeOnMessage("onPreviewMessage")

    void onPreviewMessage(string origin, string message)
        string msgType = json.get(message, "type")
        if msgType == "designer-select":
            selectedSelector = json.get(message, "selector")
```

### 17.3 Other Bridge Functions

```clean
ui.iframeSend("#preview-frame", "{\"cmd\":\"highlight\",\"id\":\"nav\"}")
// send any string to the iframe via postMessage

BoundsResult b = ui.iframeGetBounds("#preview-frame", ".hero-section")
// get bounds of an element inside the iframe (100ms timeout)

boolean ok = ui.iframeInject("#preview-frame", "document.body.style.outline='2px solid red'")
// inject and run a script inside the iframe
```

---

## 18. Incremental DOM Patching

`ui.patch` replaces the entire-`innerHTML` swap of `_ui_updateElement` with a minimal diff. Use it when updating a large area (e.g. a design canvas) to preserve scroll position, selection state, and focused inputs.

```clean
functions:
    void onSourceChange(string newSource)
        string previewHtml = canvas.renderStructuralPreview(newSource)
        ui.patch("#design-canvas-content", previewHtml)
        // selection, scroll, and focus are preserved
```

**Behavioral guarantees:**
- Text nodes are updated with `nodeValue` assignment, not element replacement
- Attributes are patched in place (add / change / remove) without touching children
- Elements with `key="X"` are matched by key value across reorders, not by position
- The currently focused element is never removed if it has a counterpart in the new HTML
- Elements with `data-component` are treated atomically — only their attributes are compared; their children are managed by the component runtime

**Browser-only:** Server-side calls are no-ops (return 0).

---

## 19. Live Streaming Elements (`cl-stream`)

Connect any element to a Server-Sent Events endpoint using the `cl-stream` directive. On mount the browser opens an `EventSource` connection; each incoming `data:` event replaces the element's `innerHTML`.

```html
<div
    class="generation-log"
    cl-stream="/api/generate/{jobId}">
    Loading...
</div>
```

- The URL may contain `{interpolated}` state expressions
- Named events (`sse.emitEvent`) are dispatched as `CustomEvent` on the element
- The `EventSource` is closed automatically when the element unmounts or the server calls `sse.close()`
- Reconnection uses the interval set by `sse.retry()`, defaulting to 3000 ms

See `03_frame_server.md §18` for the server-side `STREAM` endpoint and `sse.*` functions.

---

## 20. Context Menu Event (`oncontextmenu`)

`oncontextmenu` fires when the user right-clicks (or long-presses on mobile) an element. Always use the `.prevent` modifier to suppress the browser's native context menu.

The handler receives a `ContextMenuEvent` with fields: `clientX: number`, `clientY: number`, `targetId: string`.

```clean
component: tag="file-tree"
    events:
        showMenu(e: ContextMenuEvent):
            openContextMenu(e.clientX, e.clientY, e.targetId)
    html:
        <div id="tree" oncontextmenu.prevent="showMenu">
            ...
        </div>
```

---

## 21. Global Keyboard Shortcuts

`ui.shortcut(keys, handler, scope)` registers an application-level shortcut that fires regardless of which element has focus.

- Use `"mod"` for the platform meta key: `ctrl` on Windows/Linux, `cmd` on macOS.
- Scope: `"global"` (always fires), `"focused"` (page must be focused), or a CSS selector (fires only when an ancestor of the focused element matches the selector).
- Returns an integer ID that can be passed to `ui.removeShortcut(id)`.
- `ui.clearShortcuts()` removes all registered shortcuts.
- The handler receives a `ShortcutEvent` with fields: `keys: string`, `targetId: string`.

```clean
component: tag="app-shell"
    state:
        shortcuts: Array<integer> = []
    functions:
        setup():
            shortcuts = [
                ui.shortcut("mod+s", "save", "global"),
                ui.shortcut("mod+z", "undo", "global"),
                ui.shortcut("escape", "closeModal", "global"),
                ui.shortcut("delete", "deleteSelected", "#canvas")
            ]
        teardown():
            ui.clearShortcuts()
    events:
        save(e: ShortcutEvent):   canvas.saveCurrentFile()
        undo(e: ShortcutEvent):   canvas.undo()
        closeModal(e: ShortcutEvent): modal.close()
        deleteSelected(e: ShortcutEvent): canvas.deleteSelected()
```

---

## 22. CSS Variable Runtime Manipulation

Reads and writes CSS custom properties at runtime — no page reload, no server round-trip. Required for instant theme preview, dark/light mode toggling, and design system switching.

All four functions are **browser-only**; server-side calls are silent no-ops.

| Function | Signature | Description |
|---|---|---|
| `ui.setCssVar` | `(name, value)` | Set on `:root` |
| `ui.setCssVarOn` | `(selector, name, value)` | Set on a specific element |
| `ui.getCssVar` | `(name) -> string` | Read computed value |
| `ui.applyCssVars` | `(tokensJson)` | Set multiple variables from a JSON object in one call |

`ui.applyCssVars(json)` accepts a JSON object and applies all key-value pairs atomically — faster than looping `setCssVar` for bulk updates such as design-system switches.

```clean
events:
    applyDarkMode():
        ui.setCssVar("--color-surface-base", "oklch(8% 0 0)")
        ui.setCssVar("--color-text-primary", "oklch(95% 0 0)")

    previewDesignSystem(tokens: string):
        ui.applyCssVars(tokens)

    readCurrentBrand() -> string:
        return ui.getCssVar("--color-brand-primary")
```

---

## 23. Focus Management

Programmatic focus control is required for accessible modals, dropdowns, and command palettes. All five functions are **browser-only**.

| Function | Signature | Description |
|---|---|---|
| `ui.focus` | `(selector)` | Move focus to the first matched element |
| `ui.blur` | `(selector)` | Remove focus from the first matched element |
| `ui.getFocus` | `() -> string` | Return selector of currently focused element (`""` if body) |
| `ui.focusTrap` | `(selector) -> integer` | Constrain Tab/Shift+Tab to focusable elements inside the container; return a trap ID |
| `ui.focusTrapRelease` | `(id)` | Remove the trap and restore focus to the element that was focused before the trap was set |

Multiple traps can be stacked; they are released in LIFO order.

```clean
component: tag="confirm-modal"
    state:
        trapId: integer = 0
    events:
        open():
            trapId = ui.focusTrap("#confirm-modal")
            ui.focus("#confirm-modal-primary-button")
        close():
            ui.focusTrapRelease(trapId)
```

---

## 24. Browser Storage

`storage.local.*` persists between sessions; `storage.session.*` is cleared when the tab closes. All eight functions are **browser-only** — server-side calls return `""` silently.

Namespace keys by application to avoid collisions: `"myapp:theme"`, `"myapp:sidebar"`.

| Function | Description |
|---|---|
| `storage.local.get(key) -> string` | Read value; returns `""` if not set |
| `storage.local.set(key, value)` | Write value |
| `storage.local.remove(key)` | Delete a key |
| `storage.local.clear()` | Delete all keys for this origin |
| `storage.session.get/set/remove/clear` | Same semantics, but session-scoped |

```clean
functions:
    loadPrefs():
        string theme = storage.local.get("app:theme")
        if theme == "":
            theme = "light"
        ui.applyCssVars(themes.getTokens(theme))

    savePrefs():
        storage.local.set("app:theme", currentTheme)
```

---

## 25. File Download Trigger

Two client-side functions and one server-side response helper cover both canonical download patterns.

**Client-side (browser-only — no-ops on the server):**
- `ui.downloadText(filename, content, mimeType)` — creates a `Blob` from the content string and triggers a save dialog.
- `ui.downloadUrl(url, filename)` — navigates to a URL with a `Content-Disposition: attachment` hint.

**Server-side:**
- `res.download(filename)` — sets `Content-Disposition: attachment; filename="<filename>"` on the response; use with a binary or text body.

```clean
// Client: generate CSV in the browser
events:
    exportCsv():
        string csv = table.toCsv(data)
        ui.downloadText("export.csv", csv, "text/csv")

// Server: serve a generated file
endpoints:
    GET "/api/report":
        string pdf = reports.generatePdf()
        res.download("report.pdf")
        return res.binary(pdf, "application/pdf")
```

---

## 26. Clipboard API

`ui.clipboardWrite` and `ui.clipboardRead` use `navigator.clipboard` — both operations are async. Callbacks are Clean function names passed as strings; pass `""` to ignore an outcome. Both functions are **browser-only**; server-side calls are no-ops.

| Function | Signature | Description |
|---|---|---|
| `ui.clipboardWrite` | `(text, onSuccess, onError)` | Write text; calls `onSuccess()` or `onError(msg)` |
| `ui.clipboardRead` | `(onSuccess, onError)` | Read text; calls `onSuccess(text)` or `onError(msg)` |

```clean
state:
    copyLabel: string = "Copy"
events:
    copyApiKey():
        ui.clipboardWrite(apiKey, "onCopied", "onFailed")
    onCopied():
        copyLabel = "Copied!"
    onFailed(error: string):
        copyLabel = "Copy failed"
```

---

## 27. Resize and Intersection Observers

**Resize Observer** fires when an element's own dimensions change (not the viewport). **Intersection Observer** fires when an element's visibility ratio crosses a threshold.

| Function | Signature | Description |
|---|---|---|
| `ui.resizeObserve` | `(selector, handler) -> integer` | Observe size changes; handler receives `ResizeEntry` JSON |
| `ui.resizeUnobserve` | `(id)` | Stop observing |
| `ui.intersectObserve` | `(selector, handler, threshold) -> integer` | Observe intersection; threshold is 0.0–1.0 |
| `ui.intersectUnobserve` | `(id)` | Stop observing |

`ResizeEntry` JSON: `{"width": number, "height": number, "selector": string}`

`IntersectionEntry` JSON: `{"selector": string, "ratio": number, "isVisible": boolean}`

```clean
component: tag="lazy-image"
    state:
        loaded: boolean = false
        observerId: integer = 0
    functions:
        setup():
            observerId = ui.intersectObserve("#" + imageId, "onVisible", 0.1)
        teardown():
            ui.intersectUnobserve(observerId)
    events:
        onVisible(entry: string):
            if json.getBool(entry, "isVisible") and not loaded:
                loaded = true
                ui.intersectUnobserve(observerId)
```

---

## 28. Toast Notification System

Zero-setup: `toast.success("Saved")` works without placing any markup in the layout. The toast container is created on first use; manually placing `<div id="toast-container">` overrides the auto-created position. Design tokens are used for colors — toasts automatically match the application theme.

| Function | Signature | Description |
|---|---|---|
| `toast.success` | `(text) -> integer` | Green toast, 4 s auto-dismiss |
| `toast.error` | `(text) -> integer` | Red toast, 4 s auto-dismiss |
| `toast.warning` | `(text) -> integer` | Amber toast, 4 s auto-dismiss |
| `toast.info` | `(text) -> integer` | Surface-colored toast, 4 s auto-dismiss |
| `toast.show` | `(text, variant, duration, position) -> integer` | Full control |
| `toast.dismiss` | `(id)` | Remove a specific toast |
| `toast.dismissAll` | `()` | Remove all visible toasts |

Position values: `"top-right"`, `"top-center"`, `"top-left"`, `"bottom-right"` (default), `"bottom-center"`, `"bottom-left"`. Duration `0` means permanent (no auto-dismiss).

Configure application-wide defaults in `main.cln`:

```clean
ui:
    toast:
        position = "top-right"
        duration = 3000
        maxVisible = 5
```

```clean
events:
    saveFile():
        boolean ok = files.save(current)
        if ok:
            toast.success("Saved")
        else:
            toast.error("Save failed — check your connection")

    copyLink():
        ui.clipboardWrite(shareUrl, "onCopied", "")
    onCopied():
        toast.show("Link copied", "success", 2000, "bottom-center")
```

---

## 29. Client-Side Navigation

Frame UI provides a client-side router that intercepts link clicks and performs soft navigation — updating the page without a full browser reload.

### 29.1 Link Interception

All `<a>` elements whose `href` starts with `/` are intercepted by the router at runtime. When clicked, the router:

1. Pushes the target path onto the History API stack.
2. Sends a `GET` request to the target URL with the `X-Clean-Navigate: 1` header.
3. Replaces the current page's content with the server response.

To opt out of interception on a specific link, add `data-reload`:
```html
<a href="/full-reload-page" data-reload>Force full reload</a>
```

### 29.2 Programmatic Navigation

```clean
// Navigate to a path (triggers router render)
ui.navigate("/dashboard")

// Push to history without rendering (for redirect tracking)
ui.historyPush("/checkout/step-2", "Checkout — Step 2")

// Replace current history entry (no Back button entry)
ui.historyReplace("/login", "Sign In")

// Browser Back / Forward
ui.historyBack()
ui.historyForward()

// Read current path
string path = ui.currentPath()
```

### 29.3 View Transition Animations

Elements marked with `cl-transition` participate in the View Transitions API when navigating between pages. The value becomes the element's `view-transition-name` CSS property.

```html
<!-- Page A -->
<img src="/hero.jpg" cl-transition="hero-image">

<!-- Page B — same cl-transition value creates a morphing animation -->
<img src="/hero-detail.jpg" cl-transition="hero-image">
```

The router wraps each navigation in `document.startViewTransition()`. If the browser does not support the API, navigation proceeds without animation.

### 29.4 Server-Side Handling

Server handlers can detect soft navigation and return a partial response:

```clean
endpoints:
    GET "/dashboard" :
        boolean isNav = req.header("X-Clean-Navigate") == "1"
        if isNav
            return json({ partial: true, title: "Dashboard", html: renderWith("dashboard", data) })
        return render("dashboard")
```

---

## 30. Error Boundaries

Error boundaries catch runtime errors thrown inside a component subtree and display a fallback UI instead of crashing the entire page.

### 30.1 Declaring a Boundary

Add `cl-boundary` to any element to make it the boundary for its subtree:

```html
<div cl-boundary>
    <user-widget></user-widget>
</div>
```

When a runtime error is thrown inside `<user-widget>`, the `<div cl-boundary>` catches it. The component's `fallback:` section is rendered inside the boundary element.

### 30.2 Component Fallback Section

Components declare their fallback UI in a `fallback:` section:

```clean
component UserWidget:
    state:
        string userId = ""

    fallback:
        html:
            <div class="cl-error">
                <p>Could not load user information.</p>
                <button onclick="retry">Try again</button>
            </div>

    html:
        <div>
            <h2>{{ user.name }}</h2>
        </div>
```

### 30.3 CSS Class

When a boundary catches an error, the `cl-error` CSS class is added to the boundary element:

```css
.cl-error {
    border: 1px solid #f00;
    padding: 1rem;
    background: #fff5f5;
}
```

This class is removed when the component is successfully re-rendered (e.g. after a `retry` call).

### 30.4 Error Logging

Caught errors are logged to the browser console in the format:

```
[frame.ui] Error boundary caught in <ComponentName>: <error message>
  at <stack trace>
```

---

## 31. Plugin Contracts v2 Integration

Since frame.ui **2.10.139**, the plugin opts into the Plugin Contracts v2 lifecycle and artifact model defined in [foundation/spec/plugins/contracts/](../../../foundation/spec/plugins/contracts/). This section documents the user-visible behavior; the contract documents are authoritative.

### 31.1 The `client_init` lifecycle slot — HYDRATE_AUTO

**Closes:** HYDRATE_AUTO. **Plugin.toml:** `[lifecycle] client_init = "emit_ui_client_init"`. **Contract reference:** [lifecycle.md §3.3](../../../foundation/spec/plugins/contracts/lifecycle.md#33-client_init).

When the compiler emits the synthetic client `_start` body for a page, it invokes frame.ui's `emit_ui_client_init` slot once. The slot reads the registered component list from `_build_state_get("frame.ui:components")` and emits, per component with `client != "off"`:

- a module-level instance variable wired to the data-island wrapper
- a call to the component's `onMount()` lifecycle function

This replaces the pre-v2 inlined hydration scaffolding. Nothing in user code changes — the auto-hydration just works once the page declares one or more `component:` blocks with non-`off` hydration.

```clean
component UserList(users: any):
    client = "visible"          // triggers client_init emission for this component
    state:
        any selected = null
    events:
        onSelect(string id):
            state.selected = id
```

At build time the compiler:
1. Calls `expand_block` for each `component:`, which records the component in `_build_state_set("frame.ui:components", ...)`.
2. After all expansions, invokes the `client_init` slot. The slot iterates the recorded list and emits hydration statements.
3. Compiles the assembled client module into `{output_dir}/frontend.wasm` (see §31.2).

### 31.2 The `frontend.wasm` artifact

**Closes:** BUILD_FRONTEND, SRV004. **Plugin.toml:** `[[artifacts]] name = "frontend.wasm"`. **Contract reference:** [artifacts.md §4.1](../../../foundation/spec/plugins/contracts/artifacts.md#41-client_hydration).

The plugin declares a side-channel build artifact that the compiler emits whenever at least one component is registered:

```toml
[[artifacts]]
name            = "frontend.wasm"
purpose         = "client_hydration"
emit            = { from_module = "client_only_build" }
output_relative = "{output_dir}/frontend.wasm"
required_when   = "has_build_state.frame.ui:components"
public          = true
content_type    = "application/wasm"
```

Behavior:
- `required_when = "has_build_state.frame.ui:components"` — the artifact is only emitted when the plugin has registered at least one component during expand. Server-only API apps that import `frame.server` but no UI components do not trigger a client build.
- `emit.from_module = "client_only_build"` — the compiler triggers a nested client-mode build, applies the `client_init` lifecycle slot, and writes the resulting WASM bytes to the output path.
- `public = true` — the asset is served to the browser by the runtime; `loader.js` fetches it during hydration.

### 31.3 Server-guard stubs (v2.12.7+)

In server-only builds (no client hydration), the runtime still loads `loader.js`. Bridge functions whose `hosts` field does not include `"server"` are stubbed with no-op implementations to prevent runtime traps when SSR markup references them. Examples:

| Bridge | Server stub behavior |
|--------|----------------------|
| `_ui_render_page` | Real implementation (declares `hosts = ["server"]` with the component-tag-render callback). |
| `_ui_toggle_class` | No-op stub (returns 0). |
| `_ui_query_selector` | Returns empty string. |
| All `_storage_*` | No-op (server has no localStorage/sessionStorage). |

The full server/browser split for every bridge is in [bridge-host-classes.md §2](../../../foundation/spec/plugins/contracts/bridge-host-classes.md#2-the-hosts-field) and the plugin's `plugin.toml [bridge] functions` table.

### 31.4 Build-state bridges (`_build_state_set` / `_build_state_get`)

These bridges exist in the compiler's plugin sandbox; both declare `hosts = ["all"]`. The plugin uses them to pass the component registry from `expand_block` (called per component definition) to `emit_ui_client_init` (called once at the end of compilation). See [lifecycle.md §2.5](../../../foundation/spec/plugins/contracts/lifecycle.md#25-build-state-bridges-_build_state_set-_build_state_get).

Application code never calls these bridges directly.

### 31.5 `cln_signature` on language functions

Every `[[language.functions]]` entry in `plugin.toml` now declares its Clean Language signature via the `signature` field (e.g. `"ui.toggleClass(selector, className)"`) and explicit `params`/`returns`. The compiler uses these to type-check call sites without hardcoded resolver entries. Users do not interact with these declarations directly — they are visible in IDE hover and completion.

---

## 32. Version History

| Version | Notable changes |
|---------|-----------------|
| 2.12.15 | `cl-if`/`cl-show` route numeric-literal comparisons through `.toInteger()`. |
| 2.12.13 | Diagnostic: probe whether `expand_component` is reached. |
| 2.12.10 | Reject YAML-style component attrs with a clear error message. |
| 2.12.9 | Add `_ui_render_page` server-guard stub. |
| 2.12.8 | Gate `frontend.wasm` emission on `has_build_state.frame.ui:components`. |
| 2.12.7 | Close CLIENT_PULLS_SERVER — add server-guard stubs to `loader.js`. |
| 2.12.6 | Pass-through `events:` blocks to compiler 0.30.273 thunks. |
| 2.12.0 | Module-level instance + bare-name shims for event handlers. |
| 2.11.0 | Populate data-island wrapper before `onMount` in `client_init` splice. |
| 2.10.145 | Re-apply v2 `client_init` + `[[artifacts]]` (closes HYDRATE_AUTO). |
| 2.10.138 | Move `_ui_render_page` bridge from frame.server to frame.ui. |

---

**End of Document 05**
