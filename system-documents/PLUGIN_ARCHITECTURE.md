# Frame Plugin Architecture - FINAL IMPLEMENTATION ✅

**Status:** COMPLETE
**Date:** 2025-11-22
**Architecture:** Hybrid Approach (Library Integration + Separate Binaries)

---

## Overview

This document describes the final plugin architecture for Clean Language and Clean Frame, which successfully separates the core compiler from framework-specific plugins.

---

## Architecture Diagram

### Current State (✅ IMPLEMENTED)

```
┌─────────────────────────────────────────────────────────────────┐
│                   clean-language-compiler v0.14.0               │
│                        (PURE LANGUAGE)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Public API:                                                    │
│  ├─ compile_pure(source, file_path)                            │
│  │   └─ Compiles pure Clean Language (no plugins)              │
│  │                                                              │
│  ├─ compile_with_plugins(source, file_path, registry)          │
│  │   └─ Compiles with custom plugin registry                   │
│  │                                                              │
│  ├─ PluginRegistry (infrastructure)                            │
│  │   ├─ builder() → PluginRegistryBuilder                      │
│  │   ├─ handles(block_name) → bool                             │
│  │   └─ expand(block) → Result<Vec<Statement>>                 │
│  │                                                              │
│  └─ Version exports:                                           │
│      ├─ VERSION: &str                                          │
│      └─ MIN_PLUGIN_VERSION: &str                               │
│                                                                 │
│  Compilation Pipeline:                                         │
│  Source → Lexer → Parser → [Plugin Expansion] → HIR →          │
│           → Resolver → TypeChecker → MIR → WASM                │
│                           ↑                                     │
│                   Stage 2.5: Plugins transform here            │
│                                                                 │
│  ⚠️ NO FRAME DEPENDENCIES                                      │
│  ⚠️ NO HARDCODED PLUGINS                                       │
└─────────────────────────────────────────────────────────────────┘
                               ↓
                    Provides plugin infrastructure
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│              frame-compiler-plugins v0.1.0                      │
│                   (FRAMEWORK PLUGINS)                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Implemented Plugins:                                           │
│  ├─ WebPlugin                                                   │
│  │   ├─ Handles: "endpoints"                                    │
│  │   ├─ Transforms: HTTP routing DSL → Clean functions         │
│  │   └─ Generates: __frame_register_routes(Router router)      │
│  │                                                              │
│  └─ (Future) DataPlugin - ORM DSL                              │
│  └─ (Future) ComponentPlugin - UI DSL                          │
│                                                                 │
│  Public API:                                                    │
│  └─ create_frame_registry() → Result<PluginRegistry>           │
│                                                                 │
│  Dependencies:                                                  │
│  └─ clean-language-compiler = "0.14.0"                         │
└─────────────────────────────────────────────────────────────────┘
                               ↓
                          Used by
                               ↓
┌──────────────────────────┐        ┌─────────────────────────────┐
│  cln v0.14.0             │        │  frame v0.1.0               │
│  (Pure Compiler)         │        │  (Framework Compiler)       │
├──────────────────────────┤        ├─────────────────────────────┤
│                          │        │                             │
│  Uses:                   │        │  Uses:                      │
│  └─ compile_pure()       │        │  └─ FrameCompiler::new()   │
│                          │        │      ├─ Version check       │
│  Features:               │        │      └─ create_registry()   │
│  ✅ Pure Clean Language  │        │                             │
│  ✅ functions: blocks    │        │  └─ compile_with_plugins()  │
│  ✅ Full type system     │        │                             │
│  ✅ WebAssembly output   │        │  Features:                  │
│                          │        │  ✅ All Clean Language      │
│  NOT supported:          │        │  ✅ endpoints: blocks       │
│  ❌ endpoints:           │        │  ✅ (Future) data:          │
│  ❌ data:                │        │  ✅ (Future) component:     │
│  ❌ component:           │        │                             │
│                          │        │  Libraries:                 │
│  Binary:                 │        │  ├─ frame-server            │
│  └─ cln compile app.cln  │        │  ├─ frame-data              │
│                          │        │  └─ frame-ui                │
│                          │        │                             │
│                          │        │  Binary:                    │
│                          │        │  └─ frame build             │
└──────────────────────────┘        └─────────────────────────────┘
```

