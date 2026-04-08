# Getting Started with Clean Framework

Build your first web application in 5 minutes.

## Prerequisites

Install Clean Language and the server runtime:

```bash
# Install cleen (version manager)
curl -fsSL https://cleen.dev/install.sh | bash

# Install compiler and server
cleen install latest
cleen server install
```

## Create a New Project

```bash
cleen project create my-app --plugins=frame.server,frame.data,frame.ui,frame.auth
cd my-app
```

This creates a project with automatic file discovery:

```
my-app/
├── app.cln                  # Main entry point (plugins: block)
├── project.toml             # Project configuration
└── app/
    ├── pages/               # SSR pages (.html)          → frame.ui
    │   └── index.html       # Home page (/)
    ├── components/          # Reusable UI components (.cln) → frame.ui
    ├── layouts/             # Page layout wrappers (.html) → frame.ui
    ├── backend/             # HTTP server layer           → frame.server
    │   ├── api/             # HTTP endpoints (.cln)
    │   ├── services/        # Business logic (.cln)
    │   └── middleware/      # Request filters (.cln)
    ├── data/                # ORM layer                   → frame.data
    │   ├── models/          # Data model definitions (.cln)
    │   ├── queries/         # Reusable queries (.cln)
    │   ├── migrations/      # Schema migrations (.cln)
    │   └── repositories/    # Data access layer (.cln)
    ├── auth/                # Auth configuration (.cln)   → frame.auth
    └── public/              # Static files (CSS, images)
        └── css/
```

## Your First Page

Edit `app/pages/index.html`:

```html
<html>
<head>
    <title>My App</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <h1>Welcome to {appName}</h1>
    <p>Build fast, type-safe web applications.</p>
</body>
</html>
```

### Data for Pages (Companion File)

Pages get their data from a companion `.cln` file with the same name:

```clean
// app/pages/index.cln
functions:
	any load(Request request)
		return { appName: "My App" }
```

The companion file provides data that the HTML template can access via `{expression}` interpolation.

## Build and Run

```bash
# Compile with plugins
cln compile app.cln -o dist/app.wasm --plugins

# Run the server
cleen server run dist/app.wasm --port 3000
```

Open http://localhost:3000 in your browser.

## Add More Pages

Create `app/pages/about.html`:

```html
<html>
<body>
    <h1>About Us</h1>
    <a href="/">Back to Home</a>
</body>
</html>
```

Rebuild and the `/about` route is automatically available.

## Dynamic Routes

Create `app/pages/blog/[slug].html` for dynamic URLs:

```html
<html>
<body>
    <h1>Blog Post: {post.title}</h1>
    <p>{post.content}</p>
</body>
</html>
```

With companion loader `app/pages/blog/[slug].cln`:

```clean
functions:
	any load(Request request)
		string slug = request.params.slug
		Post post = Post.first:
			where:
				slug == slug
		return { post: post }
```

This handles URLs like `/blog/hello-world`, `/blog/my-first-post`, etc.

## Add an API Endpoint

Create `app/backend/api/users.cln`:

```clean
endpoints:
	GET "/api/users" :
		handle:
			list users = User.find:
				where:
					active == true
			return json(users)
```

Access it at `/api/users`.

## Add a Database Model

Create `app/data/models/User.cln`:

```clean
data User
	integer id : pk, auto
	string name
	string email : unique
	boolean active = true
```

Models are auto-discovered and available in your pages and API endpoints.

## Next Steps

- Read [Project Structure](PROJECT_STRUCTURE.md) for the complete folder reference
- See the [API Reference](API_REFERENCE.md) for a quick cheat sheet
- Check the [UI Specification](specification/05_frame_ui.md) for components and layouts
- Read the [Plugin Guide](PLUGIN_GUIDE.md) for plugin system details

## Quick Reference

| Command | Description |
|---------|-------------|
| `cleen project create <name>` | Create new project |
| `cln compile app.cln -o app.wasm --plugins` | Build the application |
| `cleen server run dist/app.wasm` | Run the application |

| Folder | Purpose | Plugin |
|--------|---------|--------|
| `app/pages/` | SSR pages (.html) + companion loaders (.cln) | frame.ui |
| `app/components/` | Reusable components (.cln) | frame.ui |
| `app/layouts/` | Page layouts (.html) | frame.ui |
| `app/backend/api/` | HTTP endpoints (.cln) | frame.server |
| `app/backend/services/` | Business logic (.cln) | frame.server |
| `app/data/models/` | Data model definitions (.cln) | frame.data |
| `app/data/migrations/` | Schema migrations (.cln) | frame.data |
| `app/auth/` | Auth configuration (.cln) | frame.auth |
| `app/public/` | Static files (CSS, images) | (served as-is) |

| File Pattern | Route |
|--------------|-------|
| `pages/index.html` | `/` |
| `pages/about.html` | `/about` |
| `pages/blog/[slug].html` | `/blog/:slug` |
| `backend/api/users.cln` | `/api/users` |
