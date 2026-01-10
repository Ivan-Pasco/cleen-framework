# Prompts for Native WASM Architecture

## Overview

We're changing the architecture so that:
1. **All stdlib functions compile to native WASM bytecode** (string.*, list.*, math.*, conversions)
2. **Bridge is only for platform services** (I/O, HTTP, filesystem, server APIs)
3. **Same WASM runs everywhere** - different platforms implement the bridge in their native language

---

## PROMPT 1: Clean Language Compiler

### Task: Implement stdlib functions as native WASM bytecode instead of host imports

### Current Behavior

The compiler generates import calls for most stdlib functions:

```rust
// Current: builtin_generator.rs
self.register_import_function("env", "string.split", vec![ValType::I32, ValType::I32], vec![ValType::I32])?;
```

This generates WASM that calls out to the host:
```wasm
(import "env" "string.split" (func $string.split (param i32 i32) (result i32)))
(call $string.split)
```

### Required Change

Generate native WASM bytecode for these functions, similar to how `math.abs` is already done:

```rust
// Example: math.abs is already native
let abs_instructions = vec![
    Instruction::LocalGet(0),
    Instruction::F64Abs,  // Native WASM instruction
];
self.register_function("math_abs", vec![ValType::F64], vec![ValType::F64], &abs_instructions)?;
```

### Functions to Implement as Native WASM

**String functions** (operate on memory, return pointers):
- `string.split(str, delimiter)` → list of strings
- `string.concat(a, b)` → new string
- `string.trim(str)` → new string
- `string.indexOf(str, search)` → integer
- `string.substring(str, start, end)` → new string
- `string.size(str)` / `string.length(str)` → integer
- `string.startsWith(str, prefix)` → boolean
- `string.endsWith(str, suffix)` → boolean
- `string.contains(str, search)` → boolean
- `string.toUpperCase(str)` → new string
- `string.toLowerCase(str)` → new string
- `string.replace(str, old, new)` → new string
- `string.charAt(str, index)` → string (single char)
- `string.charCodeAt(str, index)` → integer

**List functions**:
- `list.length(list)` → integer
- `list.get(list, index)` → element
- `list.set(list, index, value)` → list
- `list.push(list, value)` → list
- `list.pop(list)` → element
- `list.concat(list1, list2)` → new list
- `list.slice(list, start, end)` → new list
- `list.indexOf(list, value)` → integer
- `list.contains(list, value)` → boolean
- `list.reverse(list)` → new list
- `list.join(list, delimiter)` → string

**Conversion functions**:
- `int_to_string(int)` → string
- `float_to_string(float)` → string
- `bool_to_string(bool)` → string
- `string_to_int(str)` → integer
- `string_to_float(str)` → number

**Math functions** (some already native, complete the rest):
- `math.pow(base, exp)` → number
- `math.sqrt(n)` → number
- `math.sin(n)`, `math.cos(n)`, `math.tan(n)` → number
- `math.log(n)`, `math.log10(n)`, `math.log2(n)` → number
- `math.exp(n)` → number

### Functions that STAY as Bridge Imports

These require platform access and cannot be pure WASM:

```
bridge:io.print(msg)
bridge:io.printl(msg)
bridge:io.input(prompt)
bridge:io.input_integer(prompt)

bridge:fs.read(path)
bridge:fs.write(path, content)
bridge:fs.exists(path)
bridge:fs.delete(path)
bridge:fs.append(path, content)

bridge:http.get(url)
bridge:http.post(url, body)
bridge:http.put(url, body)
bridge:http.delete(url)
(... all http functions)

bridge:server.route(method, path, handler_id)
bridge:server.listen(port)
bridge:server.req_param(name)
bridge:server.req_body()
bridge:server.req_header(name)
```

### Memory Layout

Strings and lists need a memory layout. Current or proposed:

**String**:
```
[length: i32][data: bytes...]
```

**List**:
```
[length: i32][capacity: i32][element_type: i32][elements: ...]
```

### Implementation Approach

1. **Start with string.length** - simplest, just read i32 from pointer
2. **Implement string.concat** - allocate new memory, copy bytes
3. **Implement string.split** - most complex, needs loops and memory allocation
4. Continue with remaining functions

### Testing

After implementation, this plugin code should work:

