# Specification — Data Persistence Model

**Status:** Committed 2026-07-10. Design specification derived from framework design conversation (2026-06-30 through 2026-07-10).
**Target:** `frame.data` plugin v3.0.0 release (implementing spec files `frame-data.ebnf` v2.0.0 and `frame-data-semantics.md` v2.0.0) plus paired MCP `get_app_structure` guidance.
**Audience:** Framework and compiler implementers. Not intended as user-facing documentation yet — user-facing content lives in `documents/specification/04_frame_data.md`.

This document specifies the target data persistence model for Frame applications. It describes the folder structure, class shapes, syntax, semantics, and implementation responsibilities that together define how a Frame application declares, stores, queries, and mutates persistent data.

The design has been debated extensively. This document captures the settled decisions. All previously-open design questions were resolved during Phase 1 of the migration and are recorded in §11. Content marked **[proposed syntax]** was committed to `foundation/spec/plugins/frame-data.ebnf` v2.0.0 during Phase 1; the marker is retained as historical trace of what changed but the syntax is no longer "proposed."

---

## 1. Overview

Frame separates persistence into three layers, each with a distinct home in the folder tree:

- **Domain layer** — `app/entity/<name>.cln` — plain classes with fields, invariants, and business behavior. Persistence-ignorant.
- **Data layer** — `app/data/models/<name>.cln` — `data:` blocks paired with domain classes by shared name. Declare table mapping, storage-side fields, indexes, relations, and query methods.
- **Runtime layer** — `Database` — a top-level service provided by `frame.data`. Handles mutations (`save`, `delete`), transactions, connection routing, and lifecycle.

Two persistence surfaces are exposed to application code:

- **Reads** — via the entity's `.data` accessor: `User.data.findByEmail(email)`.
- **Writes** — via the `Database` service: `Database.save(user)`, `Database.delete(user)`, `Database.deleteOrFail(user)`.
- **Transactions** — via the top-level `transaction:` block (retained from existing Frame syntax).

The rule: **reads scope to the entity; writes are cross-cutting.**

This spec **replaces** the current Frame data model. No backwards compatibility is preserved — the block-form mutations (`Model.insert:`, `Model.update:`, `Model.delete:`) and bare-field `data:` declarations from the current spec are removed. See §13 for the migration path.

---

## 2. Design principles

Ordered by priority. Later principles yield to earlier ones when they conflict.

1. **Domain classes are persistence-ignorant.** A `class User` in `app/entity/` must be constructible, testable, and reasonable without any persistence machinery present. It knows nothing about tables, columns, drivers, connections, or query DSLs.
2. **Persistence declarations live where they belong.** Storage-side concerns (table names, column overrides, indexes, relations) live in the `data:` block, never in the domain class.
3. **Reads and writes are architecturally different.** Reads are entity-scoped (they return one entity type and attach naturally to that entity). Writes are cross-cutting (they interact with transactions, connections, event streams, cache invalidation). The persistence surface reflects this by scoping reads to the entity and centralizing writes on `Database`.
4. **The relationship between entity and data block is by name.** No inheritance, no marker types, no annotations. `class User` and `data User:` pair by sharing the identifier `User`. This is enforced by `frame.data` at plugin-compile time.
5. **Discovery-by-convention is the API.** No configuration file lists entity/data pairs. No annotations mark classes as persistable. The presence of a `data User:` block in `app/data/models/` and a `class User` in `app/entity/` is sufficient to establish the pair.
6. **Application code writes domain types.** `logic/`, `server/`, and `web/` code operates on `User`, `Order`, `Invoice` — never on `UserData`, `OrderRow`, or any persistence-facing type. The data block exists for declaration; instances are always the domain type.

---

## 3. Folder layout

```
my-app/
├── main.cln
└── app/
    ├── entity/
    │   ├── user.cln              class User
    │   ├── order.cln             class Order
    │   └── ...
    ├── data/
    │   ├── models/
    │   │   ├── user.cln          data User:
    │   │   ├── order.cln         data Order:
    │   │   └── ...
    │   ├── examples/
    │   │   ├── user_examples.cln  class UserExamples
    │   │   └── ...
    │   └── migrations/
    ├── logic/
    │   ├── auth.cln              class Auth
    │   └── ...
    ├── server/
    └── web/
```

Every persistent entity has exactly two files: one in `app/entity/` and one in `app/data/models/`. Their basenames match (`user.cln` on both sides). Their declared identifiers match (`class User` and `data User:`).

Entities without a `data:` block are permitted — a class that never persists (a session shape, a computed report, a value-carrying request) is a plain domain class with no counterpart.

Data blocks without a matching entity are not permitted. Every `data <T>:` block requires a `class <T>` in `app/entity/<name>.cln`. The `frame.data` plugin reports this as a build error.

---

## 4. Domain class specification (`app/entity/`)

### 4.1 Shape

A domain class in `app/entity/<name>.cln` is a regular Clean class following `grammar.ebnf` §6.4. It declares:

- Fields (with types, private/public visibility per §6.4a).
- Optionally an `always:` block for class invariants.
- A `functions:` block containing domain methods.

The class must be named in singular PascalCase, matching the file's basename in snake_case (e.g., `class User` in `user.cln`).

### 4.2 Field declarations

Fields declare the domain shape. Every field the domain reasons about is declared here. The domain class is the primary source of truth for what a `User` *is*.

