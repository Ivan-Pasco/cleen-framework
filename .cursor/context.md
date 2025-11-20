# AI Development Assistant Context

**Rules and conventions for AI-assisted development of Frame Framework**

## Project Overview

Frame is a full-stack framework for Clean Language that compiles to WebAssembly. This context file provides guidance for AI development assistants working on Frame Framework code.

## Core Principles

1. **Type Safety First**: Always prefer type-safe solutions
2. **Simplicity**: One clear way to do things
3. **Transparency**: No hidden magic, explicit behavior
4. **Performance**: WASM-optimized, predictable execution
5. **Security**: Sandboxed, validated, safe by default

## Clean Language Conventions

### Syntax Rules

**Indentation**: ALWAYS use tabs (never spaces)
```clean
// ✅ Correct
start()
	integer x = 5
	if x > 0
		print("positive")

// ❌ Wrong
start()
  integer x = 5  // spaces
    if x > 0     // mixing tabs/spaces
```

**Functions Must Be in Blocks**:
```clean
// ✅ Correct
functions:
	integer add(integer a, integer b)
		return a + b

// ❌ Wrong
function add(integer a, integer b)
	return a + b
```

**Lowercase Namespaces**:
```clean
// ✅ Correct
number result = math.sqrt(16)

// ❌ Wrong
number result = Math.sqrt(16)
```

**Method Calls Require Parentheses**:
```clean
// ✅ Correct
string text = value.toString()

// ❌ Wrong
string text = value.toString
```

### Naming Conventions

