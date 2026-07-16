# Clean Framework Project Structure

**This is the canonical reference for project folder layout and plugin folder ownership.** Other documents link here rather than duplicating the folder tree.

> ⚠️ **PARTIALLY ASPIRATIONAL.** The folder `app/entity/` and the endpoint code examples in this document (using `User.data.findActive()` and `Database.save(newUser)`) target the v2 data model paired with `frame.data` v4+. **The currently shipped `frame.data` v3.x** does not implement the v2 entity/data pairing or the `Database` service. If you create an `app/entity/` folder today, the plugin will not process it. The rest of the folder layout (`app/data/models/`, `app/logic/`, `app/server/`, `app/ui/`, `app/auth/`, `public/`) is accurate for the shipping plugin. For v3.x data patterns, use bare-field `data <Model>` blocks in `app/data/models/` and v1.2 block-form mutations (`Model.insert:`, `Model.update:`, `Model.delete:`).

## Standard Structure

```
my-app/
├── main.cln                     # Package declaration + target configuration + frame.data config
│
├── app/
│   ├── entity/                  # Domain classes (nouns)               → frame.data (paired w/ data/models/)
│   │
│   ├── data/                    # ORM / database layer              → frame.data
│   │   ├── models/              # Data block declarations (.cln) paired with entities by name
│   │   ├── reports/             # Cross-entity / aggregate query classes (created lazily)
│   │   ├── migrations/          # Schema migration files (.cln)
│   │   └── seeds/               # Seed data (.cln)
│   │
│   ├── logic/                   # Shared business logic                → (core compiler, no plugin)
│   │
│   ├── state/                   # Reactive client-side state, drives multiple render targets
│   │
│   ├── server/                  # HTTP API layer                    → frame.server
│   │   ├── api/                 # Endpoint handlers (endpoints: blocks)
│   │   └── middleware/          # Request filters (.cln)
│   │
│   └── ui/                     # UI plugin root                     → frame.ui
│       ├── shared/             # Cross-platform components (used by every render target)
│       └── web/                # Web rendering target
│           ├── pages/          # Auto-routed page files
│           │   ├── home.cln    # Home page (/) — Clean component
│           │   ├── about.html  # About page (/about) — HTML template
│           │   ├── about.cln   # Web adapter: URL binding + guard for about.html
│           │   └── blog/
│           │       ├── [slug].html  # Dynamic route (/blog/:slug)
│           │       └── [slug].cln   # Web adapter: URL binding + guard
│           ├── components/     # Web-specific reusable components
│           ├── layouts/        # Page layout wrappers (.html)
│           └── routes.cln      # Special routing rules only (see below)
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

    shared: [app/logic/, app/data/, app/state/]

    target: web
        plugins: [frame.ui, frame.server, frame.auth]
        entry: app/ui/web/pages/home.cln
```

The `shared:` block lists folders that compile for every target — web, mobile, desktop. You write the logic once and it just works everywhere. The `target:` blocks then add the platform-specific plugins on top.

The `public/` folder at the project root is served directly by the HTTP server without compilation.

## Naming Rule

**Folder name = plugin name after the dot.** `frame.server` → `server/`, `frame.data` → `data/`, `frame.ui` → `ui/`, `frame.auth` → `auth/`, `frame.canvas` → `canvas/`. No exceptions, no lookup table — the folder declares ownership.

Multi-target plugins nest render targets **under** their own folder rather than spreading at the project root: `frame.ui` owns `app/ui/`, with `app/ui/web/`, `app/ui/mobile/`, `app/ui/desktop/` as target subfolders and `app/ui/shared/` for code that works across targets. When a future plugin goes multi-target it follows the same pattern (e.g. `app/data/postgres/`, `app/data/sqlite/`).

The compiler reads each plugin's `[paths].owns` list from its `plugin.toml`; no folder name is hardcoded in the compiler.

## Plugin Folder Ownership

Plugins must be declared in `main.cln` via the `target:` block. Once declared, files in plugin-owned folders do not need individual `import` statements — the folder location tells the compiler which plugin processes each file.

