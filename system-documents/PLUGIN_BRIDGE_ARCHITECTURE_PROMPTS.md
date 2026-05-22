# Plugin Bridge Architecture Implementation Prompts

This document contains prompts for implementing plugin-based bridge function discovery in the Clean Language compiler. Execute these prompts in order.

---

## Implementation Order

1. Specification Update (Prompt 1)
2. Bridge Contracts Documentation (Prompt 8)
3. frame.data Plugin Manifest (Prompt 2)
4. frame.web Plugin Manifest (Prompt 3)
5. Compiler Plugin Loader (Prompt 4)
6. Compiler Semantic Analyzer (Prompt 5)
7. Compiler Code Generator (Prompt 6)
8. Compiler Pipeline Integration (Prompt 7)
9. Remove Hardcoded Functions (Prompt 9)

---

## Prompt 1: Plugin Manifest Schema Update

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/documents/specification/10_compiler_plugins.md`

```
Update the Clean Language compiler plugin specification to add bridge function declarations.

CONTEXT:
- Plugins currently declare blocks they handle but NOT the Host Bridge functions they need
- This creates coupling between plugins and compiler (compiler must hardcode bridge functions)
- Goal: Plugins declare their bridge function dependencies in plugin.toml

TASK:
1. Add a new section "Bridge Function Declarations" to the specification
2. Define the `[bridge]` section schema for plugin.toml:
   ```toml
   [bridge]
   functions = [
     { name = "_db_query", params = ["string", "string"], returns = "string", module = "env" },
   ]
   ```
3. Document supported param/return types: "string", "integer", "number", "boolean", "void"
4. Explain that `module` defaults to "env" (standard WASM import module)
5. Add examples for frame.data, frame.web, frame.auth plugins
6. Document the flow: plugin.toml → compiler reads → generates WASM imports → runtime provides implementations

CONSTRAINTS:
- Must be backwards compatible (plugins without [bridge] section still work)
- Type names must match Clean Language type system
- Follow existing documentation style in the specification
```

---

## Prompt 2: frame.data Plugin Manifest

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/plugins/frame.data/plugin.toml`

```
Update the frame.data plugin manifest to declare its Host Bridge function dependencies.

CONTEXT:
- frame.data plugin generates Clean code that calls _db_query, _db_execute, etc.
- These functions are implemented in clean-server/src/bridge.rs
- Currently the compiler has these hardcoded, breaking plugin independence

CURRENT plugin.toml:
[plugin]
name = "frame.data"
version = "1.0.0"
description = "ORM and database plugin for Clean Language"

[blocks]
handles = ["model", "query", "transaction"]

TASK:
Add [bridge] section declaring all database functions the plugin uses:

Functions to declare (check clean-server/src/bridge.rs for exact signatures):
- _db_query(sql: string, params: string) -> string  (JSON result)
- _db_execute(sql: string, params: string) -> integer  (affected rows)
- _db_begin() -> string  (transaction ID)
- _db_commit(tx_id: string) -> integer  (success/failure)
- _db_rollback(tx_id: string) -> integer  (success/failure)

FORMAT:
[bridge]
functions = [
  { name = "...", params = [...], returns = "...", description = "..." }
]

CONSTRAINTS:
- Types must use Clean Language names: "string", "integer", "number", "boolean", "void"
- Each function should have a brief description
- Order: query/execute first, then transaction functions
```

---

## Prompt 3: frame.web Plugin Manifest

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/plugins/frame.web/plugin.toml`

```
Update the frame.web plugin manifest to declare HTTP bridge function dependencies.

CONTEXT:
- HTTP functions (_http_route, _req_param, _req_body, etc.) are currently hardcoded in the compiler
- These should be declared by the frame.web plugin instead
- This allows the compiler to be plugin-agnostic

TASK:
Add [bridge] section declaring all HTTP/request functions:

Server functions:
- _http_route(method: string, path: string, handler_idx: integer) -> integer
- _http_listen(port: integer) -> integer

Request context functions:
- _req_param(name: string) -> string
- _req_query(name: string) -> string
- _req_body() -> string
- _req_header(name: string) -> string
- _req_method() -> string
- _req_path() -> string

Response functions (if any exist in bridge.rs):
- Check clean-server/src/bridge.rs for _res_* functions

