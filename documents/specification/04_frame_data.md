# Frame Data (ORM) Specification (04)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.2 (Unified Query Syntax + Many-to-Many)  
**Location:** `/documents/specification/04_frame_data.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

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

### 3.6 Find or Fail

Throws `NOT_FOUND` if no record matches. Use when existence is required.

```clean
User u = User.findOrFail:
    where:
        id == userId
```

### 3.7 Exists

Returns `true` if at least one matching record exists.

```clean
boolean taken = User.exists:
    where:
        email == newEmail
```

### 3.8 Select (Partial Fields)

Retrieve only specific fields using the `select:` sub-block.

```clean
list<User> users = User.find:
    select:
        id
        name
        email
    where:
        active == true
```

### 3.9 Offset and Pagination

Use `offset:` with `limit:` for page-based pagination.

```clean
integer page = 3
integer pageSize = 20

list<User> users = User.find:
    where:
        active == true
    order:
        createdAt desc
    limit: pageSize
    offset: (page - 1) * pageSize
```

### 3.10 Include (Eager Loading)

Load related records in a single query using the `include:` sub-block.

```clean
list<Post> posts = Post.find:
    include:
        author
        tags
    where:
        published == true
    order:
        createdAt desc
    limit: 10
```

---

## 4. Transactions

Use `transaction:` to perform atomic operations.

```clean
transaction:
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
cleen db:plan
cleen db:migrate
```

### Per-model
```clean
User.migrate()
Post.migrate()
```

**Migration files** live under `app/data/migrations/`:
```
app/data/migrations/
  001_init.sql
  002_add_posts.sql
```

Each migration includes `up` and `down` SQL for rollback.

---

## 7. Configuration

`app/data/config.cln`
```clean
data:
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

### 9.5 Counting or Aggregating

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
transaction:
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

`app/data/seed.cln`
```clean
functions:
    seed()
        User.insert:
            name  = "Admin"
            email = "admin@demo.com"
```

Run with:
```bash
cleen db:seed
```

---

## 11. CLI Summary

| Command | Description |
|----------|-------------|
| `cleen db:plan` | Show migration SQL diff |
| `cleen db:migrate` | Apply migrations |
| `cleen db:seed` | Run database seeding |
| `cleen db:reset` | Drop and recreate schema |
| `cleen db:info` | Show connection metadata |

---

## 12. Host Bridge Contracts

| Bridge | Function | Description |
|---------|-----------|-------------|
| `host:db.query` | Executes SQL and returns rows | Primary query interface |
| `host:db.tx` | Runs multiple queries transactionally | Used by `transaction:` |
| `host:db.prepare` | Prepare SQL statement (optional) | Precompiled queries |
| `host:env.get` | Read database URL or secrets | Configuration |
| `host:log.info` | ORM operations log | Migration/debug logs |

---

## 13. Guidelines

- Always use **block syntax** for queries.
- Keep **data model files small** (one per entity).
- Prefer **atomic transactions** (`transaction:`) for multi-write logic.
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

transaction:
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

## 16. Edge Cases & Important Notes

### 16.1 Migration Behavior

- `Model.migrate()` uses `CREATE TABLE IF NOT EXISTS` — it will NOT alter an existing table
- For schema changes on existing tables, use the `migrate` block with explicit `up:` and `down:` SQL
- Auto-migration diff (`_db_migration_diff`) compares declared fields against the live database schema
- Migrations are applied in alphabetical/numerical order by name

### 16.2 Transaction Limits

- Nested `transaction:` blocks are NOT supported — transactions are flat
- If a transaction block throws, the entire transaction is rolled back automatically
- Long-running transactions may time out depending on the database driver configuration

> **Constraint — no nested transactions:** A `transaction:` block cannot be started inside another `transaction:` block. Attempting to nest transactions is a compile-time error. The outer transaction must complete (commit or roll back) before a new one can be started. Service functions that each use `transaction:` internally must not be composed inside a single outer `transaction:` — extract the composed logic into a single flat transaction block.

### 16.3 Tenant Isolation

- Models with a `tenantId` field or `: tenant` constraint are automatically tenant-scoped
- All `find`, `first`, `count`, `update`, and `delete` queries include `WHERE tenant_id = <current_tenant>`
- `insert` operations auto-inject the current tenant ID
- Tenant ID is read from the session via `tenant_getId()` (requires frame.auth)
- If no session exists (unauthenticated request), tenant-scoped queries will use an empty tenant ID

---

## 17. Pagination — Offset-Based (`paginate:`)

The `paginate:` sub-block replaces manual `LIMIT`/`OFFSET` arithmetic and eliminates the need for a separate `COUNT(*)` query. The plugin computes the total count and page metadata in a single database round-trip.

Using `paginate:` and `limit:`/`offset:` in the same query is a compile-time error — they are mutually exclusive strategies.

Queries that use `paginate:` return `PagedResult<T>` instead of `Array<T>`.

### `PagedResult<T>` Fields

| Field | Type | Description |
|---|---|---|
| `items` | `Array<T>` | The records for this page |
| `totalCount` | `integer` | Total matching records across all pages |
| `totalPages` | `integer` | `ceil(totalCount / perPage)` |
| `currentPage` | `integer` | The page requested (1-based) |
| `perPage` | `integer` | Records per page |
| `hasNext` | `boolean` | `currentPage < totalPages` |
| `hasPrev` | `boolean` | `currentPage > 1` |

### Example