| Path Pattern | Owning Plugin |
|---|---|
| `app/server/`, `app/server/api/`, `app/server/middleware/` | `frame.server` |
| `app/logic/` | — (core compiler, always compiled) |
| `app/entity/` | `frame.data` (paired with `data/models/` by class name) |
| `app/data/`, `app/data/models/`, `app/data/reports/`, `app/data/migrations/`, `app/data/seeds/` | `frame.data` |
| `app/ui/`, `app/ui/shared/` | `frame.ui` (shared, multi-platform) |
| `app/ui/web/pages/`, `app/ui/web/components/`, `app/ui/web/layouts/` | `frame.ui` (web target) |
| `app/ui/web/client/` and `app/ui/web/pages/*.client.cln` | `frame.client` |
| `app/auth/` | `frame.auth` |
| `app/canvas/`, `app/canvas/scenes/` | `frame.canvas` |

## Layer Order

Each layer depends only on layers above it:

| Folder | Plugin | Depends on |
|--------|--------|------------|
| `app/entity/` | frame.data | nothing (persistence-ignorant) |
| `app/data/` | frame.data | entity/ |
| `app/logic/` | — | entity/, data/ |
| `app/state/` | — | logic/, entity/, data/ |
| `app/server/` | frame.server | logic/, entity/, data/ |
| `app/ui/shared/` | frame.ui | nothing — only needed for multi-platform projects |
| `app/ui/web/` | frame.ui | logic/, state/, ui/shared/ |
| `app/canvas/` | frame.canvas | logic/, state/ |

## Folder Reference

### `app/server/api/`

HTTP endpoint handlers using the `endpoints:` block. Owned by `frame.server`.

**Example endpoint:**
```clean
// app/server/api/users.cln
endpoints:
	GET "/api/users":
		list<User> users = User.data.findActive()
		return json(users)

	POST "/api/users":
		User newUser = User(
			name: req.body("name"),
			email: req.body("email")
		)
		Database.save(newUser)
		return json(newUser)

	GET "/api/users/:id":
		integer id = req.params.id.toInteger()
		User user = User.data.findOrFailById(id)
		return json(user)
```

Endpoints stay thin — persistence goes through `.data` for reads and `Database` for writes. Reusable queries live in the paired data block's `queries:` sub-block (see below).

### `app/entity/`

Domain classes (nouns). Each file declares one `class` describing a business object with its fields, invariants, and methods. Paired by name with a `data:` block in `app/data/models/`. Persistence-ignorant.

| File | Class |
|------|-------|
| `user.cln` | `User` |
| `blog_post.cln` | `BlogPost` |

**Example entity:**
```clean
// app/entity/user.cln
class User
	integer? id
	string email
	private string passwordHash
	string status
	datetime createdAt

	always:
		status in ["active", "pending", "suspended"]

	functions:
		public:
			boolean canPost()
				return status == "active"
```

### `app/data/models/`

Data block declarations. Paired with an entity of the same name in `app/entity/`. Owned by `frame.data`.

| File | Data Block | Entity | Table |
|------|-----------|--------|-------|
| `user.cln` | `data User:` | `class User` | `users` |
| `blog_post.cln` | `data BlogPost:` | `class BlogPost` | `blog_posts` |

**Example data block:**
```clean
// app/data/models/user.cln
data User:
	table "users"

	fields:
		id primary generated
		email required unique
		passwordHash as "password_hash" required
		status required
		createdAt as "created_at" required

	queries:
		User? findByEmail(string emailAddress)
			return User.first:
				where:
					email == emailAddress

		User findOrFailById(integer userId)
			return User.findOrFail:
				where:
					id == userId
```

### `app/data/reports/`

Cross-entity or aggregate queries. Report classes have capability-noun names (`SalesByRegion`, `MonthlyActivity`). Created lazily — only when the first cross-entity query appears. Owned by `frame.data`.

### `app/logic/`

Think of `app/logic/` as the brain of your app — the actual work of fetching and transforming data. It has no idea whether it's serving a web page, a mobile screen, or a desktop window. That's the point. You write it once and every platform uses it.

**The rule is simple: if you find yourself writing the same query in two different places, move it here.**

