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
app/web/pages/*.html      (HTML with Clean Language processing)
        │
        ▼ parse
app/web/components/*.cln      (Clean components define custom tags)
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
| `.html` | `app/web/pages/` | All pages (both dynamic and static) — file-based routing |
| `.cln` | `app/web/pages/` | Companion loaders for pages (`load()`, `guard()` functions) |
| `.cln` | `app/web/components/` | Reusable UI components |

**All pages go in `app/web/pages/`**, whether they use Clean directives or not. This gives every page consistent file-based routing (e.g., `about.html` → `/about`).

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

Pages are `.html` files in `app/web/pages/`. File paths map to URL routes.

### 3.1 Basic Page

```html
<!-- app/web/pages/index.html → / -->
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
| `app/web/pages/index.html` | `/` |
| `app/web/pages/about.html` | `/about` |
| `app/web/pages/blog/index.html` | `/blog` |
| `app/web/pages/blog/[slug].html` | `/blog/:slug` |
| `app/web/pages/users/[id]/posts.html` | `/users/:id/posts` |

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

Components are defined in `app/web/components/*.cln` using `html:` blocks for templates:

```clean
// app/web/components/UserCard.cln
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

Components are auto-discovered from `app/web/components/`. The tag name comes from the `tag=` attribute in the component definition, or is derived from the filename (PascalCase → kebab-case).

**Naming convention:**
- File: `UserCard.cln`
- Tag: `user-card`
- Class: `UserCard`

**Manual registry** (optional, in `/config/tags.cln`):

```clean
tags:
    "app-header": "app/web/components/Header"
    "app-footer": "app/web/components/Footer"
    "user-card": "app/web/components/UserCard"
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
| Web adapter | `app/web/pages/name.cln` | URL params, guard, calls `app/logic/` |
| Reactive state | `app/state/` | Client-side state, drives multiple targets |

**Page companion** (lives alongside the `.html` in `app/web/pages/`):

The companion is a thin **web adapter**. It handles what is web-specific — URL params, request context, access guards — then delegates data-fetching to `app/logic/`. It does not query the database directly.

```
app/logic/
└── users.cln            # Shared: getById(), findActive(), etc.

app/web/pages/
├── profile.html         # Template (pure HTML + { } + cl-*)
└── profile.cln          # Web adapter: params + guard, calls app/logic/users
```

**State companion** (lives in `app/state/`, drives the same view across multiple render targets):

```
app/state/dashboard.cln  ──→  app/web/pages/dashboard.cln (web)
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
// app/web/routes.cln — front door, handles all /dashboard/* pages at once
routes:
    guard: "/dashboard/*" [user]
    guard: "/admin/*" [admin]
```

```clean
// app/web/pages/profile.cln — room door, resource ownership only
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
<!-- app/web/pages/profile.html -->
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
```

### 6.4 Standard Events

| Event | Description |
|-------|-------------|
| `onclick` | Click/tap |
| `oninput` | Input value change |
| `onchange` | Input blur with change |
| `onsubmit` | Form submission |
| `onfocus` | Element focused |
| `onblur` | Element lost focus |
| `onkeydown` | Key pressed |
| `onkeyup` | Key released |
| `onmouseenter` | Mouse entered |
| `onmouseleave` | Mouse left |

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
<!-- app/web/layouts/main.html -->
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
<!-- app/web/pages/about.html -->
<page layout="main"></page>

<h1>About Us</h1>
<p>We build great software.</p>
```

### 8.3 Named Slots

```html
<!-- app/web/layouts/dashboard.html -->
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
<!-- app/web/pages/dashboard.html -->
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

See [14_frame_ui_client_communication.md](14_frame_ui_client_communication.md) for implementation details.

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

### 10.2 Scoped Styles — Co-located CSS Files

Each component can have a matching `.css` file in the same folder. The framework finds it automatically by filename and scopes all its rules to that component — no configuration needed.

```
app/web/components/
    UserCard.cln
    UserCard.css
```

`UserCard.css` is plain CSS:

```css
.user-card {
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
}
```

`UserCard.cln` has no style block:

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

CSS files in `app/web/components/` are scoped automatically. CSS in `public/css/` is global.

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

For client-server communication (HTTP, WebSocket, SSE), see [14_frame_ui_client_communication.md](14_frame_ui_client_communication.md) -- the `frame.client` plugin.

---

## 15. Examples

### 15.1 Simple Page

```html
<!-- app/web/pages/index.html -->
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
// app/web/pages/blog/[slug].cln
import "app/logic/posts"

functions:
	any load(Request request)
		string slug = request.params.slug
		Post post = posts.getBySlug(slug)
		list<Comment> comments = posts.getComments(post)
		return { post: post, comments: comments }
```

```html
<!-- app/web/pages/blog/[slug].html -->
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
// app/web/components/CommentForm.cln
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

**End of Document 05**
