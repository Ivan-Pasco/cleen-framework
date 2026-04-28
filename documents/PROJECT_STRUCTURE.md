# Clean Framework Project Structure

**This is the canonical reference for project folder layout and plugin folder ownership.** Other documents link here rather than duplicating the folder tree.

## Standard Structure

```
my-project/
├── app.cln                      # Main entry point / plugin configuration
├── project.toml                 # Project metadata and build settings
├── app/
│   ├── pages/                   # SSR page routes (.html)        → frame.ui
│   │   ├── index.html           # Home page (/)
│   │   ├── index.cln            # Companion data loader
│   │   └── blog/
│   │       ├── [slug].html      # Dynamic route (/blog/:slug)
│   │       └── [slug].cln       # Companion loader
│   ├── components/              # Reusable UI components (.cln)  → frame.ui
│   ├── layouts/                 # Page layout wrappers (.html)   → frame.ui
│   ├── backend/                 # HTTP server layer              → frame.server
│   │   ├── api/                 # HTTP endpoint handlers (.cln)
│   │   ├── services/            # Business logic (.cln)
│   │   └── middleware/          # Request filters (.cln)
│   ├── data/                    # ORM / database layer           → frame.data
│   │   ├── models/              # Data model definitions (.cln)
│   │   ├── queries/             # Reusable named queries (.cln)
│   │   ├── migrations/          # Schema migration files (.cln)
│   │   └── repositories/        # Data access layer (.cln)
│   ├── auth/                    # Auth configuration (.cln)      → frame.auth
│   ├── canvas/                  # Canvas applications            → frame.canvas
│   │   ├── scenes/              # Scene definitions (.cln)
│   │   ├── sprites/             # Sprite definitions (.cln)
│   │   └── audio/               # Audio config (.cln)
│   └── public/                  # Static assets (served as-is)
│       ├── css/
│       └── images/
└── dist/                        # Build output (generated)
```

## Plugin Folder Ownership

Plugins must be declared in `app.cln` via the `plugins:` block. Once declared, files in plugin-owned folders do not need individual `import` statements — the folder location tells the compiler which plugin processes each file.

| Path Pattern | Owning Plugin |
|---|---|
| `app/pages/`, `app/components/`, `app/layouts/` | `frame.ui` |
| `app/backend/`, `app/backend/api/`, `app/backend/services/`, `app/backend/middleware/` | `frame.server` |
| `app/data/`, `app/data/models/`, `app/data/queries/`, `app/data/migrations/`, `app/data/repositories/` | `frame.data` |
| `app/auth/` | `frame.auth` |
| `app/canvas/`, `app/canvas/scenes/`, `app/canvas/sprites/`, `app/canvas/audio/` | `frame.canvas` |

## Folder Reference

### `app/pages/`

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
app/pages/
├── profile.html         # Template with {expression} interpolation
└── profile.cln          # Companion: load() provides data, guard() checks access
```

**Example page:**
```html
<!-- app/pages/blog/[slug].html -->
<page layout="main"></page>

<article>
    <h1>{post.title}</h1>
    <p>{post.content}</p>
</article>
```

**Example companion:**
```clean
// app/pages/blog/[slug].cln
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

### `app/components/`

Reusable UI components. PascalCase filename becomes a kebab-case tag. Owned by `frame.ui`.

| File | Tag |
|------|-----|
| `Header.cln` | `<app-header>` |
| `Footer.cln` | `<app-footer>` |
| `UserCard.cln` | `<user-card>` |

**Example component:**
```clean
// app/components/UserCard.cln
component: tag="user-card"
	props:
		string userId

	html:
		<div class="user-card">
			<h3>{this.userId}</h3>
		</div>
```

### `app/layouts/`

Page layout wrappers referenced by pages via `<page layout="name">`. Owned by `frame.ui`.

**Example layout:**
```html
<!-- app/layouts/main.html -->
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

### `app/backend/api/`

HTTP endpoint handlers using the `endpoints:` block. Owned by `frame.server`.

**Example endpoint:**
```clean
// app/backend/api/users.cln
endpoints:
	GET "/api/users" :
		handle:
			list<User> users = User.find:
				where:
					active == true
			return json(users)

	POST "/api/users" :
		body:
			name : string
			email : string
		handle:
			User u = User.insert:
				name = name
				email = email
			return json(u)

	GET "/api/users/:id" :
		params:
			id : integer
		handle:
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

### `app/auth/`

Authentication and authorization configuration. Owned by `frame.auth`.

**Example auth config:**
```clean
// app/auth/auth.cln
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

### `app/public/`

Static assets served directly without processing.

Access via URL path: `/css/style.css`, `/images/logo.png`

## app.cln Configuration

The top-level `app.cln` file declares plugins. This is required.

```clean
// app.cln
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
