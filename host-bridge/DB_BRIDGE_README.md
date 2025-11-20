# Database Bridge Implementation

## Overview

The Database Bridge (`DbBridge`) provides complete database access capabilities for the Frame Framework. It supports PostgreSQL, MySQL, and SQLite through the `sqlx` crate with connection pooling, transaction management, and comprehensive error handling.

## Features

### Core Functionality

1. **Connection Pooling**
   - Configurable min/max connections
   - Automatic connection health checks
   - Connection timeout enforcement
   - Automatic reconnection on connection loss

2. **Query Execution**
   - Parameterized queries (SQL injection prevention)
   - Query timeout enforcement (default 30s)
   - Type-safe parameter binding
   - Automatic type conversion (SQL → JSON)

3. **Transaction Management**
   - ACID-compliant transactions
   - Nested operation support
   - Automatic rollback on errors
   - Thread-safe transaction tracking

4. **Error Handling**
   - Standardized error codes
   - Error message sanitization
   - Detailed error categorization
   - Security-focused error responses

## API Functions

### host:db.config

Configure database connection settings.

**Request:**
```json
{
  "fn": "host:db.config",
  "args": {
    "database_url": "postgres://user:pass@localhost/dbname",
    "max_connections": 10,
    "min_connections": 2,
    "connection_timeout": 10000,
    "query_timeout": 30000
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": null
}
```

### host:db.query

Execute SELECT queries and return rows.

**Request:**
```json
{
  "fn": "host:db.query",
  "args": {
    "sql": "SELECT * FROM users WHERE id = $1",
    "params": [123]
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "rows": [
      {"id": 123, "name": "Alice", "email": "alice@example.com"}
    ],
    "count": 1
  }
}
```

### host:db.execute

Execute INSERT/UPDATE/DELETE queries and return affected rows.

**Request:**
```json
{
  "fn": "host:db.execute",
  "args": {
    "sql": "INSERT INTO users (name, email) VALUES ($1, $2)",
    "params": ["Bob", "bob@example.com"]
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "affected_rows": 1,
    "last_insert_id": 124
  }
}
```

### host:db.transaction_begin

Start a new transaction.

**Request:**
```json
{
  "fn": "host:db.transaction_begin",
  "args": {}
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "tx_id": "tx_abc123def456"
  }
}
```

### host:db.transaction_commit

Commit a transaction.

**Request:**
```json
{
  "fn": "host:db.transaction_commit",
  "args": {
    "tx_id": "tx_abc123def456"
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": null
}
```

### host:db.transaction_rollback

Rollback a transaction.

**Request:**
```json
{
  "fn": "host:db.transaction_rollback",
  "args": {
    "tx_id": "tx_abc123def456"
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": null
}
```

### host:db.query_in_tx

Execute a SELECT query within a transaction.

**Request:**
```json
{
  "fn": "host:db.query_in_tx",
  "args": {
    "tx_id": "tx_abc123def456",
    "sql": "SELECT * FROM users WHERE id = $1",
    "params": [123]
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "rows": [...],
    "count": 1
  }
}
```

### host:db.execute_in_tx

Execute an INSERT/UPDATE/DELETE query within a transaction.

**Request:**
```json
{
  "fn": "host:db.execute_in_tx",
  "args": {
    "tx_id": "tx_abc123def456",
    "sql": "INSERT INTO users (name, email) VALUES ($1, $2)",
    "params": ["Charlie", "charlie@example.com"]
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "affected_rows": 0,
    "last_insert_id": null
  }
}
```

Note: Actual execution happens on commit, so affected_rows and last_insert_id are always 0/null for execute_in_tx.

## Error Codes

| Code | Description |
|------|-------------|
| `DB_ERROR` | General database errors |
| `CONNECTION_ERROR` | Connection failures |
| `QUERY_ERROR` | Query execution errors |
| `TRANSACTION_ERROR` | Transaction management errors |
| `VALIDATION_ERROR` | Invalid parameters or SQL, constraint violations |
| `TIMEOUT` | Query timeout exceeded |
| `PERMISSION_DENIED` | Access denied errors |
| `NOT_FOUND` | Resource not found |