```clean
class User
    integer id
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

The domain class:

- Declares field types.
- Uses `private` to hide fields from outside access (e.g., `passwordHash`).
- Uses `always:` for class invariants that must always hold.
- Uses `require` / `ensure` for method-level pre/postconditions per `grammar.ebnf` §6.5.
- Has no knowledge of tables, columns, or query mechanisms.

### 4.3 The `id` field

Persistent entities have a nullable `id` field: `integer? id` (for auto-increment primary keys) or `string? id` (for UUID-keyed types). The domain declares this field as part of what a `User` is — an identity-carrying entity that has an identity once persisted.

The data block marks this field as `primary generated` (see §5), meaning the database supplies its value on insert. Before persistence, the entity's `id` is `null`. `Database.save(entity)` mutates the entity's `id` in place from `null` to the DB-generated value. Read sites in `logic/` that need the id after save use the `!` unwrap or a null-check per Clean's standard nullable-type semantics. See §11.1 for the design rationale.

---

## 5. Data block specification (`app/data/models/`)

### 5.1 Shape [proposed syntax]

A data block in `app/data/models/<name>.cln` declares the persistence view of a paired domain class. It is introduced by the `data` keyword followed by the paired class name and a colon.

This is the **only** legal form. The bare-field declaration form from the current spec (`data User` with typed field declarations directly in the body, per `04_frame_data.md` §2.1) is removed under this spec. A `data:` block always uses the sub-block structure below.

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
        User getById(integer userId)
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

        list<User> findRecentByStatus(string status, integer limit)
            return User.find:
                where:
                    status == status
                order:
                    createdAt desc
                limit: limit
```

The identifier `User` after `data` refers to the paired domain class. The compiler and `frame.data` plugin resolve this by name — both `class User` (in `app/entity/user.cln`) and `data User:` (in `app/data/models/user.cln`) must exist.

Query bodies use Frame's existing DSL (`04_frame_data.md` §3): `User.find:`, `User.first:`, `User.findOrFail:`, `User.count:`, `User.exists:` — block form with `where:`, `order:`, `limit:`, `offset:`, `select:`, `include:` sub-blocks. Field references inside `where:` are unqualified (`email == emailAddress`, not `User.email == emailAddress`). Parameter names should not shadow field names — use `userId` for a parameter matching the `id` field, `emailAddress` for a parameter matching the `email` field.

### 5.2 Sub-blocks

A `data:` block has these sub-blocks, all optional except as noted:

- **`table <string>`** — the database table name. If omitted, defaults to the snake_case pluralization of the class name (`User` → `users`).
- **`fields:`** — required. Declares which domain fields are persisted, plus per-field constraints and column-name overrides. Field names must reference declarations on the paired domain class; unknown fields are a build error.
- **`indexes:`** — declares indexes. Each entry is either a single field name (`email`) or a parenthesized tuple (`(status, createdAt)`). Suffix `unique` to mark a unique index.
- **`relations:`** — declares foreign-key relationships. Each entry has the form `<name>: <cardinality> <TargetClass> on <foreign_key_column>` where `<cardinality>` is one of `has_many`, `has_one`, `belongs_to`.
- **`queries:`** — declares read-only query methods returning the paired domain type. See §5.4.

### 5.3 Field constraints

Inside the `fields:` block, each entry has the form:

```
<fieldName> [as "<column_name>"] [primary] [generated] [required] [unique] [default: <expr>]
```

Constraints:

- **`primary`** — this field is the table's primary key. At most one field per data block.
- **`generated`** — the database generates the value on insert (typical for auto-increment `id`).
- **`required`** — non-nullable in storage. If the domain class declares the field as non-nullable, this is redundant but harmless.
- **`unique`** — enforce uniqueness at the storage layer via a unique index (separate from `indexes:` block; this is a shorthand for single-column uniqueness).
- **`default: <expr>`** — storage-side default when the field is absent on insert.
- **`as "<column_name>"`** — the storage column name differs from the domain field name. Used for snake_case ↔ camelCase, historical naming, or renames.

Field types are inherited from the paired domain class. The data block does not restate them.

### 5.4 Query methods

The `queries:` sub-block contains read-only methods that return the paired domain type or collections of it.

Each query method:

- Declares a return type: `User`, `User?` (nullable), or `list<User>` (collection).
- Takes zero or more parameters.
- Contains exactly one query expression using the query DSL (§8).
- Is called via the entity's `.data` accessor from application code: `User.data.findByEmail(email)`.

Query methods do not mutate. They cannot call `Database.save` / `Database.delete` or open a `transaction:` block. They may call other query methods on the same data block or on other entities' data blocks.

Query methods are exposed as `public` by default; the `functions:`-block-style `public:` sub-section per §6.4b is not required inside `queries:`.

### 5.5 Restrictions

A data block cannot contain:

- A `functions:` block. Non-query methods do not belong on the data block. Any behavior that isn't a query lives on the domain class or in `logic/`.
- An `always:` block. Invariants live on the domain class.
- A `constructor`. Data blocks are not instantiable — they are declarations.

These restrictions are enforced by `frame.data` at plugin-compile time, with clear error messages naming the correct home for the misplaced construct.

---

## 6. The `.data` accessor

### 6.1 Semantics

The expression `<ClassName>.data.<method>(...)` invokes a query method declared on the `data <ClassName>:` block paired with `class <ClassName>`.

`.data` is a syntactic namespace, not a runtime object. It cannot be captured as a value:

```clean
User? user = User.data.findByEmail(email)         // valid
var facade = User.data                             // invalid — .data is not a value
```

The compiler resolves `<ClassName>.data.<method>` by:

1. Locating the paired data block for `<ClassName>` (by name matching, per §3).
2. Looking up `<method>` in the data block's `queries:` sub-block.
3. Dispatching the call to that method with the given arguments.

If no paired data block exists, or no such query method exists on it, this is a compile error.

### 6.2 Rationale

The `.data` accessor:

- **Disambiguates the entity/data pair without requiring same-name identifier coexistence in the type system.** `class User` remains a type; `data User:` is not a Clean type in the type system's identifier namespace. The compiler recognizes `User.data.<x>` as a special access path resolved by the data-block pairing.
- **Marks persistence access visually at the call site.** A reader can distinguish `user.canPost()` (domain method) from `User.data.findByEmail(email)` (persistence access) without knowing what each method does.
- **Scopes queries to their entity.** Queries returning `User` live on `User.data`. Queries returning `Order` live on `Order.data`. There is no cross-entity ambiguity.

### 6.3 What `.data` does not do

`.data` does not expose mutations. `User.data.save(user)` is not valid syntax. Mutations go through `Database` (see §7).

