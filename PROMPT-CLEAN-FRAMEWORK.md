# Clean Framework - Restructure as Plugin-Based System

## Context
The Clean Language ecosystem is implementing a new plugin architecture where:
- Plugins are written IN Clean Language itself (compiled to WASM)
- The compiler calls plugin WASM functions to expand framework blocks
- Plugins return Clean Language source code (not AST)
- Host Bridge provides runtime capabilities (HTTP, DB, File I/O)

## Your Mission
Restructure the framework to focus on **Clean Language plugins** instead of Rust crates. Keep the host-bridge but replace the Rust plugin crates with Clean Language source.

## STEP 1: Documentation First

Create/update these documents BEFORE making code changes:

### 1.1 Create `system-documents/plugin-architecture-spec.md`

```markdown
# Clean Framework Plugin Architecture

## Overview

Clean Framework provides plugins written in Clean Language that extend the compiler with framework-specific DSL blocks. These plugins compile to WASM and are loaded by the compiler.

## Plugin Structure

Each plugin is a Clean Language project:

```
plugins/<name>/
├── plugin.toml          # Plugin manifest
├── src/
│   └── main.cln         # Plugin source (Clean Language)
├── tests/
│   └── test_expand.cln  # Plugin tests
├── build.sh             # Build script
└── README.md            # Documentation
```

## Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
```

## Plugin Entry Point

Every plugin must export an `expand_block` function:

```clean
expand_block(block_name: string, attributes: string, body: string) -> string
```

- `block_name`: The DSL block identifier (e.g., "server", "route", "model")
- `attributes`: JSON string of block attributes
- `body`: The raw content inside the block
- Returns: Clean Language source code to replace the block

## Host Bridge Functions

Plugins generate code that calls host bridge functions at runtime:

### HTTP Functions
- `_http_listen(host: string, port: integer)`
- `_http_route(method: string, path: string, handler: function)`
- `_http_middleware(handler: function)`

### Database Functions
- `_db_query(sql: string, params: list) -> list`
- `_db_query_one(sql: string, params: list) -> any`
- `_db_insert(table: string, data: any) -> integer`
- `_db_update(table: string, id: integer, data: any) -> boolean`
- `_db_delete(table: string, id: integer) -> boolean`

### Auth Functions
- `_auth_create_token(user: any, secret: string) -> string`
- `_auth_verify_token(token: string, secret: string) -> any`
- `_auth_hash_password(password: string) -> string`
- `_auth_verify_password(password: string, hash: string) -> boolean`

### File Functions
- `_file_read(path: string) -> string`
- `_file_write(path: string, content: string)`
- `_file_exists(path: string) -> boolean`
- `_file_delete(path: string)`

## Build Process

1. Write plugin in Clean Language
2. Compile: `cln compile src/main.cln -o plugin.wasm`
3. Package with plugin.toml
4. Install: `cleen plugin install ./` or publish to registry
```

### 1.2 Update `README.md`

Replace current content to reflect new plugin-based structure.

### 1.3 Document each plugin

Create README.md for each plugin explaining its DSL syntax.

## STEP 2: Directory Restructure

### What to DELETE
- `frame-cli/` - replaced by cleen manager
- `frame-server/` - logic moves to frame.web plugin
- `frame-data/` - logic moves to frame.data plugin
- `frame-ui/` - logic moves to frame.ui plugin
- `frame-auth/` - logic moves to frame.auth plugin
- `frame-plugins/` - old Rust plugin system
- `frame-compiler-plugins/` - old Rust plugin system

### What to KEEP
- `host-bridge/` - runtime system access (Rust crate)
- `examples/` - update with new plugin-based examples
- `documents/` - keep existing design docs

### New Structure
```
clean-framework/
├── Cargo.toml                    # Points to host-bridge only
├── README.md                     # Updated overview
├── host-bridge/                  # KEEP - runtime imports
│   ├── Cargo.toml
│   └── src/
├── plugins/                      # NEW - Plugin sources
│   ├── frame.web/
│   │   ├── plugin.toml
│   │   ├── src/main.cln
│   │   ├── build.sh
│   │   └── README.md
│   ├── frame.data/
│   │   ├── plugin.toml
│   │   ├── src/main.cln
│   │   ├── build.sh
│   │   └── README.md
│   ├── frame.auth/
│   │   ├── plugin.toml
│   │   ├── src/main.cln
│   │   ├── build.sh
│   │   └── README.md
│   └── frame.ui/
│       ├── plugin.toml
│       ├── src/main.cln
│       ├── build.sh
│       └── README.md
├── examples/
│   ├── todo-app/
│   │   └── main.cln
│   ├── api-server/
│   │   └── main.cln
│   └── blog/
│       └── main.cln
├── system-documents/
│   ├── plugin-architecture-spec.md
│   └── (keep existing docs)
└── scripts/
    ├── build-all-plugins.sh
    └── install-plugins.sh
```

## STEP 3: Plugin Implementations

### frame.web Plugin (plugins/frame.web/src/main.cln)

```clean
// Frame Web Plugin
// Handles: server, route, middleware blocks

expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "server"
        return expand_server(attributes, body)
    if block_name == "route"
        return expand_route(attributes, body)
    if block_name == "middleware"
        return expand_middleware(attributes, body)
    return body

expand_server(attrs: string, body: string) -> string
    port = get_attr(attrs, "port", "8080")
    host = get_attr(attrs, "host", "localhost")

    return "
