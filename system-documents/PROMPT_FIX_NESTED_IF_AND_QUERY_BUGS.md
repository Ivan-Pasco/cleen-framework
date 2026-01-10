# Prompt: Fix Nested If Type Errors and Database Query Caching Bug

## Context

During comprehensive testing of the Clean Framework (endpoints + database + JSON), two critical bugs were discovered:

1. **Compiler Bug**: Deeply nested `if` blocks cause false "Cannot unify types: integer and string" errors
2. **Server Bug**: Parameterized database queries return wrong results (appears to be caching issue)

## Bug 1: Compiler - Nested If Type Unification Error

### Symptoms
When compiling Clean Language code with nested `if` statements, the compiler reports type errors on lines that:
- Are comments or blank lines
- Reference line numbers beyond the file length
- Show "Cannot unify types: integer and string" for valid string operations

### Example Code That Fails
```cln
functions:
    string __route_handler_5()
        integer passed = 0

        string j1 = "{\"a\":1}"
        any p1 = json.tryTextToData(j1)
        if p1 != null
            any aField = p1.a
            if aField != null
                integer aVal = aField.toInteger()
                if aVal == 1
                    passed = passed + 1  // ERROR reported here or nearby
                    printl("Test PASS")

        return "{\"ok\":true}"
```

### Error Output
```
error[E002]: Cannot unify types: integer and string
  --> file.cln:XX:10
   |
  XX | 	// HANDLER 3: GET /api/items/:id   <-- Points to comment line!
     |          ^^
     |          Cannot unify types: integer and string
```

### Working Pattern (from app-db.cln)
The `examples/article-blog/app-db.cln` compiles successfully. Key differences:
- Uses `if/else` within `while` loops
- Avoids deeply nested `if` without `else`
- Uses helper functions to reduce nesting

### Files to Investigate
1. **Compiler semantic analysis**: `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/semantic/`
2. **Type unification logic**: Look for type constraint solving
3. **Scope handling**: Check if nested if blocks create scope issues
4. **Line number tracking**: Error line numbers are off - check source mapping

### Reproduction Steps
```bash
# This fails with type errors
"/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/target/release/cln" compile \
    "/Users/earcandy/Documents/Dev/Clean Language/clean-framework/tests/comprehensive-test/simple-test.cln" \
    -o /tmp/test.wasm --plugins

# This succeeds
"/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/target/release/cln" compile \
    "/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/article-blog/app-db.cln" \
    -o /tmp/test.wasm --plugins
```

---

## Bug 2: Server - Database Query Returns Wrong Results

### Symptoms
When querying a single article by slug, the server returns ALL articles instead of the filtered one.

### Evidence from Server Logs
```
DEBUG: Slug received: [analog-renaissance]
DEBUG: Looking up article with slug: analog-renaissance
DB Query result: {"data":{"count":3,"rows":[...ALL 3 ARTICLES...]}}  <-- Should be count:1
DEBUG: data = 409600   <-- This looks like a memory address, not parsed data!
ERROR: Missing ok field
```

### The Query Being Executed
```cln
string sql = "SELECT title, lead, category, ... FROM articles WHERE slug = '" + slug + "'"
string result = _db_query(sql, "[]")
```

### Expected Behavior
- Query with `WHERE slug = 'analog-renaissance'` should return 1 row
- Actual: Returns all 3 rows (same as the list query)

### Possible Causes
1. **Query result caching**: Server caching query results by endpoint, not by SQL
2. **SQL string not being passed correctly**: The WHERE clause might not reach the database
3. **Memory corruption**: The `409600` value suggests pointer/memory issues
4. **WASM memory isolation**: Query strings might be shared/corrupted between calls

### Files to Investigate
1. **Clean Server**: `/Users/earcandy/Documents/Dev/Clean Language/clean-server/`
2. **Database bridge functions**: Look for `_db_query` implementation
3. **WASM runtime**: Check how strings are passed between WASM and host
4. **Request context isolation**: Ensure each request has isolated state

### Reproduction Steps
```bash
# Create test database
sqlite3 /tmp/testdb.sqlite << 'EOF'
CREATE TABLE articles (id INTEGER PRIMARY KEY, slug TEXT UNIQUE, title TEXT, ...);
INSERT INTO articles (slug, title) VALUES ('article-1', 'First'), ('article-2', 'Second');
EOF

# Start server
~/.cleen/server/0.2.2/clean-server /tmp/app-db-test.wasm --port 3000 --database "sqlite:///tmp/testdb.sqlite"

# Test - this returns wrong results
curl http://localhost:3000/articles/article-1
# Expected: Single article with slug 'article-1'
# Actual: All articles returned
```

---

## Tasks

### For Bug 1 (Compiler)
1. Add debug logging to type unification in semantic analysis
2. Create minimal reproduction case isolating the nested if pattern
3. Check if the issue is in scope management or type constraint solving
4. Fix line number tracking for error messages
5. Add regression tests for nested if blocks

### For Bug 2 (Server)
1. Add logging to trace the exact SQL being executed
2. Check if query results are being cached incorrectly
3. Verify WASM string passing for SQL queries
4. Test with different database backends (in-memory vs file)
5. Check request isolation in the runtime

## Test Files Created
- `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/tests/comprehensive-test/main.cln`
- `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/tests/comprehensive-test/crud-test.cln`
- `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/tests/comprehensive-test/simple-test.cln`

## Working Reference
The file that compiles and runs correctly (use as reference):
- `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/article-blog/app-db.cln`

## Priority
1. **Bug 1 (Compiler)** - HIGH: Blocks development of new features
2. **Bug 2 (Server)** - MEDIUM: Workaround exists (use list queries), but breaks single-record lookups
