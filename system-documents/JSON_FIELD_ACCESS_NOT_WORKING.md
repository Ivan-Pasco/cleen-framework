# JSON Field Access Still Not Working - Investigation Report

## Date: 2025-12-26

## Summary

Despite recent JSON fixes in commits 87d6695 and 82ab240 that claim to resolve the JSON field access bug, testing shows that field access on `any` type variables is still returning default values (0) instead of actual JSON field values.

## What's Working

✅ **WASM Code Generation** - No more type mismatch errors
✅ **WASM Validation** - Generated modules pass validation
✅ **JSON Parsing** - `json.tryTextToData()` returns a value (memory pointer)
✅ **Server Execution** - No runtime crashes

## What's NOT Working

❌ **Field Access on Any Type** - Returns 0 instead of actual values

### Test Code

```clean
string jsonStr = "{\"count\":4,\"name\":\"test\"}"
any parsed = json.tryTextToData(jsonStr)

printl("Parsed value: " + parsed.toString())
// Output: Parsed value: 4944 ✅ (memory pointer)

any countField = parsed.count
printl("Count field: " + countField.toString())
// Output: Count field: 0 ❌ (should be 4)
```

### Test Output

```
Testing JSON parsing...
JSON string: {"count":4,"name":"test"}
Parsed value: 4944          ← JSON object allocated in memory
Parsed is not null, trying field access...
Count field: 0              ← Field access returns 0 instead of 4
```

## Root Cause Investigation

### The Implementation (from commits)

The JSON field access fix implemented:
- `__json_get_field(any_ptr, key_ptr, key_len)` accessor function
- Complete JSON parser in WASM (~2,020 instructions)
- Support for all primitive types

### Code Generation Check

From `src/codegen/expression_generator.rs` (lines 1145-1148):

```rust
} else if self.is_any_type_variable(namespace_name) {
    // Property access on Any type (JSON object)
    // Generate: __json_get_field(any_ptr, key_ptr, key_len)
    self.generate_any_property_access(object, property, instructions)
}
```

This should generate a call to `__json_get_field` when accessing `parsed.count`.

### The Problem

**`__json_get_field` is NOT in the compiled WASM module:**

```bash
$ wasm-objdump -x /tmp/test-json-simple.wasm | grep "__json_get_field"
# NO OUTPUT - function is missing!
```

This confirms that the `generate_any_property_access` code path is **NOT being executed**.

## Why It's Not Working

Two possibilities:

### 1. Type Tracking Issue

The `is_any_type_variable()` check is failing:

```rust
fn is_any_type_variable(&self, name: &str) -> bool {
    if let Some(var_type) = self.variable_types.get(name) {
        matches!(var_type, Type::Any)
    } else {
        false
    }
}
```

When we write:
```clean
any parsed = json.tryTextToData(jsonStr)
any countField = parsed.count
```

The variable `parsed` should be in `variable_types` with `Type::Any`, but either:
- It's not being added to the map
- It's being added with a different type
- The field access happens before the variable is registered

### 2. Codegen Order Issue

Field access might be code-generated before variable types are fully registered, causing the `is_any_type_variable` check to return false.

## Required Investigation

### Step 1: Verify Type Registration

Add debug logging in `src/codegen/statement_generator.rs` around line 204-208:

```rust
if let Some(ast_type) = var_type {
    eprintln!("DEBUG: Adding variable '{}' with explicit type: {:?}", name, ast_type);
    self.variable_types.insert(name.to_string(), ast_type.clone());
} else if let Some(init_expr) = initializer {
    let inferred_type = self.infer_expression_type(init_expr)?;
    eprintln!("DEBUG: Adding variable '{}' with inferred type: {:?}", name, inferred_type);
    self.variable_types.insert(name.to_string(), inferred_type);
}
```

### Step 2: Verify Field Access Generation

Add debug logging in `src/codegen/expression_generator.rs` around line 1145:

