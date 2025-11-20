# Phase 3: Frame Data ORM Implementation Plan

**Branch:** `feature/phase-3-orm`
**Status:** In Progress
**Goal:** Implement a production-ready ORM with real database connectivity

## Overview

Phase 3 focuses on implementing the Frame Data ORM, moving from placeholder code to a fully functional database abstraction layer. This ORM will connect to databases via the Host Bridge and provide a type-safe, ergonomic API for Clean Language applications.

## Current State

### Existing Implementation (Placeholders)

**frame-data/src/lib.rs** - Connection and pool stubs
- `Connection` struct (no actual connection)
- `ConnectionPool` (not implemented)
- Basic `query()` and `execute()` methods (placeholders)

**frame-data/src/model.rs** - Model trait
- `Model` trait with CRUD operations
- All methods return dummy data
- No actual database interaction

**frame-data/src/query.rs** - Query builder
- `QueryBuilder` struct
- Methods for building queries (select, where, etc.)
- No SQL generation

**frame-data/src/schema.rs** - Schema definitions
- `Column`, `ColumnType`, `Constraint` enums
- Basic schema representation
- No schema migration support

## Phase 3 Goals

### 1. Connection Management ✅ Goals

**Implement real database connections via Host Bridge:**
- Connection pooling with configurable pool size
- Connection lifecycle management (acquire, release, health checks)
- Support for PostgreSQL, MySQL, SQLite
- Error handling and retry logic
- Connection timeout configuration

**Key Features:**
```rust
pub struct Connection {
    driver: String,
    bridge: Arc<DbBridge>, // Host Bridge integration
    transaction_depth: usize,
}

pub struct ConnectionPool {
    config: PoolConfig,
    available: Vec<Connection>,
    in_use: Vec<Connection>,
    max_size: usize,
}
```

### 2. Query Builder ✅ Goals

**Complete SQL generation:**
- SELECT queries with joins, where clauses, ordering
- INSERT queries with multiple rows, ON CONFLICT
- UPDATE queries with conditional updates
- DELETE queries with cascading
- Support for complex WHERE conditions (AND, OR, IN, LIKE, etc.)
- Parameterized queries to prevent SQL injection

**Key Features:**
```rust
impl QueryBuilder {
    pub fn select(&mut self, columns: Vec<&str>) -> &mut Self;
    pub fn from(&mut self, table: &str) -> &mut Self;
    pub fn where_clause(&mut self, condition: &str) -> &mut Self;
    pub fn join(&mut self, table: &str, on: &str) -> &mut Self;
    pub fn order_by(&mut self, column: &str, direction: OrderDirection) -> &mut Self;
    pub fn limit(&mut self, limit: u32) -> &mut Self;
    pub fn offset(&mut self, offset: u32) -> &mut Self;
    pub fn build(&self) -> (String, Vec<serde_json::Value>);
}
```

### 3. Model Implementation ✅ Goals

**Full CRUD operations via Host Bridge:**
- `create()` - Insert new records
- `find()` - Find by primary key
- `where_clause()` - Find by conditions
- `all()` - Fetch all records
- `update()` - Update existing records
- `delete()` - Delete records
- Eager loading for relationships
- Lazy loading support

**Key Features:**
```rust
#[async_trait]
pub trait Model: Sized + Serialize + DeserializeOwned {
    fn table_name() -> &'static str;
    fn primary_key() -> &'static str { "id" }

    async fn create(conn: &mut Connection, data: Self) -> Result<Self>;
    async fn find(conn: &mut Connection, id: i64) -> Result<Option<Self>>;
    async fn where_clause(conn: &mut Connection, conditions: HashMap<String, serde_json::Value>) -> Result<Vec<Self>>;
    async fn update(conn: &mut Connection, id: i64, data: Self) -> Result<Self>;
    async fn delete(conn: &mut Connection, id: i64) -> Result<bool>;
}
```

### 4. Relationships ✅ Goals

**Support for common relationship types:**
- `hasMany` - One-to-many relationships
- `belongsTo` - Many-to-one relationships
- `hasOne` - One-to-one relationships
- `manyToMany` - Many-to-many via junction tables
- Eager loading with `with()` method
- Lazy loading with accessor methods

**Example Usage:**
```rust
// Define relationship
impl Post {
    pub async fn comments(&self, conn: &mut Connection) -> Result<Vec<Comment>> {
        Comment::where_clause(conn, hashmap!{
            "post_id" => self.id.into()
        }).await
    }
}

// Eager loading
let posts = Post::with(conn, "comments").all().await?;
```

### 5. Transactions ✅ Goals

**ACID transaction support:**
- Begin, commit, rollback
- Nested transactions (savepoints)
- Automatic rollback on errors
- Transaction isolation levels

**Key Features:**
```rust
impl Connection {
    pub async fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Connection) -> Future<Output = Result<R>>,
    {
        self.begin_transaction().await?;
        match f(self).await {
            Ok(result) => {
                self.commit_transaction().await?;
                Ok(result)
            }
            Err(e) => {
                self.rollback_transaction().await?;
                Err(e)
            }
        }
    }
}
```

### 6. Schema Management ✅ Goals

**Migration system integration:**
- Parse Clean Language schema definitions
- Generate SQL CREATE TABLE statements
- Detect schema changes and generate ALTER TABLE
- Support for indexes, foreign keys, constraints
- Schema versioning and tracking

## Implementation Plan

### Week 1: Connection Management

