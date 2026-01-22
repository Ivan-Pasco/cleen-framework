# Clean Framework Project Structure

Complete reference for the automatic file discovery system.

## Standard Structure

```
my-project/
  app/
    ui/                       # Frontend (browser-facing)
      pages/                  # HTML page routes
      components/             # Custom HTML elements
      layouts/                # Page wrappers
      public/                 # Static assets
    server/                   # Backend
      api/                    # JSON API routes
      models/                 # Database schemas
      middleware/             # Request filters
    shared/                   # Shared code
      lib/                    # Utility modules
  config.cln                  # Project configuration
  dist/                       # Build output
```

## Folder Reference

### `app/ui/pages/`

HTML page routes. Files become GET routes returning HTML.

| File | Route |
|------|-------|
| `index.html` | `/` |
| `about.html` | `/about` |
| `contact.html` | `/contact` |
| `blog/index.html` | `/blog` |
| `blog/[slug].html` | `/blog/:slug` |
| `users/[id]/profile.html` | `/users/:id/profile` |

**Naming Rules:**
- Files must end with `.html`
- `index.html` maps to the folder path
- `[param].html` creates a dynamic route parameter

**Example page:**
```cln
// app/ui/pages/blog/[slug].html

// Parameter 'slug' is auto-extracted from URL
string html = "<html>
<body>
    <h1>Post: " + slug + "</h1>
</body>
</html>"

return html
```

### `app/ui/components/`

Custom HTML elements. PascalCase filename becomes kebab-case tag.

| File | Tag |
|------|-----|
| `Header.cln` | `<app-header>` |
| `Footer.cln` | `<app-footer>` |
| `UserCard.cln` | `<user-card>` |
| `BlogPostPreview.cln` | `<blog-post-preview>` |
| `cards/ProductCard.cln` | `<product-card>` |

**Naming Rules:**
- Files must end with `.cln`
- Use PascalCase for filenames
- Single-word components get `app-` prefix
- Multi-word components keep their words

**Example component:**
```cln
// app/ui/components/Header.cln

component:
    string render()
        return "<header>
            <nav>
                <a href='/'>Home</a>
                <a href='/about'>About</a>
            </nav>
        </header>"
```

### `app/ui/layouts/`

Page wrappers. Used with the `layout` attribute on pages.

| File | Layout Name |
|------|-------------|
| `main.html` | `main` |
| `admin.html` | `admin` |
| `blog.html` | `blog` |

**Example layout:**
```cln
// app/ui/layouts/main.html

return "<html>
<head>
    <title>My Site</title>
    <link rel='stylesheet' href='/public/css/style.css'>
</head>
<body>
    <app-header></app-header>
    <main>{slot}</main>
    <app-footer></app-footer>
</body>
</html>"
```

### `app/ui/public/`

Static files served as-is. Copied to `dist/public/` during build.

```
public/
  css/
    style.css
    reset.css
  js/
    app.js
  images/
    logo.png
  favicon.ico
  robots.txt
```

Access via `/public/...` URLs:
- `/public/css/style.css`
- `/public/images/logo.png`

### `app/server/api/`

JSON API endpoints. Files become routes with `/api/` prefix.

| File | Route |
|------|-------|
| `users.cln` | `/api/users` |
| `users/[id].cln` | `/api/users/:id` |
| `articles/index.cln` | `/api/articles` |
| `articles/[id]/comments.cln` | `/api/articles/:id/comments` |

**Example API:**
```cln
// app/server/api/users/[id].cln

// Return JSON
return "{\"id\": " + id + ", \"name\": \"User " + id + "\"}"
```

### `app/server/models/`

Database schemas. PascalCase name becomes snake_case table.

| File | Model | Table |
|------|-------|-------|
| `User.cln` | `User` | `users` |
| `Article.cln` | `Article` | `articles` |
| `BlogPost.cln` | `BlogPost` | `blog_posts` |

**Example model:**
```cln
// app/server/models/User.cln

data User
    integer id
    string name
    string email
    string password_hash
    string created_at
```

### `app/server/middleware/`

Request filters applied to routes.

| File | Middleware Name |
|------|-----------------|
| `auth.cln` | `auth` |
| `logging.cln` | `logging` |
| `cors.cln` | `cors` |

**Example middleware:**
```cln
// app/server/middleware/auth.cln

// Check authentication
string token = _req_header("Authorization")
if token == ""
    return _http_error(401, "Unauthorized")
end
```

### `app/shared/lib/`

Utility modules available everywhere.

| File | Module |
|------|--------|
| `validation.cln` | `validation` |
| `format.cln` | `format` |
| `helpers.cln` | `helpers` |

## Route Mapping

### URL Parameters

Use `[param]` in filenames:

```
pages/users/[id].html         → /users/:id
pages/blog/[slug]/edit.html   → /blog/:slug/edit
api/articles/[id]/comments.cln    → /api/articles/:id/comments
```

Parameters are automatically available as variables:

```cln
// pages/users/[id].html
// 'id' variable is automatically set from URL

string html = "<h1>User " + id + "</h1>"
return html
```

### Index Files

`index` files map to folder paths:

```
pages/index.html              → /
pages/blog/index.html         → /blog
api/users/index.cln               → /api/users
```

## Build Output

After `frame build`:

```
dist/
  app.wasm                # Compiled application
  .generated/
    main.cln              # Generated entry point
    components.json       # Component registry
  public/                 # Copied from app/ui/public/
    css/
    js/
    images/
```

## Configuration (config.cln)

```cln
// config.cln

config:
	project:
		name = "my-app"
		version = "1.0.0"

	build:
		output = "dist"

	server:
		port = 3000
		host = "127.0.0.1"

	auth:
		strategy = "jwt"
		secret = env("JWT_SECRET")

	database:
		driver = "sqlite"
		path = "db/app.db"
```

## Commands

```bash
# Scan project structure
frame scan
frame scan --verbose
frame scan --format json

# Build project
frame build
frame build -O 3          # Max optimization
frame build -o build      # Custom output dir

# Run
cleen server run dist/app.wasm --port 3000
```

## Legacy Mode

Projects without `app/` folder use legacy mode:

```
my-project/
  main.cln                # Entry file
  src/
    utils.cln
```

The build command looks for entry files in order:
1. `app/api/main.cln`
2. `main.cln`
3. `src/main.cln`

Or specify directly:
```bash
frame build main.cln
```