```clean
functions:
    string expand_endpoints(string body)
        list lines = string.split(body, "\n")
        integer count = list.length(lines)
        printl("Found " + int_to_string(count) + " lines")
        // Should print actual count, not 0
```

Compile and run without any host providing string.split - it's native WASM now.

### Files to Modify

- `src/codegen/builtin_generator.rs` - main implementation
- `src/codegen/mod.rs` - may need new helper functions
- `src/codegen/memory.rs` - memory allocation helpers (if exists)

---

## PROMPT 1B: Target-Based Bridge Generation (Part of Compiler)

### Task: Generate platform-specific bridge files based on `--target` flag

### Overview

The compiler should generate the appropriate bridge implementation based on the target platform. The WASM is always the same, but the bridge files differ per platform.

### Command Line Interface

```bash
# Server target (default) - no bridge file, clean-server provides it
cln compile app.cln -o dist/app.wasm --target=server

# Browser target - generates JavaScript bridge
cln compile app.cln -o dist/app.wasm --target=browser
# Outputs:
#   dist/app.wasm
#   dist/bridge.js
#   dist/loader.js
#   dist/index.html (optional template)

# Node.js target - generates Node-compatible bridge
cln compile app.cln -o dist/app.wasm --target=node
# Outputs:
#   dist/app.wasm
#   dist/bridge.mjs
#   dist/run.mjs

# Mobile targets - generate bridge stubs
cln compile app.cln -o dist/app.wasm --target=ios
# Outputs:
#   dist/app.wasm
#   dist/CleanBridge.swift

cln compile app.cln -o dist/app.wasm --target=android
# Outputs:
#   dist/app.wasm
#   dist/CleanBridge.kt
```

### Target: browser

Generate JavaScript bridge for web browsers.

**Output: `bridge.js`**
```javascript
// Auto-generated by Clean Language Compiler
// Target: browser

export class CleanBridge {
  constructor(memory) {
    this.memory = memory;
    this.decoder = new TextDecoder();
    this.encoder = new TextEncoder();
  }

  readString(ptr) {
    const view = new DataView(this.memory.buffer);
    const length = view.getInt32(ptr, true);
    const bytes = new Uint8Array(this.memory.buffer, ptr + 4, length);
    return this.decoder.decode(bytes);
  }

  writeString(str, allocFn) {
    const bytes = this.encoder.encode(str);
    const ptr = allocFn(bytes.length + 4);
    const view = new DataView(this.memory.buffer);
    view.setInt32(ptr, bytes.length, true);
    new Uint8Array(this.memory.buffer, ptr + 4, bytes.length).set(bytes);
    return ptr;
  }

  getImports(allocFn) {
    const self = this;
    return {
      env: {
        // I/O
        print: (ptr) => process.stdout?.write(self.readString(ptr)) ?? console.log(self.readString(ptr)),
        printl: (ptr) => console.log(self.readString(ptr)),
        input: (promptPtr) => self.writeString(window.prompt(self.readString(promptPtr)) || "", allocFn),

        // HTTP Client
        http_get: async (urlPtr) => {
          const url = self.readString(urlPtr);
          const response = await fetch(url);
          return self.writeString(await response.text(), allocFn);
        },
        http_post: async (urlPtr, bodyPtr) => {
          const url = self.readString(urlPtr);
          const body = self.readString(bodyPtr);
          const response = await fetch(url, { method: 'POST', body });
          return self.writeString(await response.text(), allocFn);
        },

        // Filesystem (localStorage)
        file_read: (pathPtr) => {
          const path = self.readString(pathPtr);
          const content = localStorage.getItem(`clean:${path}`);
          if (content === null) return 0; // null pointer for not found
          return self.writeString(content, allocFn);
        },
        file_write: (pathPtr, contentPtr) => {
          const path = self.readString(pathPtr);
          localStorage.setItem(`clean:${path}`, self.readString(contentPtr));
          return 1;
        },
        file_exists: (pathPtr) => {
          return localStorage.getItem(`clean:${self.readString(pathPtr)}`) !== null ? 1 : 0;
        },
        file_delete: (pathPtr) => {
          localStorage.removeItem(`clean:${self.readString(pathPtr)}`);
          return 1;
        },

        // Memory
        mem_alloc: (size) => allocFn(size),
        mem_retain: (ptr) => ptr,
        mem_release: (ptr) => {}
      }
    };
  }
}
```

