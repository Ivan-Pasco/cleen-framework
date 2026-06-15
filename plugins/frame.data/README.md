# frame.data Plugin

ORM and database plugin for Clean Language. Provides DSL blocks for data modeling and database operations.

## Blocks

### model

Defines a database model with automatic ORM methods.

**Attributes:**
- `name` (required) - Model class name (PascalCase)
- `table` (optional) - Database table name (defaults to lowercase of name)

**Example:**
```clean
import:
    frame.data

model: name="User" table="users"
    string email
    string name
    boolean active = true
    string created_at
```

**Generated Methods:**
- `save()` - Insert or update the model
- `delete()` - Delete the model from database
- `find(id)` - Find model by ID (static)
- `all()` - Get all models (static)
- `where(column, value)` - Find models matching criteria (static)
- `findBy(column, value)` - Find first model matching criteria (static)

### query

Creates a query builder for complex database queries.

**Attributes:**
- `model` (required) - Model name to query

**Example:**
```clean
query: model="User"
    where: column="active" value=true
    order: column="created_at" direction="desc"
    limit: 10
```

### transaction

Wraps operations in a database transaction with automatic rollback on error.

**Example:**
```clean
transaction:
    user = User()
    user.email = "test@example.com"
    user.save()

    post = Post()
    post.user_id = user.id
    post.title = "First Post"
    post.save()
```

## Generated Code

The plugin expands DSL blocks into Clean code that uses Host Bridge functions:

- `_db_query(sql, params)` - Execute query and return rows
- `_db_query_one(sql, params)` - Execute query and return single row
- `_db_insert(table, data)` - Insert row and return ID
- `_db_update(table, id, data)` - Update row by ID
- `_db_delete(table, id)` - Delete row by ID
- `_db_begin_transaction()` - Start transaction
- `_db_commit()` - Commit transaction
- `_db_rollback()` - Rollback transaction

## Building

```bash
./build.sh
# or
cln compile src/main.cln -o plugin.wasm
```

## Installation

```bash
cleen plugin install ./
```

## Plugin Contracts

Implements:

- [`lifecycle`](../../../foundation/spec/plugins/contracts/lifecycle.md) — `module_helpers_are_roots = true` so preamble-emitted ORM helpers (PagedResult / CursorResult constructors, query builders) survive the import-minimality BFS even when only reached through plugin-generated CRUD code.
- [`bridge-host-classes`](../../../foundation/spec/plugins/contracts/bridge-host-classes.md) — every `_db_*` bridge declares `hosts = ["server"]`; the database driver only loads on hosts that ship a runtime DB connection.