- **Classes/Components**: PascalCase (`UserProfile`, `BlogPost`)
- **Functions/Variables**: camelCase (`getUserById`, `totalCount`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_RETRIES`, `API_URL`)
- **Files**: kebab-case (`user-profile.cln`, `api-client.cln`)

### Type Annotations

Always be explicit about types:
```clean
// ✅ Good
integer count = 0
list<User> users = []
User? maybeUser = null

// ❌ Avoid
count = 0
users = []
```

## File Organization

### Project Structure

```
project/
├── app/
│   ├── api/          # Backend API endpoints
│   ├── pages/        # Frontend pages (SSR)
│   └── components/   # Reusable UI components
├── db/
│   ├── schema.cln    # Data models
│   └── migrations/   # Auto-generated SQL
├── config/
│   ├── app.cln       # App configuration
│   ├── ui.cln        # UI theme
│   ├── data.cln      # Database
│   └── auth.cln      # Authentication
├── public/           # Static assets
└── dist/             # Compiled WASM
```

### File Naming

- API endpoints: `app/api/users.cln` → `/api/users`
- Dynamic routes: `app/api/users/[id].cln` → `/api/users/:id`
- Pages: `app/pages/index.cln` → `/`
- Components: `app/components/UserCard.cln`

## Code Generation Guidelines

### When Creating Models

```clean
data ModelName
	integer id : pk, auto
	string name : min=2, max=100
	string email : unique
	datetime createdAt : default=now
```

Always include:
- Primary key with `pk, auto`
- Validation constraints (`min`, `max`, `unique`)
- Default values where appropriate
- `createdAt` timestamp for auditing

### When Creating API Endpoints

```clean
functions:
	// List with pagination
	list<Model> get(integer? page = 1, integer? limit = 20)
		return Model.find:
			order:
				createdAt desc
			limit: limit
			offset: (page - 1) * limit

	// Get single
	Model getById(integer id)
		Model? item = Model.first:
			where:
				id == id
		if item == null
			return error(404, "Not found")
		return item

	// Create
	Model post(ModelCreate body)
		return Model.insert:
			name = body.name
			email = body.email

	// Update
	Model put(integer id, ModelUpdate body)
		Model.update:
			set:
				name = body.name
			where:
				id == id
		return getById(id)

	// Delete
	void delete(integer id)
		Model.delete:
			where:
				id == id
```

### When Creating Components

```clean
component ComponentName
	props:
		prop1: Type
		prop2?: Type = defaultValue

	functions:
		Widget render()
			return (
				<div class="component-name">
					<h2>{prop1}</h2>
					if prop2 != null
						<p>{prop2}</p>
				</div>
			)

		// Event handlers
		void handleClick()
			// Handle logic
```

## Error Handling

### Always Use Structured Errors

```clean
// ✅ Good
if balance < amount
	return error(400, "Insufficient funds", {
		balance: balance,
		required: amount
	})

// ❌ Bad
if balance < amount
	return error("not enough money")
```

### Error Codes

Use consistent error codes:
- `VALIDATION_ERROR` - Input validation failed
- `NOT_FOUND` - Resource not found
- `FORBIDDEN` - Permission denied
- `DB_ERROR` - Database operation failed
- `AUTH_ERROR` - Authentication failed

## Security Guidelines

### Input Validation

Always validate user input:
```clean
functions:
	User createUser(UserForm form)
		if form.email.length() < 5
			return error(400, "Invalid email")
		if form.password.length() < 8
			return error(400, "Password too short")
		// Proceed with creation
```

### SQL Injection Prevention

Frame automatically parameterizes queries. Never build SQL strings:
```clean
// ✅ Safe - automatic parameterization
list<User> users = User.find:
	where:
		email == userInput

// ❌ Never do this
string sql = "SELECT * FROM users WHERE email = '" + userInput + "'"
```

### XSS Prevention

HTML is auto-escaped by default:
```html
<!-- ✅ Safe - automatic escaping -->
<p>{userInput}</p>

<!-- ❌ Only for trusted content -->
<div rawHtml={trustedContent}></div>
```

## Performance Best Practices

### Database Queries

```clean
// ✅ Good - paginate
list<Post> posts = Post.find:
	limit: 20
	offset: (page - 1) * 20

// ❌ Bad - load everything
list<Post> posts = Post.find:
	where: published == true
```

### UI Rendering

```html
<!-- SSR by default -->
<blog-post post-id="42"></blog-post>

<!-- Only hydrate interactive parts -->
<static-content></static-content>
<interactive-widget client="on"></interactive-widget>
```

## Testing

### Always Write Tests

```clean
tests:
	"creates user with valid data": createUser({
		name: "Alice",
		email: "alice@example.com"
	}).name = "Alice"

	"rejects invalid email": createUser({
		name: "Alice",
		email: "invalid"
	}) = error("VALIDATION_ERROR")
```

## Common Patterns

### CRUD API

See API_REFERENCE.md for complete CRUD patterns.

### Protected Routes

```clean
route /api/admin/*
	middleware: [RequireAuth, RequireRole("admin")]
```

### Form Handling

```clean
component ContactForm
	functions:
		Widget render()
			return (
				<form onSubmit="handleSubmit">
					<input type="text" name="name" required />
					<input type="email" name="email" required />
					<button type="submit">Send</button>
				</form>
			)

		void handleSubmit(FormEvent event)
			event.preventDefault()
			// Validate and submit
```

## Documentation

### When Adding New Features

1. Update relevant specification files
2. Add examples to knowledge base
3. Document in API reference
4. Add tests
5. Update changelog

### Comment Style

```clean
/// Brief description of what this does.
///
/// Longer explanation if needed. Explain why, not what
/// (code should be self-explanatory for "what").
///
/// Parameters:
///   - param1: Description
///   - param2: Description
///
/// Returns: Description of return value
functions:
	returnType functionName(param1Type param1, param2Type param2)
		// Implementation
```

## AI Assistant Behavior

### Do

- Follow all syntax rules strictly
- Use tabs for indentation (NEVER spaces)
- Always specify types explicitly
- Write secure, validated code
- Include error handling
- Add tests for new functionality
- Keep code simple and readable

### Don't

- Mix tabs and spaces
- Use uppercase namespaces (`Math` → use `math`)
- Create functions outside `functions:` blocks (except `start()`)
- Forget parentheses on method calls
- Skip input validation
- Expose secrets in code
- Create placeholder implementations

### When Uncertain

1. Check specification files in `documents/specification/`
2. Refer to examples in `examples/`
3. Consult API_REFERENCE.md
4. Ask user for clarification
5. Default to simplest, most explicit solution

## References

For detailed information, refer to:
- **ARCHITECTURE.md** - System design and internals
- **FUNCTIONAL_SPEC.md** - Complete feature specifications
- **KNOWLEDGE_BASE.md** - Technical reference and patterns
- **API_REFERENCE.md** - Complete API documentation
- **CONTRIBUTING.md** - Development workflow
- **Specification files** - `documents/specification/*.md`

## Version

This context file is for Frame Framework v1.0.

**Last Updated**: November 2025