**Output: `loader.js`**
```javascript
// Auto-generated by Clean Language Compiler
// WASM Loader for browser

import { CleanBridge } from './bridge.js';

export async function loadCleanApp(wasmPath) {
  const response = await fetch(wasmPath);
  const bytes = await response.arrayBuffer();

  // First instantiation to get memory and allocator
  const tempModule = await WebAssembly.compile(bytes);
  const tempInstance = await WebAssembly.instantiate(tempModule, {
    env: { /* minimal stubs */ }
  });

  const bridge = new CleanBridge(tempInstance.exports.memory);
  const imports = bridge.getImports(tempInstance.exports.mem_alloc);

  // Real instantiation with bridge
  const instance = await WebAssembly.instantiate(tempModule, imports);

  // Run the app
  if (instance.exports._start) {
    instance.exports._start();
  }

  return instance;
}

// Auto-run if this is the main module
loadCleanApp('./app.wasm');
```

### Target: node

Generate Node.js-compatible bridge with real filesystem access.

**Output: `bridge.mjs`**
```javascript
// Auto-generated by Clean Language Compiler
// Target: node

import * as fs from 'fs';
import * as http from 'http';
import * as https from 'https';

export class CleanBridge {
  // ... similar to browser but with:

  getImports(allocFn) {
    return {
      env: {
        // Real filesystem
        file_read: (pathPtr) => {
          const path = this.readString(pathPtr);
          const content = fs.readFileSync(path, 'utf-8');
          return this.writeString(content, allocFn);
        },
        file_write: (pathPtr, contentPtr) => {
          const path = this.readString(pathPtr);
          fs.writeFileSync(path, this.readString(contentPtr));
          return 1;
        },

        // I/O
        printl: (ptr) => console.log(this.readString(ptr)),

        // HTTP using Node's http/https
        http_get: (urlPtr) => {
          // Use http.get or https.get based on URL
        }
      }
    };
  }
}
```

### Target: ios

Generate Swift bridge stub.

**Output: `CleanBridge.swift`**
```swift
// Auto-generated by Clean Language Compiler
// Target: iOS

import Foundation
import JavaScriptCore

class CleanBridge {
    private var memory: Data

    init(memory: Data) {
        self.memory = memory
    }

    func readString(_ ptr: Int32) -> String {
        let length = memory.withUnsafeBytes { $0.load(fromByteOffset: Int(ptr), as: Int32.self) }
        let start = Int(ptr) + 4
        let bytes = memory.subdata(in: start..<(start + Int(length)))
        return String(data: bytes, encoding: .utf8) ?? ""
    }

    func getImports() -> [String: Any] {
        return [
            "printl": { (ptr: Int32) in
                print(self.readString(ptr))
            },
            "http_get": { (urlPtr: Int32) -> Int32 in
                let url = self.readString(urlPtr)
                // Use URLSession...
                return 0
            },
            "file_read": { (pathPtr: Int32) -> Int32 in
                let path = self.readString(pathPtr)
                // Use FileManager...
                return 0
            }
        ]
    }
}
```

### Target: android

Generate Kotlin bridge stub.

**Output: `CleanBridge.kt`**
```kotlin
// Auto-generated by Clean Language Compiler
// Target: Android

package com.clean.runtime

import java.nio.ByteBuffer
import java.nio.ByteOrder

class CleanBridge(private val memory: ByteBuffer) {

    init {
        memory.order(ByteOrder.LITTLE_ENDIAN)
    }

    fun readString(ptr: Int): String {
        val length = memory.getInt(ptr)
        val bytes = ByteArray(length)
        memory.position(ptr + 4)
        memory.get(bytes)
        return String(bytes, Charsets.UTF_8)
    }

    fun getImports(): Map<String, Any> {
        return mapOf(
            "printl" to { ptr: Int ->
                android.util.Log.d("Clean", readString(ptr))
            },
            "http_get" to { urlPtr: Int ->
                // Use OkHttp...
                0
            },
            "file_read" to { pathPtr: Int ->
                // Use File...
                0
            }
        )
    }
}
```

### Implementation in Compiler

Add a new module for bridge generation:

**File: `src/codegen/bridge_generator.rs`**

