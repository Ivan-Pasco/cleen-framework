# Clean Framework Project Structure

**This is the canonical reference for project folder layout and plugin folder ownership.** Other documents link here rather than duplicating the folder tree.

## Standard Structure

```
my-project/
├── app.cln                      # Project metadata + plugin configuration
├── app/
│   ├── server/                  # HTTP server layer              → frame.server
│   │   ├── api/                 # HTTP endpoint handlers (.cln)
│   │   ├── services/            # Business logic (.cln)
│   │   └── middleware/          # Request filters (.cln)
│   ├── data/                    # ORM / database layer           → frame.data
│   │   ├── models/              # Data model definitions (.cln)
│   │   ├── queries/             # Reusable named queries (.cln)
│   │   ├── migrations/          # Schema migration files (.cln)
│   │   └── repositories/        # Data access layer (.cln)
│   ├── ui/                      # UI layer                       → frame.ui
│   │   ├── pages/               # SSR page routes (.html)
│   │   │   ├── index.html       # Home page (/)
│   │   │   ├── index.cln        # Companion data loader
│   │   │   └── blog/
│   │   │       ├── [slug].html  # Dynamic route (/blog/:slug)
│   │   │       └── [slug].cln   # Companion loader
│   │   ├── components/          # Reusable UI components (.cln)
│   │   └── layouts/             # Page layout wrappers (.html)
│   ├── auth/                    # Auth configuration (.cln)      → frame.auth
│   ├── canvas/                  # Canvas applications            → frame.canvas
│   │   ├── scenes/              # Scene definitions (.cln)
│   │   ├── sprites/             # Sprite definitions (.cln)
│   │   └── audio/               # Audio config (.cln)
│   └── client/                  # Client-side communication      → frame.client
├── public/                      # Static assets (served as-is)
│   ├── css/
│   └── images/
└── dist/                        # Build output (generated)
```

## Naming Rule

**Folder name = plugin name after the dot.** `frame.server` → `server/`, `frame.data` → `data/`, `frame.ui` → `ui/`, `frame.auth` → `auth/`, `frame.canvas` → `canvas/`, `frame.client` → `client/`. No lookup table needed — the folder declares ownership.

## Plugin Folder Ownership

Plugins must be declared in `app.cln` via the `plugins:` block. Once declared, files in plugin-owned folders do not need individual `import` statements — the folder location tells the compiler which plugin processes each file.

| Path Pattern | Owning Plugin |
|---|---|
| `app/server/`, `app/server/api/`, `app/server/services/`, `app/server/middleware/` | `frame.server` |
| `app/data/`, `app/data/models/`, `app/data/queries/`, `app/data/migrations/`, `app/data/repositories/` | `frame.data` |
| `app/ui/`, `app/ui/pages/`, `app/ui/components/`, `app/ui/layouts/` | `frame.ui` |
| `app/auth/` | `frame.auth` |
| `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` | `frame.canvas` |
| `app/client/` | `frame.client` |

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

### `app/ui/pages/`

HTML page routes. Each `.html` file becomes a GET route returning server-rendered HTML. Owned by `frame.ui`.

| File | Route |
|------|-------|
| `index.html` | `/` |
| `about.html` | `/about` |
| `blog/index.html` | `/blog` |
| `blog/[slug].html` | `/blog/:slug` |
| `users/[id]/profile.html` | `/users/:id/profile` |

**Companion files**: Each page can have a matching `.cln` file (same name) that provides server-side data via `load()` and access control via `guard()`.

```
app/ui/pages/
├── profile.html         # Template with {expression} interpolation
└── profile.cln          # Companion: load() provides data, guard() checks access
```

**Example page:**
```html
<!-- app/ui/pages/blog/[slug].html -->
<page layout="main"></page>

<article>
    <h1>{post.title}</h1>
    <p>{post.content}</p>
</article>
```

**Example companion:**
```clean
// app/ui/pages/blog/[slug].cln
functions:
	any load(Request request)
		string slug = request.params.slug
		Post post = Post.first:
			where:
				slug == slug
		return { post: post }
```

**Interpolation syntax:**
- `{expression}` — HTML-escaped output (safe, default)
- `{!expression}` — Raw HTML output (trusted content only)

### `app/ui/components/`

Reusable UI components. PascalCase filename becomes a kebab-case tag. Owned by `frame.ui`.

| File | Tag |
|------|-----|
| `Header.cln` | `<app-header>` |
| `Footer.cln` | `<app-footer>` |
| `UserCard.cln` | `<user-card>` |

**Example component:**
```clean
// app/ui/components/UserCard.cln
component: tag="user-card"
	props:
		string userId

	html:
		<div class="user-card">
			<h3>{this.userId}</h3>
		</div>
```

### `app/ui/layouts/`

Page layout wrappers referenced by pages via `<page layout="name">`. Owned by `frame.ui`.

**Example layout:**
```html
<!-- app/ui/layouts/main.html -->
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

### `app/auth/`

Authentication and authorization configuration. Owned by `frame.auth`.

**Example auth config:**
```clean
// app/auth/config.cln
auth:
	session:
		cookie = "frame.sid"
		sameSite = "Lax"
		secure = true
		httpOnly = true
		timeoutMinutes = 60

	jwt:
		secret = env("JWT_SECRET")
		alg = "HS256"
		ttlMinutes = 60

	roles:
		admin: ["*"]
		editor: ["post.create", "post.edit"]
		viewer: ["post.read"]
```

### `app/canvas/`

Canvas application files. Owned by `frame.canvas`.

| Sub-folder | Purpose |
|---|---|
| `scenes/` | Scene definitions — each `.cln` file declares a `canvasScene:` block |
| `sprites/` | Sprite sheet configurations |
| `audio/` | Audio asset configurations |

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

	onKeyDown: params="key"
		// handle key press
```

### `app/client/`

Client-side communication — API calls, realtime subscriptions, live feeds. Owned by `frame.client`.

### `public/`

Static assets served directly without processing. Lives at the project root, not under `app/` — it is not compiled.

Access via URL path: `/css/style.css`, `/images/logo.png`

## app.cln Configuration

The top-level `app.cln` file is the single project manifest. It declares project metadata and plugins. No separate `project.toml` is needed.

```clean
// app.cln
project:
	name = "my-project"
	version = "1.0.0"

plugins:
	frame.ui
	frame.server
	frame.data
	frame.auth
```

## Commands

```bash
# Create project
cleen project create my-app --plugins=frame.server,frame.data,frame.ui,frame.auth

# Compile
cln compile app.cln -o dist/app.wasm --plugins

# Run server
cleen server run dist/app.wasm --port 3000

# Database migrations
cleen db:migrate
cleen db:plan

# Plugin management
cleen plugin add frame.canvas
cleen plugin list
```
