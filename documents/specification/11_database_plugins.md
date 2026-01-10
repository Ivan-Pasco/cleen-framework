# Database Plugins Specification (11)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.1
**Location:** `/docs/specification/11_database_plugins.md`

---

## 1. Introduction

Database plugins provide runtime database connectivity for Clean Language applications. Unlike compiler plugins (which transform code), database plugins are **runtime drivers** loaded by clean-server.

### Scope

This specification covers **SQL-style relational databases**. Document databases (MongoDB) and key-value stores (Redis) will be addressed in a separate specification due to their fundamentally different query models.

### Design Goals

- **Pluggable**: Add new database backends without modifying core server
- **Consistent API**: Same Clean Language API for all SQL databases
- **Runtime-Safe**: Parameterized queries prevent SQL injection
- **Connection Pooling**: Built-in connection management
- **Migration Support**: Schema migrations for all backends
- **ABI-Stable**: Plugins can be compiled independently

### Non-Goals (v1)

- Compile-time query validation (future: schema-aware tooling)
- ORM abstractions (users write SQL)
- NoSQL databases (separate spec)

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     CLEAN APPLICATION                            │
│                                                                  │
│   users = db.query("SELECT * FROM users WHERE active = ?", [1]) │
│   db.execute("INSERT INTO posts (title) VALUES (?)", [title])   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     CLEAN-SERVER                                 │
│                                                                  │
│   Bridge Functions (internal):                                   │
│   - _db_query(sql, params) -> JSON rows                         │
│   - _db_execute(sql, params) -> affected rows                   │
│   - _db_transaction_begin() -> connection_id                    │
│   - _db_transaction_commit(connection_id)                       │
│   - _db_transaction_rollback(connection_id)                     │
│                                                                  │
│   Plugin Manager:                                                │
│   - Load plugin via C-ABI vtable                                │
│   - Route calls to active driver                                 │
│   - Manage connection pool                                       │
│   - Own transaction state                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              DATABASE PLUGIN (C-ABI Interface)                   │
│                                                                  │
│   Vtable V1:                                                     │
│   - db_connect(config_json) -> status                           │
│   - db_query(conn, sql, params_json) -> result_json             │
│   - db_execute(conn, sql, params_json) -> affected              │
│   - db_begin(conn) -> status                                    │
│   - db_commit(conn) -> status                                   │
│   - db_rollback(conn) -> status                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────┬────────────┬────────────┬────────────┬────────────┐
│  SQLite    │ PostgreSQL │   MySQL    │   Oracle   │ SQL Server │
│  Plugin    │   Plugin   │   Plugin   │   Plugin   │   Plugin   │
└────────────┴────────────┴────────────┴────────────┴────────────┘
```

---

## 3. C-ABI Plugin Interface

### Why C-ABI Instead of Rust Traits

Using `Box<dyn Trait>` across dynamic library boundaries is **undefined behavior** in Rust unless the exact same compiler version and dependencies are used. This would force:
- Plugin authors to use exact compiler versions
- Rebuilding plugins on every server update
- Fragile, version-locked ecosystem

**Solution**: Use a C-ABI stable vtable interface.

### Vtable Definition (Version 1)

```rust
// clean-server/src/db/abi.rs

use std::os::raw::{c_char, c_int, c_void};

/// ABI version for compatibility checking
pub const DB_PLUGIN_ABI_VERSION: u32 = 1;

/// Plugin metadata (returned by db_plugin_info)
#[repr(C)]
pub struct DbPluginInfo {
    pub abi_version: u32,
    pub name: *const c_char,        // e.g., "sqlite"
    pub version: *const c_char,     // e.g., "1.0.0"
    pub description: *const c_char,
}

/// Connection handle (opaque to server)
pub type DbConnection = *mut c_void;

/// Result codes
pub const DB_OK: c_int = 0;
pub const DB_ERROR: c_int = -1;
pub const DB_NOT_CONNECTED: c_int = -2;
pub const DB_TRANSACTION_FAILED: c_int = -3;

/// Vtable structure - all plugins must export this
#[repr(C)]
pub struct DbPluginVtableV1 {
    /// Get plugin metadata
    pub info: extern "C" fn() -> DbPluginInfo,