---

## Implementation Details

### 1. Immutable Plugin Registry

**Design:** Plugins are registered using builder pattern, registry is immutable after creation.

```rust
// Create immutable registry
let registry = PluginRegistry::builder()
    .add(WebPlugin::new())
    .add(DataPlugin::new())
    .build()?;  // Validates conflicts, returns immutable registry

// Registry cannot be modified after creation
// registry.register(...) is deprecated
```

**Benefits:**
- Prevents plugin injection mid-compilation
- Type-safe at build time
- Clear, declarative syntax
- Automatic conflict detection

### 2. Dual Compilation APIs

**compile_pure()** - No plugins, pure language:

```rust
pub fn compile_pure(source: &str, file_path: &str) -> Result<Vec<u8>, Vec<CompilerError>> {
    let registry = PluginRegistry::builder()
        .build()
        .expect("Empty registry should always build");
    compile_with_plugins(source, file_path, &registry)
}
```

**compile_with_plugins()** - With custom registry:

```rust
pub fn compile_with_plugins(
    source: &str,
    file_path: &str,
    registry: &PluginRegistry,
) -> Result<Vec<u8>, Vec<CompilerError>> {
    // Compilation pipeline using provided registry
    // ...
}
```

### 3. Library Integration (Not Process Spawning)

**Before:**
```rust
// frame-cli spawned external process
let output = Command::new("cln")
    .arg("compile")
    .arg(input)
    .output()?;
```

**After:**
```rust
// frame-cli uses compiler as library
use clean_language_compiler::compile_with_plugins;

let wasm = compile_with_plugins(source, file_path, &self.registry)?;
```

**Benefits:**
- No process overhead (~50-100ms saved)
- Shared state and caching
- Better error propagation
- Easier debugging

### 4. Version Compatibility

**Compiler exports version:**

```rust
// In clean-language-compiler/src/lib.rs
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MIN_PLUGIN_VERSION: &str = "0.14.0";
```

**Frame checks compatibility:**

```rust
// In frame-cli/src/frame_compiler.rs
fn check_compiler_compatibility() -> Result<()> {
    // Parse and compare versions
    // Returns error if compiler < REQUIRED_COMPILER_VERSION
}
```

**Cargo.toml constraints:**

```toml
[dependencies]
clean-language-compiler = { path = "../../clean-language-compiler", version = "0.14.0" }
```

---

## Key Features

### ✅ Separation of Concerns

- **Compiler:** Framework-agnostic, pure language implementation
- **Plugins:** Framework-specific DSL transformations
- **Binaries:** Clear distinction (cln vs frame)

### ✅ Extensibility

Third-party frameworks can create their own plugins:

```rust
// my-framework-plugins/src/lib.rs
pub fn create_my_registry() -> Result<PluginRegistry> {
    PluginRegistry::builder()
        .add(MyDslPlugin::new())
        .build()
}
```

### ✅ Type Safety

- Compile-time plugin registration
- Immutable registry prevents runtime modification
- Version constraints in Cargo.toml
- Runtime version validation

### ✅ Clear APIs

- `compile_pure()` for pure language
- `compile_with_plugins()` for frameworks
- `cln` binary for language-only compilation
- `frame` binary for full framework features

---

## Migration Path

### From Old Architecture

**Before:**
```rust
// Hardcoded WebPlugin in compiler
// clean-language-compiler/src/lib.rs
fn compile() {
    let mut registry = PluginRegistry::new();
    registry.register(WebPlugin::new());  // Hardcoded!
    // ...
}
```

**After:**
```rust
// Clean separation
// Compiler provides infrastructure only
pub fn compile_pure(source: &str) -> Result<Vec<u8>> {
    let registry = PluginRegistry::builder().build()?;
    // ...
}

// Framework provides plugins
// frame-compiler-plugins/src/lib.rs
pub fn create_frame_registry() -> Result<PluginRegistry> {
    PluginRegistry::builder()
        .add(WebPlugin::new())
        .build()
}
```

