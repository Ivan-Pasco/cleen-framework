# Frame Data Plugin Manual

**Version:** 1.0.0
**Compatible with:** Clean Language Compiler 0.15.0+, Clean Server 1.0.0+

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Installation](#2-installation)
3. [Configuration](#3-configuration)
4. [Model Definition](#4-model-definition)
5. [CRUD Operations](#5-crud-operations)
6. [Transactions](#6-transactions)
7. [Relationships](#7-relationships)
8. [Raw SQL Queries](#8-raw-sql-queries)
9. [Migrations](#9-migrations)
10. [CLI Commands](#10-cli-commands)
11. [API Reference](#11-api-reference)
12. [Error Handling](#12-error-handling)
13. [Best Practices](#13-best-practices)
14. [Troubleshooting](#14-troubleshooting)

---

## 1. Introduction

The **Frame Data Plugin** (`frame.data`) is the official ORM (Object-Relational Mapping) system for Clean Language applications. It provides a block-based, fully declarative syntax for defining data models, executing queries, and managing database operations.

### Key Features

- **Block-based syntax**: Declarative model and query definitions
- **Multi-database support**: PostgreSQL, MySQL, MariaDB, SQLite
- **Type safety**: Compile-time validation of model definitions
- **Automatic migrations**: Schema changes are tracked and applied
- **Connection pooling**: Built-in connection management
- **Transaction support**: Atomic operations with automatic rollback

### Supported Databases

| Database | Status | Connection URL Prefix |
|----------|--------|----------------------|
| PostgreSQL | Full Support | `postgres://` or `postgresql://` |
| MySQL | Full Support | `mysql://` |
| MariaDB | Full Support | `mariadb://` |
| SQLite | Full Support | `sqlite://` or `sqlite:` |

---

## 2. Installation

### 2.1 Prerequisites

Before installing the Frame Data Plugin, ensure you have:

- Clean Language Compiler v0.15.0 or later
- Clean Server v1.0.0 or later
- A supported database installed (or use SQLite for development)

### 2.2 Installing the Plugin

The Frame Data Plugin is included with the standard Frame Framework installation. To verify it's available:

```bash
# Check installed plugins
cleen plugins list

# Expected output:
# frame.data    1.0.0    ORM and database plugin for Clean Language
```

If the plugin is not installed, install it manually:

```bash
# Install from the official plugin registry
cleen plugins install frame.data

# Or install from a local path
cleen plugins install --path /path/to/frame.data
```

### 2.3 Plugin Directory Structure

The plugin is installed at:

```
~/.cleen/plugins/frame.data/
├── 1.0.0/
│   ├── plugin.toml       # Plugin manifest
│   ├── plugin.wasm       # Compiled plugin
│   └── src/
│       └── main.cln      # Plugin source
```

### 2.4 Building from Source

To build the plugin from source:

```bash
cd plugins/frame.data

# Build the plugin
./build.sh

# Or manually:
cln compile src/main.cln -o plugin.wasm --plugins
```

---

## 3. Configuration

### 3.1 Database Connection

Configure database connectivity using environment variables or configuration files.

#### Environment Variables

The simplest method is using the `DATABASE_URL` environment variable:

```bash
# PostgreSQL
export DATABASE_URL="postgres://user:password@localhost:5432/myapp"

# MySQL
export DATABASE_URL="mysql://user:password@localhost:3306/myapp"

# SQLite (file-based)
export DATABASE_URL="sqlite:///path/to/database.db"

# SQLite (in-memory)
export DATABASE_URL="sqlite::memory:"
```

#### Configuration File

Create `/config/data.cln` for more detailed configuration:

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

### 3.2 Connection Pool Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `max_connections` | 10 | Maximum connections in the pool |
| `min_connections` | 2 | Minimum idle connections |
| `connection_timeout` | 10000 | Timeout in milliseconds |
| `query_timeout` | 30000 | Query timeout in milliseconds |

### 3.3 Server Configuration

When starting the Clean Server, pass database configuration:

```bash
# Using environment variable
DATABASE_URL="postgres://user:pass@localhost/myapp" clean-server app.wasm

# Or with explicit parameters
clean-server app.wasm --database-url "postgres://user:pass@localhost/myapp" --port 3000
```

---

## 4. Model Definition

### 4.1 Basic Model Syntax

Use the `data` keyword to define database entities:

```clean
data User
    integer id : pk, auto
    string  name
    string  email : unique
    integer age
    boolean active = true
    datetime createdAt : default=now
```

### 4.2 Field Types

| Type | Database Type | Description |
|------|---------------|-------------|
| `integer` | INT/BIGINT | Whole numbers |
| `number` | FLOAT/DOUBLE | Decimal numbers |
| `string` | VARCHAR/TEXT | Text data |
| `boolean` | BOOLEAN | True/false values |
| `datetime` | TIMESTAMP | Date and time |
| `date` | DATE | Date only |
| `time` | TIME | Time only |
| `json` | JSON/JSONB | JSON data |
| `blob` | BLOB/BYTEA | Binary data |

### 4.3 Field Modifiers

| Modifier | Description | Example |
|----------|-------------|---------|
| `pk` | Primary key | `integer id : pk` |
| `auto` | Auto-increment | `integer id : pk, auto` |
| `unique` | Unique constraint | `string email : unique` |
| `required` | Not null | `string name : required` |
| `default=value` | Default value | `boolean active = true` |
| `default=now` | Current timestamp | `datetime createdAt : default=now` |

### 4.4 Complete Model Example

```clean
data User
    integer id : pk, auto
    string  name : required
    string  email : unique, required
    string  password : required
    integer age
    boolean active = true
    string  role = "user"
    datetime createdAt : default=now
    datetime updatedAt
```

### 4.5 Model Naming Conventions

| Convention | Description | Example |
|------------|-------------|---------|
| Model name | PascalCase | `User`, `BlogPost` |
| Table name | Auto-generated snake_case | `users`, `blog_posts` |
| Field name | camelCase | `createdAt`, `userId` |
| Column name | Auto-generated snake_case | `created_at`, `user_id` |

---

## 5. CRUD Operations

### 5.1 Insert

Create new records using the `insert:` block:

```clean
User.insert:
    name  = "Alice"
    email = "alice@example.com"
    age   = 30
    active = true
```

Insert with variable assignment:

```clean
User newUser = User.insert:
    name  = "Bob"
    email = "bob@example.com"
```

### 5.2 Select (Find)

#### Find Multiple Records

```clean
list<User> users = User.find:
    where:
        active == true
```

#### Find with Multiple Conditions

```clean
list<User> users = User.find:
    where:
        active == true
        age >= 18
    order:
        createdAt desc
    limit: 20
```

#### Find Single Record

```clean
User? user = User.first:
    where:
        email == "alice@example.com"
```

### 5.3 Update

Update records matching conditions:

```clean
User.update:
    set:
        active = false
    where:
        email == "alice@example.com"
```

Update with multiple fields:

```clean
User.update:
    set:
        active = true
        role = "admin"
        updatedAt = now()
    where:
        id == userId
```

### 5.4 Delete

Delete records matching conditions:

```clean
User.delete:
    where:
        active == false
```

Delete by ID:

```clean
User.delete:
    where:
        id == userId
```

### 5.5 Count

Count records:

```clean
integer total = User.count:
    where:
        active == true
```

---

## 6. Transactions

### 6.1 Block-Based Transactions

Use `transaction:` for atomic operations that automatically commit on success or rollback on failure:

```clean
transaction:
    User u = User.insert:
        name = "Ana"
        email = "ana@example.com"

    Post p = Post.insert:
        title   = "Welcome"
        content = "Hello world"
        author  = u
```

### 6.2 Transaction Behavior

- **Auto-commit**: If all operations succeed, changes are committed
- **Auto-rollback**: If any operation fails, all changes are rolled back
- **Nested blocks**: All operations within `transaction:` are part of the same transaction

### 6.3 Manual Transaction Control

For advanced use cases, use explicit transaction handles:

```clean
integer txId = db.begin()

db.executeInTx(txId, "INSERT INTO orders (user_id, total) VALUES (?, ?)", [userId, total])
db.executeInTx(txId, "UPDATE inventory SET stock = stock - 1 WHERE id = ?", [productId])
db.commit(txId)
onError db.rollback(txId)
```

**Note:** The block-based `transaction:` syntax is strongly recommended as it handles commit/rollback automatically.

---

## 7. Relationships

### 7.1 One-to-Many Relationships

Define a foreign key by using another model as a field type:

```clean
data Post
    integer id : pk, auto
    string  title
    string  content
    User    author
    datetime createdAt : default=now
```

This automatically creates:
- A foreign key from `Post.author` to `User.id`
- A reverse property `user.posts`

### 7.2 Many-to-Many Relationships

Many-to-many relationships use explicit junction models:

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

### 7.3 Querying Many-to-Many

Find roles for a user:

```clean
list<Role> roles = Role.find:
    link:
        UserRole role == id
    where:
        UserRole.user == someUser
    order:
        name asc
```

Find users with a specific role:

```clean
list<User> admins = User.find:
    link:
        UserRole user == id
        Role     id == UserRole.role
    where:
        Role.name == "admin"
```

### 7.4 Creating and Removing Links

```clean
// Create link
UserRole.insert:
    user = someUser
    role = someRole

// Remove link
UserRole.delete:
    where:
        user == someUser
        role == someRole
```

---

## 8. Raw SQL Queries

### 8.1 Typed Queries

Use `db.queryAs` for queries that return typed results:

```clean
list<User> users = db.queryAs(User):
    sql: "SELECT * FROM users WHERE age > ?"
    params: [18]
```

### 8.2 Untyped Queries

For ad-hoc or complex queries:

```clean
list<map<string, any>> result = db.query:
    sql: "SELECT COUNT(*) as total FROM users"
```

### 8.3 Query with Parameters

Always use parameterized queries to prevent SQL injection:

```clean
// CORRECT: Parameterized query
list users = db.query("SELECT * FROM users WHERE id = ?", [userId])

// WRONG: Never do this (SQL injection risk)
// db.query("SELECT * FROM users WHERE id = " + userId)  // NOT SUPPORTED
```

### 8.4 Execute Statements

For INSERT, UPDATE, DELETE operations:

```clean
// Execute returns affected rows
integer affected = db.execute(
    "UPDATE users SET active = ? WHERE last_login < ?",
    [false, oldDate]
)

// Insert returns new ID
integer newId = db.insert(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    [name, email]
)
```

---

## 9. Migrations

### 9.1 Automatic Migrations

Frame Data automatically generates migrations based on model changes:

```bash
# Preview migration SQL
frame db:plan

# Apply migrations
frame db:migrate
```

### 9.2 Migration Files

Migrations are stored in `/db/migrations/`:

```
/db/migrations/
    001_init.sql
    002_add_posts.sql
    003_add_user_roles.sql
```

### 9.3 Migration File Format

```sql
-- migrations/001_create_users.sql

-- migrate:up
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

-- migrate:down
DROP TABLE users;
```

### 9.4 Per-Model Migrations

Run migrations for specific models:

```clean
User.migrate()
Post.migrate()
```

### 9.5 Migration State

Migration state is tracked in the `_clean_migrations` table:

```sql
CREATE TABLE _clean_migrations (
    id INTEGER PRIMARY KEY,
    version VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    checksum VARCHAR(64)
);
```

---

## 10. CLI Commands

### 10.1 Database Commands

| Command | Description |
|---------|-------------|
| `frame db:plan` | Show pending migration SQL |
| `frame db:migrate` | Apply pending migrations |
| `frame db:seed` | Run database seeding |
| `frame db:reset` | Drop and recreate schema |
| `frame db:info` | Show database connection info |

### 10.2 Migration CLI

```bash
# Create new migration
cleen db migrate create add_posts_table
# Creates: migrations/002_add_posts_table.sql

# Run pending migrations
cleen db migrate up

# Rollback last migration
cleen db migrate down

# Rollback to specific version
cleen db migrate down --to 001

# Show migration status
cleen db migrate status

# Verify migrations (check checksums)
cleen db migrate verify
```

### 10.3 Seeding

Create seed data in `/db/seed.cln`:

```clean
functions:
    void seed()
        User.insert:
            name  = "Admin"
            email = "admin@example.com"
            role  = "admin"

        User.insert:
            name  = "Demo User"
            email = "demo@example.com"
```

Run with:

```bash
frame db:seed
```

---

## 11. API Reference

### 11.1 Model Static Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `Model.find:` | Find multiple records | `list<Model>` |
| `Model.first:` | Find single record | `Model?` |
| `Model.insert:` | Create new record | `Model` |
| `Model.update:` | Update records | `integer` (affected rows) |
| `Model.delete:` | Delete records | `integer` (deleted rows) |
| `Model.count:` | Count records | `integer` |
| `Model.migrate()` | Run migrations | `void` |

### 11.2 Query Sub-Blocks

| Block | Description | Example |
|-------|-------------|---------|
| `where:` | Filter conditions | `where: active == true` |
| `order:` | Sort results | `order: createdAt desc` |
| `limit:` | Limit results | `limit: 20` |
| `offset:` | Skip results | `offset: 10` |
| `link:` | Join relationships | `link: UserRole user == id` |

### 11.3 Database Functions

| Function | Description |
|----------|-------------|
| `db.query(sql, params)` | Execute SELECT, returns rows |
| `db.queryOne(sql, params)` | Execute SELECT, returns single row or null |
| `db.execute(sql, params)` | Execute INSERT/UPDATE/DELETE |
| `db.insert(sql, params)` | Execute INSERT, returns new ID |
| `db.begin()` | Start transaction |
| `db.commit(txId)` | Commit transaction |
| `db.rollback(txId)` | Rollback transaction |

### 11.4 Host Bridge Contracts

| Bridge Function | Description |
|-----------------|-------------|
| `host:db.query` | Execute SQL and return rows |
| `host:db.tx` | Run multiple queries transactionally |
| `host:db.prepare` | Prepare SQL statement |
| `host:env.get` | Read database URL or secrets |
| `host:log.info` | ORM operation logging |

---

## 12. Error Handling

### 12.1 Error Codes

| Code | Category | Description |
|------|----------|-------------|
| 1001 | Connection | Connection failed |
| 1002 | Connection | Connection timeout |
| 1003 | Connection | Authentication failed |
| 2001 | Query | Query failed |
| 2002 | Query | Query timeout |
| 2003 | Query | Syntax error |
| 2004 | Query | Constraint violation |
| 3001 | Transaction | Transaction failed |
| 3002 | Transaction | Transaction timeout |
| 3003 | Transaction | Deadlock detected |

### 12.2 Error Response Format

All errors follow the standard envelope format:

```json
{
    "ok": false,
    "err": {
        "code": "VALIDATION_ERROR",
        "field": "email",
        "message": "must be unique"
    }
}
```

### 12.3 Clean Language Error Handling

Clean Language uses `onError` for error handling:

```clean
// Handle errors with fallback value
User user = User.first:
    where:
        email == "alice@example.com"
onError null

// Handle errors with error logging
User.insert:
    name  = "Alice"
    email = "alice@example.com"
onError print("Database error: " + error.message) +
```

---

## 13. Best Practices

### 13.1 Model Design

- Keep **one model per file** for better organization
- Use **meaningful field names** (e.g., `createdAt` not `ca`)
- Always include **timestamps** (`createdAt`, `updatedAt`)
- Use **appropriate field types** (don't store dates as strings)

### 13.2 Query Optimization

- Use **indexed columns** in WHERE clauses
- Add **LIMIT** to prevent large result sets
- Use **COUNT** instead of fetching all records when only counting
- Use **transactions** for multiple related operations

### 13.3 Security

- **Always use parameterized queries** - never concatenate user input
- **Validate input** before database operations
- **Use transactions** for data integrity
- **Hash passwords** before storing (use `crypto.hash()`)

### 13.4 Transaction Guidelines

- Keep transactions **short** to avoid lock contention
- Use `transaction:` blocks for **related operations**
- Handle **errors properly** - don't leave orphaned transactions

### 13.5 Migration Guidelines

- **Never modify** applied migrations - create new ones
- **Test migrations** on development database first
- **Include rollback** (`migrate:down`) for all migrations
- **Commit migrations** with related code changes

---

## 14. Troubleshooting

### 14.1 Connection Issues

**Error: Connection failed**

```
Error: Connection to database failed
Code: 1001
```

**Solutions:**
1. Verify `DATABASE_URL` is correct
2. Check database server is running
3. Verify credentials (username/password)
4. Check network connectivity and firewall rules
5. Ensure database exists

**Diagnostic commands:**

```bash
# Test PostgreSQL connection
psql "$DATABASE_URL" -c "SELECT 1"

# Test MySQL connection
mysql -h localhost -u user -p database -e "SELECT 1"

# Check SQLite file
ls -la /path/to/database.db
```

### 14.2 Query Issues

**Error: Query timeout**

```
Error: Query execution timeout
Code: 2002
```

**Solutions:**
1. Add indexes to frequently queried columns
2. Use LIMIT for large result sets
3. Increase `query_timeout` configuration
4. Optimize complex queries

### 14.3 Transaction Issues

**Error: Transaction timeout**

```
Error: Transaction exceeded maximum time
Code: 3002
```

**Solutions:**
1. Break large transactions into smaller ones
2. Increase `max_transaction_time` if needed
3. Avoid long-running operations in transactions

### 14.4 Migration Issues

**Error: Migration checksum mismatch**

```
Error: Migration 001_init.sql has been modified
```

**Solutions:**
1. Never modify applied migrations
2. Create a new migration for schema changes
3. If necessary, reset database: `frame db:reset` (WARNING: destroys data)

### 14.5 Common Mistakes

| Mistake | Solution |
|---------|----------|
| String concatenation in SQL | Use parameterized queries |
| Missing transactions for related updates | Wrap in `transaction:` block |
| Large unbounded queries | Always use `limit:` |
| Storing passwords in plaintext | Use `crypto.hashPassword()` |
| Not handling null results | Check for `null` on `first:` queries |

---

## Appendix A: Database-Specific Notes

### PostgreSQL

- Full JSON/JSONB support
- Array column support
- UUID type support
- Recommended for production

### MySQL/MariaDB

- Use MySQL 8.0+ for best compatibility
- JSON support available
- UTF8MB4 encoding recommended

### SQLite

- Great for development and testing
- Single-file database
- No server required
- Use `:memory:` for in-memory testing

---

## Appendix B: Complete Example Application

```clean
// models/User.cln
data User
    integer id : pk, auto
    string  name : required
    string  email : unique, required
    string  password : required
    boolean active = true
    datetime createdAt : default=now

// models/Post.cln
data Post
    integer id : pk, auto
    string  title : required
    string  content
    User    author
    boolean published = false
    datetime createdAt : default=now

// main.cln
start()
    // Run migrations
    User.migrate()
    Post.migrate()

    // Create user in transaction
    transaction:
        User admin = User.insert:
            name     = "Admin"
            email    = "admin@example.com"
            password = crypto.hashPassword("secret123")

        Post.insert:
            title   = "Welcome"
            content = "Welcome to our blog!"
            author  = admin
            published = true

    // Query users
    list<User> activeUsers = User.find:
        where:
            active == true
        order:
            createdAt desc
        limit: 10

    // Count posts
    integer postCount = Post.count:
        where:
            published == true

    print("Active users: " + activeUsers.length().toString()) +
    print("Published posts: " + postCount.toString()) +
```

---

**End of Frame Data Plugin Manual**