`.data` does not expose fields. `User.data.email` is not a valid access. Fields live on domain instances (`user.email`) or on the persistence declaration (readable only via query results).

---

## 7. The `Database` service

### 7.1 Purpose

`Database` is the top-level service provided by `frame.data`. It handles:

- **Mutations** — `Database.save(entity)`, `Database.delete(entity)`, `Database.deleteOrFail(entity)`.
- **Transactions** — the top-level `transaction:` block (existing Frame syntax, unchanged; see §7.3).
- **Bulk operations** — `Database.saveAll([...])`, `Database.deleteAll([...])`.

Connection routing (`Database.readReplica.<query>` / `Database.primary.<query>`) is out of scope for v3.0. All reads and writes route through a single connection. See §11.2 for the deferral rationale.

### 7.2 Mutation operations

```clean
Database.save(user)                    // insert if id is null, update otherwise
Database.delete(user)                  // delete by primary key; no-op if row missing
Database.deleteOrFail(user)            // delete by primary key; raises NOT_FOUND if row missing
```

Semantics:

- **`save`** — persists the given entity. If the entity's `id` is `null` (per §4.3), inserts a new row and mutates the entity's `id` in place to the DB-generated value. Otherwise, updates the existing row identified by that `id`.
- **`delete`** — lenient variant. Removes the row identified by the entity's primary key. If no row exists, this is a no-op — no error is raised. Retry-safe and idempotent. Matches SQL's default `DELETE ... WHERE id = ?` behaviour of returning zero affected rows without error.
- **`deleteOrFail`** — strict variant. Same as `delete` but raises `NOT_FOUND` if no row matches the entity's primary key. Use when the caller needs an explicit signal that the row existed and was removed. See §11.3 for the design rationale.

`save`, `delete`, and `deleteOrFail` all validate the entity's `always:` invariants before persisting or removing. If any invariant fails, the operation raises `RUN005 — Assertion Failure` (per `semantic-rules.md` line 797) and does not persist.

### 7.3 Transactions

The top-level `transaction:` block is Frame's existing atomic-writes syntax. It is unchanged by this spec.

```clean
transaction:
    Database.save(order)
    for CartItem item in items:
        OrderLine line = OrderLine(0, order.id, item.productId, item.quantity, item.price)
        Database.save(line)
    Database.save(payment)
```

Semantics (per `documents/specification/04_frame_data.md` §4):

- All `Database.save` / `Database.delete` calls inside the block execute atomically.
- If any call raises an error (including invariant failures), the entire transaction rolls back.
- **Nested `transaction:` blocks are not supported.** Attempting to nest transactions is a compile-time error. Service functions that each use `transaction:` internally must not be composed inside a single outer `transaction:` — extract the composed logic into a single flat transaction block.

The `transaction:` block is a language-level construct, not part of the `Database` service surface. It is invoked directly at the top level of application code wherever atomicity is required.

### 7.4 Rationale for centralizing writes

Writes have concerns that don't scope to any single entity: transaction boundaries, connection routing, event emission, cache invalidation, bulk operations, replica coordination. Attaching writes to an entity (`user.data.save()`) would either fracture these concerns across every entity or force the entity to know about them.

Centralizing on `Database` gives every write concern a natural home. This matches the design of Ecto (`Repo`), Entity Framework (`DbContext`), Hibernate (`Session`), and Doctrine (`EntityManager`).

---

## 8. Query DSL

This spec inherits Frame's existing query DSL from `documents/specification/04_frame_data.md` §3. Query methods inside the `queries:` sub-block (§5.4) use this DSL. This section summarizes the DSL for reference; the authoritative definition is `04_frame_data.md`.

### 8.1 Query verbs

Verbs on the entity type follow the block form `<Entity>.<verb>:` with sub-blocks:

- **`find`** — returns a collection. Returns an empty `list<T>` if no rows match. Return type: `list<T>`.
- **`first`** — returns at most one row. Returns null if no row matches. Return type: `T?`.
- **`findOrFail`** — returns exactly one row. Raises `NOT_FOUND` if no row matches. Return type: `T` (not nullable).
- **`count`** — returns the number of matching rows. Return type: `integer`.
- **`exists`** — returns `true` if at least one row matches. Return type: `boolean`.

Return-type-to-verb correspondence is enforced by the compiler: `T` requires `findOrFail`, `T?` requires `first`, `list<T>` requires `find`, `integer` requires `count`, `boolean` requires `exists`.

### 8.2 Query sub-blocks

Each query verb takes a block with the following sub-blocks:

- **`where:`** — filter predicates. One predicate per line (implicit `and` between lines).
- **`order:`** — sort clauses. One `<field> [asc|desc]` per line. Default direction is `asc`.
- **`limit: <integer>`** — cap the returned row count.
- **`offset: <integer>`** — skip N rows before returning results (used for pagination).
- **`select:`** — retrieve only specific fields. One field name per line.
- **`include:`** — eager-load related records via declared relations.
- **`set:`** — used by `update` verb only; declares column assignments.

### 8.3 Field references inside `where:` and `order:`

Field references are **unqualified**. Inside a `where:` or `order:` block, the field name alone refers to the current row's column on the entity being queried:

```clean
list<User> users = User.find:
    where:
        status == "active"
        createdAt > startOfMonth
    order:
        createdAt desc
```

The bare `status`, `createdAt` names refer to columns on `User`. Values from the enclosing method scope (`startOfMonth`) are used directly.

For joined queries via `include:`, fields on the linked model are referenced by the linked model's name: `author.status`, `tags.name`. See `04_frame_data.md` §3.10 and §7 for the join syntax.

### 8.4 DSL restrictions

The query DSL is a **subset** of what raw SQL supports. Deliberately in scope:

- Comparison, boolean logic, `and`/`or`/`not`.
- `order:` on any indexed or non-indexed column.
- `limit:` and `offset:` for pagination.
- Explicit joins over declared `relations:` via `include:`.

Deliberately out of scope:

