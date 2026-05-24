# Clean Framework Project Structure

**This is the canonical reference for project folder layout and plugin folder ownership.** Other documents link here rather than duplicating the folder tree.

## Standard Structure

```
my-app/
├── main.cln                     # Package declaration + target configuration
│
├── app/
│   ├── data/                    # ORM / database layer              → frame.data
│   │   ├── models/              # Data model definitions (.cln)
│   │   ├── migrations/          # Schema migration files (.cln)
│   │   └── seeds/               # Seed data (.cln)
│   │
│   ├── logic/                   # Shared data-fetching & business logic, reusable across all targets
│   │
│   ├── state/                   # Reactive client-side state, drives multiple render targets
│   │
│   ├── server/                  # HTTP API layer                    → frame.server
│   │   ├── api/                 # Endpoint handlers (endpoints: blocks)
│   │   └── middleware/          # Request filters (.cln)
│   │
│   ├── ui/                      # Abstract component library        → frame.ui
│   │   └── (cross-platform components — any target)
│   │
│   └── web/                     # Web rendering layer               → frame.ui (web)
│       ├── pages/               # Auto-routed page files
│       │   ├── home.cln         # Home page (/) — Clean component
│       │   ├── about.html       # About page (/about) — HTML template
│       │   ├── about.cln        # Web adapter: URL binding + guard for about.html
│       │   └── blog/
│       │       ├── [slug].html  # Dynamic route (/blog/:slug)
│       │       └── [slug].cln   # Web adapter: URL binding + guard
│       ├── components/          # Web-specific reusable components
│       ├── layouts/             # Page layout wrappers (.html)
│       └── routes.cln           # Special routing rules only (see below)
│
└── public/
    ├── css/
    ├── images/
    └── fonts/
```

## main.cln — Package Entry Point

The `main.cln` file is the project manifest. It declares metadata and which plugins each build target uses. No separate `package.clean.toml` is needed.

```clean
package: MyApp
    version: "1.0.0"

    target: web
        plugins: [frame.ui, frame.server, frame.data, frame.auth]
        entry: app/web/pages/home.cln
```

Multiple targets share the same `app/data/`, `app/logic/`, and `app/state/` folders — logic is written once and runs on every target. The `public/` folder at the project root is served directly by the HTTP server without compilation.

## Naming Rule

**Folder name = plugin name after the dot.** `frame.server` → `server/`, `frame.data` → `data/`, `frame.ui` → `ui/` and `web/`, `frame.auth` → `auth/`, `frame.canvas` → `canvas/`. No lookup table needed — the folder declares ownership.

## Plugin Folder Ownership

Plugins must be declared in `main.cln` via the `target:` block. Once declared, files in plugin-owned folders do not need individual `import` statements — the folder location tells the compiler which plugin processes each file.

| Path Pattern | Owning Plugin |
|---|---|
| `app/server/`, `app/server/api/`, `app/server/middleware/` | `frame.server` |
| `app/data/`, `app/data/models/`, `app/data/migrations/`, `app/data/seeds/` | `frame.data` |
| `app/web/pages/`, `app/web/components/`, `app/web/layouts/` | `frame.ui` |
| `app/ui/` | `frame.ui` |
| `app/auth/` | `frame.auth` |
| `app/canvas/`, `app/canvas/scenes/` | `frame.canvas` |

## Layer Order

Each layer depends only on layers above it:

| Folder | Plugin | Depends on |
|--------|--------|------------|
| `app/data/` | frame.data | nothing |
| `app/logic/` | — | data/ |
| `app/state/` | — | logic/, data/ |
| `app/server/` | frame.server | logic/, data/ |
| `app/ui/` | frame.ui | nothing |
| `app/web/` | frame.ui | logic/, state/, ui/ |
| `app/canvas/` | frame.canvas | logic/, state/ |

## Folder Reference

### `app/server/api/`

HTTP endpoint handlers using the `endpoints:` block. Owned by `frame.server`.

**Example endpoint:**
```clean
// app/server/api/users.cln
endpoints:
	GET "/api/users":
		list<User> users = User.find:
			where:
				active == true
		return json(users)

	POST "/api/users":
		User u = User.insert:
			name = req.body("name")
			email = req.body("email")
		return json(u)

	GET "/api/users/:id":
		integer id = req.params.id.toInteger()
		User user = User.first:
			where:
				id == id
		return json(user)
```

### `app/data/models/`

Data model definitions. PascalCase filename becomes a snake_case database table. Owned by `frame.data`.

| File | Model | Table |
|------|-------|-------|
| `User.cln` | `User` | `users` |
| `BlogPost.cln` | `BlogPost` | `blog_posts` |

**Example model:**
```clean
// app/data/models/User.cln
data User
	integer id : pk, auto
	string name
	string email : unique
	string passwordHash
	boolean active = true
	datetime createdAt : default=now
```

### `app/logic/`

Shared data-fetching and business logic. Functions here are reusable across all targets (web, mobile, desktop) — they never receive an HTTP `Request` and have no knowledge of web-specific concepts.

**Example:**
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