```clean
endpoints:
    GET "/api/posts":
        integer page    = int(req.query("page") ?? "1")
        integer perPage = 20

        PagedResult<Post> result = Post.find:
            where:
                published == true
            order:
                createdAt desc
            paginate:
                page:    page
                perPage: perPage

        return json({
            items:       result.items,
            totalPages:  result.totalPages,
            currentPage: result.currentPage,
            hasNext:     result.hasNext
        })
```

---

## 18. Pagination — Cursor-Based (`cursor:`)

Cursor-based pagination is more efficient than offset pagination for large datasets because it does not issue a `COUNT(*)` query and does not scan skipped rows. Use it for infinite scroll and feeds where the absolute page number does not matter.

Queries that use `cursor:` return `CursorResult<T>` instead of `Array<T>`.

### `cursor:` Sub-Fields

| Field | Description |
|---|---|
| `after:` | Cursor value from the previous page's `nextCursor`; pass `""` to start from the beginning |
| `perPage:` | Maximum records to return |
| `by:` | The field used as the cursor key — must be monotonically ordered (typically `id` or `createdAt`) |

### `CursorResult<T>` Fields

| Field | Type | Description |
|---|---|---|
| `items` | `Array<T>` | The records for this page |
| `nextCursor` | `string` | Opaque cursor to pass as `after:` for the next page; `""` if no more pages |
| `hasPrev` | `boolean` | Whether a previous page exists |
| `count` | `integer` | Number of items in this page (may be < `perPage` on the last page) |

### Example

```clean
endpoints:
    GET "/api/feed":
        string  cursor  = req.query("cursor") ?? ""
        integer perPage = 25

        CursorResult<Post> result = Post.find:
            where:
                published == true
            order:
                id desc
            cursor:
                after:   cursor
                perPage: perPage
                by:      id

        return json({
            items:      result.items,
            nextCursor: result.nextCursor
        })
```

---

## 19. Dynamic Sort Safety

When an `order:` clause uses a runtime variable for the field name, the plugin validates the value against the model's declared fields before executing the query. An unknown field name produces a `ValidationError`, not a SQL error — this prevents SQL injection through sort parameters.

```clean
// Safe — field name is validated at query time even though it comes from user input
string sortField = req.query("sort")
string sortDir   = req.query("dir")

Array<Post> posts = Post.find:
    order: sortField sortDir   // ValidationError if sortField is not a Post field
    limit: 20
```

The validation is performed at the ORM layer before any SQL is generated. The allowed values for `sortField` are exactly the field names declared on the model; any other value is rejected with `VALIDATION_ERROR`.

---

## 20. Runtime Validation — `Model.validate(field, value)`

Run validation rules at call sites that aren't a full `insert:` / `update:` (e.g. a "is this email available?" check before showing a form, or partial-update validation).

```clean
string error = User.validate("email", req.json("email"))
if error.length > 0:
    return badRequest(json({ field: "email", error: error }))
```

| Returns | Meaning |
|---|---|
| `""` (empty) | Value is valid for the field |
| Non-empty string | Validation message (e.g. `"must be a valid email address"`) |

The function checks the rules declared on the model field (`min`, `max`, `email`, `range`, custom validators). It does not write to the database.

---

## 21. Common Patterns

### 21.1 Soft delete

Add a nullable `deletedAt` timestamp; never `DELETE FROM`, always filter on it:

```clean
data User:
    integer id            primary
    string  email         unique
    string  name
    timestamp deletedAt?  // nullable — null means active
```

Standard query helper:

```clean
functions:
    Array<User> activeUsers():
        return User.find:
            where:
                deletedAt == null
```

Soft-delete instead of physical:

```clean
User.update:
    where:
        id == userId
    set:
        deletedAt = time.now()
```

### 21.2 Multi-tenant scoping

Every tenant-scoped model carries a `tenantId` foreign key. Every query in tenant context calls `tenant_getId()` (provided by frame.auth — requires session claims with a tenant id).

```clean
data Project:
    integer id           primary
    integer tenantId     foreign User.tenantId
    string  name
    timestamp createdAt
```

Query:

```clean
Array<Project> projects = Project.find:
    where:
        tenantId == tenant_getId()
    order:
        createdAt desc
```

`tenant_getId()` returns 0 when called outside a session context (e.g. background jobs); guard accordingly. See [06_frame_auth.md §8](06_frame_auth.md#8-multi-tenant) for the underlying claims contract.

### 21.3 Audit log (append-only)

```clean
data AuditEvent:
    integer id           primary
    integer actorId      foreign User.id
    string  action       // "user.created", "post.deleted", ...
    string  details      // JSON payload as string
    timestamp at         default = time.now()
```

Never `update:` or `delete:` audit rows; only `insert:`. Add an index on `(actorId, at desc)` for actor-history queries.

### 21.4 Tree (self-referential)

```clean
data Category:
    integer id        primary
    integer parentId? foreign Category.id   // null for roots
    string  name
    integer depth     // denormalized — keep in sync on insert/update
```

Self-reference requires `parentId?` to be nullable; the root has `parentId == null`.

---

## 22. Plugin Contracts v2 Integration

frame.data **2.1.x** opts into [lifecycle.md §3.1](../../../foundation/spec/plugins/contracts/lifecycle.md#31-module_helpers) with `module_helpers_are_roots = true`. Preamble-emitted ORM helpers (`PagedResult` / `CursorResult` constructors, query builders) are reachable only through plugin-generated CRUD code, so they must be tree-shake roots. Every `_db_*` bridge declares `hosts = ["server"]` ([bridge-host-classes.md §2](../../../foundation/spec/plugins/contracts/bridge-host-classes.md#2-the-hosts-field)); the database driver only loads on hosts that ship a runtime DB connection.

---

**End of Document 04 — Frame Data (ORM) Specification (Unified Block Syntax)**