CONSTRAINTS:
- Match exact signatures from clean-server/src/bridge.rs
- Use Clean Language type names
- Group functions logically (server, request, response)
```

---

## Prompt 4: Compiler Plugin Loader

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/plugins/mod.rs` (or appropriate file)

```
Extend the compiler's plugin loader to read bridge function declarations from plugin.toml.

CONTEXT:
- Plugins are loaded from ~/.cleen/plugins/<name>/<version>/
- plugin.toml contains metadata, exports, and blocks
- Need to add parsing of [bridge] section

CURRENT FLOW:
1. Compiler detects plugin usage (use frame.data, etc.)
2. Loads plugin WASM for block expansion
3. Does NOT read bridge function declarations

NEW FLOW:
1. Compiler detects plugin usage
2. Loads plugin.toml
3. Parses [bridge].functions array
4. Returns BridgeFunctionDeclaration structs to caller

TASK:
1. Create BridgeFunctionDeclaration struct:
   ```rust
   pub struct BridgeFunctionDeclaration {
       pub name: String,
       pub params: Vec<String>,  // Clean type names
       pub returns: String,      // Clean type name or "void"
       pub module: String,       // WASM import module, default "env"
       pub description: Option<String>,
   }
   ```

2. Add function to parse [bridge] section from plugin.toml:
   ```rust
   pub fn load_bridge_declarations(plugin_path: &Path) -> Result<Vec<BridgeFunctionDeclaration>, PluginError>
   ```

3. Handle missing [bridge] section gracefully (return empty vec)

4. Validate:
   - Function names start with underscore (convention for bridge functions)
   - Types are valid Clean types
   - No duplicate function names

CONSTRAINTS:
- Use toml crate for parsing (already a dependency)
- Error messages should indicate plugin name and file path
- Must be backwards compatible with plugins without [bridge] section
```

---

## Prompt 5: Compiler Semantic Analyzer Integration

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/semantic/mod.rs`

```
Modify the semantic analyzer to dynamically register bridge functions from loaded plugins.

CONTEXT:
- Currently bridge functions (_http_route, _db_query, etc.) are hardcoded in register_builtin_functions()
- Plugin loader now provides BridgeFunctionDeclaration from plugin.toml
- Need to register these dynamically instead of hardcoding

CURRENT CODE (to be replaced):
```rust
// HTTP server functions (internal bridge functions for Frame runtime)
self.function_table.insert("_http_route".to_string(), vec![(vec![...], Type::Integer, 3)]);
self.function_table.insert("_db_query".to_string(), vec![(vec![...], Type::String, 2)]);
// ... more hardcoded functions
```

TASK:
1. Add method to register bridge functions from declarations:
   ```rust
   pub fn register_bridge_functions(&mut self, declarations: &[BridgeFunctionDeclaration]) {
       for decl in declarations {
           let params = self.convert_types(&decl.params);
           let return_type = self.convert_type(&decl.returns);
           self.function_table.insert(
               decl.name.clone(),
               vec![(params, return_type, decl.params.len())]
           );
       }
   }
   ```

2. Add type conversion helper:
   ```rust
   fn convert_type(&self, type_name: &str) -> Type {
       match type_name {
           "string" => Type::String,
           "integer" => Type::Integer,
           "number" => Type::Number,
           "boolean" => Type::Boolean,
           "void" => Type::Unit,
           _ => Type::Any,
       }
   }
   ```

3. Remove hardcoded _db_* and _http_* function registrations
   - Keep only core language builtins (print, printl, etc.)
   - Bridge functions come from plugins

4. Update SemanticAnalyzer::new() to accept optional bridge declarations

CONSTRAINTS:
- Maintain backwards compatibility: if no declarations provided, use empty set
- Type conversion must handle all Clean Language types
- Function arity (param count) must be tracked for validation
```

---

## Prompt 6: Compiler Code Generator Integration

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/codegen/mod.rs`

