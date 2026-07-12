# Frame Data (ORM) Specification (04)

**Project:** Frame — Full-Stack Framework for Clean Language
**Version:** 2.0 (Entity/Data Pairing + `.data` Accessor + `Database` Service)
**Location:** `/documents/specification/04_frame_data.md`
**Committed spec:** `clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md`
**Plugin spec:** `foundation/spec/plugins/frame-data.ebnf` v2.0.0, `foundation/spec/plugins/frame-data-semantics.md` v2.0.0

---

> **See also:**
> - [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities.
> - [SPEC_DATA_PERSISTENCE_MODEL.md](../../system-documents/SPEC_DATA_PERSISTENCE_MODEL.md) — the full committed design spec with rationales.

## 1. Purpose

**Frame Data** is the integrated ORM of the Clean Language ecosystem.

Frame Data separates persistence into three layers, each with a distinct home in the folder tree:

- **Domain layer** — `app/entity/<name>.cln` — plain classes with fields, invariants, and business behavior. Persistence-ignorant.
- **Data layer** — `app/data/models/<name>.cln` — `data:` blocks paired with domain classes by shared name. Declare table mapping, storage-side fields, indexes, relations, and query methods.
- **Runtime layer** — `Database` — a top-level service provided by `frame.data`. Handles mutations (`save`, `delete`), transactions, connection routing, and lifecycle.

Two persistence surfaces are exposed to application code:

- **Reads** — via the entity's `.data` accessor: `User.data.findByEmail(email)`.
- **Writes** — via the `Database` service: `Database.save(user)`, `Database.delete(user)`, `Database.deleteOrFail(user)`.
- **Transactions** — via the top-level `transaction:` block.

The rule: **reads scope to the entity; writes are cross-cutting.**

> **v2 replaces v1.** No backwards compatibility. `Model.insert:`, `Model.update:`, `Model.delete:`, `Model.upsert:` block-form mutations from v1 are removed; use `Database.save` / `Database.delete` / `Database.deleteOrFail`. Bare-field `data:` declarations from v1 are removed; use the sub-block form. See §14 (Migration from v1) for the transition path.

---

## 2. Domain Class Definition (`app/entity/`)

### 2.1 Declaring an Entity

Entity classes live in `app/entity/<name>.cln`. Each file declares one class using core Clean class syntax. Entity-ness comes from folder location plus name-based pairing with a `data:` block — no new keyword is introduced.

```clean
class User
    integer? id
    string email
    private string passwordHash
    string status
    datetime createdAt

    always:
        status in ["active", "pending", "suspended"]

    functions:
        public:
            boolean canPost()
                return status == "active"

            boolean verifiesPassword(string password)
                return password.verify(passwordHash)

            void promote()
                require status == "pending"
                status = "active"
```

**Rules:**

- File basename matches the class name in snake_case (`class User` in `user.cln`; `class UserProfile` in `user_profile.cln`).
- One entity per file.
- The `id` field is nullable (`integer?` or `string?`). It is `null` before persistence; `Database.save` mutates it in place to the DB-generated value after INSERT.
- Domain classes are persistence-ignorant. They cannot import from `app/data/`, `app/logic/`, or `app/server/`.
- `always:` invariants are enforced by `Database.save` before persisting. Invariant failure raises `RUN005` and does not persist.
- Business methods live in the `functions:` block with a `public:` sub-section for methods callable from outside.

### 2.2 Computed / Derived Properties

Domain properties that are computed from other fields (not stored) are methods, not fields:

```clean
class User
    string firstName
    string lastName
    string email

    functions:
        public:
            string fullName()
                return firstName + " " + lastName

            string emailDomain()
                return email.after("@")
```

Callers write `user.fullName()` with parentheses.

---

## 3. Data Block Definition (`app/data/models/`)

### 3.1 Declaring a Data Block

Data blocks live in `app/data/models/<name>.cln` and pair with the entity of the same name via the `data <T>:` sub-block form. The `<T>` after `data` must exactly match a `class <T>` declaration in `app/entity/<basename>.cln`.

```clean
data User:
    table "users"

    fields:
        id primary generated
        email required unique
        passwordHash as "password_hash" required
        status required
        createdAt as "created_at" required

    indexes:
        (status, createdAt)
        email

    relations:
        orders: has_many Order on user_id
        profile: has_one Profile on user_id

    queries:
        User findOrFailById(integer userId)
            return User.findOrFail:
                where:
                    id == userId

        User? findByEmail(string emailAddress)
            return User.first:
                where:
                    email == emailAddress

        list<User> findActive()
            return User.find:
                where:
                    status == "active"

        list<User> findRecentByStatus(string status, integer resultLimit)
            return User.find:
                where:
                    status == status
                order:
                    createdAt desc
                limit: resultLimit
```

**Rules:**

- File basename matches the paired entity's basename.
- One data block per file.
- The `fields:` sub-block is required; every persisted field is declared here.
- Field types come from the paired entity (not restated).
- Every field in `fields:` must exist on the paired entity (field alignment enforced at plugin-compile time).
- Every non-optional entity field must appear in `fields:` (or be a method declared in the entity's `functions:` block, which is not a field).

### 3.2 Sub-blocks

**`table "<name>"`** — SQL table name. Defaults to snake_case pluralization of the class name (`User` → `users`, `UserProfile` → `user_profiles`).

**`fields:`** — required. Declares which entity fields are persisted with per-field constraints and column-name overrides.

Field constraints:
- `primary` — primary key.
- `generated` — DB-generated (typical for `id`).
- `required` — non-null.
- `unique` — unique index (shorthand for single-column uniqueness).
- `default: <expr>` — storage-side default when the field is absent on insert.
- `as "<column_name>"` — override the SQL column name (used for snake_case ↔ camelCase mappings, historical naming, or renames).

**`indexes:`** — declares indexes. Each entry is either a single field name (`email`) or a parenthesized tuple (`(status, createdAt)`). Suffix `unique` to mark a unique index.

**`relations:`** — declares foreign-key relationships. Each entry has the form `<name>: <cardinality> <TargetClass> on <fk_column>`. Cardinalities: `has_many`, `has_one`, `belongs_to`. The `<name>` is used in `include:` (eager loading), in filter-scope references inside `where:` when included, and generates an auto-reverse property on the target.

**`queries:`** — declares read-only methods that return the paired entity type. See §4 (Query DSL) for method body syntax.

### 3.3 Restrictions

A data block cannot contain:
- A `functions:` block. Non-query methods belong on the entity or in `logic/`.
- An `always:` block. Invariants live on the entity.
- A `constructor`. Data blocks are not instantiable.

---

## 4. Query DSL

The query DSL is invoked on the entity class name with a dot accessor and a verb, followed by a block body with sub-blocks. Used both inside `queries:` sub-blocks on data blocks (defining reusable queries) and inline in `logic/` for one-off queries.

### 4.1 Query Verbs

| Verb | Returns | Semantics |
|---|---|---|
| `find` | `list<T>` | Returns matching rows (possibly empty). |
| `first` | `T?` | Returns first matching row or `null`. |
| `findOrFail` | `T` | Returns first matching row; raises `NOT_FOUND` if none. |
| `count` | `integer` | Returns count of matching rows. |
| `exists` | `boolean` | Returns `true` if at least one row matches. |

Verb-to-return-type correspondence is enforced by the compiler.

### 4.2 Simple Select

```clean
list<User> users = User.find:
    where:
        status == "active"
```

### 4.3 Filtered with Ordering and Limit

```clean
list<User> users = User.find:
    where:
        status == "active"
        age >= 18
    order:
        createdAt desc
    limit: 20
```

Field references inside `where:` and `order:` are unqualified — the field name refers to the current row's column on the queried entity.

### 4.4 Retrieve One Record

```clean
User? u = User.first:
    where:
        email == "alice@example.com"
```

### 4.5 Find Or Fail

Raises `NOT_FOUND` if no row matches.

```clean
User u = User.findOrFail:
    where:
        id == userId
```

### 4.6 Count and Exists

```clean
integer total = User.count:
    where:
        status == "active"

boolean taken = User.exists:
    where:
        email == candidateEmail
```

### 4.7 Column Projection

Retrieve only specific fields with `select:`. Return type remains the model; unselected columns receive zero values.

```clean
list<User> users = User.find:
    select:
        id
        email
    where:
        status == "active"
```

### 4.8 Pagination

Offset-based with `limit:` and `offset:`:

```clean
integer page = 3
integer perPage = 20

list<User> users = User.find:
    where:
        status == "active"
    order:
        createdAt desc
    limit: perPage
    offset: (page - 1) * perPage
```

Page-based with `paginate:`:

```clean
PagedResult<User> result = User.paginate:
    where:
        status == "active"
    page: 3
    perPage: 20
```

Cursor-based (keyset) pagination:

```clean
CursorResult<User> result = User.cursor:
    where:
        status == "active"
    after: previousCursor
    perPage: 20
    by: id
```

### 4.9 Eager Loading (`include:`)

Load related records in a single query:

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

`include:` triggers eager loading (JOIN or IN clause). Guaranteed N+1-free.

### 4.10 Filtering on Included Relations

When a relation is `include:`d, its columns become referenceable in `where:` via the relation name:

```clean
list<User> users = User.find:
    include:
        orders
    where:
        tenantId == currentTenantId
        orders.status == "pending"
```

The join is explicit at the source level. Filtering on a relation that isn't `include:`d is a compile-time error.

### 4.11 Many-to-Many via `link:`

For many-to-many relationships, use `link:` with a junction model (see §9).

---

## 5. Persistence Operations (`Database` Service)

The `Database` service is the top-level object providing mutation and bulk operations. Its methods are called statically as `Database.<method>(entity)`.

### 5.1 Save

`Database.save(entity)` inserts if the entity's `id` is `null`; updates otherwise. On INSERT, mutates `entity.id` in place to the DB-generated value.

```clean
User newUser = User(
    email: emailInput,
    passwordHash: passwordInput.hash(),
    status: "pending",
    createdAt: time.now()
)

Database.save(newUser)
// newUser.id is now populated
```

Before persisting, `Database.save` evaluates the entity's `always:` invariants. Any invariant failure raises `RUN005 — Assertion Failure` and does not persist.

### 5.2 Delete

Two variants, mirroring the `first` vs `findOrFail` pattern on reads:

```clean
Database.delete(user)         // lenient — no-op if row missing
Database.deleteOrFail(user)   // strict — raises NOT_FOUND if row missing
```

Use `Database.delete` for retry-safe / idempotent flows. Use `Database.deleteOrFail` when the caller needs an explicit signal that the row existed.

### 5.3 Bulk Save and Delete

```clean
list<User> newUsers = [...]
Database.saveAll(newUsers)

list<User> toRemove = [...]
Database.deleteAll(toRemove)
```

`Database.deleteAll` follows lenient semantics per element (no `Database.deleteAllOrFail` in v3.0).

Inside a `transaction:` block, bulk operations commit atomically with the surrounding transaction.

### 5.4 Update via Load-Mutate-Save

There is no `update:` block form. Update by loading the entity, mutating it, and calling `Database.save`:

```clean
User user = User.data.findOrFailById(userId)
user.promote()
Database.save(user)
```

### 5.5 What `Database` does NOT expose

- `Database.find` / `Database.query` — reads use the query DSL on the entity or via the `.data` accessor.
- `Database.transaction` — the top-level `transaction:` block is the transaction primitive (§6).
- `Database.readReplica` / `Database.primary` — reserved for v3.1+. v3.0 is single-connection.

`Database` cannot be captured as a runtime value. It is a compile-time namespace whose methods dispatch to plugin-generated code.

---

## 6. The `.data` Accessor

Query methods declared inside a data block's `queries:` sub-block are callable from `logic/` via the entity's `.data` accessor:

```clean
// app/data/models/user.cln
data User:
    ...
    queries:
        User? findByEmail(string emailAddress)
            return User.first:
                where:
                    email == emailAddress

// app/logic/auth.cln
class Auth
    functions:
        public:
            User? lookupByEmail(string emailInput)
                return User.data.findByEmail(emailInput)
```

**Rules:**

- `<Entity>.data.<method>` is a compile-time namespace, not a runtime value. `var x = User.data` is an error.
- Only methods declared in the paired data block's `queries:` sub-block are dispatchable.
- Query methods must return `T`, `T?`, or `list<T>` where `T` is the paired entity. Aggregate returns and cross-entity types belong in report classes (see §11).
- Query method bodies are a single `return` statement invoking the query DSL. No local variables, arithmetic, or conditionals — the body is a direct DSL call.
- Query methods have implicit `public` visibility; no `public:` sub-section required.

---

## 7. Transactions

Multiple mutations can be grouped atomically using the top-level `transaction:` block.

```clean
transaction:
    User user = User(
        email: "ana@example.com",
        status: "pending",
        createdAt: time.now()
    )
    Database.save(user)

    Profile profile = Profile(
        userId: user.id!,
        displayName: "Ana"
    )
    Database.save(profile)
```

**Semantics:**

- Any error inside the block triggers automatic rollback of all writes.
- Reads inside a `transaction:` block use the transaction's connection and see uncommitted writes made earlier in the same transaction.
- Nested `transaction:` blocks are a compile-time error. Extract composed logic into a single flat block.
- Long-running transactions may hit driver-configured timeouts (default 30 seconds).

---

## 8. Raw Queries (Escape Hatch)

For queries the DSL doesn't express (aggregations, window functions, vendor-specific SQL, deep joins), use raw SQL. These typically live inside report classes (§11).

Typed:

```clean
list<UserSummary> summaries = db.queryAs(UserSummary):
    sql: "SELECT id, email, COUNT(*) AS post_count FROM users JOIN posts ON ..."
    params: []
```

Untyped:

```clean
list<map<string, any>> results = db.query:
    sql: "SELECT COUNT(*) AS total FROM users"
```

Raw SQL bypasses ORM tenant scoping — include explicit `WHERE tenant_id = ?` when tenant scoping is required.

---

## 9. Many-to-Many Relationships

Many-to-many uses an explicit junction model, not implicit `many<>` syntax.

### 9.1 Defining a Junction Model

```clean
// app/entity/role.cln
class Role
    integer? id
    string name
    string description

// app/entity/user_role.cln
class UserRole
    integer? id
    integer userId
    integer roleId
    datetime assignedAt

// app/data/models/user_role.cln
data UserRole:
    table "user_roles"

    fields:
        id primary generated
        userId as "user_id" required
        roleId as "role_id" required
        assignedAt as "assigned_at" required

    indexes:
        (userId, roleId) unique

    relations:
        user: belongs_to User on user_id
        role: belongs_to Role on role_id
```

### 9.2 Creating and Removing Links

```clean
UserRole ur = UserRole(
    userId: user.id!,
    roleId: role.id!,
    assignedAt: time.now()
)
Database.save(ur)

// Remove
UserRole existing = UserRole.data.findOrFailByUserAndRole(user.id!, role.id!)
Database.delete(existing)
```

### 9.3 Link Syntax (Querying Many-to-Many)

Use `link:` inside `find:` to traverse junction models:

**Roles for a given user:**

```clean
list<Role> roles = Role.find:
    link:
        UserRole role == id
    where:
        UserRole.userId == currentUserId
```

**Users with a given role:**

```clean
list<User> users = User.find:
    link:
        UserRole user == id
    where:
        UserRole.roleId == adminRoleId
```

Inside `link:`, bare `id` refers to the outer (primary) model's key.

---

## 10. Report Classes (`app/data/reports/`)

Queries that don't return a single entity type (aggregates, cross-entity structs, summary counts) live in report classes under `app/data/reports/`.

**Naming:** capability-noun style — `SalesByRegion`, `MonthlyActivity`, `ActivityCohort`. Not entity-scoped.

**Return types:** anything that is not a persistent entity — value types from `app/types/`, plain `map<string, any>`, custom struct types.

**Example:**

```clean
// app/types/region_revenue.cln
class RegionRevenue
    string region
    number revenue

// app/data/reports/sales_by_region.cln
class SalesByRegion
    functions:
        public:
            list<RegionRevenue> forQuarter(integer year, integer quarter)
                return db.queryAs(RegionRevenue):
                    sql: "SELECT region, SUM(total) AS revenue FROM orders WHERE year = ? AND quarter = ? GROUP BY region"
                    params: [year, quarter]
```

Report classes may use `db.query` / `db.queryAs` for raw SQL. They are the natural home for queries that exceed the DSL's expressiveness.

**Folder is lazy** — `app/data/reports/` is created only when the first report class appears.

---

## 11. Migrations

### 11.1 CLI

```
cleen db:migrate                # Apply pending migrations
cleen db:migrate:status         # Show which migrations have run
cleen db:migrate:rollback       # Roll back the last migration
```

### 11.2 Per-Migration File

Migration files live in `app/data/migrations/` with the naming convention `NNN_description.cln`.

```clean
// app/data/migrations/003_add_user_status_index.cln
migrate "003_add_user_status_index":
    up:
        "CREATE INDEX idx_users_status ON users (status)"
    down:
        "DROP INDEX idx_users_status"
```

**Rules:**

- Every `migrate:` block requires both `up:` and `down:` sub-blocks.
- Migrations execute in filename order (ascending).
- `Model.migrate()` uses `CREATE TABLE IF NOT EXISTS` — it does NOT alter columns on existing tables. Column changes require an explicit `migrate:` block.

Schema helpers available in `up:` / `down:`:
`createTable`, `dropTable`, `addColumn`, `dropColumn`, `renameColumn`, `createIndex`, `dropIndex`, `addForeignKey`, `dropForeignKey`, `execute`.

---

## 12. Configuration

Database configuration lives in `main.cln` under the `frame.data:` block. (v1's `app/data/config.cln` is removed under v2.)

```clean
target:
    plugins:
        frame.server
        frame.data
        frame.ui

frame.data:
    default:
        driver: postgres
        url: env("DATABASE_URL")
        pool:
            max: 10
            idleTimeout: 30000
    dev:
        driver: sqlite
        url: ".frame/data.db"
    test:
        driver: examples
        source: "app/data/examples/"
```

**Rules:**

- `password` (or full URLs containing credentials) MUST use `env(...)` — literal credentials in `main.cln` are a compile-time error.
- Environment-scoped blocks (`default:`, `dev:`, `test:`, `production:`, etc.) resolve at runtime per `CLEAN_ENV`.
- CLI flags override environment (`frame-cli serve --examples` forces examples driver).
- Supported drivers in v3.0: `postgres`, `sqlite`, `examples`.

Connection routing (`Database.readReplica.*` / `Database.primary.*`) is reserved for v3.1+ and not available in v3.0.

---

## 13. Example End-to-End Flow

A complete register/login flow using v2 patterns.

### 13.1 Entity

```clean
// app/entity/user.cln
class User
    integer? id
    string email
    private string passwordHash
    string status
    datetime createdAt

    always:
        status in ["active", "pending", "suspended"]

    functions:
        public:
            boolean canPost()
                return status == "active"

            boolean verifiesPassword(string password)
                return password.verify(passwordHash)

            void promote()
                require status == "pending"
                status = "active"
```

### 13.2 Data Block

```clean
// app/data/models/user.cln
data User:
    table "users"

    fields:
        id primary generated
        email required unique
        passwordHash as "password_hash" required
        status required
        createdAt as "created_at" required

    queries:
        User? findByEmail(string emailAddress)
            return User.first:
                where:
                    email == emailAddress

        User findOrFailById(integer userId)
            return User.findOrFail:
                where:
                    id == userId
```

### 13.3 Logic Layer

```clean
// app/logic/auth.cln
class Auth
    functions:
        public:
            integer register(string email, string password)
                User? existing = User.data.findByEmail(email)
                require existing == null

                User newUser = User(
                    email: email,
                    passwordHash: password.hash(),
                    status: "pending",
                    createdAt: time.now()
                )

                Database.save(newUser)
                return newUser.id!

            User login(string email, string password)
                User? user = User.data.findByEmail(email)

                require user != null
                require user.verifiesPassword(password)
                require user.canPost()

                return user

            void deleteAccount(integer userId)
                User user = User.data.findOrFailById(userId)
                Database.delete(user)
```

### 13.4 Server Endpoints

```clean
// app/server/api/auth.cln
endpoints:
    POST "/api/register":
        integer userId = Auth.register(req.body.email, req.body.password)
        return json({ id: userId })

    POST "/api/login":
        User user = Auth.login(req.body.email, req.body.password)
        return json({ id: user.id!, email: user.email })

    DELETE "/api/users/:id":
        Auth.deleteAccount(parseInt(req.params.id))
        return json({ ok: true })
```

---

## 14. Migration from v1

**v2 replaces v1 with no backwards compatibility.** All Frame applications using v1 syntax must migrate before upgrading to `frame.data` v3.0.

**Migration mechanism:** AI-with-MCP. Point an AI instance (Claude Code or equivalent) at the app and request migration. The updated MCP responses (`get_specification`, `get_quick_reference`, `get_app_structure`) plus this document plus the plugin's rejection diagnostics for removed syntax are sufficient for AI-driven migration. No dedicated `cleen migrate` command is built.

### 14.1 What is removed

- Bare-field `data <T>` declarations (fields declared directly under `data <T>` without a `fields:` sub-block).
- `Model.insert:`, `Model.update:`, `Model.delete:`, `Model.upsert:`, `Model.insert_id:` block-form mutations.
- Business behavior in `data:` blocks (`functions:` block inside `data:` was already E-STRUCT-003 under v1; explicitly removed under v2).
- `app/data/config.cln` file (config moves to `main.cln`).

### 14.2 What is added

- `app/entity/` folder for domain classes paired with data blocks by name.
- Sub-block `data <T>:` form with `table`, `fields:`, `indexes:`, `relations:`, `queries:`.
- `.data` accessor for reads (`User.data.findByEmail(...)`).
- `Database` service for writes (`Database.save(user)`, `Database.delete(user)`, `Database.deleteOrFail(user)`, `Database.saveAll(list)`, `Database.deleteAll(list)`).
- Pairing verification at plugin-compile time (`E-STRUCT-012`).
- `app/data/reports/` folder for aggregate and cross-entity queries.

### 14.3 What is retained

- Query DSL verbs and sub-blocks (`find`, `first`, `findOrFail`, `count`, `exists` with `where:`, `order:`, `limit:`, `offset:`, `select:`, `include:`).
- `transaction:` block form and semantics (nested transactions still a compile-time error).
- Raw SQL escape hatches (`db.query`, `db.queryAs`).
- Migration file format and CLI.

### 14.4 Migration steps for each entity

**Step 1 — Create the domain class.**

Extract every field, invariant, and single-entity business method into `app/entity/<name>.cln`:

```clean
class User
    integer? id
    string email
    private string passwordHash
    string status
    datetime createdAt

    always:
        status in ["active", "pending", "suspended"]

    functions:
        public:
            boolean canPost()
                return status == "active"
            // ... other business methods extracted from logic/ classes
```

Methods that were in `logic/` capability classes but operate only on a single entity's state move here. Methods that orchestrate across entities (e.g., `Auth.register`, which touches User and sends email) stay in `logic/`.

**Step 2 — Rewrite the data block in the sub-block form.**

```clean
data User:
    table "users"

    fields:
        id primary generated
        email required unique
        passwordHash as "password_hash" required
        status required
        createdAt as "created_at" required

    queries:
        User? findByEmail(string emailAddress)
            return User.first:
                where:
                    email == emailAddress
```

Every reusable query becomes a named method in the `queries:` sub-block.

**Step 3 — Rewrite call sites.**

Reads (inline block form → named method):

```clean
// Before
User? u = User.first:
    where:
        email == inputEmail

// After
User? u = User.data.findByEmail(inputEmail)
```

Writes (block form → Database service):

```clean
// Before
User.insert:
    email = "ana@example.com"
    status = "pending"
    createdAt = time.now()

// After
User newUser = User(
    email: "ana@example.com",
    status: "pending",
    createdAt: time.now()
)
Database.save(newUser)
```

Updates (block form → load-mutate-save):

```clean
// Before
User.update:
    set:
        status = "active"
    where:
        id == userId

// After
User user = User.data.findOrFailById(userId)
user.promote()
Database.save(user)
```

Deletes (block form → Database.delete):

```clean
// Before
User.delete:
    where:
        id == userId

// After
User user = User.data.findOrFailById(userId)
Database.delete(user)
```

**Step 4 — Confirm the migration is complete.**

Search the codebase for these patterns and confirm none remain:

- `<Entity>.insert:` (block-form insert)
- `<Entity>.update:` (block-form update)
- `<Entity>.delete:` (block-form delete)
- `<Entity>.upsert:` (block-form upsert)
- `data <Entity>` without a colon (bare-field form)
- `functions:` inside a `data:` block

---

## 15. Host Bridge Contracts

Frame Data uses the standard host bridge contract (`host:db.*`) for driver-level operations.

| Bridge | Purpose |
|---|---|
| `host:db.query` | Execute a parameterized SELECT and return rows |
| `host:db.execute` | Execute a parameterized INSERT / UPDATE / DELETE |
| `host:db.tx` | Wrap a sequence of statements in a transaction |
| `host:db.migrate` | Apply DDL for schema migrations |

The bridge details are documented in `documents/specification/frame_bridge_contracts.md`.

---

## 16. Guidelines

- **One entity per file, one data block per file, one report per file.** Keep files small and focused.
- **Domain purity:** Business methods on the entity, orchestration in `logic/`, persistence in `data/` blocks and `Database`.
- **Query naming:** Prefer entity-scoped names (`findActive`, `findByEmail`, `findOrFailById`) that describe what is fetched.
- **Reserve `logic/` for verbs (`Auth`, `Checkout`, `Mailer`).** Nouns are entities or reports.
- **Do not add methods to `data:` blocks other than `queries:`.**
- **Use raw SQL only when the DSL doesn't cover a case — and prefer to put it inside a report class.**

---

## 17. Summary

Frame Data v2:

- **Entity in `app/entity/`** — nouns, business methods, `always:` invariants, persistence-ignorant.
- **Data block in `app/data/models/`** — schema declaration + reusable queries. Paired to entity by name.
- **`.data` accessor** — read queries from `logic/`.
- **`Database` service** — writes from `logic/`. Mutates entity `id` in place on INSERT.
- **`transaction:` block** — atomic multi-write.
- **Report class in `app/data/reports/`** — aggregate or cross-entity queries.
- **Raw SQL via `db.query` / `db.queryAs`** — escape hatch.

The query DSL (`find`, `first`, `findOrFail`, `count`, `exists`) and `transaction:` block are retained from v1 unchanged. Block-form mutations (`Model.insert:`, etc.) and bare-field `data:` declarations are removed and replaced by the entity/data pairing plus `Database` service.

For rationale, alternatives considered, and design decisions, see `clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md`.