## Type Mappings

### PostgreSQL / MySQL

| SQL Type | JSON Type | Example |
|----------|-----------|---------|
| INTEGER, BIGINT | number | 123 |
| TEXT, VARCHAR | string | "Alice" |
| BOOLEAN | boolean | true |
| TIMESTAMP | string (ISO 8601) | "2025-11-19T12:00:00.000" |
| JSON, JSONB | object/array | {"key": "value"} |
| NULL | null | null |

### SQLite

| SQL Type | JSON Type |
|----------|-----------|
| INTEGER | number |
| TEXT | string |
| REAL | number |
| BLOB | string (base64) |
| NULL | null |

## Security Features

1. **SQL Injection Prevention**
   - All queries use parameterized statements
   - No raw SQL execution without parameters
   - Parameter binding enforced at type level

2. **Error Message Sanitization**
   - SQL queries removed from error messages
   - Error messages truncated to 200 characters
   - Prevents information leakage

3. **Connection String Validation**
   - URL parsing and validation
   - Configuration parameter validation
   - Secure connection defaults

4. **Transaction Isolation**
   - Read Committed isolation level (default)
   - Proper transaction state tracking
   - Prevents transaction leakage

## Usage Examples

### Simple Query

```rust
use host_host::{DbBridge, DbConfig};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut bridge = DbBridge::new();

    // Configure
    bridge.configure(DbConfig {
        database_url: "postgres://localhost/mydb".to_string(),
        max_connections: 10,
        min_connections: 2,
        connection_timeout: 10000,
        query_timeout: 30000,
    }).await.unwrap();

    // Query
    let result = bridge.call("query", json!({
        "sql": "SELECT * FROM users WHERE active = $1",
        "params": [true]
    })).await.unwrap();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
```

### Transaction

```rust
// Begin transaction
let tx_begin = bridge.call("transaction_begin", json!({})).await.unwrap();
let tx_id = tx_begin["data"]["tx_id"].as_str().unwrap();

// Execute operations
bridge.call("execute_in_tx", json!({
    "tx_id": tx_id,
    "sql": "INSERT INTO users (name, email) VALUES ($1, $2)",
    "params": ["Alice", "alice@example.com"]
})).await.unwrap();

bridge.call("execute_in_tx", json!({
    "tx_id": tx_id,
    "sql": "INSERT INTO posts (title, user_id) VALUES ($1, $2)",
    "params": ["First Post", 1]
})).await.unwrap();

// Commit
bridge.call("transaction_commit", json!({"tx_id": tx_id})).await.unwrap();
```

## Testing

The implementation includes comprehensive tests covering:

- Connection configuration
- Query execution (SELECT, INSERT, UPDATE, DELETE)
- Transaction management (begin, commit, rollback)
- Error handling (validation, constraints, timeouts)
- Parameter binding and type conversion

Run tests:
```bash
cargo test --lib db::tests
```

## Performance Considerations

1. **Connection Pooling**: Reuses connections to avoid overhead
2. **Query Timeout**: Prevents long-running queries from blocking
3. **Parameterized Queries**: Allows query plan caching
4. **Async Implementation**: Non-blocking I/O using tokio
5. **Type Conversion**: Efficient JSON serialization

## Future Enhancements

Potential improvements:

1. **Prepared Statements**: Cache frequently-used queries
2. **Batch Operations**: Execute multiple queries in one call
3. **Streaming Results**: Support for large result sets
4. **Query Builder**: Type-safe query construction
5. **Migration Support**: Schema versioning and migrations
6. **Read Replicas**: Support for read/write splitting

## Dependencies

- `sqlx` (0.7): Database driver and connection pooling
- `tokio`: Async runtime
- `serde_json`: JSON serialization
- `uuid`: Transaction ID generation
- `chrono`: Date/time handling (via sqlx)

## References

- [Frame Bridge Contracts Specification](../documents/specification/frame_bridge_contracts.md)
- [Frame Data Specification](../documents/specification/04_frame_data.md)
- [sqlx Documentation](https://docs.rs/sqlx/)