- Aggregations (`sum`, `avg`, `group by`, `having`) beyond the built-in `count` verb. Complex aggregations are handled by ordinary methods in `logic/` that call the DSL for their base fetches, or by raw SQL via `db.query` / `db.queryAs` (`04_frame_data.md` §5).
- Window functions, CTEs, `distinct`, subqueries.
- Vendor-specific SQL.

The DSL must produce identical results across every backing driver (Postgres, SQLite, examples driver). Divergence between drivers is a driver bug.

---

## 9. Multiple backing drivers

### 9.1 Driver configuration

`frame.data` supports multiple drivers via `main.cln` configuration:

```clean
frame.data:
    default:
        driver: postgres
        url: env("DATABASE_URL")
        pool: 10
    dev:
        driver: examples
        source: "app/data/examples/"
    test:
        driver: examples
        source: "app/data/examples/"
```

Standard 12-factor precedence:

1. CLI flag (`frame-cli serve --examples` forces the `examples` driver).
2. Environment (`CLEAN_ENV=dev` selects the `dev:` block).
3. Config default (`default:`).

### 9.2 Supported drivers

- **`postgres`** — PostgreSQL. Full DSL support.
- **`sqlite`** — SQLite. Full DSL support.
- **`examples`** — reads from `app/data/examples/*.cln` at startup, serves the DSL from an in-memory index. Mutations vanish on restart. Transactions are no-ops. See paired framework proposal for full driver spec.

Additional drivers (MySQL, MongoDB, other stores) are future work and out of scope here.

### 9.3 Release-build guard

`frame-cli build --release` refuses to bundle a build with `driver: examples` unless invoked with `--allow-examples-in-release`. This prevents accidental production deployment of the test-repository driver.

Enforcement point: `frame-cli` reads a manifest field emitted by `frame.data` and validates it during the release build. Compiler emits the manifest field; CLI enforces the guard.

---

## 10. Field alignment and pairing verification

### 10.1 Pairing check

`frame.data` verifies at plugin-compile time that every `data <T>:` block has a paired `class <T>` in `app/entity/<basename>.cln`. Missing pairs are a build error (`E-STRUCT-012` per paired MCP proposal).

### 10.2 Field alignment

`frame.data` verifies field alignment between the domain class and the data block:

- Every field named in the data block's `fields:` sub-block must exist as a declared field on the paired domain class.
- Every non-optional field declared on the domain class must be represented in the data block's `fields:` sub-block.
- Methods declared inside the class's `functions:` block (including computed properties per §11.6) are not fields and are excluded from the alignment check.

Field types are read from the domain class. The data block does not restate them; a type mismatch is impossible because the type has one source.

### 10.3 Column-name override

If the storage column name differs from the domain field name, the data block uses `as "<column_name>"`. Example:

```clean
fields:
    passwordHash as "password_hash" required
    createdAt as "created_at" required
```

The domain sees `passwordHash`; the database sees `password_hash`. No automatic case conversion — the developer states the mapping explicitly.

---

## 11. Design decisions

Each subsection below records a design decision, the alternatives considered, and the rationale. All decisions in §11 were resolved during Phase 1 of the migration (see `foundation/management/cross-component-prompts/framework-data-persistence-migration-phase-1-spec.md`).

### 11.1 Sentinel value for unpersisted ids

**Decision:** Optional `id` (`integer?` or `string?`) with mutate-in-place semantics on `Database.save`.

**How it works:**

- The domain class declares `id` as nullable: `integer? id` or `string? id`.
- A newly-constructed entity has `id == null`. The named-argument constructor (§11.7) omits `id`.
- `Database.save(user)` writes an INSERT if the entity's `id` is null and mutates `user.id` in place to the DB-generated value. Subsequent calls to `Database.save(user)` (with `id` now set) issue UPDATE.
- Read sites in `logic/` that need the id after save use the `!` unwrap (or a null-check) — a small tax accepted for type honesty.

**Example:**

```clean
class User
    integer? id           // null before save, DB-assigned after
    string email
    string status
    datetime createdAt

// logic/
User u = User(email: "a@b.com", status: "pending", createdAt: time.now())
require u.id == null

Database.save(u)
require u.id != null      // mutated in place

integer userId = u.id!    // ! unwrap at read site
```

**Alternatives considered and rejected:**

- **Sentinel value (`0` for `integer id`)** — code smell at every constructor call site (`User(0, email, ...)`); conflates "unpersisted" with "id = 0"; race-condition risk on retry.
- **Framework-provided factory (`Database.new(User, ...)`)** — introduces a second construction path; interacts awkwardly with `always:` invariants; extra API surface for one specific concern.
- **Optional id with return-new-entity `Database.save`** — every save site becomes `user = Database.save(user)`; extra ceremony without proportional benefit; mutate-in-place matches Rails/Django/Ecto behaviour that developers already expect.

**Ecosystem precedent:** Ecto (Elixir) uses this exact pattern (`%User{id: nil}` → `%User{id: 42}` after `Repo.insert`) and Elixir developers accept the small null-check tax cleanly.

### 11.2 Connection routing (read replicas, primary/replica split)

**Decision:** Deferred to v3.1+. v3.0 ships primary-only routing.

**Rationale:**

- Replica routing is a real design commitment with subtle semantics (replication lag, transaction routing, health checks, fail-over policy) that are better made against concrete user need.
- Most small-to-medium Frame apps will never need it. Building it now solves a problem no current user has.
- Retrofit is additive, not breaking. When `Database.readReplica.<query>` and `Database.primary.<query>` land in v3.1+, bare `Database.<query>` continues to work — it defaults to primary, matching v3.0 behaviour.
- Every mature ORM added replica support gradually (Rails in 6.0, ~15 years after 1.0). We are not the first framework in this situation.

**What ships in v3.0:** `main.cln` config has a single `url:` per driver. All reads and writes go to the same connection. No `read_url:` alongside `url:` in v3.0 either — that surface waits for v3.1.

**What's NOT ruled out for v3.1+:** the full replica API (`Database.readReplica.<query>` / `Database.primary.<query>`), configurable selection strategies, health checks, and transaction-scope enforcement (reads inside a `transaction:` block route to primary).

