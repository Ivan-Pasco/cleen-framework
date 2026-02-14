# Frame UI Specification (05)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.1
**Location:** `/docs/specification/05_frame_ui.md`

---

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
app/ui/pages/*.html      (HTML with Clean Language processing)
        │
        ▼ parse
app/ui/components/*.cln      (Clean components define custom tags)
        │
        ▼ compile
dist/app.wasm                (SSR renderer)
        │
        ▼ serve
HTML + loader.js             (Browser receives rendered page)
```

**Key principle**: Pages are HTML. Components are Clean. The compiler bridges them.

### 2.1 File Extensions

| Extension | Purpose |
|-----------|---------|
| `.html` | HTML pages that need Clean Language processing (data binding, components, etc.) |
| `.html` | Static HTML files served as-is without processing |
| `.cln` | Clean Language source files (components, logic, etc.) |

The `.html` extension indicates that the file contains HTML that should be processed by the Clean Language compiler. This allows:
- Clear distinction between static and dynamic pages
- Standard HTML tooling support (editors, linters, formatters)
- IDEs can recognize the file as HTML for syntax highlighting

### 2.2 Project Structure

Clean Framework uses automatic file discovery. Place files in the `app/` folder:

```
app/
  ui/                         # Frontend (browser-facing)
    pages/                    # HTML page routes
      index.html          # → /
      about.html          # → /about
      blog/
        index.html        # → /blog
        [slug].html       # → /blog/:slug
    components/               # Custom HTML elements
      Header.cln              # → <app-header>
      UserCard.cln            # → <user-card>
    layouts/                  # Page wrappers
      main.html
    public/                   # Static assets (CSS, images, JS)
      css/
      images/
  server/                     # Backend
    api/                      # JSON API endpoints (→ /api/*)
    models/                   # Database schemas
    middleware/               # Request filters
  shared/                     # Shared code
    lib/                      # Utility modules
```

**Build command:**
```bash
frame build           # Discovers files, generates main.cln, compiles to WASM
frame scan            # Preview discovered routes/components
```

---

## 3. Page Structure (HTML)

Pages are `.html` files in `app/ui/pages/`. File paths map to URL routes.

### 3.1 Basic Page

```html
<!-- app/ui/pages/index.html → / -->
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
| `app/ui/pages/index.html` | `/` |
| `app/ui/pages/about.html` | `/about` |
| `app/ui/pages/blog/index.html` | `/blog` |
| `app/ui/pages/blog/[slug].html` | `/blog/:slug` |
| `app/ui/pages/users/[id]/posts.html` | `/users/:id/posts` |

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

Components are defined in `app/ui/components/*.cln`:

```clean
// app/ui/components/UserCard.cln
component: tag="user-card"
    props:
        string userId
        boolean showAvatar = true

    functions:
        string render()
            user = User.find(userId)
            html = "<div class=\"user-card\">"
            if showAvatar
                html = html + "<img src=\"" + user.avatar + "\" alt=\"\">"
            html = html + "<h3>" + user.name + "</h3>"
            html = html + "</div>"
            return html
```

### 4.3 The `html:` Template Block

The `html:` block provides a clean, declarative way to write HTML templates inside components and functions. It replaces verbose string concatenation with actual HTML that supports `{expression}` interpolation.

#### In Components

Use `html:` instead of a manual `render()` function:

```clean
// app/ui/components/UserCard.cln
component: tag="user-card"
    props:
        string userId
        boolean showAvatar = true

    html:
        <div class="user-card">
            <h3>{this.userId}</h3>
        </div>

    functions:
        string getAvatar()
            return "/avatars/" + this.userId + ".png"
```

The plugin automatically generates a `render()` function from the `html:` block at compile time. The generated code is identical to hand-written string concatenation.

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

Components are auto-discovered from `app/ui/components/`. The tag name comes from the `tag=` attribute in the component definition, or is derived from the filename (PascalCase → kebab-case).

**Naming convention:**
- File: `UserCard.cln`
- Tag: `user-card`
- Class: `UserCard`

**Manual registry** (optional, in `/config/tags.cln`):

```clean
tags:
    "app-header": "app/components/Header"
    "app-footer": "app/components/Footer"
    "user-card": "app/components/UserCard"
```

---

## 5. Data Binding

### 5.1 Interpolation Syntax

Use double curly braces for dynamic values:

```html
<h1>Hello, {{user.name}}</h1>
<p>You have {{notifications.count}} notifications</p>
<span class="price">{{formatCurrency(product.price)}}</span>
```

**Rules:**
- `{{expression}}` - HTML-escaped by default (XSS-safe)
- `{{{expression}}}` - Raw HTML (use only for trusted content)
- Expressions can include property access, function calls, and simple operations

### 5.2 Page Data

Data for pages comes from a companion `.cln` file or inline `<script type="text/clean">`:

```html
<!-- app/ui/pages/profile.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{{user.name}}'s Profile</title>
</head>
<body>
    <h1>{{user.name}}</h1>
    <p>Member since {{formatDate(user.createdAt)}}</p>

    <script type="text/clean">
        data:
            user = User.find(params.id)
    </script>
</body>
</html>
```

Or in a separate file:

```clean
// app/ui/pages/profile.cln
page: path="/profile/[id]"
    data:
        user = User.find(params.id)
```

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
        <a href="/blog/{{post.slug}}">{{post.title}}</a>
    </li>
</ul>

<!-- With index -->
<ol>
    <li cl-iterate="item, index in items">
        {{index + 1}}. {{item.name}}
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
<!-- app/ui/layouts/main.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{title}} - MyApp</title>
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
<!-- app/ui/pages/about.html -->
<page layout="main"></page>

<h1>About Us</h1>
<p>We build great software.</p>
```

### 8.3 Named Slots

```html
<!-- app/ui/layouts/dashboard.html -->
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
<!-- app/ui/pages/dashboard.html -->
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

The `cl-bind` attribute creates two-way data binding:

```html
<input type="text" cl-bind="searchQuery">
<textarea cl-bind="content"></textarea>
<select cl-bind="selectedCountry">
    <option cl-iterate="country in countries" value="{{country.code}}">
        {{country.name}}
    </option>
</select>
```

### 9.3 Form Validation

```html
<input
    type="email"
    cl-bind="email"
    required
    cl-validate="email"
    error-message="Please enter a valid email">

<span cl-if="errors.email" class="error">
    {{errors.email}}
</span>
```

---

## 10. Styling

### 10.1 Standard CSS

Use standard CSS files in `/public/css/`:

```html
<head>
    <link rel="stylesheet" href="/css/main.css">
</head>
```

### 10.2 Scoped Styles

Add scoped styles within components:

```html
<!-- In component definition or page -->
<style scoped>
    .user-card {
        padding: 1rem;
        border: 1px solid #e5e7eb;
    }
</style>
```

### 10.3 CSS Variables (Theming)

Define theme in `/config/ui.cln`:

```clean
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
        <h1>{{post.title}}</h1>
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
    aria-expanded="{{isOpen}}"
    onclick="toggleMenu">
    <icon-x></icon-x>
</button>

<div role="alert" aria-live="polite" cl-if="notification">
    {{notification.message}}
</div>
```

---

## 12. Security

### 12.1 XSS Prevention

- `{{expression}}` is HTML-escaped by default
- Use `{{{expression}}}` only for trusted, sanitized HTML
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

Configure CSP in `/config/server.cln`:

```clean
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
/app
├── /components          # Clean component definitions
│   ├── Header.cln
│   ├── Footer.cln
│   ├── UserCard.cln
│   └── ...
├── /pages               # HTML pages
│   ├── index.html
│   ├── about.html
│   ├── /blog
│   │   ├── index.html
│   │   └── [slug].html
│   └── /dashboard
│       └── index.html
├── /layouts             # Layout templates
│   ├── main.html
│   └── admin.html
└── /partials            # Reusable HTML fragments
    ├── nav.html
    └── sidebar.html

/config
├── tags.cln             # Component registry (optional)
└── ui.cln               # Theme configuration

/public
├── /css
│   └── main.css
├── /js
│   └── app.js
└── /images
    └── logo.svg

/dist                    # Build output
├── server.wasm
├── ui.wasm
├── manifest.islands.json
└── /public
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
2. Calls `_start()` — the WASM module registers event handlers via `_ui_on_event`
3. Events are dispatched via document-level delegation using CSS selectors
4. Handlers make targeted DOM updates via bridge functions (`_ui_update_element`, `_ui_toggle_class`, etc.)

**Client-side code pattern:**

```clean
start:
    integer s = 0
    s = _ui_on_event(".btn-save", "click", 0)
    s = _ui_on_event(".search-input", "input", 1)

functions:
    handle_event_0()
        string value = _ui_event_value()
        integer s = _ui_update_element("#status", "Saved!")
        s = _ui_set_timeout(2, 2000)

    handle_event_1()
        string query = _ui_event_value()
        integer s = _ui_filter_by_text(".card", "data-name", "data-desc", query)

    handle_event_2()
        integer s = _ui_update_element("#status", "")
```

All bridge functions are declared in `plugin.toml` and implemented in `loader.js`. See the Bridge Contracts specification for the complete function reference.

---

## 15. Examples

### 15.1 Simple Page

```html
<!-- app/ui/pages/index.html -->
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

```html
<!-- app/ui/pages/blog/[slug].html -->
<page layout="main"></page>

<script type="text/clean">
    data:
        post = Post.find: where: slug=params.slug
        comments = post.comments: order: createdAt="desc"
</script>

<article class="blog-post">
    <header>
        <h1>{{post.title}}</h1>
        <p class="meta">
            By {{post.author.name}} on {{formatDate(post.createdAt)}}
        </p>
    </header>

    <div class="content">
        {{{post.content}}}
    </div>

    <section class="comments">
        <h2>Comments ({{comments.length}})</h2>

        <div cl-iterate="comment in comments" class="comment">
            <comment-card comment-id="{{comment.id}}"></comment-card>
        </div>

        <comment-form post-id="{{post.id}}" client="on"></comment-form>
    </section>
</article>
```

### 15.3 Interactive Component

```clean
// app/ui/components/CommentForm.cln
component: tag="comment-form" client="on"
    props:
        string postId

    state:
        string content = ""
        boolean submitting = false

    functions:
        string render()
            return "<form class=\"comment-form\">" +
                "<textarea bind=\"content\" placeholder=\"Write a comment...\"></textarea>" +
                "<button type=\"submit\" onclick=\"submitComment\" disabled=\"" + submitting.toString() + "\">" +
                    (if submitting then "Posting..." else "Post Comment") +
                "</button>" +
            "</form>"

        void submitComment()
            submitting = true
            Comment.create(postId: postId, content: content)
            content = ""
            submitting = false
```

---

## 16. Migration from v1

If migrating from Clean-first syntax:

| v1 (Clean DSL) | v2 (HTML-first) |
|----------------|-----------------|
| `page:` blocks | `.html` files |
| `component:` with layout | `.cln` with `render()` |
| Clean templating | HTML + `{{}}` |
| `render()` returns Widget | `render()` returns string |

---

**End of Document 05**
