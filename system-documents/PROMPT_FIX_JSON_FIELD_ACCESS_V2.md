# Fix JSON Field Access Bug - Phase 2

## Current Status (2025-12-29)

The initial fix for JSON field access partially worked. **Integer field access now works correctly**, but **boolean, object, and array field access is still broken**.

## Test Results Summary

### Test 1: Simple JSON with mixed types
```cln
string jsonStr = "{\"ok\":true,\"count\":42}"
any parsed = json.tryTextToData(jsonStr)

any okField = parsed.ok        // Returns 0 (WRONG - should be true)
any countField = parsed.count  // Returns 42 (CORRECT!)
```

**Output:**
```
parsed pointer: true
okField: 0
countField: 42
countInt: 42
SUCCESS: count is 42
```

### Test 2: Nested objects
```cln
string jsonStr = "{\"data\":{\"count\":4,\"rows\":[...]},\"ok\":true}"
any parsed = json.tryTextToData(jsonStr)

any dataObj = parsed.data      // Returns something that toString() shows as "true" (WRONG)
any countField = dataObj.count // Still returns 4 somehow (WORKS)
```

**Output:**
```
DEBUG: data = 26960
DEBUG: dataObj = true          // WRONG - should be object representation
DEBUG: countField = 4          // Correct!
```

### Test 3: Array field access
```cln
string jsonStr = "{\"items\":[{\"name\":\"first\",\"value\":1},{\"name\":\"second\",\"value\":2}]}"
any parsed = json.tryTextToData(jsonStr)

any items = parsed.items       // Returns something invalid
any first = list.get(items, 0) // Returns 0 (WRONG)
any firstName = first.name     // Returns 0 (WRONG)
```

**Output:**
```
parsed = 4776
items = true                   // WRONG - should be array
first = 0                      // WRONG - list.get fails
firstName = 0                  // WRONG
```

## Bug Analysis

The `__json_get_field` function (or how the compiler calls it) has different behaviors for different JSON value types:

| JSON Type | Field Access | Result |
|-----------|-------------|--------|
| Integer   | `parsed.count` where count=42 | ✅ Returns 42 correctly |
| Boolean   | `parsed.ok` where ok=true | ❌ Returns 0 |
| String    | `parsed.name` where name="test" | ❓ Untested |
| Object    | `parsed.data` where data={...} | ❌ Returns invalid (shows as "true") |
| Array     | `parsed.items` where items=[...] | ❌ Returns invalid (shows as "true") |

## Hypothesis

The `__json_get_field` runtime function may be correctly extracting values, but the **type tagging or return value handling** differs by JSON type:

1. **Integer values** - Returned directly as i32, works correctly
2. **Boolean values** - May be returning type tag (0/1) instead of actual value
3. **Object/Array values** - May be returning type tag or flag instead of pointer to nested structure

OR the compiler is generating different code paths for different inferred types.

## Files to Investigate

### 1. Runtime JSON Implementation
Location: `src/stdlib/json_class.rs`

Check `__json_get_field` implementation:
- How does it handle different JSON value types?
- Does it return the correct pointer for nested objects/arrays?
- Is there type tagging that needs to be unwrapped?

### 2. MIR Codegen for AnyGetField
Location: `src/codegen/mir_codegen.rs` (around lines 2791-2811)

```rust
MirOperation::AnyGetField { object, key } => {
    // Check if this handles all JSON types correctly
    // Verify the return value handling
}
```

### 3. JSON Value Type Handling
Check if there's type-specific handling for JSON values in:
- `src/runtime/json.rs` (if exists)
- `src/stdlib/` - any JSON-related functions

## Reproduction Steps