Page companions and API endpoints call into `app/logic/` rather than querying the database directly, so the same logic can serve a web page, a mobile screen, and a REST endpoint without duplication.

### `app/web/pages/`

Page files auto-routed by file path. Each file becomes a GET route. Owned by `frame.ui`. Accepts both `.html` templates and `.cln` components.

| File | Route |
|------|-------|
| `home.cln` | `/` |
| `about.html` | `/about` |
| `blog/index.html` | `/blog` |
| `blog/[slug].html` | `/blog/:slug` |
| `users/[id]/profile.html` | `/users/:id/profile` |

**Companion files**: Each `.html` page can have a matching `.cln` file (same name). The companion is a thin **web adapter** — it parses URL params, enforces guards, then delegates data-fetching to `app/logic/`. It never queries the database directly.

```
app/web/pages/
├── about.html               # HTML template
└── about.cln                # Web adapter: URL binding + guard for about.html
```

**Example page:**
```html
<!-- app/web/pages/blog/[slug].html -->
<page layout="main"></page>

<article>
    <h1>{post.title}</h1>
    <p>{post.content}</p>
</article>
```

**Example companion** (delegates to `app/logic/posts`):
```clean
// app/web/pages/blog/[slug].cln
import "app/logic/posts"

functions:
	any load(Request request)
		string slug = request.params.slug
		Post post = posts.getBySlug(slug)
		return { post: post }
```

**Interpolation syntax:**
- `{expression}` — HTML-escaped output (safe, default)
- `{!expression}` — Raw HTML output (trusted content only)

### `app/web/routes.cln`

Special-case routing rules that cannot be expressed as page files. Use only when needed — regular pages go in `pages/` with no routing declaration required.

Handles: cross-cutting guards, URL redirects, pretty-URL rewrites, error pages.

```clean
// app/web/routes.cln
routes:
	redirect: "/old-about" → "/about"
	redirect: "/blog/:slug" → "/posts/:slug"

	guard: "/admin/*" [admin]
	guard: "/dashboard/*" [user]

	error: 404 → "pages/not-found.cln"
	error: 500 → "pages/error.cln"

	rewrite: "/:username" → "pages/profile/[id].cln"
		resolve: username → User.first({ handle: username })
```

**Rule:** if a page maps cleanly to a URL, use `pages/`. Only use `routes.cln` for behavior that cannot be expressed as a file name.

### `app/state/`

Reactive client-side state. Each file drives the same view across multiple render targets — web, mobile, desktop — without duplicating logic.

```
app/state/dashboard.cln  ──→  app/web/pages/dashboard.cln (web)
                          ──→  app/desktop/screens/dashboard.cln (future)
```

State uses the `state:` block:

```clean
// app/state/dashboard.cln
state:
	string userName = ""
	boolean loading = true

	computed:
		string displayName
			return userName ?? "Guest"
```

### `app/web/components/`

Web-specific reusable components. PascalCase filename becomes a kebab-case tag. Owned by `frame.ui`.

| File | Tag |
|------|-----|
| `Header.cln` | `<app-header>` |
| `Footer.cln` | `<app-footer>` |
| `UserCard.cln` | `<user-card>` |

**Example component:**
```clean
// app/web/components/UserCard.cln
component: tag="user-card"
	props:
		string userId

	html:
		<div class="user-card">
			<h3>{this.userId}</h3>
		</div>
```

### `app/web/layouts/`

Page layout wrappers referenced by pages via `<page layout="name">`. Owned by `frame.ui`.

**Example layout:**
```html
<!-- app/web/layouts/main.html -->
<html>
<head>
    <title>My Site</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <app-header></app-header>
    <main><slot></slot></main>
    <app-footer></app-footer>
</body>
</html>
```

### `app/canvas/`

Canvas application files. Owned by `frame.canvas`.

| Sub-folder | Purpose |
|---|---|
| `scenes/` | Scene definitions — each `.cln` file declares a `canvasScene:` block |
| `components/` | Canvas-specific reusable components |

**Example scene:**
```clean
// app/canvas/scenes/main.cln
canvasScene: width=800 height=600
	draw:
		canvas.clear "black"
		canvas.text "Hello Canvas" 100 100 24 "white"

	onFrame: param="dt"
		// per-frame update logic

	onPointerDown: params="x,y"
		// handle click at (x, y)
```

### `public/`

Static assets served directly without processing. Lives at the project root, outside `app/` — not compiled or processed by any plugin.

| Sub-folder | Purpose |
|---|---|
| `css/` | Stylesheets, linked via `<link>` tags |
| `images/` | Images, served at `/images/...` |
| `fonts/` | Web fonts, served at `/fonts/...` |

Access via URL path: `/css/style.css`, `/images/logo.png`, `/fonts/inter.woff2`

## Commands

```bash
# Create project
cleen project create my-app --plugins=frame.server,frame.data,frame.ui,frame.auth

# Compile
cln compile main.cln -o dist/app.wasm --plugins

# Run server
cleen server run dist/app.wasm --port 3000

# Database migrations
cleen db:migrate
cleen db:plan

# Plugin management
cleen plugin add frame.canvas
cleen plugin list
```
