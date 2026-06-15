# Frame Framework - API Reference

**Complete API Documentation**

## Table of Contents

1. [Clean Language Core API](#clean-language-core-api)
2. [Standard Library](#standard-library)
3. [Frame Data (ORM)](#frame-data-orm)
4. [Frame Server](#frame-server)
5. [Frame UI](#frame-ui)
6. [Frame Auth](#frame-auth)
7. [Host Bridge](#host-bridge)
8. [CLI Commands](#cli-commands)

For full specification details, see:

| Category | Spec Document |
|----------|---------------|
| Server / HTTP | [03_frame_server.md](specification/03_frame_server.md) |
| ORM / Data | [04_frame_data.md](specification/04_frame_data.md) |
| UI Components | [05_frame_ui.md](specification/05_frame_ui.md) |
| Auth | [06_frame_auth.md](specification/06_frame_auth.md) |
| Host Bridge | [frame_bridge_contracts.md](specification/frame_bridge_contracts.md) |
| Canvas | [12_frame_canvas.md](specification/12_frame_canvas.md) |
| CLI | [02_frame_cli.md](specification/02_frame_cli.md) |

---

## Clean Language Core API

### Type System

#### Primitive Types

**`integer`** - 32-bit signed integer
```clean
integer count = 42
integer negative = -17
```

**`integer:N`** - Sized integers
```clean
integer:8 tiny = 127           // -128 to 127
integer:16 small = 32767       // -32,768 to 32,767
integer:64 large = 9223372036854775807

integer:8u byte = 255          // 0 to 255 (unsigned)
integer:64u huge = 18446744073709551615
```

**`number`** - 64-bit floating point
```clean
number pi = 3.14159
number scientific = 6.02e23
```

**`string`** - UTF-8 text
```clean
string name = "Alice"
string empty = ""
string interpolated = "Hello, {name}!"
```

**`boolean`** - True or false
```clean
boolean isActive = true
boolean isEmpty = false
```

#### Collection Types

**`list<T>`** - Homogeneous list
```clean
list<integer> numbers = [1, 2, 3, 4, 5]
list<string> names = ["Alice", "Bob", "Charlie"]
list<any> mixed = []  // Generic list
```

**`matrix<T>`** - 2D list
```clean
matrix<number> grid = [
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0]
]
```

#### Nullable Types

```clean
integer? maybeNumber = null
User? maybeUser = User.first: where: id == 42
```

#### Type Summary

| Type | Description | Example |
|------|-------------|---------|
| `integer` | 32-bit signed | `integer count = 42` |
| `integer:8 / :16 / :64` | Sized integers | `integer:64 large = 9223372036854775807` |
| `integer:8u / :64u` | Unsigned variants | `integer:8u byte = 255` |
| `number` | 64-bit float | `number pi = 3.14159` |
| `string` | UTF-8 text, `{var}` interpolation | `string s = "Hello, {name}!"` |
| `boolean` | `true` / `false` | `boolean active = true` |
| `list<T>` | Homogeneous list | `list<integer> nums = [1, 2, 3]` |
| `matrix<T>` | 2D list | `matrix<number> grid = [[1.0, 2.0]]` |
| `T?` | Nullable | `User? user = null` |
| `any` | Generic / untyped | used in `list<any>` |

### Type Conversion

**`.toInteger()`** - Convert to integer
```clean
number decimal = 3.14
integer whole = decimal.toInteger()  // 3
```

**`.toNumber()`** - Convert to number
```clean
integer count = 42
number decimal = count.toNumber()    // 42.0
```

**`.toString()`** - Convert to string
```clean
integer age = 25
string text = age.toString()         // "25"
```

**`.toBoolean()`** - Convert to boolean
```clean
integer zero = 0
boolean flag = zero.toBoolean()      // false

integer nonZero = 5
boolean other = nonZero.toBoolean()  // true
```

---

## Standard Library

### Math Module

**`math.sqrt(number)`** - Square root
```clean
number result = math.sqrt(16)  // 4.0
```

**`math.abs(number)`** - Absolute value
```clean
number absolute = math.abs(-42.5)  // 42.5
```

**`math.max(number, number)`** - Maximum of two numbers
```clean
number largest = math.max(10.5, 7.2)  // 10.5
```

**`math.min(number, number)`** - Minimum of two numbers
```clean
number smallest = math.min(10.5, 7.2)  // 7.2
```

**`math.floor(number)`** - Round down
```clean
number down = math.floor(3.7)  // 3.0
```

**`math.ceil(number)`** - Round up
```clean
number up = math.ceil(3.2)  // 4.0
```

**`math.round(number)`** - Round to nearest
```clean
number rounded = math.round(3.5)  // 4.0
```

**`math.sin(number)`** - Sine (radians)
```clean
number sine = math.sin(math.pi() / 2)  // 1.0
```

**`math.cos(number)`** - Cosine (radians)
```clean
number cosine = math.cos(0)  // 1.0
```

**`math.tan(number)`** - Tangent (radians)
```clean
number tangent = math.tan(math.pi() / 4)  // 1.0
```

**`math.pi()`** - Pi constant
```clean
number pi = math.pi()  // 3.14159...
```

**`math.e()`** - Euler's number
```clean
number e = math.e()  // 2.71828...
```

**Quick reference:**
```
math.sqrt(n)   math.abs(n)    math.max(a, b)  math.min(a, b)
math.floor(n)  math.ceil(n)   math.round(n)
math.sin(n)    math.cos(n)    math.tan(n)
math.pi()      math.e()
```

### String Module

**`string.length(string)`** - Get string length
```clean
integer len = string.length("Hello")  // 5
```

**`string.concat(string, string)`** - Concatenate strings
```clean
string combined = string.concat("Hello", " World")  // "Hello World"
```

**`string.contains(string, string)`** - Check if contains substring
```clean
boolean has = string.contains("Hello World", "World")  // true
```

**`string.split(string, string)`** - Split by delimiter
```clean
list<string> parts = string.split("a,b,c", ",")  // ["a", "b", "c"]
```

**`string.upper(string)`** - Convert to uppercase
```clean
string upper = string.upper("hello")  // "HELLO"
```

**`string.lower(string)`** - Convert to lowercase
```clean
string lower = string.lower("HELLO")  // "hello"
```

**`string.trim(string)`** - Remove leading/trailing whitespace
```clean
string trimmed = string.trim("  hello  ")  // "hello"
```

**`string.replace(string, string, string)`** - Replace first occurrence
```clean
string replaced = string.replace("hello world", "world", "there")  // "hello there"
```

**`string.replaceAll(string, string, string)`** - Replace all occurrences
```clean
string replaced = string.replaceAll("foo foo", "foo", "bar")  // "bar bar"
```

**Quick reference:**
```
string.length(s)               string.concat(a, b)
string.contains(s, sub)        string.split(s, delim)
string.upper(s)                string.lower(s)
string.trim(s)                 string.replace(s, old, new)
string.replaceAll(s, old, new)
```

### List Module

**`list.size(list<any>)`** - Get list size
```clean
integer size = list.size([1, 2, 3])  // 3
```

**`list.get(list<any>, integer)`** - Get element at index
```clean
integer value = list.get([10, 20, 30], 1)  // 20
```

**`list.add(list<any>, any)`** - Add element to list
```clean
list<integer> numbers = [1, 2, 3]
numbers = list.add(numbers, 4)  // [1, 2, 3, 4]
```

**`list.remove(list<any>, integer)`** - Remove element at index
```clean
list<integer> numbers = [1, 2, 3]
numbers = list.remove(numbers, 1)  // [1, 3]
```

**`list.contains(list<any>, any)`** - Check if contains element
```clean
boolean has = list.contains([1, 2, 3], 2)  // true
```

**`list.sort(list<any>)`** - Sort list ascending
```clean
list<integer> sorted = list.sort([3, 1, 2])  // [1, 2, 3]
```

**`list.reverse(list<any>)`** - Reverse list
```clean
list<integer> reversed = list.reverse([1, 2, 3])  // [3, 2, 1]
```

**`list.join(list<string>, string)`** - Join strings with separator
```clean
string joined = list.join(["a", "b", "c"], ", ")  // "a, b, c"
```

**Quick reference:**
```
list.size(l)          list.get(l, i)       list.add(l, v)
list.remove(l, i)     list.contains(l, v)  list.sort(l)
list.reverse(l)       list.join(l, sep)
```

### File Module

_(desktop / CLI / server only, via `bridge:fs`)_

**`file.read(string)`** - Read file contents
```clean
string content = file.read("data.txt")
```

**`file.write(string, string)`** - Write to file
```clean
file.write("output.txt", "Hello, World!")
```

**`file.exists(string)`** - Check if file exists
```clean
boolean exists = file.exists("config.json")
```

**`file.delete(string)`** - Delete file
```clean
file.delete("temp.txt")
```

### Http Module

_(outbound client requests, via `bridge:http`)_

**`http.get(string)`** - HTTP GET request
```clean
string response = http.get("https://api.example.com/users")
```

**`http.post(string, string)`** - HTTP POST request
```clean
string response = http.post("https://api.example.com/users", "{\"name\":\"Alice\"}")
```

**`http.put(string, string)`** - HTTP PUT request
```clean
string response = http.put("https://api.example.com/users/1", "{\"name\":\"Bob\"}")
```

**`http.delete(string)`** - HTTP DELETE request
```clean
string response = http.delete("https://api.example.com/users/1")
```

---

## Frame Data (ORM)

**Plugin:** `frame.data`
**Owned folders:** `app/data/`, `app/data/models/`, `app/data/`, `app/data/migrations/`, `app/data/`
**Spec:** [04_frame_data.md](specification/04_frame_data.md)

### Plugin Registration

Plugins are loaded automatically by folder location. For explicit registration in `main.cln`:

```clean
plugins:
    frame.data
```

Files placed in `app/data/` are processed by `frame.data` automatically without any `import:` statement.

### Model Definition

**`data` keyword** - Define a model
```clean
data User
    integer id : pk, auto
    string name : min=2, max=100
    string email : unique
    integer age : min=0, max=150
    boolean active = true
    datetime createdAt : default=now
```

**Field Attributes**:
- `pk` - Primary key
- `auto` - Auto-increment
- `unique` - Unique constraint
- `index` - Create index
- `default=<value>` - Default value
- `min=<value>` - Minimum value / minimum length
- `max=<value>` - Maximum value / maximum length
- `onDelete=cascade` - Cascading delete for foreign keys
- `nullable` - Allow null values

**Example model file** (`app/data/models/User.cln`):
```clean
data User
    integer id : pk, auto
    string name : min=2, max=100
    string email : unique, index
    string passwordHash
    boolean active = true
    datetime createdAt : default=now
    datetime? updatedAt
```

### Query Operations

**`.find:`** - Query multiple records
```clean
list<User> users = User.find:
    where:
        active == true
        age >= 18
    order:
        name asc
    limit: 20
    offset: 0
```

**`.first:`** - Get single record
```clean
User? user = User.first:
    where:
        email == "alice@example.com"
```

**`.insert:`** - Create record
```clean
User newUser = User.insert:
    name = "Alice"
    email = "alice@example.com"
    age = 30
```

**`.update:`** - Update records
```clean
User.update:
    set:
        active = false
        lastLogin = now()
    where:
        id == userId
```

**`.delete:`** - Delete records
```clean
User.delete:
    where:
        active == false
        createdAt < now().minusDays(365)
```

**`.count:`** - Count records
```clean
integer count = User.count:
    where:
        active == true
```

**Quick reference:**
```clean
// Query
list<User> users = User.find:
    where: active == true
    order: name asc
    limit: 20

// Single record
User? u = User.first: where: id == userId

// Insert
User u = User.insert: name = "Alice"  email = "alice@example.com"

// Update
User.update: set: active = false  where: id == userId

// Delete
User.delete: where: active == false

// Count
integer n = User.count: where: active == true
```

### Where Clause Operators

```clean
// Equality
where: name == "Alice"

// Comparison
where: age >= 18
where: score < 100
where: price != 0

// Logical AND (multiple where conditions)
where:
    active == true
    age >= 18

// Null checks
where: deletedAt == null
where: email != null

// String matching
where: name like "Ali%"
```

### Order Clause

```clean
order:
    name asc

order:
    createdAt desc
    name asc
```

### Transactions

**`transaction:`** - Execute transaction
```clean
transaction:
    User user = User.insert:
        name = "Alice"
        email = "alice@example.com"

    Post.insert:
        title = "Hello"
        author = user
        published = true
```

All operations inside `transaction:` run atomically. If any operation fails, the entire transaction is rolled back.

### Relationships

**One-to-Many**:
```clean
data User
    integer id : pk, auto
    string name

data Post
    integer id : pk, auto
    string title
    User author  // Foreign key to User

// Query posts by user
list<Post> posts = Post.find:
    where:
        author.id == userId

// Query with author data
list<Post> posts = Post.find:
    where:
        author.id == userId
    order:
        createdAt desc
```

**Many-to-Many** (via explicit junction table):
```clean
data User
    integer id : pk, auto
    string name

data Role
    integer id : pk, auto
    string name : unique

data UserRole
    User user : onDelete=cascade
    Role role : onDelete=cascade
    unique user, role

// Query users with a specific role
list<User> admins = User.find:
    link:
        UserRole user == id
        Role id == UserRole.role
    where:
        Role.name == "admin"

// Assign role to user
UserRole.insert:
    user = currentUser
    role = adminRole
```

### Migrations

Migrations are auto-generated from schema diffs. Use the CLI to manage them:

```bash
cleen db:plan      # Preview pending migrations
cleen db:migrate   # Apply pending migrations
cleen db:rollback  # Roll back last migration
```

---

## Frame Server

**Plugin:** `frame.server`
**Owned folders:** `app/server/`, `app/server/api/`, `app/server/middleware/`
**Handled blocks:** `server`, `endpoints`
**Spec:** [03_frame_server.md](specification/03_frame_server.md)

### Plugin Registration

Files placed in `app/server/` are processed by `frame.server` automatically. For explicit registration:

```clean
plugins:
    frame.server
```

### Endpoint Blocks

**`endpoints:`** - Define HTTP routes
```clean
endpoints:
    GET "/users":
        return json(User.find:)

    POST "/users":
        User u = User.insert(req.json(User))
        return json(u)

    GET "/users/:id":
        User? user = User.first: where: id == req.params.id.toInteger()
        return json(user)

    PUT "/users/:id":
        User u = User.update(req.params.id.toInteger(), req.json(User))
        return json(u)

    DELETE "/users/:id":
        User.delete: where: id == req.params.id.toInteger()
        return json({ deleted: true })
```

**With guards and caching:**
```clean
endpoints:
    GET "/admin/stats" [admin] cache(60):
        return adminStats()

    POST "/posts" [auth]:
        return createPost()

    GET "/public/data" cache(300):
        return publicData()
```

**Full handler example** (`app/server/api/users.cln`):
```clean
endpoints:
    GET "/users":
        list<User> users = User.find:
            where: active == true
            order: name asc
        return json(users)

    POST "/users":
        User newUser = User.insert(req.json(User))
        return json(newUser)

    GET "/users/:id":
        integer id = req.params.id.toInteger()
        User? user = User.first: where: id == id
        if user == null
            return notFound("User not found")
        return json(user)
```

### Request Object

```clean
// Available inside handler functions
Request req
    string method       // "GET", "POST", "PUT", "DELETE", etc.
    string path         // "/api/users/42"
    map headers         // HTTP request headers
    string body         // Raw request body string
    map query           // Query string parameters (?key=value)
    map params          // Path parameters (:id, :slug, etc.)
    map cookies         // Request cookies
    Session? session    // Current session (if authenticated)
```

**Accessing request data:**
```clean
// Path parameter
integer userId = req.params.id.toInteger()

// Query parameter
integer page = req.query.page.toInteger()
integer limit = req.query.limit.toInteger()

// Request body field
string name = req.body.name
string email = req.body.email

// Header
string contentType = req.headers["Content-Type"]
```

### Response Helpers

**`json(data)`** - JSON response (200)
```clean
return json(user)
return json(users)
return json({message: "Created", id: newUser.id})
```

**`html(string)`** - HTML response (200)
```clean
return html("<h1>Hello</h1>")
```

**`redirect(string)`** - Redirect (302)
```clean
return redirect("/dashboard")
return redirect("/login")
```

**`notFound(string)`** - 404 error
```clean
return notFound("User not found")
return notFound("Resource does not exist")
```

**`forbidden(string)`** - 403 error
```clean
return forbidden("Access denied")
```

**`badRequest(string)`** - 400 error
```clean
return badRequest("Invalid email format")
```

**`serverError(string)`** - 500 error
```clean
return serverError("Database connection failed")
```

**Custom response:**
```clean
return Response(201, {id: newUser.id}, {"Content-Type": "application/json"})
```

### Middleware

```clean
middleware RequireAuth
    functions:
        Request handle(Request req)
            Session? session = auth.session.getCurrent()
            if session == null
                return redirect("/login")
            return req

middleware RateLimit
    functions:
        Request handle(Request req)
            // Rate limiting logic
            return req
```

**Applying middleware to endpoints:**
```clean
endpoints:
    GET "/api/protected" middleware(RequireAuth):
        return protectedHandler()

    GET "/api/admin" middleware(RequireAuth, RequireAdmin):
        return adminHandler()
```

### File-Based Routing

Routes are automatically derived from file paths under `app/server/api/`:

```
app/server/api/users.cln              → /api/users
app/server/api/users/[id].cln         → /api/users/:id
app/server/api/posts/[id]/comments.cln → /api/posts/:id/comments
```

Page routes from `app/ui/web/pages/`:
```
app/ui/web/pages/index.html                   → /
app/ui/web/pages/about.html                   → /about
app/ui/web/pages/blog/[slug].html             → /blog/:slug
```

---

## Frame UI

**Plugin:** `frame.ui`
**Owned folders:** `app/ui/web/pages/`, `app/ui/web/components/`, `app/ui/web/layouts/`
**Handled blocks:** `component`, `screen`, `page`, `html`
**Spec:** [05_frame_ui.md](specification/05_frame_ui.md)

### Plugin Registration

Files in `app/ui/web/pages/`, `app/ui/web/components/`, and `app/ui/web/layouts/` are processed by `frame.ui` automatically. For explicit registration:

```clean
plugins:
    frame.ui
```

### Component Definition

```clean
component ComponentName
    inputs:
        prop1: Type
        prop2?: Type = defaultValue

    functions:
        Widget render()
            return html:
                <div class="container">
                    <h1>{prop1}</h1>
                </div>

        // Lifecycle hooks
        onMount()
        onUnmount()
        onUpdate()
```

**Example component** (`app/ui/web/components/UserCard.cln`):
```clean
component UserCard
    inputs:
        user: User
        showEmail?: boolean = false

    functions:
        Widget render()
            return html:
                <div class="user-card">
                    <h2>{user.name}</h2>
                    if showEmail
                        <p class="email">{user.email}</p>
                    <p class="joined">Joined: {user.createdAt}</p>
                </div>
```

### SSR Pages

Pages are standard HTML files with Clean Language template directives (`app/ui/web/pages/`):

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{pageTitle}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <h1>Welcome, {user.name}</h1>

    <ul cl-iterate="post in posts">
        <li>
            <a href="/blog/{post.slug}">{post.title}</a>
        </li>
    </ul>

    <div cl-if="user.isAdmin">
        <a href="/admin">Admin Panel</a>
    </div>
</body>
</html>
```

### Template Directives

**`{variable}`** - Interpolation (auto-escaped)
```html
<p>{user.name}</p>
<p>Count: {totalCount}</p>
<a href="/users/{user.id}">Profile</a>
```

**`cl-if`** - Conditional rendering
```html
<div cl-if="user.isAdmin">Admin only content</div>
<p cl-if="items.length == 0">No items found</p>
```

**`cl-iterate`** - Loop rendering
```html
<ul cl-iterate="item in items">
    <li>{item.name}</li>
</ul>

<tr cl-iterate="user in users">
    <td>{user.name}</td>
    <td>{user.email}</td>
</tr>
```

**`cl-bind`** - Two-way data binding (client-hydrated components)
```html
<input cl-bind="searchQuery" type="text" />
```

### Clean Language HTML Syntax

Inside `.cln` component files, use the `html:` block:

**Elements:**
```clean
html:
    <div class="container">
        <h1>Title</h1>
        <p>Paragraph</p>
    </div>
```

**Interpolation:**
```clean
html:
    <p>{variable}</p>
    <p>Count: {count}</p>
    <p>User: {user.name}</p>
```

**Conditionals:**
```clean
html:
    if condition
        <p>Shown when true</p>
    else
        <p>Shown when false</p>
```

**Loops:**
```clean
html:
    iterate item in itemList
        <li>{item.name}</li>
```

**Events:**
```clean
html:
    <button onClick="handleClick">Click me</button>
    <input onInput="handleInput" />
    <form onSubmit="handleSubmit">...</form>
```

### Hydration Strategies

Control when and whether a component is hydrated on the client:

| Attribute | Behavior |
|-----------|----------|
| _(omitted)_ | SSR only - no client JavaScript |
| `client="on"` | SSR + hydrate immediately on load |
| `client="visible"` | SSR + hydrate when scrolled into view |
| `client="idle"` | SSR + hydrate during browser idle time |
| `client="only"` | Client render only - no SSR |

```html
<!-- SSR only (default) -->
<user-card user="{user}"></user-card>

<!-- SSR + immediate hydration -->
<search-box client="on"></search-box>

<!-- SSR + hydrate when visible -->
<comment-section client="visible"></comment-section>

<!-- SSR + hydrate when idle -->
<analytics-widget client="idle"></analytics-widget>

<!-- Client-only (no SSR) -->
<interactive-chart client="only"></interactive-chart>
```

### Layouts

Layouts wrap pages with shared structure (`app/ui/web/layouts/`):

```clean
component MainLayout
    inputs:
        title: string
        content: Widget

    functions:
        Widget render()
            return html:
                <!DOCTYPE html>
                <html>
                    <head>
                        <title>{title}</title>
                        <link rel="stylesheet" href="/css/style.css" />
                    </head>
                    <body>
                        <header>
                            <nav>...</nav>
                        </header>
                        <main>{content}</main>
                        <footer>...</footer>
                    </body>
                </html>
```

---

## Frame Auth

**Plugin:** `frame.auth`
**Owned folders:** `app/auth/`
**Handled blocks:** `auth`, `protected`, `login`, `roles`
**Spec:** [06_frame_auth.md](specification/06_frame_auth.md)

### Plugin Registration

Files in `app/auth/` are processed by `frame.auth` automatically. For explicit registration:

```clean
plugins:
    frame.auth
```

### Auth Configuration

```clean
auth:
    provider: session  // "session" or "jwt"
    secret: env.get("AUTH_SECRET")
    sessionTtl: 86400  // 24 hours in seconds
    cookieName: "frame_session"
    secureCookie: true
    sameSite: "strict"
```

### Session Management

**Create session:**
```clean
Session session = auth.session.create(userId, claims: {
    email: user.email,
    role: user.role
})
```

**Set session cookie on response:**
```clean
Response response = auth.session.setCookie(session, redirect("/dashboard"))
```

**Get current session:**
```clean
Session? session = auth.session.getCurrent()
if session == null
    return redirect("/login")
integer userId = session.userId
```

**Destroy session:**
```clean
auth.session.destroyCurrent()
return redirect("/login")
```

**Access session claims:**
```clean
Session? session = auth.session.getCurrent()
if session != null
    string role = session.claims.role
    string email = session.claims.email
```

### JWT

**Sign token:**
```clean
Token token = auth.jwt.sign({
    sub: userId,
    email: user.email,
    role: user.role,
    exp: now().plusMinutes(60)
})
string tokenString = token.value
```

**Verify token:**
```clean
Claims? claims = auth.jwt.verify(tokenString)
if claims != null
    integer userId = claims.sub.toInteger()
    string role = claims.role
```

**Refresh token:**
```clean
Token? newToken = auth.jwt.refresh(tokenString)
if newToken == null
    return forbidden("Token expired")
```

### Roles and Permissions

**Role definitions** (`app/auth/roles.cln`):
```clean
roles:
    admin:
        permissions: ["*"]

    editor:
        permissions: ["post.create", "post.update", "post.publish"]

    viewer:
        permissions: ["post.read"]
```

**Check permission:**
```clean
boolean canPublish = auth.can(user, "post.publish")
if !canPublish
    return forbidden("Insufficient permissions")
```

**Check role:**
```clean
boolean isAdmin = auth.hasRole(user, "admin")
```

### Route Guards

**In endpoint blocks:**
```clean
endpoints:
    GET "/admin/*" [admin]:
        return adminHandler()

    POST "/api/posts/publish" [editor, admin]:
        return publishPost()

    GET "/api/profile" [auth]:
        return getProfile()
```

**Guarded route groups** — apply inline `[role]` to each route individually:
```clean
endpoints:
    GET "/dashboard" [auth]:
        return dashboard()
    GET "/profile" [auth]:
        return profile()
    PUT "/profile" [auth]:
        return updateProfile()

    GET "/admin" [admin]:
        return adminDashboard()
    GET "/admin/users" [admin]:
        return listUsers()
    DELETE "/admin/users/:id" [admin]:
        return deleteUser()
```

**`login:` block** - Login flow configuration:
```clean
login:
    endpoint: POST "/auth/login"
    redirect: /dashboard
    failRedirect: /login?error=1
    handler: handleLogin

functions:
    Response handleLogin(Request req)
        string email = req.body.email
        string password = req.body.password

        User? user = User.first: where: email == email
        if user == null
            return redirect("/login?error=invalid")

        boolean valid = auth.verifyPassword(password, user.passwordHash)
        if !valid
            return redirect("/login?error=invalid")

        Session session = auth.session.create(user.id, claims: {
            email: user.email,
            role: user.role
        })
        return auth.session.setCookie(session, redirect("/dashboard"))
```

### Password Utilities

**Hash password:**
```clean
string hash = auth.hashPassword(plainTextPassword)
```

**Verify password:**
```clean
boolean valid = auth.verifyPassword(plainTextPassword, storedHash)
```

### CSRF Protection

CSRF protection is enabled by default for POST, PUT, PATCH, DELETE requests. To include the token in forms:

```html
<form method="POST" action="/api/users">
    <input type="hidden" name="_csrf" value="{csrfToken}">
    <!-- form fields -->
</form>
```

---

## Frame Canvas

**Plugin:** `frame.canvas`
**Owned folders:** `app/canvas/`, `app/canvas/scenes/`
**Handled blocks:** `canvasScene`, `draw`, `onFrame`
**Spec:** [12_frame_canvas.md](specification/12_frame_canvas.md)

### Scene Definition

```clean
canvasScene GameScene
    width: 800
    height: 600

    functions:
        onFrame(number deltaTime)
            // Called every frame with delta time in seconds
            update(deltaTime)
            draw:
                clearCanvas()
                drawBackground()
                drawSprites()

        onMount()
            // Called when scene becomes active
            loadAssets()

        onUnmount()
            // Called when scene is destroyed
            cleanup()
```

### Drawing Operations

**`draw:` block** - Drawing context
```clean
draw:
    // Clear
    clearCanvas()
    fillBackground("#1a1a2e")

    // Shapes
    fillRect(x, y, width, height, color)
    strokeRect(x, y, width, height, color, lineWidth)
    fillCircle(cx, cy, radius, color)
    strokeCircle(cx, cy, radius, color, lineWidth)
    fillPolygon(points, color)

    // Text
    fillText(text, x, y, font, color)
    strokeText(text, x, y, font, color, lineWidth)

    // Images
    drawImage(imageId, x, y)
    drawImageScaled(imageId, x, y, width, height)
    drawImageCropped(imageId, srcX, srcY, srcW, srcH, dstX, dstY, dstW, dstH)

    // Transforms
    save()
    translate(x, y)
    rotate(angle)
    scale(sx, sy)
    restore()
```

### Input Handling

```clean
canvasScene MyScene
    functions:
        onKeyDown(string key)
            if key == "ArrowLeft"
                player.x = player.x - speed

        onKeyUp(string key)
            // Handle key release

        onMouseDown(number x, number y, integer button)
            // Handle mouse click at (x, y)

        onMouseMove(number x, number y)
            // Handle mouse movement

        onTouchStart(number x, number y)
            // Handle touch input
```

---

## Host Bridge

The Host Bridge is the only interface between WASM modules and system resources. All calls use a standard JSON envelope.

**Request envelope:**
```json
{ "fn": "bridge:namespace.function", "args": { ... } }
```

**Success response:**
```json
{ "ok": true, "data": { ... } }
```

**Error response:**
```json
{ "ok": false, "err": { "code": "ERROR_CODE", "message": "...", "details": {} } }
```

Full contracts: [frame_bridge_contracts.md](specification/frame_bridge_contracts.md)

### HTTP Bridge (`bridge:http`)

**`bridge:http.request`** - Make outbound HTTP request
```json
{
  "fn": "bridge:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/data",
    "headers": {"Accept": "application/json"},
    "timeout": 5000
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "status": 200,
    "headers": {"Content-Type": "application/json"},
    "body": "{\"users\": []}"
  }
}
```

### Database Bridge (`bridge:db`)

**`bridge:db.query`** - Execute SQL query
```json
{
  "fn": "bridge:db.query",
  "args": {
    "sql": "SELECT * FROM users WHERE id=$1",
    "params": [42]
  }
}
```

**`bridge:db.tx`** - Execute transaction
```json
{
  "fn": "bridge:db.tx",
  "args": {
    "ops": [
      {"sql": "INSERT INTO users (name) VALUES ($1)", "params": ["Alice"]},
      {"sql": "INSERT INTO posts (title, user_id) VALUES ($1, $2)", "params": ["Hello", 1]}
    ]
  }
}
```

**`bridge:db.exec`** - Execute non-query SQL (DDL, UPDATE, DELETE)
```json
{
  "fn": "bridge:db.exec",
  "args": {
    "sql": "UPDATE users SET active=$1 WHERE id=$2",
    "params": [false, 42]
  }
}
```

### Crypto Bridge (`bridge:crypto`)

**`bridge:crypto.hash`** - Hash data
```json
{
  "fn": "bridge:crypto.hash",
  "args": {
    "algo": "sha256",
    "data": "base64:SGVsbG8="
  }
}
```

**`bridge:crypto.random`** - Generate cryptographically secure random bytes
```json
{
  "fn": "bridge:crypto.random",
  "args": {"bytes": 32}
}
```

**`bridge:crypto.bcrypt`** - Hash password with bcrypt
```json
{
  "fn": "bridge:crypto.bcrypt",
  "args": {
    "password": "plaintext",
    "cost": 12
  }
}
```

**`bridge:crypto.bcryptVerify`** - Verify bcrypt hash
```json
{
  "fn": "bridge:crypto.bcryptVerify",
  "args": {
    "password": "plaintext",
    "hash": "$2b$12$..."
  }
}
```

### Log Bridge (`bridge:log`)

**`bridge:log.info`** - Structured info log
```json
{
  "fn": "bridge:log.info",
  "args": {"event": "user.login", "userId": 42, "ip": "1.2.3.4"}
}
```

**`bridge:log.warn`** - Warning log
```json
{
  "fn": "bridge:log.warn",
  "args": {"event": "rate.limit.approaching", "userId": 42, "count": 90}
}
```

**`bridge:log.error`** - Error log
```json
{
  "fn": "bridge:log.error",
  "args": {"event": "db.query.failed", "error": "connection refused", "sql": "SELECT ..."}
}
```

### Environment Bridge (`bridge:env`)

**`bridge:env.get`** - Read environment variable
```json
{
  "fn": "bridge:env.get",
  "args": {"key": "DATABASE_URL"}
}
```

**Response:**
```json
{"ok": true, "data": {"value": "postgres://..."}
```

### Time Bridge (`bridge:time`)

**`bridge:time.now`** - Current timestamp
```json
{
  "fn": "bridge:time.now",
  "args": {}
}
```

**Response:**
```json
{"ok": true, "data": {"unix": 1700000000, "iso": "2023-11-14T22:13:20Z"}
```

### Filesystem Bridge (`bridge:fs`)

_(server / desktop / CLI only)_

**`bridge:fs.read`** - Read file
```json
{
  "fn": "bridge:fs.read",
  "args": {"path": "/data/config.json"}
}
```

**`bridge:fs.write`** - Write file
```json
{
  "fn": "bridge:fs.write",
  "args": {"path": "/data/output.txt", "content": "Hello"}
}
```

**`bridge:fs.exists`** - Check file existence
```json
{
  "fn": "bridge:fs.exists",
  "args": {"path": "/data/config.json"}
}
```

**`bridge:fs.delete`** - Delete file
```json
{
  "fn": "bridge:fs.delete",
  "args": {"path": "/tmp/temp.txt"}
}
```

### System Bridge (`bridge:sys`)

**`bridge:sys.info`** - System information
```json
{
  "fn": "bridge:sys.info",
  "args": {}
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "platform": "linux",
    "arch": "x86_64",
    "memoryMb": 8192,
    "cpuCount": 4
  }
}
```

### Standard Error Codes

| Code | Description |
|------|-------------|
| `DB_ERROR` | Database operation failed |
| `AUTH_ERROR` | Authentication or authorization failed |
| `VALIDATION_ERROR` | Input validation failed |
| `NOT_FOUND` | Resource not found |
| `FORBIDDEN` | Permission denied |
| `NETWORK_FAIL` | Network or HTTP operation failed |
| `TIMEOUT` | Operation timed out |
| `BUILD_FAIL` | Compilation or build error |
| `FS_ERROR` | Filesystem operation failed |
| `CRYPTO_ERROR` | Cryptographic operation failed |

---

## CLI Commands

**Spec:** [02_frame_cli.md](specification/02_frame_cli.md)

### Project Management

**`frame new <name>`** - Create new project
```bash
cleen project create myapp
cleen project create myapp --template=api
cleen project create myapp --template=fullstack
cleen project create myapp --template=canvas
```

**`frame serve`** - Start development server
```bash
cleen serve
cleen serve --port=3000
cleen serve --port=3000 --verbose
cleen serve --host=0.0.0.0 --port=8080
```

**`frame build`** - Build for production
```bash
cleen build
cleen build --target=web
cleen build --target=server
cleen build --target=server --release
cleen build --target=pwa
cleen build --target=desktop
cleen build --target=mobile
```

### Database Commands

**`frame db:plan`** - Show pending migration plan
```bash
cleen db:plan
```

**`frame db:migrate`** - Apply pending migrations
```bash
cleen db:migrate
cleen db:migrate --up
cleen db:migrate --down
cleen db:migrate --steps=3
```

**`frame db:rollback`** - Roll back last migration
```bash
cleen db:rollback
```

**`frame db:seed`** - Run database seed files
```bash
cleen db:seed
cleen db:seed --file=users
```

**`frame db:reset`** - Drop, recreate, and re-seed database
```bash
cleen db:reset
```

### Plugin Management

**`cleen plugin add <name>`** - Install a plugin
```bash
cleen plugin add frame.data
cleen plugin add frame.auth
cleen plugin add frame.canvas
```

**`cleen plugin list`** - List installed plugins
```bash
cleen plugin list
```

**`cleen plugin remove <name>`** - Remove a plugin
```bash
cleen plugin remove frame.canvas
```

### Code Generation

**`frame api:generate <name>`** - Generate API endpoint scaffold
```bash
cleen api:generate users
cleen api:generate posts --model=Post
```

**`frame component:generate <name>`** - Generate UI component scaffold
```bash
cleen component:generate UserCard
cleen component:generate NavBar --with-styles
```

### Platform Commands

**`frame mobile:init`** - Initialize mobile app support
```bash
cleen mobile:init
cleen mobile:init --platform=ios
cleen mobile:init --platform=android
```

**`frame desktop:init`** - Initialize desktop app support
```bash
cleen desktop:init
cleen desktop:init --platform=macos
cleen desktop:init --platform=windows
```

**`frame pwa:init`** - Initialize Progressive Web App support
```bash
cleen pwa:init
```

### Global Flags

All commands support these global flags:

| Flag | Description |
|------|-------------|
| `--verbose` | Enable verbose output |
| `--json` | Machine-readable JSON output |
| `--target=<platform>` | Target platform override |
| `--config=<path>` | Path to custom config file |

---

## Error Handling

### Error Response Format

All errors from the framework and Host Bridge use this standard envelope:

```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable description",
    "details": {}
  }
}
```

### Error Handling in Clean Language

```clean
User? user = User.first:
    where: id == userId

if user == null
    return notFound("User {userId} not found")

// onError syntax for propagating errors
Data newUser = User.insert:
    name = req.body.name
    email = req.body.email
onError err
    return badRequest("Could not create user: {err.message}")
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource not found |
| `FORBIDDEN` | 403 | Permission denied |
| `AUTH_ERROR` | 401 | Authentication failed |
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `DB_ERROR` | 500 | Database operation failed |
| `NETWORK_FAIL` | 502 | Network or HTTP error |
| `TIMEOUT` | 504 | Operation timed out |
| `BUILD_FAIL` | 500 | Compilation or build error |

---

## Project Structure Reference

See **[PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md)** — canonical reference for folder layout, plugin ownership, and file extension conventions.

---

For more information:
- [Getting Started Guide](./GETTING_STARTED.md)
- [Plugin Guide](./PLUGIN_GUIDE.md)
- [Project Structure](./PROJECT_STRUCTURE.md)
- [Specification Index](./specification/frame_internal_map.md)