**Days 1-2: Host Bridge Integration**
- Create `DbBridge` trait for Host Bridge communication
- Implement connection establishment via bridge calls
- Add connection health checks
- Error handling and logging

**Days 3-4: Connection Pooling**
- Implement connection pool with configurable size
- Add connection acquisition and release logic
- Implement connection timeout and retry
- Add pool statistics and monitoring

**Day 5: Testing**
- Unit tests for connection management
- Integration tests with mock bridge
- Error handling tests
- Load testing for connection pool

### Week 2: Query Builder

**Days 1-2: SELECT Queries**
- Implement SELECT with column selection
- Add WHERE clause building
- Implement JOIN operations
- Add ORDER BY, LIMIT, OFFSET

**Days 3-4: DML Queries**
- Implement INSERT with parameterization
- Add UPDATE with conditional updates
- Implement DELETE with cascading
- Add bulk operations

**Day 5: Testing**
- Unit tests for all query types
- SQL generation verification
- Parameterization security tests
- Complex query integration tests

### Week 3: Model CRUD

**Days 1-2: Create and Read**
- Implement create() with INSERT
- Add find() with SELECT by PK
- Implement where_clause() with filtering
- Add all() for full table scans

**Days 3-4: Update and Delete**
- Implement update() with conditional updates
- Add delete() with cascading
- Implement soft deletes
- Add batch operations

**Day 5: Testing**
- Unit tests for all CRUD operations
- Integration tests with test database
- Error handling tests
- Performance benchmarks

### Week 4: Relationships & Transactions

**Days 1-2: Relationships**
- Implement hasMany relationships
- Add belongsTo relationships
- Implement eager loading
- Add lazy loading

**Days 3-4: Transactions**
- Implement transaction begin/commit/rollback
- Add savepoint support
- Implement automatic rollback on error
- Add isolation level configuration

**Day 5: Final Testing & Documentation**
- Comprehensive integration tests
- End-to-end workflow tests
- Performance benchmarking
- Documentation update

## Testing Strategy

### Unit Tests
- Individual component testing
- Mock Host Bridge for isolation
- Edge case coverage
- Error condition testing

### Integration Tests
- Real database connections (PostgreSQL in CI)
- Full CRUD workflows
- Relationship testing
- Transaction testing
- Migration testing

### Performance Tests
- Connection pool efficiency
- Query execution time
- Bulk operation performance
- Memory usage under load

## Success Criteria

### Functional Requirements
- ✅ All CRUD operations work via Host Bridge
- ✅ Queries generate correct SQL
- ✅ Relationships load correctly
- ✅ Transactions work atomically
- ✅ Connection pool manages resources efficiently

### Non-Functional Requirements
- ✅ 100% test coverage for ORM core
- ✅ < 1ms query building overhead
- ✅ Support for 1000+ concurrent connections
- ✅ Memory-safe (no leaks)
- ✅ Graceful error handling

### Code Quality
- ✅ No unsafe code blocks
- ✅ Clear error messages
- ✅ Comprehensive documentation
- ✅ Type-safe API
- ✅ Idiomatic Rust

## Dependencies

### New Dependencies Needed
```toml
# Database drivers (via Host Bridge)
# No direct database drivers needed - goes through bridge

# Async runtime
async-trait = "0.1"

# Utilities
dashmap = "5.5"  # Concurrent hashmap for connection pool
parking_lot = "0.12"  # Faster mutexes

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Migration from Placeholder Code

### Before (Placeholder)
```rust
pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
    // Placeholder - would use Host Bridge
    Ok(vec![])
}
```

### After (Real Implementation)
```rust
pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
    let request = DbRequest {
        operation: "query".to_string(),
        sql: sql.to_string(),
        params,
    };

    let response = self.bridge.execute(request).await?;

    match response {
        DbResponse::Rows(rows) => Ok(rows),
        DbResponse::Error(err) => Err(anyhow!(err)),
        _ => Err(anyhow!("Unexpected response type")),
    }
}
```

## Risks and Mitigation

### Risk: Host Bridge Latency
**Mitigation:** Connection pooling, query batching, caching

### Risk: SQL Injection
**Mitigation:** Always use parameterized queries, no string concatenation

### Risk: Connection Leaks
**Mitigation:** RAII pattern with Drop trait, connection pool monitoring

### Risk: Transaction Deadlocks
**Mitigation:** Timeout configuration, deadlock detection, automatic retry

## Documentation

### User Documentation
- ORM API reference
- Query builder guide
- Relationship usage examples
- Transaction best practices
- Migration guide

### Developer Documentation
- Architecture overview
- Host Bridge integration
- Adding new database drivers
- Performance tuning
- Troubleshooting guide

## Deliverables

1. ✅ Fully functional ORM with Host Bridge integration
2. ✅ Comprehensive test suite (100% coverage)
3. ✅ Performance benchmarks
4. ✅ Documentation and examples
5. ✅ Migration from placeholder code

## Next Steps After Phase 3

With the ORM complete, Phase 4 will focus on:
- **Frame UI:** Component system, SSR, hydration
- **Frame Auth:** Authentication and authorization
- **Frame Server:** Enhanced routing and middleware
- **Integration:** Connecting all layers together

## Timeline

**Estimated Duration:** 4 weeks (1 month)
**Start Date:** 2025-01-20
**Target Completion:** 2025-02-20

This timeline is aggressive but achievable with focused development and existing foundation from Phase 1 (Host Bridge) and Phase 2 (CLI).