    /// Create a new connection
    /// config_json: JSON string with connection parameters
    /// Returns: connection handle or null on failure
    pub connect: extern "C" fn(config_json: *const c_char) -> DbConnection,

    /// Close a connection
    pub disconnect: extern "C" fn(conn: DbConnection) -> c_int,

    /// Check if connection is alive
    pub is_connected: extern "C" fn(conn: DbConnection) -> c_int,

    /// Execute a SELECT query
    /// sql: SQL query string
    /// params_json: JSON array of parameters
    /// result_out: pointer to store result JSON string (caller must free)
    /// Returns: DB_OK or error code
    pub query: extern "C" fn(
        conn: DbConnection,
        sql: *const c_char,
        params_json: *const c_char,
        result_out: *mut *mut c_char,
    ) -> c_int,

    /// Execute INSERT/UPDATE/DELETE
    /// Returns: number of affected rows, or negative error code
    pub execute: extern "C" fn(
        conn: DbConnection,
        sql: *const c_char,
        params_json: *const c_char,
    ) -> i64,

    /// Execute INSERT and return last insert ID
    pub insert: extern "C" fn(
        conn: DbConnection,
        sql: *const c_char,
        params_json: *const c_char,
    ) -> i64,

    /// Begin transaction on this connection
    pub begin_transaction: extern "C" fn(conn: DbConnection) -> c_int,

    /// Commit transaction
    pub commit: extern "C" fn(conn: DbConnection) -> c_int,

    /// Rollback transaction
    pub rollback: extern "C" fn(conn: DbConnection) -> c_int,

    /// Free a result string allocated by the plugin
    pub free_result: extern "C" fn(result: *mut c_char),

    /// Get last error message
    pub last_error: extern "C" fn(conn: DbConnection) -> *const c_char,
}

/// Plugin entry point - every plugin must export this symbol
/// Returns pointer to static vtable
pub type DbPluginEntry = extern "C" fn() -> *const DbPluginVtableV1;
pub const DB_PLUGIN_ENTRY_SYMBOL: &str = "db_plugin_init";
```

### Plugin Implementation Example

```rust
// clean-db-sqlite/src/lib.rs

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use rusqlite::{Connection, params_from_iter};

static VTABLE: DbPluginVtableV1 = DbPluginVtableV1 {
    info: plugin_info,
    connect: plugin_connect,
    disconnect: plugin_disconnect,
    is_connected: plugin_is_connected,
    query: plugin_query,
    execute: plugin_execute,
    insert: plugin_insert,
    begin_transaction: plugin_begin_transaction,
    commit: plugin_commit,
    rollback: plugin_rollback,
    free_result: plugin_free_result,
    last_error: plugin_last_error,
};

#[no_mangle]
pub extern "C" fn db_plugin_init() -> *const DbPluginVtableV1 {
    &VTABLE
}

extern "C" fn plugin_info() -> DbPluginInfo {
    DbPluginInfo {
        abi_version: 1,
        name: b"sqlite\0".as_ptr() as *const c_char,
        version: b"1.0.0\0".as_ptr() as *const c_char,
        description: b"SQLite database driver\0".as_ptr() as *const c_char,
    }
}

extern "C" fn plugin_connect(config_json: *const c_char) -> DbConnection {
    let config_str = unsafe { CStr::from_ptr(config_json).to_str().unwrap_or("{}") };
    // Parse config, open connection
    // Return connection as opaque pointer
    // ...
}

// ... other implementations
```

---

## 4. Transaction Model

### Connection-Bound Transactions

Transactions are **bound to a specific connection**, not a transaction ID. This matches how databases actually work.

```
┌─────────────────────────────────────────────────────────────────┐
│                     CONNECTION POOL                              │
│                                                                  │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
│   │ Conn 1  │ │ Conn 2  │ │ Conn 3  │ │ Conn 4  │              │
│   │ (idle)  │ │ (in tx) │ │ (idle)  │ │ (query) │              │
│   └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
│                                                                  │
│   Transaction State (managed by SERVER, not plugin):            │
│   - Conn 2: tx_started=true, tx_start_time=...                 │
└─────────────────────────────────────────────────────────────────┘
```

### Server-Side Transaction Management

The **server owns transaction state**, not the plugin. This ensures:
- Consistent timeout handling
- Proper cleanup on errors
- Resource limit enforcement

```rust
// clean-server/src/db/transaction.rs

