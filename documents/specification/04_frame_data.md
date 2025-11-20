# Frame Data (ORM) Specification (04)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.2 (Unified Query Syntax + Many-to-Many)  
**Location:** `/docs/specification/04_frame_data.md`

---

## 1. Purpose

**Frame Data** is the integrated ORM of the Clean Language ecosystem.  
It provides a **block-based, fully declarative syntax** for defining, querying, and manipulating data.  
All ORM actions follow Clean’s indentation and minimalism principles — there is **only one way** to declare and query data.

---

## 2. Model Definition

### 2.1 Declaring Models
Use the `data` keyword to define persisted entities.

```clean
data User
    integer id : pk, auto
    string  name
    string  email : unique
    integer age
    boolean active = true
    datetime createdAt : default=now
```

### 2.2 Relations
Relations are typed and automatically inferred.

```clean
data Post
    string title
    string content
    User   author
```

This declaration generates:
- A foreign key from `Post.author` → `User.id`.
- Automatic reverse property `user.posts`.

---

## 3. CRUD Operations

### 3.1 Insert

```clean
User.insert:
    name  = "Alice"
    email = "alice@example.com"
    age   = 30
    active = true
```

### 3.2 Select

#### Simple Select
```clean
list<User> users = User.find:
    where:
        active == true
```

#### Filtered with Ordering and Limit
```clean
list<User> users = User.find:
    where:
        active == true
        age >= 18
    order:
        createdAt desc
    limit: 20
```

#### Retrieve One Record
```clean
User? u = User.first:
    where:
        email == "alice@example.com"
```

### 3.3 Update

Updates can target individual records or conditions.

```clean
User.update:
    set:
        active = false
    where:
        email == "alice@example.com"
```

### 3.4 Delete
```clean
User.delete:
    where:
        age < 18
```

### 3.5 Count
```clean
integer total = User.count:
    where:
        active == true
```

---

## 4. Transactions

Use `Data.tx:` to perform atomic operations.

```clean
Data.tx:
    User u = User.insert:
        name = "Ana"
        email = "ana@example.com"

    Post p = Post.insert:
        title   = "Welcome"
        author  = u
```

If any step fails, all changes roll back automatically.

---

## 5. Raw Queries (Typed SQL)

Use `db.queryAs` for typed custom SQL:

```clean
list<User> users = db.queryAs(User):
    sql: "SELECT * FROM users WHERE age > ?"
    params: [18]
```

Or untyped for ad-hoc operations:

```clean
list<map<string, any>> result = db.query:
    sql: "SELECT COUNT(*) FROM users"
```

---

## 6. Migrations

Frame generates migrations automatically based on model diffs.

### CLI
```bash
frame db:plan
frame db:migrate
```

### Per-model
```clean
User.migrate()
Post.migrate()
```

**Migration files** live under `/db/migrations/`:
```
/db/migrations/
  001_init.sql
  002_add_posts.sql
```

Each migration includes `up` and `down` SQL for rollback.

---

## 7. Configuration

`/config/data.cln`
```clean
data:
    default:
        engine = "postgres"
        host = "localhost"
        port = 5432
        database = "frame_app"
        user = "admin"
        password = env("DB_PASSWORD")
        pool:
            max = 10
            idleTimeout = 30000
```

Default engine: `sqlite` (local development).  
Supported engines: `postgres`, `mysql`, `sqlite`.

---

## 8. Validation

All models and operations are validated at compile-time and runtime.

| Type | Validation |
|------|-------------|
| string | `min`, `max`, regex, email |
| integer | range |
| boolean | truthy values only |
| relation | existence & FK constraint |
| all | respect default values and required fields |

**Example error**
```json
{ "ok": false, "err": { "code": "VALIDATION_ERROR", "field": "email", "message": "must be unique" } }
```

---

## 9. Many-to-Many Relationships

In Frame Data, **many-to-many** relationships are defined explicitly through a **junction model**.  
This keeps the system simple, predictable, and fully consistent with Clean Language principles — there is **only one clear way** to represent many-to-many data.

### 9.1 Defining a Junction Model

```clean
data User
    integer id : pk, auto
    string  name
    string  email : unique

data Role
    integer id : pk, auto
    string  name : unique

data UserRole
    User user  : onDelete=cascade
    Role role  : onDelete=cascade
    unique user, role
```

- `UserRole` is the single source of truth for the relation.
- Use `unique user, role` or `primary user, role` for duplicates protection.
- Use `onDelete=cascade` to clean orphan links automatically.

