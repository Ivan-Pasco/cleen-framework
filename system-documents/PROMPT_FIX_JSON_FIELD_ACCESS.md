# Fix JSON Field Access Bug in Clean Language Compiler

## Bug Summary

When accessing a field on an `any` type variable (e.g., `parsed.count` where `parsed` is the result of `json.tryTextToData`), the compiler generates incorrect WASM code. Instead of calling `__json_get_field`, it generates a call to `int_to_string`, causing field access to return 0/null.

## Root Cause Identified (2025-12-29)

**THE BUG**: The compiler generates a call to function 282 (`int_to_string`) instead of function 309 (`__json_get_field`) when compiling field access on `any` type.

### Evidence from WASM Disassembly

The compiled WASM for `parsed.count` shows (lines 5308-5312):
```wasm
i32.const 4200     ; Push pointer to string "count"
local.set 6        ; Store in local 6
local.get 1        ; Push parsed JSON object (any_ptr)
local.get 6        ; Push string "count" pointer (key_ptr)
call 282           ; WRONG! This is int_to_string, not __json_get_field
```

**What it SHOULD be:**
```wasm
local.get 1        ; Push any_ptr (parsed JSON object)
i32.const <offset> ; Push key_ptr (pointer to content of "count")
i32.const 5        ; Push key_len (length of "count")
call 309           ; Call __json_get_field (signature: i32,i32,i32 -> i32)
```

### Function Index Reference (from compiled WASM)

- `func[282]` = `int_to_string` - Signature: `(i32) -> i32` - **WRONG function being called**
- `func[309]` = Unnamed, signature `(i32, i32, i32) -> i32` - **This is __json_get_field**
- `func[312]` = `json.tryTextToData` - Signature: `(i32) -> i32`

## Reproduction

Test file: `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/tests/cln/stdlib/json/json_field_access_bug.cln`

```bash
# Compile
cln compile tests/cln/stdlib/json/json_field_access_bug.cln -o /tmp/json-bug-test.wasm --plugins

# Run
~/.cleen/server/0.2.2/clean-server /tmp/json-bug-test.wasm --port 3002
```

Expected: `PASS: Field access works correctly`
Actual: `FAIL: Field access returned '0' instead of '4'`

## Code Path Analysis

The compiler uses MIR (Mid-level Intermediate Representation) for code generation. The expected flow:

### 1. MIR Builder (`src/mir/mir_builder.rs:4058-4124`)
When processing `TastExpressionKind::PropertyAccess`, it should check:
```rust
if matches!(object.expr_type, ConcreteType::Any) {
    // Generate AnyGetField operation
    MirOperation::AnyGetField { object, key }
}
```

**INVESTIGATE**: Is `object.expr_type` correctly identified as `ConcreteType::Any`?

### 2. MIR Codegen (`src/codegen/mir_codegen.rs:2791-2811`)
When processing `MirOperation::AnyGetField`:
```rust
// Load object (any_ptr)
self.load_operand(object)?;
// Load key string expanded to (content_ptr, len) - this pushes TWO values
self.load_string_argument_for_print(key)?;
// Get function index
let json_get_field_idx = self.get_or_register_json_get_field()?;
// Call __json_get_field
self.current_instructions.push(Instruction::Call(json_get_field_idx));
```

**INVESTIGATE**: Is `get_or_register_json_get_field()` returning the correct index (should be 309)?

### 3. Function Registration (`src/stdlib/json_class.rs:243-258`)
```rust
register_stdlib_function_with_locals(
    codegen,
    "__json_get_field",
    &[WasmType::I32, WasmType::I32, WasmType::I32], // any_ptr, key_ptr, key_len
    Some(WasmType::I32),                            // returns any pointer
    ...
);
```

**INVESTIGATE**: Is this function correctly registered in the function_map?

## Likely Failure Points