pub struct TransactionContext {
    connection_id: u64,
    started_at: Instant,
    timeout: Duration,
}

pub struct ConnectionPool {
    connections: Vec<PooledConnection>,
    active_transactions: HashMap<u64, TransactionContext>,
    max_transaction_time: Duration,
}

impl ConnectionPool {
    pub fn begin_transaction(&mut self) -> Result<u64, DbError> {
        // 1. Get connection from pool (or wait)
        let conn = self.acquire_connection()?;

        // 2. Call plugin's begin_transaction
        let status = unsafe { (self.vtable.begin_transaction)(conn.handle) };
        if status != DB_OK {
            return Err(DbError::TransactionFailed);
        }

        // 3. Track transaction state in server
        let tx_id = conn.id;
        self.active_transactions.insert(tx_id, TransactionContext {
            connection_id: tx_id,
            started_at: Instant::now(),
            timeout: self.max_transaction_time,
        });

        Ok(tx_id)
    }

    pub fn commit(&mut self, tx_id: u64) -> Result<(), DbError> {
        let ctx = self.active_transactions.remove(&tx_id)
            .ok_or(DbError::NoActiveTransaction)?;

        let conn = self.get_connection(ctx.connection_id)?;
        let status = unsafe { (self.vtable.commit)(conn.handle) };

        // Return connection to pool
        self.release_connection(conn);

        if status != DB_OK {
            return Err(DbError::CommitFailed);
        }
        Ok(())
    }
}
```

### Clean Language Transaction API

```clean
// Option 1: Block-based (recommended, auto commit/rollback)
db.transaction:
    db.execute("INSERT INTO orders ...")
    db.execute("UPDATE inventory ...")
    // Auto-commit on success, auto-rollback on error

// Option 2: Explicit handles (for advanced use)
integer txId = db.begin()
db.executeInTx(txId, "INSERT INTO orders ...")
db.executeInTx(txId, "UPDATE inventory ...")
db.commit(txId)
// Or: db.rollback(txId)
```

---

## 5. Clean Language API

### Naming Convention

The public API uses **friendly names** without underscores. Internally, these map to `_db_*` bridge functions.

| Public API | Internal Bridge | Description |
|------------|-----------------|-------------|
| `db.query(sql, params)` | `_db_query` | SELECT query, returns rows |
| `db.queryOne(sql, params)` | `_db_query_one` | Single row or null |
| `db.execute(sql, params)` | `_db_execute` | INSERT/UPDATE/DELETE |
| `db.insert(sql, params)` | `_db_insert` | INSERT, returns new ID |
| `db.begin()` | `_db_begin` | Start transaction |
| `db.commit(txId)` | `_db_commit` | Commit transaction |
| `db.rollback(txId)` | `_db_rollback` | Rollback transaction |

### Query Examples

```clean
// Basic query - returns list of rows
list users = db.query("SELECT * FROM users", [])

// Query with parameters (prevents SQL injection)
list user = db.query("SELECT * FROM users WHERE id = ?", [userId])

// Query with multiple parameters
list posts = db.query(
    "SELECT * FROM posts WHERE author_id = ? AND status = ?",
    [authorId, "published"]
)

// Single row query (returns row or null)
any user = db.queryOne("SELECT * FROM users WHERE email = ?", [email])
if user != null
    printl("Found user: " + user.name)
```

### Execute Examples

```clean
// Insert - returns affected rows
integer affected = db.execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    [name, email]
)

// Insert returning ID
integer newId = db.insert(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    [name, email]
)

// Update
integer updated = db.execute(
    "UPDATE users SET name = ? WHERE id = ?",
    [newName, userId]
)

// Delete
integer deleted = db.execute(
    "DELETE FROM users WHERE id = ?",
    [userId]
)
```

### Transaction Examples

```clean
// Block-based transaction (recommended)
db.transaction:
    db.execute("INSERT INTO orders (user_id, total) VALUES (?, ?)", [userId, total])
    integer orderId = db.insert("SELECT last_insert_rowid()")
    db.execute("INSERT INTO order_items (order_id, product_id) VALUES (?, ?)", [orderId, productId])
    db.execute("UPDATE products SET stock = stock - 1 WHERE id = ?", [productId])
    // Auto-commits if all succeed
    // Auto-rollbacks if any fails