**When should something live here?** Simple rule: if you find the same database query in more than one file, move it to `app/logic/`. If it's only used in one place, it's fine to keep it in the companion or endpoint for now.

The most common trigger is when both a page companion and an API endpoint need the same data. That duplication is the signal to extract it here.

Functions here never receive an HTTP `Request` and have no knowledge of web-specific concepts. The compiler always processes `app/logic/` regardless of which plugins are active — no plugin declaration is needed.

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

### `app/ui/web/pages/`

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
app/ui/web/pages/
├── about.html               # HTML template
└── about.cln                # Web adapter: URL binding + guard for about.html
```

**Example page:**
```html
<!-- app/ui/web/pages/blog/[slug].html -->
<page layout="main"></page>

<article>
    <h1>{post.title}</h1>
    <p>{post.content}</p>
</article>
```

**Example companion** (delegates to `app/logic/posts`):
```clean
// app/ui/web/pages/blog/[slug].cln
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

### `app/ui/web/routes.cln`

Special-case routing rules that cannot be expressed as page files. Use only when needed — regular pages go in `pages/` with no routing declaration required.

Handles: cross-cutting guards, URL redirects, pretty-URL rewrites, error pages.

```clean
// app/ui/web/routes.cln
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

**Two-tier access control:**

Think of it like a building. The front door checks if you're allowed in at all. Individual office doors check if you belong in that specific room.

- **Front door** (`routes.cln` guards) — "are you logged in?", "do you have the admin role?" — declared once, applies to all matching routes automatically.
- **Room door** (companion `guard()`) — "does this post actually belong to you?" — page-specific checks only.

```clean
// routes.cln — front door, handles all /dashboard/* pages at once
routes:
    guard: "/dashboard/*" [user]
    guard: "/admin/*" [admin]
```

```clean
// app/ui/web/pages/posts/[id].cln — room door, resource ownership only
import "app/logic/posts"

functions:
    any guard(Request request)
        Post post = posts.getById(request.params.id)
        if post.authorId != request.auth.userId
            return redirect("/403")
        return null

    any load(Request request)
        Post post = posts.getById(request.params.id)
        return { post: post }
```

### `app/state/`

Reactive client-side state. Each file drives the same view across multiple render targets — web, mobile, desktop — without duplicating logic.

```
app/state/dashboard.cln  ──→  app/ui/web/pages/dashboard.cln (web)
                          ──→  app/ui/desktop/screens/dashboard.cln (future)
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

### `app/ui/web/components/`

Web-specific reusable components. PascalCase filename becomes a kebab-case tag. Owned by `frame.ui`.

| File | Tag |
|------|-----|
| `Header.cln` | `<app-header>` |
| `Footer.cln` | `<app-footer>` |
| `UserCard.cln` | `<user-card>` |

Each component automatically links a CSS file from `public/css/components/` matching its tag name. The framework injects `<link rel="stylesheet" href="/css/components/{tag}.css">` into the page `<head>` when the component renders — deduplicated, so one link tag per tag name per page. Override with the `css=` attribute.

```
app/ui/web/components/
    UserCard.cln     ← component definition (no styles block)

public/css/components/
    user-card.css    ← linked automatically for tag="user-card"
```

**Example component:**
```clean
// app/ui/web/components/UserCard.cln
component: tag="user-card"
	inputs:
		string userId

	html:
		<div class="user-card">
			<h3>{inputs.userId}</h3>
		</div>
```

**CSS file (public/css/components/user-card.css):**
```css
.user-card {
	padding: 1rem;
	border: 1px solid #e5e7eb;
	border-radius: 6px;
}
```

**Custom CSS path (optional):**
```clean
component: tag="user-card" css="/css/shared/cards.css"
```

**Going multi-platform?** When you add a second target (mobile, desktop), put components that work on every target in `app/ui/shared/`. For web-only projects, keep everything in `app/ui/web/components/` — adding `app/ui/shared/` before you need it just creates an extra folder with nothing in it.

### `app/ui/web/layouts/`

Page layout wrappers referenced by pages via `<page layout="name">`. Owned by `frame.ui`.

**Example layout:**
```html
<!-- app/ui/web/layouts/main.html -->
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