```rust
} else if self.is_any_type_variable(namespace_name) {
    eprintln!("DEBUG: Generating any property access for {}.{}", namespace_name, property);
    self.generate_any_property_access(object, property, instructions)
} else {
    eprintln!("DEBUG: NOT generating any property access for {} (not any type)", namespace_name);
    Err(CompilerError::codegen_error(
```

### Step 3: Check variable_types Map

Add debug logging in `is_any_type_variable`:

```rust
fn is_any_type_variable(&self, name: &str) -> bool {
    eprintln!("DEBUG: Checking if '{}' is any type", name);
    eprintln!("DEBUG: variable_types contains: {:?}", self.variable_types);
    if let Some(var_type) = self.variable_types.get(name) {
        eprintln!("DEBUG: Found type for '{}': {:?}", name, var_type);
        matches!(var_type, Type::Any)
    } else {
        eprintln!("DEBUG: No type found for '{}'", name);
        false
    }
}
```

### Step 4: Recompile and Test

```bash
cd "/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler"
cargo build --release 2>&1 | tee /tmp/build.log

"./target/release/cln" compile /tmp/test-json-simple.cln -o /tmp/test-json-debug.wasm --plugins 2>&1 | tee /tmp/compile.log

# Check if __json_get_field is now in the WASM
wasm-objdump -x /tmp/test-json-debug.wasm | grep "__json_get_field"
```

Expected debug output should show:
1. Variable `parsed` being added with type `Type::Any`
2. Field access checking `parsed` and finding it's an any type
3. Generating `__json_get_field` call

## Alternative Issue: FieldAccess vs PropertyAccess

Check if the AST is using `FieldAccess` or `PropertyAccess` for `parsed.count`. The code might be looking for the wrong expression type.

Look in `src/codegen/expression_generator.rs` for how different access patterns are handled:

```rust
Expression::FieldAccess { .. } => { ... }
Expression::PropertyAccess { .. } => { ... }
```

## Test Files

### Minimal Test Case

File: `/tmp/test-json-simple.cln`

```clean
import:
	frame.server

functions:
	string __route_handler_0()
		string jsonStr = "{\"count\":4,\"name\":\"test\"}"
		any parsed = json.tryTextToData(jsonStr)
		any countField = parsed.count

		if countField == null
			return "ERROR: Count field is null"

		integer count = countField.toInteger()
		return "SUCCESS: count=" + count.toString()

start()
	integer status = _http_route("GET", "/", 0)
```

### Real-World Test

File: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/article-blog/app-db.cln`

Lines 86-116 show the same issue with database query results.

## Expected vs Actual

### Expected Behavior

```
Parsed value: 4944           ← Object pointer
Count field: 4               ← Actual field value
SUCCESS: count=4
```

### Actual Behavior

```
Parsed value: 4944           ← Object pointer ✅
Count field: 0               ← Default value ❌
ERROR: Count field is null
```

## Impact

**CRITICAL** - Blocks all JSON-based functionality:
- Database query result parsing
- API response handling
- Configuration files
- Any external data integration

Affects:
- Article-blog example (`examples/article-blog/app-db.cln`)
- All Frame Data ORM examples
- Any application using JSON responses

## Next Steps

1. **Add debug logging** as specified above
2. **Recompile** with debug output enabled
3. **Analyze logs** to find where type tracking fails
4. **Fix the issue** in the appropriate codegen file
5. **Test** with both minimal and real-world examples
6. **Verify** `__json_get_field` appears in compiled WASM
7. **Confirm** field access returns actual values

## Files to Modify

- `src/codegen/statement_generator.rs` - Variable type registration
- `src/codegen/expression_generator.rs` - Field access code generation
- `src/codegen/expression_generator.rs` - Type checking logic

## Compiler Version

- **Commit**: 82ab240 (latest)
- **Version**: 0.20.8
- **Build**: December 26, 2025

---

**Created**: 2025-12-26
**Status**: Investigation Required
**Priority**: CRITICAL