```rust
pub enum Target {
    Server,   // No bridge file needed
    Browser,  // Generate JS for web
    Node,     // Generate JS for Node.js
    iOS,      // Generate Swift
    Android,  // Generate Kotlin
}

pub struct BridgeGenerator {
    target: Target,
    output_dir: PathBuf,
}

impl BridgeGenerator {
    pub fn generate(&self) -> Result<Vec<PathBuf>, Error> {
        match self.target {
            Target::Server => Ok(vec![]), // clean-server provides bridge
            Target::Browser => self.generate_browser_bridge(),
            Target::Node => self.generate_node_bridge(),
            Target::iOS => self.generate_ios_bridge(),
            Target::Android => self.generate_android_bridge(),
        }
    }

    fn generate_browser_bridge(&self) -> Result<Vec<PathBuf>, Error> {
        let bridge_js = include_str!("templates/browser/bridge.js");
        let loader_js = include_str!("templates/browser/loader.js");

        fs::write(self.output_dir.join("bridge.js"), bridge_js)?;
        fs::write(self.output_dir.join("loader.js"), loader_js)?;

        Ok(vec![
            self.output_dir.join("bridge.js"),
            self.output_dir.join("loader.js"),
        ])
    }

    // ... other generators
}
```

### Files to Add/Modify

- `src/codegen/bridge_generator.rs` - new module for bridge generation
- `src/codegen/templates/` - template files for each target
  - `templates/browser/bridge.js`
  - `templates/browser/loader.js`
  - `templates/node/bridge.mjs`
  - `templates/node/run.mjs`
  - `templates/ios/CleanBridge.swift`
  - `templates/android/CleanBridge.kt`
- `src/main.rs` or CLI handling - add `--target` flag
- `src/codegen/mod.rs` - integrate bridge generation after WASM generation

---

## PROMPT 2: Clean Server

### Task: Remove stdlib host functions, keep only bridge functions

### Current Behavior

Clean server implements ALL host functions including string.split, list.length, etc.

### Required Change

After the compiler generates native WASM for stdlib, clean-server only needs to implement bridge functions:

**Remove these implementations** (will be native WASM):
- All `string.*` functions
- All `list.*` functions
- All `math.*` functions (except those truly needing host)
- Conversion functions (int_to_string, etc.)

**Keep these implementations** (bridge/platform services):
- `bridge:io.*` - print, printl, input
- `bridge:fs.*` - file operations
- `bridge:http.*` - HTTP client functions
- `bridge:server.*` - HTTP server functions (_http_route, _req_param, etc.)

### Import Naming

Update imports to use `bridge:` namespace:

```rust
// Before
linker.func_wrap("env", "string.split", |...| { ... })?;
linker.func_wrap("env", "printl", |...| { ... })?;

// After - only bridge functions remain
linker.func_wrap("env", "bridge:io.printl", |...| { ... })?;
linker.func_wrap("env", "bridge:fs.read", |...| { ... })?;
linker.func_wrap("env", "bridge:http.get", |...| { ... })?;
```

Or keep current naming but just remove stdlib implementations.

### Testing

After changes:
1. Compile a Clean app with new compiler
2. Run with updated clean-server
3. String/list operations should work (native WASM)
4. HTTP/file operations should work (bridge)

---

## PROMPT 3: Clean Framework

### Task: Document the bridge specification

### File: Update `documents/specification/frame_bridge_contracts.md`

Document the standard bridge interface:

1. **bridge:io** - Console I/O
2. **bridge:fs** - Filesystem
3. **bridge:http** - HTTP client
4. **bridge:server** - HTTP server (Frame-specific)
5. **bridge:env** - Environment variables
6. **bridge:time** - Time/date operations
7. **bridge:crypto** - Cryptographic operations

For each function:
- Name and signature
- Parameters (types, meaning)
- Return value
- Platform availability (browser, server, CLI, mobile)
- Example usage

---

## Implementation Order

1. **Compiler - Native WASM Stdlib** (PROMPT 1) - Most critical, enables everything else
2. **Compiler - Target-Based Bridge Generation** (PROMPT 1B) - Generate platform bridges
3. **Clean Server** (PROMPT 2) - Simplify to bridge-only
4. **Framework Docs** (PROMPT 3) - Document the bridge spec (DONE)

After step 1, plugins will work immediately. Step 2 enables browser/mobile deployment. Steps 3-4 complete the ecosystem.