// Explicit transaction (advanced)
integer txId = db.begin()
boolean success = false
try:
    db.executeInTx(txId, "INSERT INTO orders ...", [...])
    db.executeInTx(txId, "UPDATE inventory ...", [...])
    db.commit(txId)
    success = true
finally:
    if not success
        db.rollback(txId)
```

---

## 6. Type Safety Clarification

### Current Scope (v1): Runtime Safety

Version 1 provides **runtime safety**, not compile-time type checking:

| Feature | Status | Description |
|---------|--------|-------------|
| Parameterized queries | ✅ v1 | Prevents SQL injection |
| Type coercion | ✅ v1 | Params auto-converted to DB types |
| Result as JSON | ✅ v1 | Rows returned as dynamic objects |
| Query syntax validation | ❌ Future | Compile-time SQL parsing |
| Schema-aware types | ❌ Future | `db.query<User>(...)` |

### Future: Schema-Aware Tooling (v2+)

Future versions may add:

```clean
// Future: Typed queries (requires schema introspection)
User user = db.query<User>("SELECT * FROM users WHERE id = ?", [userId])

// Future: Compile-time SQL validation
// Compiler checks SQL syntax against schema.toml
```

This will be a separate spec when implemented.

---

## 7. Plugin Directory Structure

```
~/.cleen/db-plugins/
├── sqlite/
│   └── 1.0.0/
│       ├── plugin.toml           # Plugin manifest
│       ├── libclean_db_sqlite.dylib  (macOS)
│       ├── libclean_db_sqlite.so     (Linux)
│       └── clean_db_sqlite.dll       (Windows)
├── postgres/
│   └── 1.0.0/
│       ├── plugin.toml
│       └── libclean_db_postgres.dylib
└── mysql/
    └── 1.0.0/
        └── ...
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "clean-db-postgres"
version = "1.0.0"
driver = "postgres"
description = "PostgreSQL driver for Clean Language"

[abi]
version = 1                      # Must match DB_PLUGIN_ABI_VERSION
entry_symbol = "db_plugin_init"  # Default, can be customized

[compatibility]
min_server_version = "1.0.0"

[library]
macos = "libclean_db_postgres.dylib"
linux = "libclean_db_postgres.so"
windows = "clean_db_postgres.dll"

[features]
connection_pooling = true
prepared_statements = true
json_columns = true              # PostgreSQL JSONB support
array_columns = true             # PostgreSQL arrays
```

---

## 8. Server Configuration

### Project Configuration (config.cln)

```cln
// config.cln

config:
	database:
		driver = "postgres"           // sqlite, postgres, mysql
		host = "localhost"
		port = 5432
		database = "myapp"
		username = "user"
		password = env("DB_PASSWORD") // Environment variable
		pool_size = 10
		max_transaction_time = "30s"  // Transaction timeout

		options:
			ssl_mode = "require"
			connect_timeout = "10s"
```

### Environment Variables

```bash
# Connection string (alternative to config)
DATABASE_URL=postgres://user:pass@localhost/myapp

# Or individual variables
DB_DRIVER=postgres
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp
DB_USER=user
DB_PASSWORD=secret
```

---

## 9. Migration System

### Migration State Management

Migration state is stored in a **server-managed table**, not by plugins. This ensures consistency across all database backends.

```sql
-- Created automatically by clean-server
CREATE TABLE _clean_migrations (
    id INTEGER PRIMARY KEY,
    version VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    checksum VARCHAR(64)  -- SHA256 of migration content
);
```

### Migration File Format

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

### Migration CLI

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
# Output:
#   001_create_users      Applied  2025-01-15 10:30:00
#   002_add_posts_table   Pending

# Verify migrations (check checksums)
cleen db migrate verify
```

### Migration Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│                    MIGRATION PROCESS                             │
│                                                                  │
│   1. Read migrations/ directory                                  │
│   2. Query _clean_migrations table                              │
│   3. Compare: find pending migrations                            │
│   4. For each pending migration:                                │
│      a. Begin transaction                                        │
│      b. Execute UP SQL via plugin                               │
│      c. Insert into _clean_migrations                           │
│      d. Commit transaction                                       │
│   5. Report results                                              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 10. Security Considerations

