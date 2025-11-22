# Clean Language Plugin Development Guide

This guide explains how to create compiler plugins for Clean Language and framework extensions like Clean Frame.

---

## Table of Contents

1. [Overview](#overview)
2. [Plugin Architecture](#plugin-architecture)
3. [Creating a Plugin](#creating-a-plugin)
4. [Plugin Registration](#plugin-registration)
5. [Version Compatibility](#version-compatibility)
6. [Testing Plugins](#testing-plugins)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

---

## Overview

### What are Plugins?

Plugins extend Clean Language with Domain-Specific Language (DSL) features. They transform framework-specific blocks (like `endpoints:`, `data:`, `component:`) into standard Clean Language AST before the HIR transformation stage.

### Compilation Pipeline

```text
Source → Lexer → Parser → [Plugin Expansion] → HIR → Resolver → TypeChecker → MIR → WASM
                              ↑
                      Plugins transform here (Stage 2.5)
```

### When to Use Plugins

Use plugins for:
- Framework-specific DSL blocks
- Domain-specific languages built on Clean Language
- Code generation from declarative syntax
- Syntactic sugar that compiles to Clean Language

**Do NOT use plugins for:**
- Runtime functionality (use libraries instead)
- Core language features (contribute to the compiler)
- Simple macros (consider Clean Language features first)

---

## Plugin Architecture

### Core Components

```rust
use clean_language_compiler::plugins::{
    FrameworkPlugin,
    FrameworkBlock,
    PluginResult,
    PluginRegistry,
};
use clean_language_compiler::ast::Statement;

pub struct MyPlugin;

impl FrameworkPlugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my.plugin"  // Unique plugin identifier
    }

    fn handles(&self) -> &'static [&'static str] {
        &["myblock"]  // DSL block identifiers this plugin handles
    }

    fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
        // Transform DSL block into Clean Language AST
        todo!()
    }

    fn validate(&self, block: &FrameworkBlock) -> PluginResult<()> {
        // Optional: Validate block content before expansion
        Ok(())
    }

    fn version(&self) -> &'static str {
        "1.0.0"  // Plugin version
    }
}
```

### FrameworkBlock Structure

```rust
pub struct FrameworkBlock {
    pub name: String,              // Block identifier (e.g., "endpoints")
    pub content: String,           // Block content as string
    pub attributes: Vec<FrameworkAttribute>,  // Block attributes
    pub location: Option<SourceLocation>,     // Source location for errors
}

pub struct FrameworkAttribute {
    pub name: String,                      // Attribute name
    pub value: Option<String>,             // Attribute value (if any)
    pub location: Option<SourceLocation>,  // Attribute location
}
```

---

## Creating a Plugin

### Step 1: Set Up Plugin Crate

Create a new Rust crate for your plugin:

```bash
cargo new my-framework-plugins --lib
cd my-framework-plugins
```

Add dependencies to `Cargo.toml`:

```toml
[package]
name = "my-framework-plugins"
version = "0.1.0"
edition = "2021"

[dependencies]
# Required: Clean Language compiler for plugin infrastructure
clean-language-compiler = { path = "../clean-language-compiler", version = "0.14.0" }

# Optional: Common utilities
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
```

### Step 2: Implement FrameworkPlugin Trait

```rust
// src/my_plugin.rs

use clean_language_compiler::plugins::{
    FrameworkPlugin, FrameworkBlock, PluginResult, PluginError,
};
use clean_language_compiler::ast::{Statement, Function, Expression, Type};

pub struct MyPlugin;

impl FrameworkPlugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my.framework"
    }

    fn handles(&self) -> &'static [&'static str] {
        &["myblock"]
    }

    fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
        // Step 1: Parse the block content
        let config = self.parse_block_content(&block.content)
            .map_err(|e| PluginError::ParseError {
                message: e.to_string(),
                line: 0,
                column: 0,
                location: block.location.clone(),
            })?;

        // Step 2: Generate Clean Language AST
        let statements = self.generate_statements(config);

        Ok(statements)
    }

    fn validate(&self, block: &FrameworkBlock) -> PluginResult<()> {
        // Validate block structure before expansion
        if block.content.trim().is_empty() {
            return Err(PluginError::ValidationFailed {
                plugin_name: self.name().to_string(),
                message: "Block content cannot be empty".to_string(),
                location: block.location.clone(),
            });
        }

        Ok(())
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl MyPlugin {
    pub fn new() -> Self {
        Self
    }

    fn parse_block_content(&self, content: &str) -> Result<MyConfig, String> {
        // Parse DSL syntax
        todo!()
    }

    fn generate_statements(&self, config: MyConfig) -> Vec<Statement> {
        // Generate Clean Language AST
        vec![]
    }
}
```

### Step 3: Create Plugin Registry Factory

```rust
// src/lib.rs

mod my_plugin;

pub use my_plugin::MyPlugin;

use clean_language_compiler::plugins::{PluginRegistry, PluginError};

/// Create a plugin registry with all framework plugins
pub fn create_my_registry() -> Result<PluginRegistry, PluginError> {
    PluginRegistry::builder()
        .add(MyPlugin::new())
        // Add more plugins here
        .build()
}
```

---

## Plugin Registration

### Immutable Registry Pattern

The `PluginRegistry` is **immutable after creation**. This prevents plugin injection mid-compilation.

```rust
use my_framework_plugins::create_my_registry;
use clean_language_compiler::compile_with_plugins;

// Build immutable registry
let registry = create_my_registry()?;

// Use registry for compilation
let wasm = compile_with_plugins(source, file_path, &registry)?;
```

### Builder Pattern

```rust
use clean_language_compiler::plugins::PluginRegistry;
use my_framework_plugins::MyPlugin;

let registry = PluginRegistry::builder()
    .add(MyPlugin::new())
    .add(AnotherPlugin::new())
    .build()?;  // Returns Result<PluginRegistry, PluginError>
```

### Conflict Detection

The builder automatically detects conflicts:

```rust
let result = PluginRegistry::builder()
    .add(PluginA { handles: &["myblock"] })
    .add(PluginB { handles: &["myblock"] })  // Conflict!
    .build();

// Returns Err(PluginError::RegistrationConflict {
//     block_name: "myblock",
//     existing_plugin: "PluginA",
//     new_plugin: "PluginB",
// })
```

---

## Version Compatibility

### Compiler Version Constraints

Always specify compiler version in `Cargo.toml`:

```toml
[dependencies]
clean-language-compiler = { path = "../clean-language-compiler", version = "0.14.0" }
```

### Runtime Version Checking

Add version validation in your framework compiler:

```rust
use clean_language_compiler::{VERSION as COMPILER_VERSION, MIN_PLUGIN_VERSION};

const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");
const REQUIRED_COMPILER_VERSION: &str = "0.14.0";

fn check_compiler_compatibility() -> Result<()> {
    let compiler_parts: Vec<&str> = COMPILER_VERSION.split('.').collect();
    let required_parts: Vec<&str> = REQUIRED_COMPILER_VERSION.split('.').collect();

    if compiler_parts.len() < 2 || required_parts.len() < 2 {
        return Ok(());  // Allow if version parsing fails
    }

    let compiler_major: u32 = compiler_parts[0].parse()?;
    let compiler_minor: u32 = compiler_parts[1].parse()?;
    let required_major: u32 = required_parts[0].parse()?;
    let required_minor: u32 = required_parts[1].parse()?;

    if compiler_major < required_major ||
       (compiler_major == required_major && compiler_minor < required_minor) {
        bail!(
            "Compiler version incompatibility:\n  \
            Framework {} requires Clean Language Compiler >= {}\n  \
            Found compiler version: {}",
            FRAMEWORK_VERSION,
            REQUIRED_COMPILER_VERSION,
            COMPILER_VERSION
        );
    }

    Ok(())
}
```

### Semantic Versioning

Follow semantic versioning:
- **Major version**: Breaking changes to plugin API
- **Minor version**: New features, backward compatible
- **Patch version**: Bug fixes only

---

## Testing Plugins

### Unit Tests

Test individual plugin methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clean_language_compiler::plugins::FrameworkBlockBuilder;

    #[test]
    fn test_plugin_expands_simple_block() {
        let plugin = MyPlugin::new();

        let block = FrameworkBlockBuilder::new("myblock")
            .content("key: value")
            .build();

        let result = plugin.expand(&block);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert!(!statements.is_empty());
    }

    #[test]
    fn test_plugin_validates_empty_block() {
        let plugin = MyPlugin::new();

        let block = FrameworkBlockBuilder::new("myblock")
            .content("")
            .build();

        let result = plugin.validate(&block);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.name(), "my.framework");
        assert_eq!(plugin.handles(), &["myblock"]);
        assert!(!plugin.version().is_empty());
    }
}
```

### Integration Tests

Test end-to-end compilation:

```rust
// tests/integration.rs

use my_framework_plugins::create_my_registry;
use clean_language_compiler::compile_with_plugins;

#[test]
fn test_compile_with_myblock() {
    let source = r#"
        myblock:
            key: value
            another: setting

        functions:
            void start()
                print("Hello")
    "#;

    let registry = create_my_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed");

    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_invalid_myblock_syntax() {
    let source = r#"
        myblock:
            invalid syntax here!!!
    "#;

    let registry = create_my_registry().unwrap();
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_err(), "Should fail with invalid syntax");
}
```

---

## Best Practices

### 1. Error Handling

Provide clear, actionable error messages:

```rust
fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
    let config = self.parse_block_content(&block.content)
        .map_err(|e| PluginError::ParseError {
            message: format!(
                "Invalid myblock syntax: {}\nExpected format:\n  key: value",
                e
            ),
            line: 0,
            column: 0,
            location: block.location.clone(),
        })?;

    // ...
}
```

### 2. Location Tracking

Preserve source locations for better error reporting:

```rust
use clean_language_compiler::ast::SourceLocation;

fn generate_function(&self, location: &Option<SourceLocation>) -> Function {
    Function {
        name: "generated_function".to_string(),
        // ...
        location: location.clone(),  // Preserve location
    }
}
```

### 3. Idempotency

Plugin expansion should be deterministic:

```rust
// ✅ Good: Deterministic generation
fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
    let statements = self.parse_and_generate(&block.content)?;
    Ok(statements)
}

// ❌ Bad: Non-deterministic (uses randomness, timestamps, etc.)
fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
    let id = rand::random::<u64>();  // Non-deterministic!
    // ...
}
```

### 4. Minimal Dependencies

Keep plugin crates lightweight:

```toml
[dependencies]
# Required
clean-language-compiler = { version = "0.14.0" }

# Optional (only if needed)
serde = { version = "1.0", features = ["derive"] }
```

### 5. Documentation

Document your DSL syntax:

```rust
/*!
 * # MyBlock DSL
 *
 * The `myblock:` block defines...
 *
 * ## Syntax
 *
 * ```clean
 * myblock:
 *     key: value
 *     another: setting
 * ```
 *
 * ## Expansion
 *
 * This generates Clean Language code equivalent to:
 *
 * ```clean
 * functions:
 *     void __my_init()
 *         // Generated code
 * ```
 */
```

---

## Examples

### Example 1: Configuration Plugin

Transforms configuration blocks into constants:

```clean
config:
    app_name: "My App"
    version: "1.0.0"
```

Expands to:

```clean
functions:
    string getAppName()
        return "My App"

    string getVersion()
        return "1.0.0"
```

Implementation:

```rust
impl FrameworkPlugin for ConfigPlugin {
    fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
        let config: HashMap<String, String> = self.parse_config(&block.content)?;

        let mut statements = Vec::new();

        for (key, value) in config {
            let function_name = format!("get{}", capitalize(&key));

            let function = Function {
                name: function_name,
                parameters: vec![],
                return_type: Type::String,
                body: Block {
                    statements: vec![
                        Statement::Return {
                            value: Some(Expression::StringLiteral {
                                value: value,
                                location: block.location.clone(),
                            }),
                            location: block.location.clone(),
                        }
                    ],
                },
                // ...
            };

            statements.push(Statement::FunctionDeclaration(function));
        }

        Ok(statements)
    }
}
```

### Example 2: WebPlugin (Endpoints DSL)

See `frame-compiler-plugins/src/web.rs` for a complete, production example.

Key features:
- Parses HTTP routes (GET /path -> handler)
- Generates router registration function
- Handles parameter extraction
- Supports multiple HTTP methods

---

## Troubleshooting

### Common Issues

**Issue:** Plugin not being invoked
- **Solution:** Check that block name matches `handles()` exactly
- **Solution:** Verify plugin is registered in the registry

**Issue:** Compilation errors after expansion
- **Solution:** Use `debug!()` to log generated AST
- **Solution:** Test generated Clean Language code independently

**Issue:** Version conflicts
- **Solution:** Update compiler version in Cargo.toml
- **Solution:** Check version compatibility with `check_compiler_compatibility()`

**Issue:** Parser errors in expanded code
- **Solution:** Ensure generated AST has valid syntax
- **Solution:** Use AST builders instead of string concatenation

---

## Resources

- **Compiler Plugin API:** `clean-language-compiler/src/plugins/mod.rs`
- **WebPlugin Example:** `frame-compiler-plugins/src/web.rs`
- **Integration Tests:** `frame-cli/tests/frame_compiler_integration.rs`
- **AST Definitions:** `clean-language-compiler/src/ast/mod.rs`

---

## Contributing

When contributing plugins:

1. Follow the [Clean Language Specification](../../../clean-language-compiler/documentation/Clean_Language_Specification.md)
2. Add comprehensive tests
3. Document DSL syntax clearly
4. Provide error messages with examples
5. Check version compatibility
6. Submit PR with examples

---

**Last Updated:** 2025-11-22
**Compiler Version:** 0.14.0
