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
frame new my-app
cd my-app
```

This creates a project with automatic file discovery:

```
my-app/
  app/
    ui/
      pages/
        index.html     # Home page (/)
      components/          # Reusable UI components
      layouts/             # Page layouts
      public/              # Static files (CSS, images)
    server/
      api/                 # JSON API endpoints
      models/              # Database models
      middleware/          # Request filters
  config.cln               # Project configuration
```

## Your First Page

Edit `app/ui/pages/index.html`:

```cln
// Home page
string html = "<html>
<head>
    <title>My App</title>
    <link rel='stylesheet' href='/public/css/style.css'>
</head>
<body>
    <h1>Welcome to Clean Framework</h1>
    <p>Build fast, type-safe web applications.</p>
</body>
</html>"

return html
```

## Build and Run

```bash
# Build the application
frame build

# Run it
cleen server run dist/app.wasm --port 3000
```

Open http://localhost:3000 in your browser.

## Add More Pages

Create `app/ui/pages/about.html`:

```cln
string html = "<html>
<body>
    <h1>About Us</h1>
    <a href='/'>Back to Home</a>
</body>
</html>"

return html
```

Rebuild and the `/about` route is automatically available.

## Dynamic Routes

Create `app/ui/pages/blog/[slug].html` for dynamic URLs:

```cln
// The slug parameter is automatically extracted from the URL
string html = "<html>
<body>
    <h1>Blog Post: " + slug + "</h1>
</body>
</html>"

return html
```

This handles URLs like `/blog/hello-world`, `/blog/my-first-post`, etc.

## Add an API Endpoint

Create `app/server/api/users.cln`:

```cln
// Returns JSON
return "[{\"id\": 1, \"name\": \"Alice\"}, {\"id\": 2, \"name\": \"Bob\"}]"
```

Access it at `/api/users`.

## Discover Your Project

See what will be built:

```bash
frame scan
```

Output:
```
Discovered 3 items:

Pages (2):
  GET /
  GET /about

API Routes (1):
  GET /api/users
```

## Add a Database Model

Create `app/server/models/User.cln`:

```cln
data User
    integer id
    string name
    string email
```

Models are auto-discovered and available in your pages and API endpoints.

## Next Steps

- Read [Project Structure](PROJECT_STRUCTURE.md) for the complete folder reference
- Check the [Examples](../examples/) for real-world patterns
- See the [UI Specification](specification/05_frame_ui.md) for components and layouts

## Quick Reference

| Command | Description |
|---------|-------------|
| `frame new <name>` | Create new project |
| `frame scan` | Preview discovered files |
| `frame build` | Build to dist/app.wasm |
| `cleen server run dist/app.wasm` | Run the application |

| Folder | Purpose |
|--------|---------|
| `app/ui/pages/` | HTML routes (.html) |
| `app/ui/components/` | Custom elements (.cln) |
| `app/server/api/` | JSON endpoints (.cln) |
| `app/server/models/` | Database schemas (.cln) |
| `app/ui/public/` | Static files |

| File Pattern | Route |
|--------------|-------|
| `pages/index.html` | `/` |
| `pages/about.html` | `/about` |
| `pages/blog/[slug].html` | `/blog/:slug` |
| `api/users.cln` | `/api/users` |
| `api/users/[id].cln` | `/api/users/:id` |
