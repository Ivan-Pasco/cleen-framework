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

### Type Conversion

**`toInteger()`** - Convert to integer
```clean
number decimal = 3.14
integer whole = decimal.toInteger()  // 3
```

**`toNumber()`** - Convert to number
```clean
integer count = 42
number decimal = count.toNumber()    // 42.0
```

**`toString()`** - Convert to string
```clean
integer age = 25
string text = age.toString()         // "25"
```

**`toBoolean()`** - Convert to boolean
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

### File Module

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
- `min=<value>` - Minimum value
- `max=<value>` - Maximum value

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

### Transactions

**`Data.tx:`** - Execute transaction
```clean
Data.tx:
    User user = User.insert:
        name = "Alice"
        email = "alice@example.com"

    Post.insert:
        title = "Hello"
        author = user
        published = true
```

### Relationships

**One-to-Many**:
```clean
data User
    integer id : pk, auto
    string name

data Post
    integer id : pk, auto
    string title
    User author  // Foreign key

// Query
list<Post> posts = Post.find:
    where:
        author.id == userId
```

**Many-to-Many**:
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

// Query users with role
list<User> admins = User.find:
    link:
        UserRole user == id
        Role id == UserRole.role
    where:
        Role.name == "admin"
```

---

## Frame Server

### Request/Response

**Request Object**:
```clean
Request req
    string method      // "GET", "POST", etc.
    string path        // "/api/users"
    map headers        // HTTP headers
    string body        // Request body
    map query          // Query parameters
    map params         // Path parameters
```

**Response Helpers**:
```clean
// JSON response
Response json(map data)
    return Response(200, data, {"Content-Type": "application/json"})

// Redirect
Response redirect(string url)
    return Response(302, "", {"Location": url})

// Error
Response error(integer status, string message)
    return Response(status, {error: message})
```

### Middleware

```clean
middleware MiddlewareName
    functions:
        Request handle(Request req)
            // Process request
            // Return modified request or error
            return req

// Apply to route
route /api/protected/*
    middleware: [RequireAuth]
```

### File-Based Routing

**API Routes** (`app/api/`):
```
users.cln          → /api/users
users/[id].cln     → /api/users/:id
posts/[id]/comments.cln → /api/posts/:id/comments
```

**Page Routes** (`app/pages/`):
```
index.cln          → /
about.cln          → /about
blog/[slug].cln    → /blog/:slug
```

---

## Frame UI

### Component Definition

```clean
component ComponentName
    props:
        prop1: Type
        prop2?: Type = defaultValue

    functions:
        Widget render()
            return (/* HTML */)

        // Lifecycle hooks
        onMount()
        onUnmount()
        onUpdate()
```

### HTML Syntax

**Elements**:
```clean
<div class="container">
    <h1>Title</h1>
    <p>Paragraph</p>
</div>
```

**Interpolation**:
```clean
<p>{variable}</p>
<p>Count: {count}</p>
<p>User: {user.name}</p>
```

**Conditionals**:
```clean
if condition
    <p>Shown when true</p>
else
    <p>Shown when false</p>
```

**Loops**:
```clean
iterate item in list
    <li>{item.name}</li>
```

**Events**:
```clean
<button onClick="handleClick">Click me</button>
<input onInput="handleInput" />
<form onSubmit="handleSubmit">...</form>
```

### Rendering Modes

**`client` attribute**:
```html
<!-- SSR only (default) -->
<component-name></component-name>

<!-- SSR + immediate hydration -->
<component-name client="on"></component-name>

<!-- SSR + hydrate when visible -->
<component-name client="visible"></component-name>

<!-- SSR + hydrate when idle -->
<component-name client="idle"></component-name>

<!-- Client-only (no SSR) -->
<component-name client="only"></component-name>
```

---

## Frame Auth

### Session Management

**Create session**:
```clean
Session session = auth.session.create(userId, claims: {
    email: user.email,
    role: user.role
})
```

**Set session cookie**:
```clean
Response response = auth.session.setCookie(session, redirect("/dashboard"))
```

**Get current session**:
```clean
Session? session = auth.session.getCurrent()
```

**Destroy session**:
```clean
auth.session.destroyCurrent()
```

### JWT

**Sign token**:
```clean
Token token = auth.jwt.sign({
    sub: userId,
    email: user.email,
    role: user.role,
    exp: now().plusMinutes(60)
})
```

**Verify token**:
```clean
Claims? claims = auth.jwt.verify(tokenString)
if claims != null
    // Valid token
```

### Permissions

**Check permission**:
```clean
boolean canPublish = auth.can(user, "post.publish")
```

**Route guards**:
```clean
route /admin/*
    guard: role("admin")

route /api/posts/publish
    guard: permission("post.publish")
```

---

## Host Bridge

### HTTP Bridge

**`host:http.request`** - Make HTTP request
```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/data",
    "headers": {"Accept": "application/json"},
    "timeout": 5000
  }
}
```

### Database Bridge

**`host:db.query`** - Execute SQL query
```json
{
  "fn": "host:db.query",
  "args": {
    "sql": "SELECT * FROM users WHERE id=$1",
    "params": [42]
  }
}
```

**`host:db.tx`** - Execute transaction
```json
{
  "fn": "host:db.tx",
  "args": {
    "ops": [
      {"sql": "INSERT INTO users (name) VALUES ($1)", "params": ["Alice"]},
      {"sql": "INSERT INTO posts (title, user_id) VALUES ($1, $2)", "params": ["Hello", 1]}
    ]
  }
}
```

### Crypto Bridge

**`host:crypto.hash`** - Hash data
```json
{
  "fn": "host:crypto.hash",
  "args": {
    "algo": "sha256",
    "data": "base64:SGVsbG8="
  }
}
```

**`host:crypto.random`** - Generate random bytes
```json
{
  "fn": "host:crypto.random",
  "args": {"bytes": 32}
}
```

### Log Bridge

**`host:log.info`** - Info log
```json
{
  "fn": "host:log.info",
  "args": {"event": "user.login", "userId": 42}
}
```

---

## CLI Commands

### Project Management

**`frame new <name>`** - Create new project
```bash
frame new myapp
frame new myapp --template=api
```

**`frame serve`** - Start dev server
```bash
frame serve
frame serve --port=3000 --verbose
```

**`frame build`** - Build for production
```bash
frame build
frame build --target=web
frame build --target=server --release
```

### Database

**`frame db:plan`** - Show migration plan
```bash
frame db:plan
```

**`frame db:migrate`** - Run migrations
```bash
frame db:migrate
frame db:migrate --up
frame db:migrate --down
```

**`frame db:seed`** - Seed database
```bash
frame db:seed
```

### Platform

**`frame mobile:init`** - Initialize mobile app
```bash
frame mobile:init
```

**`frame desktop:init`** - Initialize desktop app
```bash
frame desktop:init
```

**`frame pwa:init`** - Initialize PWA
```bash
frame pwa:init
```

---

## Error Handling

### Error Response Format

```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": {}
  }
}
```

### Common Error Codes

- `DB_ERROR` - Database operation failed
- `AUTH_ERROR` - Authentication failed
- `VALIDATION_ERROR` - Input validation failed
- `NOT_FOUND` - Resource not found
- `FORBIDDEN` - Permission denied
- `NETWORK_FAIL` - Network operation failed
- `TIMEOUT` - Operation timed out

---

For more information:
- [Architecture Guide](./ARCHITECTURE.md)
- [Functional Specification](./FUNCTIONAL_SPEC.md)
- [Knowledge Base](./KNOWLEDGE_BASE.md)
- [Contributing Guide](./CONTRIBUTING.md)