---

## Usage Examples

### Example 1: Pure Clean Language Compilation

```rust
use clean_language_compiler::compile_pure;

let source = r#"
    functions:
        void start()
            print("Hello, World!")
"#;

let wasm = compile_pure(source, "app.cln")?;
std::fs::write("app.wasm", wasm)?;
```

### Example 2: Framework Compilation with Plugins

```rust
use clean_language_compiler::compile_with_plugins;
use frame_compiler_plugins::create_frame_registry;

let source = r#"
    endpoints:
        GET /users -> listUsers

    functions:
        void listUsers()
            print("Listing users")
"#;

let registry = create_frame_registry()?;
let wasm = compile_with_plugins(source, "app.cln", &registry)?;
std::fs::write("app.wasm", wasm)?;
```

### Example 3: Custom Framework

```rust
use clean_language_compiler::plugins::PluginRegistry;
use my_framework::MyPlugin;

let registry = PluginRegistry::builder()
    .add(MyPlugin::new())
    .build()?;

let wasm = compile_with_plugins(source, path, &registry)?;
```

---

## Testing

### Test Coverage

1. **Unit Tests:**
   - Plugin trait implementations
   - Registry builder functionality
   - Version validation logic

2. **Integration Tests:**
   - End-to-end compilation with plugins
   - Multi-file projects
   - Error handling
   - Version compatibility

3. **Test Results:**
   - ✅ All 5 integration tests passing
   - ✅ All 4 frame_compiler unit tests passing
   - ✅ All 311 compiler tests passing

---

## Performance

### Compilation Speed

**Before (Process Spawning):**
- Frame CLI → spawn cln process → compile
- Overhead: ~50-100ms per compilation
- No caching between compilations

**After (Library Integration):**
- Frame CLI → compile_with_plugins() → compile
- Overhead: ~0ms (direct function call)
- Shared caching potential

### Memory Usage

- **Before:** Separate process memory
- **After:** Shared memory, more efficient

---

## Benefits Summary

| Aspect | Before | After |
|--------|--------|-------|
| **Separation** | ❌ WebPlugin in compiler | ✅ Plugins in separate crate |
| **Extensibility** | ❌ Hardcoded plugins | ✅ Custom plugin registries |
| **Performance** | ⚠️ Process spawning | ✅ Direct library calls |
| **Type Safety** | ⚠️ Runtime registration | ✅ Compile-time builder |
| **Versioning** | ❌ No version checks | ✅ Automatic validation |
| **Testing** | ⚠️ Limited | ✅ Comprehensive suite |
| **Dependencies** | ❌ Compiler → Frame | ✅ Frame → Compiler |

---

## Future Enhancements

### 1. DataPlugin (ORM DSL)

```clean
data:
    User
        id: integer primary_key
        name: string
        email: string unique
```

### 2. ComponentPlugin (UI DSL)

```clean
component:
    UserCard(user: User)
        div class="card"
            h2 {user.name}
            p {user.email}
```

### 3. Plugin Discovery

Automatic plugin loading from config:

```toml
# frame.toml
[plugins]
web = "0.1.0"
data = "0.1.0"
ui = "0.1.0"
```

---

## Resources

- **Plugin Development Guide:** [PLUGIN_DEVELOPMENT_GUIDE.md](./PLUGIN_DEVELOPMENT_GUIDE.md)
- **Migration Status:** [PLUGIN_MIGRATION_STATUS.md](./PLUGIN_MIGRATION_STATUS.md)
- **Compiler API:** `clean-language-compiler/src/plugins/mod.rs`
- **WebPlugin Source:** `frame-compiler-plugins/src/web.rs`
- **Integration Tests:** `frame-cli/tests/frame_compiler_integration.rs`

---

**Last Updated:** 2025-11-22
**Compiler Version:** 0.14.0
**Frame Version:** 0.1.0
**Status:** Production Ready ✅