### 11.3 Semantics of `Database.delete` on missing row

**Decision:** Both variants. `Database.delete(entity)` is lenient (no-op if row missing). `Database.deleteOrFail(entity)` is strict (raises `NOT_FOUND` if row missing).

**How it works:**

```clean
// lenient — retry-safe, idempotent
Database.delete(user)     // no error if row already gone

// strict — raises NOT_FOUND if row missing
Database.deleteOrFail(user)
```

**Rationale:**

- Mirrors the existing query DSL's `first` (lenient, returns `T?`) vs `findOrFail` (strict, raises). Same design language on both sides.
- Callers pick per site based on whether they need idempotency (retry logic, distributed systems) or explicit failure signal (correctness-sensitive flows).
- One extra method on `Database` is trivial cost; developer clarity is significant benefit.

**Ecosystem precedent:** Rails ActiveRecord (`destroy` vs `destroy!`), Ecto (`Repo.delete` vs `Repo.delete!`), Doctrine (via explicit checks). This pattern is well-established.

### 11.4 Nested transactions

**Decision:** Compile-time error. Inherited unchanged from the current Frame spec (`04_frame_data.md` §4).

Nested `transaction:` blocks are rejected at compile time. Service functions that each use `transaction:` internally must not be composed inside a single outer `transaction:` — extract the composed logic into a single flat transaction block.

This spec inherits that decision without change. No further work required.

### 11.5 Cross-entity joins in the query DSL

**Decision:** Filter on `include:`d relations only. Full general joins are out of scope; escape to raw SQL for cases the DSL doesn't cover.

**How it works:**

- A query that filters on a joined field must first `include:` the relation.
- The `where:` clause may then reference relation-scoped fields (`orders.status`).
- The plugin generates the join at query time. The join is explicit at the source level.

**Example:**

```clean
list<User> users = User.find:
    include:
        orders
    where:
        tenantId == currentTenantId
        orders.status == "pending"
```

**Rationale:**

- Explicit enough to be predictable: the developer sees `include: orders` and knows a join happens. No implicit-join performance surprises.
- Permissive enough to be useful for the common "user with their orders" or "post with its author" cases that show up constantly.
- The `include:`d relation's data is loaded into the result. If a query needs only filter-scope (no payload load), that's expressible in a future revision via an additional `join:` clause — deferred to keep v3.0 scope tight.

**Alternatives considered and rejected:**

- **No cross-entity joins in the DSL** — forces two-query patterns or raw SQL for common cases; awkward at call sites; N+1 risk.
- **Fully general joins without `include:`** — implicit joins are notorious for performance surprises; the developer can't tell at a glance whether a `where:` clause fires a join.

**Escape hatch:** for queries the DSL doesn't support (deep joins, aggregations, window functions, vendor-specific SQL), use `db.query` / `db.queryAs` per `04_frame_data.md` §5. Report classes in `app/data/reports/` (see §11.8) are the natural home for such queries.

### 11.6 Domain-only fields (computed properties)

**Decision:** Reuse the existing `functions:` block. Computed properties are methods on the class. No new `computed:` sub-block is introduced.

**How it works:**

- A domain class declares stored fields at the top of the class body.
- Computed properties are declared as methods inside the `functions:` block.
- Callers use method-call syntax with parentheses: `user.fullName()`.
- The plugin's field-alignment check compares the class's declared fields against the paired data block's `fields:` sub-block. Methods are not fields, so no special handling is required.

**Example:**

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

**Rationale:**

- Zero new grammar. Uses only existing Clean constructs.
- Field-alignment check is naturally correct: fields declared in the class must appear in `data:` `fields:`; methods don't participate in the check.
- The parenthesised call `user.fullName()` honestly signals "this is computed at access time" rather than blurring stored and derived data.
- The ergonomic cost (parentheses vs. bare `user.fullName`) is small and offset by the honesty.

**Alternatives considered and rejected:**

- **Explicit `computed:` sub-block with property-style access** — cleaner ergonomics but requires new plugin grammar and adds a language concept for something already expressible with methods.
- **Regular fields with data block silently omitting them** — silent-bug factory; a forgotten field is indistinguishable from an intentionally domain-only one at code review time.

### 11.7 Named-argument construction

**Decision:** Already supported by Clean. No new language feature required.

Per `foundation/spec/grammar.ebnf` line 399, `named_argument = parameter_name , ":" , argument_expression` applies wherever an argument list appears, including constructor invocations. Per `foundation/spec/semantic-rules.md` §named arguments (around lines 397–454), the language already supports:

- Named-only calls.
- Positional-only calls.
- Mixed calls (with the rule that positional arguments precede named arguments).
- Uniqueness (each parameter satisfied by exactly one positional or named argument).

**Example (already valid Clean syntax):**

```clean
User newUser = User(
    email: emailInput,
    passwordHash: passwordInput.hash(),
    status: "pending",
    createdAt: time.now()
)
```

**What this means for the plugin spec:** nothing new to add. The plugin uses the existing constructor invocation syntax the language already provides. This subsection is retained in the spec as a rediscovery record so future readers know the design decision was checked, not overlooked.

### 11.8 Where cross-entity queries live

**Decision:** Hybrid rule sharpened by return type.

- **Return type is a persistent entity (`T`, `list<T>`, `T?` where `T` has a paired `data:` block)** → query lives in that entity's `queries:` sub-block. This includes queries whose `where:` filters on `include:`d relations — the result is still typed as the primary entity.
- **Return type is not a persistent entity (aggregates, cross-entity struct types, plain values, `map<string, any>`)** → query lives in `app/data/reports/<name>.cln` as `class <ReportName>` with capability-noun names (`SalesByRegion`, `MonthlyActivity`, `ActivityCohort`).
- **The `app/data/reports/` folder is created lazily** — not required in projects that only have entity-typed queries. Matches Frame's "don't pre-create empty folders" convention.

**Example — entity-typed query on the data block:**

