# Getting Started with Clean Framework

Build your first web application in 5 minutes.

## Prerequisites

Install Clean Language and the server runtime:

```bash
# Install cleen (version manager)
curl -fsSL https://cleen.dev/install.sh | bash

# Install compiler and all framework plugins
cleen install latest
cleen frame install latest
```

## Create a New Project

```bash
cleen project create my-app --plugins=frame.server,frame.data,frame.ui,frame.auth
cd my-app
```

This creates a project with automatic file discovery:

```
my-app/
├── main.cln                  # Project metadata + plugin configuration
├── app/
│   ├── server/api/          # HTTP endpoints (.cln)      → frame.server
│   ├── data/models/         # Data model definitions     → frame.data
│   ├── ui/pages/            # SSR pages (.html)          → frame.ui
│   ├── ui/components/       # Reusable UI components     → frame.ui
│   └── auth/                # Auth configuration         → frame.auth
└── public/css/              # Static styles
```

For the full folder reference, see [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md).

## Your First Page

Edit `app/web/pages/index.html`:

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
// app/web/pages/index.cln
functions:
	any load(Request request)
		return { appName: "My App" }
```

The companion file provides data that the HTML template can access via `{expression}` interpolation.

## Build and Run

```bash
# Compile with plugins
cln compile main.cln -o dist/app.wasm --plugins

# Run the server
cleen server run dist/app.wasm --port 3000
```

Open http://localhost:3000 in your browser.

## Add More Pages

Create `app/web/pages/about.html`:

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

Create `app/web/pages/blog/[slug].html` for dynamic URLs:

```html
<html>
<body>
    <h1>Blog Post: {post.title}</h1>
    <p>{post.content}</p>
</body>
</html>
```

Put the query in `app/logic/posts.cln` so it can be reused by other pages and API endpoints:

```clean
// app/logic/posts.cln
functions:
	Post getBySlug(string slug)
		return Post.first:
			where:
				slug == slug
```

Then the companion `app/web/pages/blog/[slug].cln` is a thin web adapter that just binds the URL param and calls the logic:

```clean
// app/web/pages/blog/[slug].cln
import "app/logic/posts"

functions:
	any load(Request request)
		string slug = request.params.slug
		return { post: posts.getBySlug(slug) }
```

This handles URLs like `/blog/hello-world`, `/blog/my-first-post`, etc.

## Add an API Endpoint

Create `app/server/api/users.cln`:

```clean
endpoints:
	GET "/api/users":
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
| `cln compile main.cln -o app.wasm --plugins` | Build the application |
| `cleen server run dist/app.wasm` | Run the application |

| Folder | Purpose | Plugin |
|--------|---------|--------|
| `app/server/api/` | HTTP endpoints (.cln) | frame.server |
| `app/logic/` | Business logic (.cln) | frame.server |
| `app/data/models/` | Data model definitions (.cln) | frame.data |
| `app/data/migrations/` | Schema migrations (.cln) | frame.data |
| `app/web/pages/` | SSR pages (.html) + companion loaders (.cln) | frame.ui |
| `app/web/components/` | Reusable components (.cln) | frame.ui |
| `app/web/layouts/` | Page layouts (.html) | frame.ui |
| `app/auth/` | Auth configuration (.cln) | frame.auth |
| `public/` | Static files (CSS, images) | (served as-is) |

| File Pattern | Route |
|--------------|-------|
| `app/web/pages/index.html` | `/` |
| `app/web/pages/about.html` | `/about` |
| `app/web/pages/blog/[slug].html` | `/blog/:slug` |
| `app/server/api/users.cln` | `/api/users` |
