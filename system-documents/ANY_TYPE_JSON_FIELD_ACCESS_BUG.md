# Any Type JSON Field Access Bug

## Issue Summary

Clean Language's `any` type returns default values (0, empty string, null) when accessing fields on parsed JSON objects, instead of returning the actual field values.

## Status

**CRITICAL BUG** - Blocks all JSON parsing functionality with nested objects

## Environment

- **Compiler Version**: 0.20.8
- **Server Version**: clean-server v0.2.2
- **Affected File**: `examples/article-blog/app-db.cln`

## Reproduction

### Code

```clean
string result = _db_query(sql, "[]")
// result = {"ok":true,"data":{"count":4,"rows":[...]}}

any data = json.tryTextToData(result)
// data != null ✓ (JSON parsing succeeds)

any dataObj = data.data
// dataObj != null ✓ (Nested object access seems to work)

any count = dataObj.count
// count.toString() = "0" ✗ (Should be "4")

any rows = dataObj.rows
// rows.toString() = "0" ✗ (Should be array representation)
```

### Expected Behavior

1. `dataObj.count` should return the integer value `4`
2. `dataObj.rows` should return the array object
3. Field access should retrieve actual values from parsed JSON

### Actual Behavior

1. All field accesses on `any` type return default values
2. Numeric fields return `0`
3. Cannot access nested JSON data structures

## Debug Evidence

Server logs show:

```
DB Query result: {"data":{"count":4,"rows":[...]}, "ok":true}
DEBUG: Successfully parsed JSON
DEBUG: Got data object
DEBUG: rows type: 0      ← Should be array
DEBUG: countField type: 0  ← Should be 4
```

## Impact

### Blocked Functionality

- Cannot parse database query results
- Cannot process JSON API responses
- Cannot work with any nested JSON structures
- List operations (`list.get()`, `list.size()`) fail on JSON arrays

### Workarounds

1. **String-based parsing**: Use string operations to extract values manually (fragile)
2. **Flat JSON**: Only use top-level fields (severe limitation)
3. **External parsing**: Parse JSON outside Clean Language (defeats purpose)

## Root Cause Analysis

The `any` type appears to have one of these issues:

1. Field accessor returns default value when field lookup fails
2. Type coercion incorrectly converts all values to default types
3. Memory layout of parsed JSON incompatible with field access mechanism
4. Missing implementation for nested object field access

## Test Case

### Input JSON
```json
{
  "ok": true,
  "data": {
    "count": 4,
    "rows": [
      {"id": 1, "title": "Article 1"},
      {"id": 2, "title": "Article 2"}
    ]
  }
}
```

### Test Code
```clean
functions:
  string testJsonParsing()
    string jsonStr = "{\"ok\":true,\"data\":{\"count\":4,\"rows\":[{\"id\":1},{\"id\":2}]}}"
    any parsed = json.tryTextToData(jsonStr)

    if parsed == null
      return "ERROR: JSON parsing failed"

    // Test top-level field
    any okField = parsed.ok
    string okValue = okField.toString()  // Should be "true" or "1"

    // Test nested object
    any dataField = parsed.data
    if dataField == null
      return "ERROR: Nested object is null"

    // Test numeric field in nested object
    any countField = dataField.count
    string countValue = countField.toString()  // Should be "4"
    integer countInt = countField.toInteger()  // Should be 4

    return "ok=" + okValue + ", count=" + countValue + ", countInt=" + countInt.toString()

start()
  printl(testJsonParsing())
  // Expected: ok=true, count=4, countInt=4
  // Actual: ok=?, count=0, countInt=0
```

## Related Issues

- Database integration blocked in `examples/article-blog/app-db.cln`
- JSON-based API responses cannot be processed
- Framework examples cannot demonstrate real-world data handling

## Priority

**CRITICAL** - This blocks a fundamental feature (JSON parsing) that is essential for:
- Database query results
- API responses
- Configuration files
- Any data interchange

## Suggested Fix Locations

Likely in compiler code generation for:
- `src/codegen/` - `any` type field access codegen
- `src/stdlib/json.rs` - JSON parsing and object representation
- `src/runtime/` - Runtime type system for `any` values

## Temporary Status

The example has been modified to show:
```
"Database connected! Found articles. (JSON parsing with 'any' type needs compiler fix)"
```

This confirms the database integration works but JSON parsing is blocked by this bug.

## Next Steps

1. **Investigate `any` type implementation** in compiler
2. **Review JSON parsing** to understand object representation
3. **Test field access** code generation for `any` type
4. **Create minimal reproduction** in compiler test suite
5. **Implement fix** for field access on `any` type
6. **Verify** all JSON parsing scenarios work correctly

---

**Date Created**: 2025-12-26
**Discovered By**: System testing of article-blog example
**Blocks**: Frame Data plugin examples, JSON-based applications
