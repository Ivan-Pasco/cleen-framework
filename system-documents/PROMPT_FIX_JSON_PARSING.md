# Prompt: Fix JSON Parsing and Any Type Field Access

## Context

Recent work was done to add JSON support to the Clean Language compiler. The user reports that fixes and improvements were made to JSON processing, but we need to verify if the issues identified below still persist and fix any remaining problems.

## Known Issues to Verify

### Issue 1: `any` Type Field Access Returns Default Values

When parsing JSON with `json.tryTextToData()`, field access on the returned `any` type returns default values (0, null, empty string) instead of actual field values.

**Test Case:**

```clean
string jsonStr = "{\"ok\":true,\"data\":{\"count\":4,\"rows\":[{\"id\":1},{\"id\":2}]}}"
any parsed = json.tryTextToData(jsonStr)

printl("Parsed object: " + parsed.toString())
// Expected: Some representation of the object
// Actual (before fix): "0"

any okField = parsed.ok
printl("ok field: " + okField.toString())
// Expected: "true" or "1"
// Actual (before fix): "0"

any dataField = parsed.data
printl("data field: " + dataField.toString())
// Expected: Some representation of nested object
// Actual (before fix): "0"

any countField = dataField.count
printl("count value: " + countField.toString())
// Expected: "4"
// Actual (before fix): "0"
```

### Issue 2: WASM Code Generation Bugs in Latest Build

The latest compiler build (Dec 26 00:00) generates invalid WASM with type mismatches.

**Error Examples:**

```
error: type mismatch in call, expected [i32, i32] but got [i32]
error: type mismatch in local.set, expected [i32] but got []
error: type mismatch in f64.store, expected [i32, f64] but got [f64, i32]
```

These suggest:
- Missing function parameters
- Empty stack when expecting values
- Parameters passed in wrong order for f64.store operations

### Issue 3: List Operations on JSON Arrays

Once field access is fixed, need to verify that list operations work on JSON arrays:

```clean
any data = json.tryTextToData("{\"rows\":[{\"id\":1},{\"id\":2},{\"id\":3}]}")
any rows = data.rows
integer count = list.size(rows)  // Should return 3
any firstItem = list.get(rows, 0)  // Should return first object
integer id = firstItem.id.toInteger()  // Should return 1
```

## Real-World Test Case

The article-blog example (`examples/article-blog/app-db.cln`) provides a comprehensive real-world test:

**Database Query Returns:**
```json
{
  "ok": true,
  "data": {
    "count": 4,
    "rows": [
      {
        "slug": "renaissance-analog",
        "title": "The Renaissance of Analog",
        "category": "Culture",
        "author": "Elena Marchetti",
        "thumbnail_image": "https://...",
        "lead": "In an age of...",
        "read_time": "8",
        "featured": 1
      },
      // ... 3 more articles
    ]
  }
}
```

**Required Functionality:**

1. Parse the JSON response
2. Access `data.data.count` to get `4`
3. Access `data.data.rows` to get the array
4. Use `list.size()` and `list.get()` on the rows array
5. Access fields on each article object: `article.title.toString()`, etc.

**Current Code (lines 77-159 in app-db.cln):**

```clean
string getArticleCardsFromDB()
    string sql = "SELECT slug, title, category, author, thumbnail_image, lead, read_time, featured FROM articles ORDER BY featured DESC, created_at DESC LIMIT 4"
    string result = _db_query(sql, "[]")
    printl("DB Query result: " + result)

    string c = ""

    any data = json.tryTextToData(result)
    printl("DEBUG: data = " + data.toString())

    any dataObj = data.data
    printl("DEBUG: dataObj = " + dataObj.toString())

    any countField = dataObj.count
    printl("DEBUG: countField = " + countField.toString())

    integer count = countField.toInteger()
    printl("DEBUG: Article count after toInteger: " + count.toString())

    any rows = dataObj.rows

    integer i = 0
    while i < count
        any article = list.get(rows, i)
        string title = article.title.toString()
        // ... build HTML cards
        i = i + 1

    return c
```

**Previous Debug Output (Issue Present):**

```
DB Query result: {"ok":true,"data":{"count":4,"rows":[...]}}
DEBUG: data = 0
DEBUG: dataObj = 0
DEBUG: countField = 0
DEBUG: Article count after toInteger: 0
```

All field accesses returned 0 instead of actual values.

## Verification Steps

1. **Check if `json.tryTextToData()` works:**
   ```bash
   cd "/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler"

   # Create minimal test file
   cat > /tmp/test-json.cln << 'EOF'
   functions:
       string testJson()
           string json = "{\"count\":4,\"name\":\"test\"}"
           any parsed = json.tryTextToData(json)
           printl("Parsed: " + parsed.toString())

           any count = parsed.count
           printl("Count: " + count.toString())

           any name = parsed.name
           printl("Name: " + name.toString())

           return "Done"

   start()
       printl(testJson())
   EOF

   # Compile
   cargo run --release --bin cln -- compile /tmp/test-json.cln -o /tmp/test-json.wasm

   # Validate WASM
   wasm-validate /tmp/test-json.wasm
   ```

2. **Expected output when running:**
   ```
   Parsed: {count:4,name:"test"}  (or similar object representation)
   Count: 4
   Name: test
   Done
   ```