### 9.2 Creating and Removing Links

```clean
UserRole.insert:
    user = someUser
    role = someRole

UserRole.delete:
    where:
        user == someUser
        role == someRole
```

### 9.3 Link Syntax

The `link:` block defines relationships in a simple, context-aware way.
Inside a `find:` block, the unqualified field `id` refers to the current model, and fields written after the linked model name belong to that model.

Example:
```clean
link:
    UserRole role == id
```
means *link the UserRole model where its `role` field matches the current model’s `id`*.

Multiple links can be listed on separate lines:
```clean
link:
    UserRole user == id
    Role     id == UserRole.role
```

### 9.4 Querying Many-to-Many Relations

#### Roles for a given user
```clean
list<Role> roles = Role.find:
    link:
        UserRole role == id
    where:
        UserRole.user == someUser
    order:
        name asc
```

#### Users with a given role
```clean
list<User> users = User.find:
    link:
        UserRole user == id
        Role     id == UserRole.role
    where:
        Role.name == "admin"
```

#### Users with at least one role
```clean
list<User> users = User.find:
    link:
        UserRole user == id
    where:
        UserRole.role != null
```

### 9.4 Counting or Aggregating

```clean
list<map<string, any>> counts = db.query:
    sql: """
        SELECT u.id, u.name, COUNT(ur.role) AS total
        FROM users u
        JOIN user_roles ur ON ur.user = u.id
        GROUP BY u.id, u.name
        ORDER BY total DESC
    """
```

### 9.5 Using Transactions

```clean
Data.tx:
    User u = User.insert:
        name  = "Alice"
        email = "alice@x.com"

    Role r = Role.first:
        where:
            name == "admin"

    UserRole.insert:
        user = u
        role = r
```

### 9.6 Naming and Conventions

| Convention | Description |
|-------------|--------------|
| **Model name** | PascalCase combining entities (e.g., `UserRole`, `PostTag`) |
| **Table name (DB)** | Auto-generated (e.g., `user_roles`, `post_tags`) |
| **Composite keys** | Use `unique user, role` or `primary user, role` |
| **Cascade deletes** | Recommended for data integrity |

### 9.7 Why Explicit Junctions

- **Transparent** — relationships are visible and controllable.  
- **Consistent** — same syntax as all other `data` models.  
- **Predictable** — clear structure for compilers and tools.  
- **One obvious way** — no alternate `many<>` syntax.

---

## 10. Seeds

`/db/seed.cln`
```clean
functions:
    seed()
        User.insert:
            name  = "Admin"
            email = "admin@demo.com"
```

Run with:
```bash
frame db:seed
```

---

## 11. CLI Summary

| Command | Description |
|----------|-------------|
| `frame db:plan` | Show migration SQL diff |
| `frame db:migrate` | Apply migrations |
| `frame db:seed` | Run database seeding |
| `frame db:reset` | Drop and recreate schema |
| `frame db:info` | Show connection metadata |

---

## 12. Host Bridge Contracts

| Bridge | Function | Description |
|---------|-----------|-------------|
| `host:db.query` | Executes SQL and returns rows | Primary query interface |
| `host:db.tx` | Runs multiple queries transactionally | Used by `Data.tx` |
| `host:db.prepare` | Prepare SQL statement (optional) | Precompiled queries |
| `host:env.get` | Read database URL or secrets | Configuration |
| `host:log.info` | ORM operations log | Migration/debug logs |

---

## 13. Guidelines

- Always use **block syntax** for queries.
- Keep **data model files small** (one per entity).
- Prefer **atomic transactions** (`Data.tx:`) for multi-write logic.
- Avoid embedding business logic in models — use services.
- Migrations should be committed with related code.

---

## 14. Example End-to-End Flow

```clean
User.migrate()
Post.migrate()

User.insert:
    name = "Alice"
    email = "alice@x.com"

list<User> admins = User.find:
    where:
        active == true
        age >= 18
    order:
        createdAt desc

Data.tx:
    User u = User.first:
        where:
            email == "alice@x.com"

    Post.insert:
        title   = "Hello"
        content = "Clean ORM is amazing"
        author  = u
```

---

## 15. Summary

Frame Data uses **only block-based ORM syntax** to ensure consistency, clarity, and minimal mental load.  
It turns database interaction into clean, declarative statements — readable for humans, predictable for compilers, and intuitive for AI tools.

---

**End of Document 04 — Frame Data (ORM) Specification (Unified Block Syntax)**