```clean
// app/data/models/user.cln
data User:
    queries:
        list<User> withRecentOrders(integer days)
            return User.find:
                include:
                    orders
                where:
                    orders.createdAt > time.now() - days.days()
```

**Example — non-entity-typed query in a report class:**

```clean
// app/data/reports/sales_by_region.cln
class SalesByRegion
    functions:
        public:
            list<RegionRevenue> forQuarter(integer year, integer quarter)
                return db.queryAs(RegionRevenue):
                    sql: "SELECT region, SUM(total) AS revenue FROM orders WHERE ..."
                    params: [year, quarter]
```

**Rationale:**

- The rule is decidable from the return type. No judgment call at every new query.
- Report classes' methods can return either dedicated value types (declared in `app/types/`) for type-safe access or plain `map<string, any>` for flexible ad-hoc reporting.
- The report-class location and naming (`app/data/reports/`, capability-noun class names) matches Frame's existing conventions for capability classes in `logic/`, so developers already know the pattern.

**Alternatives considered and rejected:**

- **All queries on entity data blocks (return-type ambiguity)** — forces cross-entity queries onto one arbitrary entity; misleading; awkward for reports with synthetic return types.
- **All queries in `logic/`** — blurs the data/logic separation; puts nouns (report classes) in a folder reserved for verbs (use cases).
- **All queries in dedicated `app/data/queries/<entity>.cln` classes; nothing in `data:` blocks** — sacrifices colocation of entity queries with their schema; adds a folder and naming convention to teach without proportional benefit.

**Escape hatch relationship:** report classes are the natural home for queries that use `db.query` / `db.queryAs` (raw SQL). The DSL covers entity-typed queries; report classes cover everything else.

---

## 12. Implementation responsibilities

### 12.1 Compiler responsibilities

- Parse the `data <T>:` block with sub-blocks (`table`, `fields:`, `indexes:`, `relations:`, `queries:`). This **replaces** the current `data:` block form; the bare-field form (`data User` with typed fields directly in the body) is no longer legal.
- Reject block-form mutations at parse or semantic-analysis time — `<Entity>.insert:`, `<Entity>.update:`, `<Entity>.delete:` are no longer valid syntax under this spec. Emit a diagnostic pointing developers to `Database.save(entity)` / `Database.delete(entity)`.
- The query DSL (`find`, `first`, `findOrFail`, `count`, `exists` with `where:`, `order:`, `limit:`, `offset:`, `select:`, `include:` sub-blocks) is retained from `04_frame_data.md` §3.
- Parse the `.data` accessor path (`<ClassName>.data.<method>(...)`). This is new syntax.
- The top-level `transaction:` block form is retained from `04_frame_data.md` §4.
- Emit grammar-productions in `foundation/spec/grammar.ebnf` for the new sub-blocks and the `.data` accessor.
- Emit semantic rules for `.data` accessor resolution (§6.1) and the entity/data pairing constraint (§10.1).

### 12.2 `frame.data` plugin responsibilities

- Verify entity/data pairing (§10.1).
- Verify field alignment (§10.2).
- Emit generated persistence code from `data:` block declarations (table DDL, index creation, query implementations).
- Implement the `Database` service surface (§7).
- Implement supported drivers (`postgres`, `sqlite`, `examples`).
- Enforce the release-build guard for the `examples` driver (§9.3).
- Provide the DSL runtime that turns `where:` clauses into driver-appropriate queries.

### 12.3 MCP guidance responsibilities

The MCP `get_app_structure` response must:

- Document the folder layout (§3).
- Document the domain-class-in-`entity/` and data-block-in-`data/models/` convention.
- Document the `.data` accessor as the read surface.
- Document `Database` as the write surface.
- Document field-alignment rules and the pairing check.

The paired MCP proposal (`compiler-mcp-app-structure-add-entity-folder-and-dbc.md`) contains this guidance.

---

## 13. Migration from current Frame data model

This spec **replaces** the current Frame data model. No backwards compatibility is preserved. Existing Frame applications must be migrated wholesale; there is no coexistence period, no opt-in path, no legacy fallback.

The current spec's DSL (`04_frame_data.md` §3) — `find`, `first`, `findOrFail`, `count`, `exists` verbs with `where:`, `order:`, `limit:`, `offset:`, `select:`, `include:` sub-blocks — is adopted as the query DSL of this spec because it is well-designed. The `transaction:` block (§4) is adopted for the same reason. Everything else changes.

### 13.1 What is removed

- **Bare-field `data:` declarations** (`data User` with typed field declarations directly in the body, per `04_frame_data.md` §2.1). The `data:` block always uses the sub-block form defined in §5 of this spec.
- **Block-form mutations** — `Model.insert:`, `Model.update:`, `Model.delete:` are no longer valid. All mutations go through `Database.save(entity)` and `Database.delete(entity)` (§7).
- **Business behavior in `data:` blocks** — a `data:` block is a persistence declaration only. Fields, indexes, relations, queries. No `functions:` block, no methods that aren't queries.
- **Business behavior scattered in `logic/` capability classes when it belongs to a single entity** — methods that operate on a single entity's own state move to `class <Entity>` in `app/entity/`.

### 13.2 What is added

- **`app/entity/` folder** — domain classes paired with data blocks by name.
- **`data <T>:` sub-block structure** — `table`, `fields:`, `indexes:`, `relations:`, `queries:` (§5).
- **`.data` accessor** — `<Entity>.data.<method>(...)` invokes query methods declared on the paired data block (§6).
- **`Database` service** — top-level object providing `Database.save(entity)`, `Database.delete(entity)` (§7).
- **Pairing verification** — the `frame.data` plugin verifies every `data:` block has a matching `class` in `app/entity/` and vice versa (§10).

### 13.3 What is unchanged

- **Query DSL verbs and sub-blocks** — `find`, `first`, `findOrFail`, `count`, `exists` with `where:`, `order:`, `limit:`, `offset:`, `select:`, `include:` (§8).
- **`transaction:`** block form and the "no nested transactions" rule (§7.3).
- **Raw SQL escape hatches** — `db.query`, `db.queryAs` remain available for cases the DSL doesn't cover.