1. **Type Recognition Failure**: `object.expr_type` is NOT `ConcreteType::Any`
   - The MIR builder takes the wrong branch (GetElementPtr instead of AnyGetField)
   - This would explain why __json_get_field isn't being called at all

2. **MirOperation Not Generated**: `AnyGetField` isn't being created
   - Check if the code path reaches line 4081 in mir_builder.rs

3. **Wrong Function Index Lookup**: `get_or_register_json_get_field()` returns wrong index
   - Check `function_map.get("__json_get_field")` mapping

4. **Different Code Path**: A legacy code path bypasses the MIR system

## Files to Investigate

1. **Type Inference** - Verify `any` type propagates correctly:
   - `src/typechecker/type_inference.rs:3890` - `json.tryTextToData` return type
   - `src/resolver/symbol_table.rs:1099` - Function signature registration

2. **MIR Builder** - Check AnyGetField generation:
   - `src/mir/mir_builder.rs:4058-4124` - PropertyAccess handling
   - Check if `object.expr_type` is `ConcreteType::Any`

3. **MIR Codegen** - Check WASM generation:
   - `src/codegen/mir_codegen.rs:2791-2811` - AnyGetField processing
   - `src/codegen/mir_codegen.rs:4972-4989` - `get_or_register_json_get_field()`

4. **Function Registration** - Verify correct function index:
   - `src/stdlib/json_class.rs:243-258` - `__json_get_field` registration
   - `src/codegen/mod.rs:6188-6256` - `register_function_with_locals`

## Debugging Strategy

Add debug output to trace the code path:

### 1. In `mir_builder.rs:4081`:
```rust
eprintln!("DEBUG PropertyAccess: object.expr_type = {:?}", object.expr_type);
if matches!(object.expr_type, ConcreteType::Any) {
    eprintln!("DEBUG: Taking AnyGetField path");
    // ... existing code
} else {
    eprintln!("DEBUG: Taking GetElementPtr path (WRONG for any type!)");
    // ... existing code
}
```

### 2. In `mir_codegen.rs:2791`:
```rust
MirOperation::AnyGetField { object, key } => {
    eprintln!("DEBUG: Processing AnyGetField operation");
    // ...
    let json_get_field_idx = self.get_or_register_json_get_field()?;
    eprintln!("DEBUG: __json_get_field index = {}", json_get_field_idx);
    // ...
}
```

### 3. In `json_class.rs` after registration:
```rust
let idx = register_stdlib_function_with_locals(...)?;
eprintln!("DEBUG: __json_get_field registered at index {}", idx);
```

## Fix Verification

After fixing, verify with:

### 1. Compile the test file:
```bash
cln compile tests/cln/stdlib/json/json_field_access_bug.cln -o /tmp/json-bug-test.wasm --plugins
```

### 2. Check the WASM disassembly:
```bash
wasm2wat /tmp/json-bug-test.wasm -o /tmp/json-bug-test.wat
# Search for the testSimpleJsonParsing function
# Verify it calls function 309 (or correct __json_get_field index) with 3 arguments
# NOT function 282 (int_to_string)
```

### 3. Run the test:
```bash
~/.cleen/server/0.2.2/clean-server /tmp/json-bug-test.wasm --port 3002
```

Expected output:
```
=== JSON Field Access Bug Test ===

Test 1: Simple object parsing
PASS: Field access works correctly

Test 2: Nested object parsing
PASS: Nested field access works

=== Expected Results ===
Both tests should PASS
```

## Summary

The bug is in the **Clean Language Compiler**. The generated WASM code calls function 282 (`int_to_string`) instead of function 309 (`__json_get_field`) when accessing fields on `any` type variables.

Most likely cause: The MIR builder is not recognizing `object.expr_type` as `ConcreteType::Any`, causing it to take the wrong code path.

## Priority

**CRITICAL** - This bug breaks:
- All JSON API request handling
- Any application that needs to parse and access JSON fields
- The entire POST/PUT/PATCH workflow for web applications
