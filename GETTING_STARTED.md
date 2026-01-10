# Getting Started with Frame Framework

Welcome to Frame! This guide will walk you through creating your first full-stack application with the Frame Framework.

## Table of Contents

1. [Installation](#installation)
2. [Your First Application](#your-first-application)
3. [Understanding the Architecture](#understanding-the-architecture)
4. [Core Concepts](#core-concepts)
5. [Next Steps](#next-steps)

## Installation

### Prerequisites

- **Clean Language Compiler** - Install from [clean-lang.org](https://clean-lang.org)
- **Database** - SQLite (included) or PostgreSQL
- **Node.js** (optional) - For TypeScript SDK generation

### Install Frame CLI

```bash
# Install Frame CLI globally
cargo install frame-cli

# Verify installation
frame --version
```

## Your First Application

### 1. Create a New Project

```bash
# Create a new Frame application
frame new my-blog --template=full-stack

# Navigate into the project
cd my-blog
```

This creates a project structure:

```
my-blog/
├── src/
│   ├── main.cln        # Application entry point
│   ├── models/         # Data models
│   ├── components/     # UI components
│   └── pages/          # Page templates
├── migrations/         # Database migrations
├── static/             # Static assets
├── package.clean.toml  # Project manifest
└── README.md
```

### 2. Set Up the Database

```bash
# Create the database
frame db:create

# Run migrations
frame db:migrate
```

### 3. Start the Development Server

```bash
# Start dev server with hot reload
frame serve
```

Visit http://localhost:3000 - Your app is running! 🎉

## Understanding the Architecture

Frame uses a unique architecture that combines the best of server and client rendering:

```
┌─────────────────────────────────────────┐
│           Browser (Client)              │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   Islands (Interactive JS)       │  │
│  │   - Comments form                │  │
│  │   - Search widget                │  │
│  │   - Theme toggle                 │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   Static HTML (No JS)            │  │
│  │   - Header                       │  │
│  │   - Footer                       │  │
│  │   - Content                      │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
                  ↑
                  │ HTML + Islands Manifest
                  │
┌─────────────────────────────────────────┐
│          Server (Frame)                  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   Server-Side Rendering (SSR)    │  │
│  │   - Renders full HTML            │  │
│  │   - Runs all components          │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   API Endpoints                  │  │
│  │   - GET /api/posts               │  │
│  │   - POST /api/posts              │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   Database ORM                   │  │
│  │   - Type-safe queries            │  │
│  │   - Relationships                │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**Key Benefits:**
- ⚡ **Fast initial load** - SSR renders complete HTML
- 🎯 **Selective interactivity** - Only islands use JavaScript
- 🔍 **SEO friendly** - All content is in the HTML
- ♿ **Accessible** - Works without JavaScript

## Core Concepts

### 1. Data Models

Define your database schema with the `data` keyword:

```clean
// models/Post.cln
import frame-data:*

data Post {
    id: integer
    title: string
    content: string
    author_id: integer
    published: boolean
    created_at: string

    // Relationships
    belongsTo: User
    hasMany: Comment

    // Table configuration
    tableName: "posts"
}
```

**Query the data:**

```clean
// Find all published posts
data posts = Post.find()
    .where("published", true)
    .orderBy("created_at", "desc")
    .with("author")  // Eager load relationship
    .limit(10)
    .execute()

// Find specific post
data post = Post.find(123).execute()

// Create new post
data newPost = Post.create({
    title: "My First Post",
    content: "Hello, Frame!",
    author_id: currentUser.id,
    published: true
})

// Update post
post.title = "Updated Title"
post.save()

// Delete post
post.delete()
```

### 2. HTTP Endpoints

Define your API with the `endpoints:` block:

```clean
// Declare your routes
endpoints:
    GET "/api/posts" -> listPosts
    POST "/api/posts" -> createPost
        guard: requireAuth
    GET "/api/posts/{id}" -> getPost
    PUT "/api/posts/{id}" -> updatePost
        guard: requireAuth
    DELETE "/api/posts/{id}" -> deletePost
        guard: requireAuth

// Implement the handlers
functions:
    Widget listPosts() {
        data posts = Post.find()
            .where("published", true)
            .orderBy("created_at", "desc")
            .execute()

        return json({ posts: posts })
    }

    Widget createPost() {
        data body = request.body()

        // Validate input
        data validator = FormValidator.new()
        validator.addRule("title", required())
        validator.addRule("content", required())

        data result = validator.validate(body)

        if !result.isValid() {
            return badRequest({ errors: result.errors })
        }

        // Create post
        data post = Post.create(body)

        return created({ post: post })
    }

    Widget getPost() {
        data id = request.param("id").toInteger()
        data post = Post.find(id).execute()

        if post == null {
            return notFound({ error: "Post not found" })
        }

        return json({ post: post })
    }
```

**Authentication guards:**

```clean
functions:
    boolean requireAuth() {
        data user = Auth.currentUser()
        return user != null
    }
```

### 3. UI Components

Create reusable UI components:

```clean
// components/PostCard.cln
import frame-ui:*

component PostCard {
    // Props passed from parent
    data post: Post

    Widget render() {
        return html(
            <div class="post-card">
                <h2>{post.title}</h2>
                <p class="post-meta">
                    By {post.author.name} on {post.created_at}
                </p>
                <p>{post.excerpt}</p>
                <a href="/posts/{post.id}" class="read-more">
                    Read more →
                </a>
            </div>
        )
    }
}
```

**Use in pages:**

```clean
// pages/HomePage.cln
Widget homePage() {
    data posts = Post.find()
        .where("published", true)
        .limit(10)
        .execute()

    return html(
        <html>
            <head>
                <title>My Blog</title>
                <link rel="stylesheet" href="/static/css/frame.css" />
            </head>
            <body>
                <header>
                    <h1>Welcome to My Blog</h1>
                </header>

                <main>
                    {posts.map(post =>
                        <PostCard post={post} />
                    )}
                </main>
            </body>
        </html>
    )
}
```

### 4. Islands Architecture

Make components interactive with the `client` attribute:

```clean
// components/CommentForm.cln
component CommentForm {
    // This component will hydrate on the client
    client: "idle"  // Hydrate when browser is idle

    data postId: integer
    state comment: string = ""
    state submitting: boolean = false

    Widget render() {
        return html(
            <form class="comment-form">
                <textarea
                    value={comment}
                    onInput={handleInput}
                    placeholder="Write a comment..."
                />
                <button
                    onClick={submitComment}
                    disabled={submitting}
                    class="btn btn-primary"
                >
                    {submitting ? "Submitting..." : "Post Comment"}
                </button>
            </form>
        )
    }

    onInput handleInput(event) {
        comment = event.target.value
    }

    onClick submitComment() {
        submitting = true

        fetch("/api/posts/" + postId.toString() + "/comments", {
            method: "POST",
            body: JSON.stringify({ body: comment })
        })
        .then(response => response.json())
        .then(data => {
            comment = ""
            submitting = false
            // Optionally reload comments or update UI
        })
        .catch(error => {
            submitting = false
            alert("Failed to post comment")
        })
    }
}
```

**Hydration strategies:**

- `client="on"` - Always hydrate (most interactive)
- `client="off"` - Never hydrate (static HTML only)
- `client="visible"` - Hydrate when scrolled into view
- `client="idle"` - Hydrate when browser is idle
- `client="only"` - Client-side only (no SSR)

### 5. Authentication

Secure your application with built-in authentication:

```clean
// Configure auth in start()
void start() {
    Auth.configure({
        sessionSecret: env.get("SESSION_SECRET"),
        sessionExpiry: 86400,  // 24 hours
        jwtSecret: env.get("JWT_SECRET")
    })

    // Define roles and permissions
    Auth.defineRole("admin", ["*"])  // All permissions
    Auth.defineRole("user", [
        "posts.read",
        "comments.create"
    ])

    Server.listen(3000)
}

// Login endpoint
functions:
    Widget login() {
        data body = request.body()

        data user = User.find()
            .where("email", body.email)
            .execute()

        if user == null {
            return unauthorized({ error: "Invalid credentials" })
        }

        data valid = Auth.verifyPassword(body.password, user.password)

        if !valid {
            return unauthorized({ error: "Invalid credentials" })
        }

        // Create session
        data session = Auth.createSession(user)

        return json({ user: user, session: session })
    }

    Widget logout() {
        Auth.destroySession()
        return noContent()
    }
```

**Check permissions:**

```clean
import frame-auth:can

functions:
    Widget createPost() {
        data user = Auth.currentUser()

        if !can(user, "posts.create") {
            return forbidden({ error: "Permission denied" })
        }

        // Create post...
    }
```

### 6. Form Validation

Validate user input on both client and server:

**Server-side:**

```clean
import frame-ui:validation

functions:
    Widget createPost() {
        data validator = FormValidator.new()
        validator.addRule("title", required())
        validator.addRule("title", minLength(5))
        validator.addRule("content", required())
        validator.addRule("content", minLength(100))

        data body = request.body()
        data result = validator.validate(body)

        if !result.isValid() {
            return badRequest({
                error: "Validation failed",
                errors: result.errors
            })
        }

        // Process valid data...
    }
```

**Client-side:**

```html
<script src="/static/js/frame-validate.js"></script>
<script>
    const validator = new FrameValidation.FormValidator();
    validator.addRule('title', FrameValidation.required());
    validator.addRule('title', FrameValidation.minLength(5));
    validator.addRule('content', FrameValidation.required());

    FrameValidation.attachToForm(
        document.querySelector('#post-form'),
        validator
    );
</script>
```

### 7. Theming

Built-in support for multiple themes:

**Enable theming:**

```html
<link rel="stylesheet" href="/static/css/frame.css" />
<script src="/static/js/frame-theme.js"></script>
<script>
    // Initialize theme system
    FrameTheme.init();
</script>
```

**Add theme toggle:**

```clean
component ThemeToggle {
    client: "on"

    Widget render() {
        return html(
            <div class="theme-switcher">
                <button onClick="FrameTheme.setTheme('light')">☀️ Light</button>
                <button onClick="FrameTheme.setTheme('dark')">🌙 Dark</button>
                <button onClick="FrameTheme.setTheme('high-contrast')">◐ High Contrast</button>
            </div>
        )
    }
}
```

The framework includes:
- Automatic dark mode detection
- localStorage persistence
- Three built-in themes
- Support for custom themes

## Next Steps

### 1. Generate API Documentation

```bash
# Generate OpenAPI 3.1 spec
frame api:openapi openapi.json

# Generate TypeScript SDK
frame api:sdk typescript client.ts

# Use the SDK in your frontend
import api from './client';
const { data, error } = await api.listPosts();
```

### 2. Database Migrations

```bash
# Create a new migration
frame db:create-migration add_tags_to_posts

# Edit the migration file in migrations/
# Then apply it
frame db:migrate

# Rollback if needed
frame db:rollback
```

### 3. Deploy to Production

```bash
# Build for server deployment
frame build --target=server

# Build for web (WASM)
frame build --target=web

# Build for mobile
frame build --target=ios
frame build --target=android
```

### 4. Explore Examples

Check out the complete examples in the `examples/` directory:
- **blog-app** - Full-featured blog with all framework features
- **todo-app** - Simple task manager
- **e-commerce** - Product catalog with cart

### 5. Read the Documentation

- [Frame Server Guide](documents/specification/03_frame_server.md) - HTTP server and endpoints
- [Frame Data Guide](documents/specification/04_frame_data.md) - Database ORM
- [Frame UI Guide](documents/specification/05_frame_ui.md) - Components and Islands
- [Frame Auth Guide](documents/specification/06_frame_auth.md) - Authentication
- [Islands Architecture Guide](system-documents/islands-architecture.md) - Deep dive

## Common Patterns

### Pagination

```clean
functions:
    Widget listPosts() {
        data page = request.query("page", "1").toInteger()
        data limit = 10

        data posts = Post.find()
            .orderBy("created_at", "desc")
            .limit(limit)
            .offset((page - 1) * limit)
            .execute()

        data total = Post.count().execute()

        return json({
            posts: posts,
            pagination: {
                page: page,
                limit: limit,
                total: total,
                pages: math.ceil(total / limit)
            }
        })
    }
```

### Search

```clean
functions:
    Widget search() {
        data query = request.query("q", "")

        if query.length() < 3 {
            return badRequest({ error: "Query too short" })
        }

        data posts = Post.find()
            .whereContains("title", query)
            .orWhereContains("content", query)
            .orderBy("created_at", "desc")
            .limit(20)
            .execute()

        return json({ posts: posts, query: query })
    }
```

### File Uploads

```clean
functions:
    Widget uploadAvatar() {
        data file = request.file("avatar")

        if file == null {
            return badRequest({ error: "No file uploaded" })
        }

        // Save file
        data path = "/uploads/" + file.name
        file.saveTo(path)

        // Update user
        data user = Auth.currentUser()
        user.avatar_url = path
        user.save()

        return json({ avatar_url: path })
    }
```

## Troubleshooting

### Build Errors

If you encounter build errors:
1. Check `frame-cli` is up to date: `frame --version`
2. Verify your Clean Language version matches
3. Run `frame build --verbose` for detailed logs

### Database Issues

If migrations fail:
1. Check database connection: `frame db:status`
2. Verify migration syntax
3. Check database permissions

### Runtime Errors

Check logs for detailed error messages:
```bash
# View server logs
frame serve --verbose

# Check WASM errors
frame build --debug
```

## Getting Help

- **Documentation**: [Frame Framework Docs](documents/README.md)
- **Examples**: Check the `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/clean-lang/frame/issues)
- **Community**: [Discord Server](https://discord.gg/clean-lang)

---

**Congratulations!** 🎉 You're now ready to build amazing full-stack applications with Frame Framework.

Happy coding!