### 13.4 Migration steps for an existing entity

Assume `app/data/models/user.cln` currently contains:

```clean
data User
    integer id : pk, auto
    string  email : unique
    string  passwordHash
    string  status
    datetime createdAt : default=now
```

And business behavior for `User` lives in various `logic/` capability classes (`Auth`, `UserService`, etc.), including inline `User.first:`, `User.insert:`, `User.update:` calls at call sites.

**Step 1 — Create the domain class.**

Extract every field, invariant, and single-entity method into `app/entity/user.cln`:

```clean
class User
    integer id
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

Methods that were in `logic/` capability classes but operate only on a single `User`'s state (`canPost`, `verifiesPassword`, `promote`) move here. Methods that orchestrate across entities (`Auth.register`, which touches User and sends email) stay in `logic/`.

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
```

Every reusable query (any query used at more than one call site) becomes a named method in the `queries:` sub-block.

**Step 3 — Rewrite all call sites.**

Read call sites — inline block-form queries become `.data` accessor calls:

```clean
// Before
User? u = User.first:
    where:
        email == inputEmail

// After
User? u = User.data.findByEmail(inputEmail)
```

Write call sites — block-form mutations become `Database` service calls:

```clean
// Before
User.insert:
    email = "ana@example.com"
    status = "pending"
    createdAt = time.now()

// After
User newUser = User(
    0,
    "ana@example.com",
    "hashed_password",
    "pending",
    time.now()
)
Database.save(newUser)
```

Update call sites — read, mutate, save:

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

Delete call sites:

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
- `data <Entity>` without a colon (bare-field form)
- `functions:` inside a `data:` block

Migration tooling (a `frame-cli migrate` command that automates these transformations) is out of scope for this spec but is worth building. Given no backwards compatibility, migration tooling is not optional for real-world adoption — it's required.

---

## 14. Design decisions log

For traceability, the key decisions in this spec came out of the framework design conversation. Each was debated with alternatives; the chosen design is named alongside the rejected ones.

| Decision | Chosen | Rejected |
|---|---|---|
| Entity/data-block relationship | Paired by name, verified by plugin | Inheritance (`class User is UserData`); same-name coexistence (`class User` + `data: User` in language type system); `persist:` sub-block on domain |
| Read surface | `.data` accessor on entity | Static methods directly on entity (`User.findByEmail`); repository classes (`UserQueries.findByEmail`) |
| Write surface | Top-level `Database` service | Instance methods (`user.save()`); entity-scoped writes (`User.data.save`); repository classes; block-form mutations (`User.insert:`, `User.update:`, `User.delete:`) |
| Query method block | `queries:` sub-block inside `data:` | Reusing `functions:`; separate query classes |
| Data-block methods | Queries only, no mutations | Both queries and mutations |
| Field-list source of truth | Domain class | Data block; both (with alignment); either |
| Column-name override | Explicit `as "column_name"` | Automatic case conversion; convention-driven |
| Cross-entity queries | Deferred to `app/data/reports/` if needed | Inline in single-entity data blocks |
| Backwards compatibility with `04_frame_data.md` | None — replace wholesale | Additive coexistence with legacy DSL; opt-in migration per entity |
| Mutation surface count | One (`Database.save`/`Database.delete`) | Two (`Database.save` + `User.insert:` block form coexisting) |
| Query DSL verbs and sub-blocks | Retained from `04_frame_data.md` §3 | Reinvent (`get`/`first`/`find` with different sub-blocks) |
| `transaction:` block form | Retained from `04_frame_data.md` §4 | `Database.transaction:` method form; new grammar |

---

## 15. Status

**Committed spec, effective 2026-07-10. Retargeted 2026-07-16 to aspirational. Reactivated 2026-07-17 as Option A execution target following plugin owner decision.**

Phase 1 of the data-persistence migration completed 2026-07-10. This spec is now the **active execution target** for the next `frame.data` release.

Paired artifacts:

- **`foundation/spec/plugins/frame-data.ebnf` v2.0.0** — the formal grammar for the plugin's DSL. Committed 2026-07-10.
- **`foundation/spec/plugins/frame-data-semantics.md` v2.0.0** — the semantic rules governing plugin behavior. Committed 2026-07-10.
- **`clean-framework/system-documents/DRAFT_MIGRATION_PLAN_DATA_PERSISTENCE_MODEL.md`** — the detailed migration plan.
- **`foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md`** — the per-sub-cycle LOC re-projection for Option A execution.

### Version history

| Date | Event |
|---|---|
| 2026-07-10 | Phase 1 complete. Spec committed. Cross-component prompts placed. Two collision options offered: A (adopt v2 in sub-cycles) or B (finish v1.2 on typed emission first). |
| 2026-07-10 → 2026-07-16 | Plugin team shipped `frame.data` v3.0.4 → v3.0.10 implementing v1.2 on typed emission (Option B trajectory), no response to collision handoff. |
| 2026-07-16 | Aspirational-warning banners added to user-facing docs and book chapters (commits `a4e0fe2`, `643a042`). Migration status recorded as Option B implicit. |
| 2026-07-17 | **Plugin owner decision: adopt Option A. Sub-cycle v3.x plan retired. This spec becomes the active execution target.** Aspirational warnings scheduled for removal as Phase 2 progresses. |
| 2026-07-17 | Phase 2 partial + Phase 4 partial landed in a single session. Sub-cycle 1 (v2) partial (commit `ccd5c2d`, `clean-framework`): v2 sub-block parser (`is_v2_sub_block_form`, `extract_v2_sub_block`, `extract_v2_table_line`, `extract_v2_field_names`), `expand_data_model` routes v2 form via synthesized v1-shape body (types stubbed pending Sub-cycle 3 pairing), `validate_data_model` enforces DAT-M015 on v2 form, dispatcher rejects `Model.insert/insert_id/update/upsert/delete` block forms with `FRAME-DATA-E021` pointing at `Database.save`/`Database.delete`. Source parses (cln check: 133 functions, 0 errors); plugin.wasm unchanged. Phase 4 MCP updates (commit `c53e6ec2`, `clean-language-compiler`): `tool_get_app_structure` rewritten end-to-end for v2 (adds `app/entity/`, `app/data/reports/`, DAT-P pairing note, updated Law 1 dependency direction, extended decision tree with entity/data/reports steps, E-STRUCT-012..015 and FRAME-DATA-E021 in forbidden-patterns table, v2 example capability); `tool_get_quick_reference` §Rule A rewritten to teach `Entity.data.<method>` for reads and `Database.save/delete/deleteOrFail/saveAll/deleteAll` for writes with v1→v2 migration recipes. Track B (book chapter rewrites, ~230 v1 references) skipped because 5/7 target chapters had uncommitted work from a parallel session; deferred until a clean workspace is available. |