```
Modify the code generator to dynamically create WASM imports from plugin bridge declarations.

CONTEXT:
- Currently has separate methods: register_http_imports(), register_file_imports()
- These hardcode WASM import generation for specific functions
- Need to replace with dynamic generation from BridgeFunctionDeclaration

CURRENT PATTERN (to be generalized):
```rust
fn register_http_imports(&mut self) -> Result<(), CompilerError> {
    // _http_route(methodPtr, methodLen, pathPtr, pathLen, handlerIdx) -> i32
    let route_type = self.add_function_type(&[I32, I32, I32, I32, I32], Some(I32))?;
    self.import_section.import("env", "_http_route", EntityType::Function(route_type));
    // ... more hardcoded imports
}
```

TASK:
1. Create generic method for registering bridge imports:
   ```rust
   pub fn register_bridge_imports(&mut self, declarations: &[BridgeFunctionDeclaration]) -> Result<(), CompilerError> {
       for decl in declarations {
           let wasm_params = self.convert_to_wasm_params(&decl.params);
           let wasm_return = self.convert_to_wasm_return(&decl.returns);

           let type_index = self.add_function_type(&wasm_params, wasm_return)?;
           self.import_section.import(&decl.module, &decl.name, EntityType::Function(type_index));

           self.function_map.insert(decl.name.clone(), self.function_count);
           self.function_count += 1;
       }
       Ok(())
   }
   ```

2. Handle string parameters correctly:
   - Clean "string" → WASM (i32, i32) for (pointer, length)
   - This is critical for bridge function calling convention

   ```rust
   fn convert_to_wasm_params(&self, params: &[String]) -> Vec<WasmType> {
       let mut wasm_params = Vec::new();
       for param in params {
           match param.as_str() {
               "string" => {
                   wasm_params.push(WasmType::I32); // pointer
                   wasm_params.push(WasmType::I32); // length
               }
               "integer" | "boolean" => wasm_params.push(WasmType::I32),
               "number" => wasm_params.push(WasmType::F64),
               _ => wasm_params.push(WasmType::I32),
           }
       }
       wasm_params
   }
   ```

3. Remove hardcoded register_http_imports(), register_database_imports() etc.
   - Replace calls in mir_codegen.rs with single register_bridge_imports() call

4. Update MirCodeGenerator to:
   - Load bridge declarations from all used plugins
   - Pass to register_bridge_imports()

CONSTRAINTS:
- String expansion to (ptr, len) is CRITICAL - bridge functions expect this format
- Must maintain function index ordering (imports before internal functions)
- Keep non-bridge imports (print, printl, math functions) - these are core, not plugin-based
```

---

## Prompt 7: Compiler Main Pipeline Integration

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/codegen/mir_codegen.rs`

```
Update the MIR code generator to load and use plugin bridge declarations.

CONTEXT:
- MirCodeGenerator orchestrates the compilation pipeline
- Currently calls individual register_*_imports() methods
- Need to load plugin declarations and pass to unified registration

CURRENT CODE:
```rust
// In generate() method:
self.wasm_generator.register_http_imports().map_err(|e| vec![e])?;
self.wasm_generator.register_file_imports().map_err(|e| vec![e])?;
```

TASK:
1. Add plugin bridge loading at start of generate():
   ```rust
   // Load bridge declarations from all plugins used by this compilation
   let bridge_declarations = self.load_plugin_bridge_declarations()?;
   ```

2. Replace individual register calls with unified call:
   ```rust
   // Register all bridge imports from plugins
   self.wasm_generator.register_bridge_imports(&bridge_declarations).map_err(|e| vec![e])?;
   ```

3. Implement load_plugin_bridge_declarations():
   ```rust
   fn load_plugin_bridge_declarations(&self) -> Result<Vec<BridgeFunctionDeclaration>, Vec<CompilerError>> {
       let mut all_declarations = Vec::new();

       // Get list of plugins used (from AST analysis or explicit imports)
       let used_plugins = self.detect_used_plugins();

       for plugin_name in used_plugins {
           let plugin_path = get_plugin_path(&plugin_name)?;
           let declarations = load_bridge_declarations(&plugin_path)?;
           all_declarations.extend(declarations);
       }

       Ok(all_declarations)
   }
   ```

4. Implement detect_used_plugins():
   - Scan AST for framework blocks (model:, query:, endpoints:, etc.)
   - Map block types to plugin names
   - Return list of plugin names to load

CONSTRAINTS:
- Must handle case where no plugins are used (empty declarations)
- Plugin detection should be based on actual usage, not hardcoded list
- Error handling: missing plugin should give clear error message
```

---

## Prompt 8: Bridge Function Documentation

**File:** `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/documents/specification/frame_bridge_contracts.md`

```
Update the Host Bridge contracts documentation to serve as the source of truth for bridge functions.