### SQL Injection Prevention

- **All queries use parameterized statements**
- Raw SQL string interpolation is **not supported**
- Parameters are type-checked and escaped by the driver

```clean
// CORRECT: Parameterized query
db.query("SELECT * FROM users WHERE id = ?", [userId])

// WRONG: String interpolation (not supported)
// db.query("SELECT * FROM users WHERE id = " + userId)  // Won't compile
```

### Connection Security

| Feature | SQLite | PostgreSQL | MySQL |
|---------|--------|------------|-------|
| SSL/TLS | N/A | ✅ Required | ✅ Optional |
| Certificate validation | N/A | ✅ | ✅ |
| Encrypted passwords | N/A | ✅ | ✅ |

### Resource Limits

```toml
[database.limits]
max_query_time = "30s"        # Query timeout
max_transaction_time = "60s"  # Transaction timeout
max_rows_returned = 10000     # Prevent accidental huge queries
max_connections = 20          # Pool size limit
```

### Plugin Trust Model

| Source | Trust Level | Verification |
|--------|-------------|--------------|
| Official plugins | High | Signed, SHA256 verified |
| Community plugins | Medium | SHA256 checksum |
| Local plugins | User-defined | None (manual install) |

---

## 11. Supported Databases

| Database | Plugin Name | Priority | Notes |
|----------|-------------|----------|-------|
| SQLite | clean-db-sqlite | P0 | File-based, great for dev |
| PostgreSQL | clean-db-postgres | P0 | Full-featured, recommended |
| MySQL | clean-db-mysql | P1 | MySQL 8.0+ |
| MariaDB | clean-db-mysql | P1 | Uses MySQL plugin |
| SQL Server | clean-db-sqlserver | P2 | Microsoft SQL Server |
| Oracle | clean-db-oracle | P3 | Requires Oracle client |

### Out of Scope (Separate Spec)

| Database | Type | Notes |
|----------|------|-------|
| MongoDB | Document | Different query model |
| Redis | Key-Value | Different API surface |
| Elasticsearch | Search | Specialized queries |
| DynamoDB | Key-Value | AWS-specific |

---

## 12. Implementation Roadmap

### Phase 1: Core Infrastructure
- [ ] Define C-ABI vtable interface
- [ ] Implement plugin loader in clean-server
- [ ] Server-side connection pooling
- [ ] Transaction state management
- [ ] Bridge function integration

### Phase 2: SQLite Plugin
- [ ] Implement SQLite driver
- [ ] In-memory and file database support
- [ ] Test with example application

### Phase 3: PostgreSQL Plugin
- [ ] Implement Postgres driver
- [ ] SSL/TLS support
- [ ] Connection pooling

### Phase 4: Migration System
- [ ] Migration file parser
- [ ] `_clean_migrations` table management
- [ ] CLI commands (up, down, status)

### Phase 5: MySQL Plugin
- [ ] Implement MySQL driver
- [ ] MariaDB compatibility

### Phase 6: Additional Features
- [ ] Query timeouts
- [ ] Resource limits
- [ ] Plugin signing

---

## 13. Error Handling

### Error Codes

```rust
pub enum DbErrorCode {
    // Connection errors
    ConnectionFailed = 1001,
    ConnectionTimeout = 1002,
    AuthenticationFailed = 1003,

    // Query errors
    QueryFailed = 2001,
    QueryTimeout = 2002,
    SyntaxError = 2003,
    ConstraintViolation = 2004,

    // Transaction errors
    TransactionFailed = 3001,
    TransactionTimeout = 3002,
    DeadlockDetected = 3003,

    // Plugin errors
    PluginNotFound = 4001,
    ABIVersionMismatch = 4002,
    PluginLoadFailed = 4003,
}
```

### Clean Language Error Handling

```clean
// Errors throw exceptions that can be caught
try:
    db.execute("INSERT INTO users ...", [...])
catch DbError as error
    printl("Database error: " + error.message)
    printl("Error code: " + error.code.toString())
```

---

**End of Document 11 (v1.1)**