### Option A adoption — what changes

- **Sub-cycle v3.x plan retired.** The pre-baked design in `reports/2026-07-03-session-4-frame-data-deferred-v4.md` §2 (v1.2-on-typed-emission) is no longer the target. `frame.data` v3.0.10 stands as the last v1.2 release; no v3.0.11 is planned.
- **Sub-cycle re-baking proposal activated.** The per-sub-cycle LOC targets in `foundation/management/reports/2026-07-10-frame-data-v2-spec-subcycle-rebaking.md` §3 are the execution plan:
  - Sub-cycle 1 (v2): ~1400 LOC — data-model expansion + `.data` accessor + preamble + dispatcher + Am 10 spec construction.
  - Sub-cycle 2 (v2): ~1050 LOC — read verbs (query DSL retained unchanged from v1.2) + DAT-Q020 include-relation path resolution.
  - Sub-cycle 3 (v2): ~1050 LOC — `Database` service class generation + pairing verification + invariant enforcement + rejection diagnostics for removed v1.2 syntax.
  - Sub-cycle 4 (v2): ~1200 LOC — DSL parsers + migrate + integration + smoke test.
- **Total v2-adapted projection:** ~4700 v3 LOC across four sub-cycles.
- **Phase 2 execution prompt** (`foundation/management/cross-component-prompts/framework-data-persistence-migration-phase-2-plugin.md`) is now unblocked. The scope-collision handoff (`...-phase-2-scope-collision.md`) is resolved by Option A adoption.
- **Phase 4 (MCP updates + AI-driven verification)** is unblocked as Phase 2 progresses. MCP responses should now teach v2 syntax again — the same v2 that Phase 2 will implement.

### What continues to require care

- **Aspirational warning banners in user-facing docs and book chapters** remain in place until Phase 2 delivers a working v2 plugin. Removing them prematurely (before the plugin ships) would produce documentation that describes syntax the shipped plugin still rejects. As each sub-cycle completes and the plugin surface catches up to the spec, the corresponding warnings can be removed. Full banner removal happens when Sub-cycle 4 ships.
- **Existing Frame apps using v1.2 syntax** will break when v2 ships. The AI-with-MCP migration mechanism (Phase 4 §7) is the intended migration path — no dedicated `cleen migrate` tool is built. Users need to be informed of the breaking change with sufficient lead time before v2 ships.
- **Studio and other downstream consumers** currently migrating to v1.2 patterns (`framework-frame-data-two-residual-v1-2-gaps.md`, `studio-frame-data-residual-gaps-status.md`) will need a separate migration path from v1.2 to v2. Not blocked by Phase 2 — can happen in parallel once v2 ships.

### Phase 1 completion record (unchanged)

- All eight design decisions in §11 resolved with rationale, alternatives, and ecosystem precedent recorded.
- `foundation/spec/plugins/frame-data.ebnf` rewritten from v1.2.0 to v2.0.0.
- `foundation/spec/plugins/frame-data-semantics.md` rewritten from v1.2.0 to v2.0.0.
- Two earlier overlapping proposals reconciled with supersession headers.
- Developer approval per Principle 25 recorded for all spec-file modifications.

### Migration phase status (updated 2026-07-17 after Sub-cycle 1 partial + MCP updates)

- **Phase 1 (Plugin spec finalization):** ✅ Complete 2026-07-10.
- **Phase 2 (frame.data plugin rewrite):** ▶️ **Sub-cycle 1 partial landed (commit `ccd5c2d`).** Remaining Sub-cycle 1 work: `.data` accessor generation on the entity class (DAT-A001..A005), entity/data-block pairing verification (DAT-P001..P005), type resolution from paired entity (DAT-M016 — currently stubbed as `string`), `indexes:`/`relations:`/`queries:` sub-block semantic consumers (the parser recognizes them as boundaries but does not yet emit code from them), Am 10 spec construction changes, and rejection diagnostic for bare-field `data:` (v1). Sub-cycles 2-4 (read verbs with DAT-Q020, Database service + pairing + invariants, DSL parsers + migrate + smoke test) all still pending — per-sub-cycle LOC targets sum to ~3300 additional LOC. The v3.0.10 binary (v1.2 semantics) remains what `cleen install latest` ships until the plugin owner runs `comita` for a v2 tag.
- **Phase 3 (docs and books rewrite):** ⏸️ **Warnings in place until Phase 2 ships.** Banners in 11 files (8 clean-framework docs + 3 book chapters) will be removed as Phase 2 sub-cycles land. Book chapters beyond ch12/ch13/ch14 (~230 v1 references across ~25 files) attempted this session but skipped: 5/7 target chapters (ch25, ch29, ch32, ch33, ch34) had uncommitted edits from a parallel session, matching the "active parallel activity" guardrail in the handoff prompt. Track B awaits a clean workspace.
- **Phase 4 (MCP updates + AI-driven verification):** ▶️ **MCP updates complete (commit `c53e6ec2`).** `tool_get_app_structure` and `tool_get_quick_reference` §Rule A now teach v2 patterns. Earlier session's reset risk did not recur — commit landed and survived cargo check. The AI-driven verification step still blocks on a working v2 plugin from Phase 2 (Sub-cycles 2-4).