// Generated server code
start()
    print(\"Server starting on " + host + ":" + port + "\")
    " + body + "
    _http_listen(\"" + host + "\", " + port + ")
"

expand_route(attrs: string, body: string) -> string
    method = get_attr(attrs, "method", "GET")
    path = get_attr(attrs, "path", "/")

    return "
_http_route(\"" + method + "\", \"" + path + "\", (request) -> any
    " + body + "
)
"

expand_middleware(attrs: string, body: string) -> string
    return "
_http_middleware((request, next) -> any
    " + body + "
    return next(request)
)
"

get_attr(json: string, key: string, default_val: string) -> string
    // Parse JSON attribute - simplified
    // Real implementation would parse JSON properly
    return default_val
```

### frame.data Plugin (plugins/frame.data/src/main.cln)

```clean
// Frame Data Plugin
// Handles: model, query, transaction blocks

expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "model"
        return expand_model(attributes, body)
    if block_name == "query"
        return expand_query(attributes, body)
    if block_name == "transaction"
        return expand_transaction(attributes, body)
    return body

expand_model(attrs: string, body: string) -> string
    name = get_attr(attrs, "name", "Model")
    table = get_attr(attrs, "table", name.toLowerCase())

    return "
class " + name + "
    integer id
    " + body + "

    save() -> boolean
        if this.id == 0
            this.id = _db_insert(\"" + table + "\", this)
            return this.id > 0
        return _db_update(\"" + table + "\", this.id, this)

    static find(id: integer) -> " + name + "
        data = _db_query_one(\"SELECT * FROM " + table + " WHERE id = ?\", [id])
        return " + name + ".fromData(data)

    static all() -> list<" + name + ">
        rows = _db_query(\"SELECT * FROM " + table + "\", [])
        return rows.map((row) -> " + name + ".fromData(row))

    delete() -> boolean
        return _db_delete(\"" + table + "\", this.id)
"

expand_query(attrs: string, body: string) -> string
    return "_db_query(" + body + ", [])"

expand_transaction(attrs: string, body: string) -> string
    return "
_db_begin_transaction()
try
    " + body + "
    _db_commit()
onError(e)
    _db_rollback()
    throw e
"

get_attr(json: string, key: string, default_val: string) -> string
    return default_val
```

### frame.auth Plugin (plugins/frame.auth/src/main.cln)

```clean
// Frame Auth Plugin
// Handles: auth, protected, login blocks

expand_block(block_name: string, attributes: string, body: string) -> string
    if block_name == "auth"
        return expand_auth(attributes, body)
    if block_name == "protected"
        return expand_protected(attributes, body)
    if block_name == "login"
        return expand_login(attributes, body)
    return body

expand_auth(attrs: string, body: string) -> string
    strategy = get_attr(attrs, "strategy", "jwt")
    secret = get_attr(attrs, "secret", "")

    return "
// Auth configuration
string _auth_strategy = \"" + strategy + "\"
string _auth_secret = \"" + secret + "\"
" + body + "
"

expand_protected(attrs: string, body: string) -> string
    return "
_http_middleware((request, next) -> any
    token = request.header(\"Authorization\")
    if token == \"\"
        return {\"status\": 401, \"body\": \"Unauthorized\"}

    user = _auth_verify_token(token, _auth_secret)
    if user == null
        return {\"status\": 401, \"body\": \"Invalid token\"}

    request.user = user
    return next(request)
)
" + body + "
"

expand_login(attrs: string, body: string) -> string
    return "
_http_route(\"POST\", \"/login\", (request) -> any
    " + body + "
)
"

get_attr(json: string, key: string, default_val: string) -> string
    return default_val
```

## STEP 4: Example Applications

### examples/todo-app/main.cln

```clean
import:
    frame.web
    frame.data

model: name="Todo" table="todos"
    string title
    boolean completed = false
    string created_at

server: port=3000

    route: method="GET" path="/todos"
        todos = Todo.all()
        return {"status": 200, "body": todos}

    route: method="GET" path="/todos/:id"
        id = request.param("id").toInteger()
        todo = Todo.find(id)
        if todo == null
            return {"status": 404, "body": "Not found"}
        return {"status": 200, "body": todo}

    route: method="POST" path="/todos"
        todo = Todo.new()
        todo.title = request.body("title")
        todo.save()
        return {"status": 201, "body": todo}

    route: method="DELETE" path="/todos/:id"
        id = request.param("id").toInteger()
        todo = Todo.find(id)
        if todo != null
            todo.delete()
        return {"status": 204}
```

## STEP 5: Build Scripts

### scripts/build-all-plugins.sh

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGINS_DIR="$SCRIPT_DIR/../plugins"

for plugin_dir in "$PLUGINS_DIR"/*/; do
    plugin_name=$(basename "$plugin_dir")
    echo "Building $plugin_name..."

    cd "$plugin_dir"
    if [ -f "build.sh" ]; then
        bash build.sh
    else
        cln compile src/main.cln -o plugin.wasm
    fi

    echo "✓ $plugin_name built successfully"
done

echo ""
echo "All plugins built successfully!"
```

### plugins/frame.web/build.sh

```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"
cln compile src/main.cln -o plugin.wasm
echo "Built frame.web plugin -> plugin.wasm"
```

## Important Notes

1. **Documentation First** - Update specs before deleting any code
2. **Keep host-bridge** - It provides runtime capabilities
3. **Plugins in Clean Language** - All framework logic moves to .cln files
4. **Test each plugin** - Ensure expand_block works correctly
5. **Update root Cargo.toml** - Should only reference host-bridge

## Start by:
1. Create `system-documents/plugin-architecture-spec.md`
2. Update `README.md` with new structure
3. Create the `plugins/` directory
4. Implement frame.web plugin first (simplest)
5. Test with compiler: `cln compile example.cln -o out.wasm --plugins`
6. Then implement frame.data, frame.auth, frame.ui
7. Delete old Rust crates (frame-cli, frame-server, etc.)

Begin with the documentation.
