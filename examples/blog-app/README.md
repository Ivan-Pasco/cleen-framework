# Frame Blog - Full-Stack Example Application

A complete blog application built with the Frame Framework, demonstrating all core features including SSR, Islands Architecture, authentication, database ORM, validation, and theming.

## Features Demonstrated

### 🚀 Framework Features

- **Server-Side Rendering (SSR)** - Fast initial page loads
- **Islands Architecture** - Selective client hydration for interactivity
- **Authentication** - Session-based auth with login/logout
- **Database ORM** - Type-safe database queries and relationships
- **Form Validation** - Client and server-side validation
- **Theming** - Multiple themes with dark mode support
- **API Generation** - OpenAPI spec and TypeScript SDK
- **Migrations** - Database schema management

### 📝 Application Features

- User registration and authentication
- Create, read, update, delete blog posts
- Comment system with nested replies
- User profiles
- Tag-based post filtering
- Search functionality
- Responsive design
- Dark mode toggle

## Project Structure

```
blog-app/
├── src/
│   ├── main.cln           # Application entry point
│   ├── models/
│   │   ├── User.cln       # User data model
│   │   ├── Post.cln       # Blog post model
│   │   └── Comment.cln    # Comment model
│   ├── components/
│   │   ├── Header.cln     # Site header with nav
│   │   ├── PostCard.cln   # Post preview card
│   │   ├── PostList.cln   # List of posts (island)
│   │   ├── CommentForm.cln # Comment submission form (island)
│   │   └── ThemeToggle.cln # Theme switcher (island)
│   └── pages/
│       ├── HomePage.cln   # Landing page
│       ├── PostPage.cln   # Single post view
│       ├── LoginPage.cln  # Login form
│       └── ProfilePage.cln # User profile
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_posts.sql
│   └── 003_create_comments.sql
└── README.md (this file)
```

## Getting Started

### Prerequisites

- Frame CLI installed
- SQLite or PostgreSQL database

### Installation

1. **Create the database:**
   ```bash
   frame db:create
   ```

2. **Run migrations:**
   ```bash
   frame db:migrate
   ```

3. **Start development server:**
   ```bash
   frame serve
   ```

4. **Visit the application:**
   Open http://localhost:3000 in your browser

### Generate API Documentation

Generate OpenAPI spec and TypeScript SDK:

```bash
# Generate OpenAPI 3.1 specification
frame api:openapi openapi.json

# Generate TypeScript SDK for frontend
frame api:sdk typescript src/api-client.ts
```

## Key Code Examples

### 1. Data Models with Relationships

```clean
// models/Post.cln
data Post {
	id: integer
	title: string
	content: string
	author_id: integer
	created_at: string

	// Relationship to User
	belongsTo: User

	// Relationship to Comments
	hasMany: Comment
}
```

### 2. API Endpoints

```clean
// HTTP API endpoints
endpoints:
	GET "/api/posts":
		return json(Post.find: order: createdAt desc)

	POST "/api/posts" [auth]:
		Post post = Post.insert(req.json(Post))
		return json(post)

	GET "/api/posts/:id":
		Post? post = Post.first: where: id == req.params.id.toInteger()
		if post == null
			return notFound()
		return json(post)

	PUT "/api/posts/:id" [auth]:
		Post post = Post.update(req.params.id.toInteger(), req.json(Post))
		return json(post)

	DELETE "/api/posts/:id" [auth]:
		Post.delete: where: id == req.params.id.toInteger()
		return json({ deleted: true })

	POST "/api/login":
		return auth.session.login(req.json(Credentials))

	POST "/api/logout":
		return auth.session.logout()

	GET "/api/me" [auth]:
		return json(req.context.user)
```

### 3. Components with Islands

```clean
// components/PostList.cln
component PostList {
	// Island component for client-side interactivity
	client: "visible"  // Hydrate when visible

	data posts: Array<Post>
	data currentPage: integer

	Widget render() {
		return html(
			<div class="post-list">
				{posts.map(post =>
					<PostCard post={post} />
				)}
				<Pagination page={currentPage} />
			</div>
		)
	}

	// Client-side event handler
	onClick loadMore() {
		// Fetch more posts
	}
}
```

### 4. Form Validation

```clean
// Validate post creation
import frame-ui:validation

functions:
	Widget createPostPage() {
		data validator = FormValidator.new()
		validator.addRule("title", required())
		validator.addRule("title", minLength(5))
		validator.addRule("content", required())
		validator.addRule("content", minLength(100))

		data result = validator.validate(formData)

		if result.isValid() {
			// Create post
		} else {
			// Show errors
		}
	}
```

### 5. Authentication Guard

```clean
// Protect routes with authentication
endpoints:
	POST "/api/posts" [auth]:
		Post post = Post.create(req.json(Post))
		return json(post)
```

### 6. Database Queries

```clean
// Complex queries with relationships
import frame-data:*

functions:
	Widget listPosts() {
		data posts = Post.find()
			.where("published", true)
			.orderBy("created_at", "desc")
			.limit(10)
			.with("author")  // Eager load author
			.with("comments") // Eager load comments
			.execute()

		return json(posts)
	}
```

## Architecture Highlights

### SSR + Islands = Best of Both Worlds

- **Server-Side Rendering** provides fast initial load and SEO
- **Islands Architecture** adds interactivity only where needed
- Non-interactive components stay static HTML
- Interactive islands hydrate on client (visible, idle, or on-load)

### Type-Safe Full Stack

- Database schema → Clean models → TypeScript SDK
- Single source of truth for data types
- Compile-time type checking prevents runtime errors

### Security by Default

- CSRF protection on all POST/PUT/PATCH/DELETE
- SQL injection prevention with parameterized queries
- XSS prevention with automatic HTML escaping
- HTTP-only secure cookies for sessions

## Performance

This example demonstrates Frame's performance characteristics:

- **< 50ms** First paint for SSR pages
- **< 100ms** API response times
- **Minimal JavaScript** Only islands need client-side JS
- **Progressive Enhancement** Works without JavaScript

## Learning Path

1. **Start with `src/main.cln`** - See the application entry point
2. **Explore `models/`** - Understand data modeling
3. **Read `components/`** - Learn component patterns
4. **Study `endpoints:`** - API route definitions
5. **Check `migrations/`** - Database schema evolution

## Deployment

Build for production:

```bash
# Build for server deployment
frame build --target=server

# Build for web (WASM)
frame build --target=web

# Build for mobile
frame build --target=ios
frame build --target=android
```

## Further Reading

- [Frame Framework Documentation](../../documents/README.md)
- [Islands Architecture Guide](../../system-documents/islands-architecture.md)
- [Authentication Guide](../../system-documents/authentication-guide.md)
- [Database ORM Guide](../../documents/specification/04_frame_data.md)

## License

MIT License - See LICENSE file for details

---

**This example demonstrates production-ready patterns for building modern web applications with Frame Framework.**