### Quick Test
```bash
# Create test file
cat > /tmp/json-type-test.cln << 'EOF'
functions:
    string test()
        string json = "{\"num\":42,\"bool\":true,\"str\":\"hello\",\"obj\":{\"x\":1},\"arr\":[1,2,3]}"
        any p = json.tryTextToData(json)

        printl("Testing field access on different JSON types:")
        printl("")

        printl("1. Integer field:")
        any numField = p.num
        printl("   p.num = " + numField.toString())
        printl("   toInteger = " + numField.toInteger().toString())

        printl("2. Boolean field:")
        any boolField = p.bool
        printl("   p.bool = " + boolField.toString())

        printl("3. String field:")
        any strField = p.str
        printl("   p.str = " + strField.toString())

        printl("4. Object field:")
        any objField = p.obj
        printl("   p.obj = " + objField.toString())
        any xField = objField.x
        printl("   p.obj.x = " + xField.toString())

        printl("5. Array field:")
        any arrField = p.arr
        printl("   p.arr = " + arrField.toString())
        any firstElem = list.get(arrField, 0)
        printl("   p.arr[0] = " + firstElem.toString())

        return "DONE"

start()
    printl("=== JSON Type Field Access Test ===")
    printl(test())
EOF

# Compile
~/.cleen/bin/cln compile /tmp/json-type-test.cln -o /tmp/json-type-test.wasm --plugins

# Run
~/.cleen/server/0.2.2/clean-server /tmp/json-type-test.wasm --port 3010
```

### Expected Output
```
=== JSON Type Field Access Test ===
Testing field access on different JSON types:

1. Integer field:
   p.num = 42
   toInteger = 42

2. Boolean field:
   p.bool = true       // Currently returns 0

3. String field:
   p.str = hello

4. Object field:
   p.obj = [object]    // Currently returns "true"
   p.obj.x = 1

5. Array field:
   p.arr = [array]     // Currently returns "true"
   p.arr[0] = 1        // Currently returns 0
DONE
```

## Fix Strategy

### Option A: Fix in Runtime (`__json_get_field`)

If the issue is in how the runtime extracts values:

1. Check the JSON value type before returning
2. For objects/arrays, return pointer to nested JSON structure
3. For booleans, return 1 for true, 0 for false (or proper any-typed value)
4. For strings, return pointer to string content

### Option B: Fix in Compiler (Type-Aware Code Generation)

If the compiler generates different code for different inferred types:

1. Ensure `MirOperation::AnyGetField` generates uniform code
2. Check if there's type inference affecting the code path
3. Verify the function call signature matches `__json_get_field(any_ptr, key_ptr, key_len) -> any`

### Option C: Fix `any` Type Handling

If the issue is in how `any` type values are stored/retrieved:

1. Check `any` type memory layout
2. Verify type tags are preserved through field access
3. Ensure `toString()` on `any` values handles all types

## Debug Approach

Add debug output to trace the issue:

### In `json_class.rs`:
```rust
// In __json_get_field or equivalent
eprintln!("DEBUG __json_get_field:");
eprintln!("  any_ptr = {}", any_ptr);
eprintln!("  key = {:?}", key);
eprintln!("  value_type = {:?}", value_type);
eprintln!("  returning = {}", result);
```

### In `mir_codegen.rs`:
```rust
// In AnyGetField handling
eprintln!("DEBUG AnyGetField codegen:");
eprintln!("  object operand = {:?}", object);
eprintln!("  key = {:?}", key);
eprintln!("  function_idx = {}", json_get_field_idx);
```

## Success Criteria

After the fix, all of these should work:

```cln
string json = "{\"count\":4,\"ok\":true,\"data\":{\"x\":1},\"items\":[1,2]}"
any p = json.tryTextToData(json)

// All should return correct values:
integer count = p.count.toInteger()  // 4
boolean ok = p.ok == true            // true (comparison should work)
any data = p.data                    // valid object pointer
integer x = data.x.toInteger()       // 1
any items = p.items                  // valid array pointer
integer first = list.get(items, 0).toInteger()  // 1
```

## Priority

**CRITICAL** - This bug blocks:
- Any web application that parses JSON API responses
- Database query result parsing (which returns JSON)
- Configuration file parsing
- Any real-world Clean Language application

## Related Files

- Compiler: `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/`
- Test file: `tests/cln/stdlib/json/json_field_access_bug.cln`
- Previous bug doc: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/system-documents/PROMPT_FIX_JSON_FIELD_ACCESS.md`