3. **If WASM validation fails**, check for:
   - Type mismatches in function calls
   - f64.store parameter order issues
   - Missing stack values for local.set

4. **Test with article-blog example:**
   ```bash
   cd "/Users/earcandy/Documents/Dev/Clean Language/clean-framework"

   # Ensure database exists
   sqlite3 /tmp/articles.db < /tmp/create-articles.sql

   # Compile
   ~/.cleen/bin/cln compile examples/article-blog/app-db.cln -o /tmp/app-db-test.wasm --plugins

   # Run server
   DATABASE_URL="sqlite:///tmp/articles.db" ~/.cleen/server/0.2.2/clean-server /tmp/app-db-test.wasm --port 3000

   # Test endpoint
   curl http://localhost:3000/

   # Check server logs for DEBUG output
   ```

5. **Success Criteria:**
   - `parsed.toString()` shows object representation (not "0")
   - `dataObj.count` returns `4`
   - `list.size(rows)` returns `4`
   - Homepage displays 4 article cards with correct titles, authors, etc.

## Files to Review

### Compiler Code
- `src/stdlib/json.rs` - JSON parsing implementation
- `src/codegen/` - Code generation for `any` type field access
- `src/codegen/mir_codegen.rs` - Check for issues around field access and type conversions
- `src/runtime/` - Runtime type system for `any` values

### Test Files
- `examples/article-blog/app-db.cln` - Real-world test case
- Create `tests/cln/json/` directory with comprehensive JSON tests

### Required Test Coverage

Create tests in `tests/cln/json/`:

1. **01_basic_parsing.cln** - Parse simple JSON objects
2. **02_nested_objects.cln** - Parse nested JSON structures
3. **03_arrays.cln** - Parse and access JSON arrays
4. **04_field_access.cln** - Access fields of various types (string, int, bool, null)
5. **05_list_operations.cln** - list.size() and list.get() on JSON arrays
6. **06_type_conversions.cln** - .toString(), .toInteger() on JSON values

## Expected Fixes

### Fix 1: `any` Type Field Access

The `any` type should properly store and retrieve field values from parsed JSON objects. Check if:

- JSON objects are properly represented in memory
- Field access codegen correctly retrieves values from the object structure
- Type conversions (.toString(), .toInteger()) work on JSON values

### Fix 2: WASM Code Generation

Ensure all generated WASM instructions have:

- Correct number of parameters for function calls
- Proper stack management (values present when needed)
- Correct parameter order for all instructions (especially f64.store)

### Fix 3: List Operations

Verify that JSON arrays can be used with:

- `list.size(array)` returns correct count
- `list.get(array, index)` returns correct element
- Elements can have their fields accessed

## How to Test After Fix

```bash
# 1. Build fresh compiler
cd "/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler"
cargo build --release

# 2. Run JSON tests
cargo test --test json_tests  # If test suite exists

# 3. Test article-blog example
cd "/Users/earcandy/Documents/Dev/Clean Language/clean-framework"

# Compile with latest compiler
~/.cleen/bin/cln compile examples/article-blog/app-db.cln -o /tmp/app-db-fixed.wasm --plugins

# Validate WASM
wasm-validate /tmp/app-db-fixed.wasm
# Should show: "valid" (no errors)

# Run server
DATABASE_URL="sqlite:///tmp/articles.db" ~/.cleen/server/0.2.2/clean-server /tmp/app-db-fixed.wasm --port 3000 > /tmp/test-server.log 2>&1 &

# Wait and test
sleep 2
curl -s http://localhost:3000/ | grep -o '<article class="card' | wc -l
# Should output: 4

# Check logs
grep "DEBUG:" /tmp/test-server.log
# Should show actual values, not zeros

# Stop server
pkill -f clean-server
```

## Success Metrics

- [ ] `json.tryTextToData()` returns usable object (toString() not "0")
- [ ] Field access works: `obj.field` returns actual value
- [ ] Nested access works: `obj.nested.field` returns value
- [ ] Array access works: `list.size()` and `list.get()` work on JSON arrays
- [ ] Type conversions work: `.toString()`, `.toInteger()` on JSON values
- [ ] WASM validation passes with no errors
- [ ] Article-blog example displays all 4 articles correctly
- [ ] Server logs show correct debug values (count=4, not 0)

## Additional Context

### Database Setup

The test database at `/tmp/articles.db` contains:

```sql
CREATE TABLE articles (
    id INTEGER PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    category TEXT,
    author TEXT,
    author_title TEXT,
    author_image TEXT,
    thumbnail_image TEXT,
    cover_image TEXT,
    lead TEXT,
    content TEXT,
    read_time TEXT,
    featured INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Contains 4 sample articles
```

### Environment

- **Compiler Version**: 0.20.8 (should be latest with JSON fixes)
- **Server Version**: clean-server v0.2.2
- **Database**: SQLite 3.x
- **Platform**: macOS (Darwin 25.1.0)

### Key Question

**Have the recent JSON fixes resolved these issues?**

Please verify each issue and fix any that remain. The article-blog example should serve as the ultimate integration test - if it can display articles from the database, JSON parsing is fully functional.

---

**Priority**: HIGH - Blocks database integration and all JSON-based features
**Created**: 2025-12-26
**Compiler Path**: `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler`
**Test App**: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/article-blog/app-db.cln`