CONTEXT:
- Plugins declare bridge functions in plugin.toml
- Server implements bridge functions in bridge.rs
- Documentation should define the contract between them

TASK:
1. Add comprehensive "Database Bridge Functions" section:

   ## Database Functions (frame.data plugin)

   ### _db_query
   - **Signature:** `_db_query(sql: string, params: string) -> string`
   - **WASM Signature:** `(i32, i32, i32, i32) -> i32` (sql_ptr, sql_len, params_ptr, params_len)
   - **Description:** Execute SELECT query, return JSON result
   - **Params JSON:** Array of values, e.g., `[42, "active"]`
   - **Returns JSON:** `{"ok": true, "data": {"rows": [...], "count": N}}`
   - **Errors:** `{"ok": false, "err": {"code": "...", "message": "..."}}`

2. Document all database functions:
   - _db_query
   - _db_execute
   - _db_begin
   - _db_commit
   - _db_rollback

3. Add "HTTP Bridge Functions" section for frame.web:
   - _http_route
   - _http_listen
   - _req_param, _req_query, _req_body, _req_header, _req_method, _req_path

4. Add "String Passing Convention" section:
   - Explain (pointer, length) format
   - Document length-prefix format for return values
   - Memory allocation responsibilities

5. Add "Error Envelope Format" section:
   - Standard ok/err structure
   - Error codes by category

CONSTRAINTS:
- This document becomes the authoritative reference
- Plugin.toml declarations must match this spec
- Server implementations must match this spec
- Include version compatibility notes
```

---

## Prompt 9: Remove Hardcoded Bridge Functions from Compiler

**Files:**
- `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/builtins/registry.rs`
- `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/semantic/mod.rs`
- `/Users/earcandy/Documents/Dev/Clean Language/clean-language-compiler/src/resolver/resolver_impl.rs`

```
Remove hardcoded bridge function registrations from the compiler core.

CONTEXT:
- Bridge functions are now declared in plugin.toml files
- Compiler dynamically loads and registers them
- Hardcoded registrations create duplication and coupling

TASK:
1. In builtins/registry.rs:
   - Remove register_database_functions() method
   - Remove register_http_server_functions() method
   - Remove BuiltinCategory::Database (if only used for bridge functions)
   - Keep core builtins: IO, Math, String, List, Type, Compare, etc.

2. In semantic/mod.rs (register_builtin_functions):
   - Remove _http_route, _http_listen registrations
   - Remove _req_param, _req_query, _req_body, etc. registrations
   - Remove _db_query, _db_execute registrations
   - Keep: print, printl, input, toString, toInteger, etc.

3. In resolver/resolver_impl.rs:
   - Remove hardcoded HTTP function registrations
   - Remove hardcoded database function registrations
   - Keep core language function registrations

4. Search for any other files with hardcoded _http_* or _db_* registrations:
   ```bash
   grep -r "_http_route\|_db_query\|_req_param" src/
   ```

VERIFICATION:
After changes, compile a simple program WITHOUT plugins - should work.
Compile a program WITH frame.data plugin - should load declarations from plugin.toml.

CONSTRAINTS:
- Do NOT remove print, printl, input - these are core language features
- Do NOT remove Math, String methods - these are core
- Only remove bridge functions that will come from plugins
- Ensure backwards compatibility during transition
```

---

## Testing Checklist

After implementing all prompts, verify:

- [ ] Simple Clean program compiles without plugins
- [ ] Program using `model:` block loads frame.data plugin declarations
- [ ] Program using `endpoints:` block loads frame.web plugin declarations
- [ ] WASM output has correct imports for bridge functions
- [ ] Runtime (clean-server) successfully links all imports
- [ ] Database queries work end-to-end
- [ ] HTTP routing works end-to-end
- [ ] Error messages are clear when plugin is missing
- [ ] Error messages are clear when bridge function is undefined

---

## Rollback Plan

If issues arise, the changes can be rolled back by:

1. Restoring hardcoded function registrations in compiler
2. Removing [bridge] sections from plugin.toml (optional, backwards compatible)
3. Reverting codegen to use individual register_*_imports() methods

The plugin.toml [bridge] sections are additive and won't break anything if the compiler doesn't read them.
