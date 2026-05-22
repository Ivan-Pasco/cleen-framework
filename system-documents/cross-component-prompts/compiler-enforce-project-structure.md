# Compiler — Parse and Enforce Plugin `[enforcement]` Rules

Component: clean-language-compiler
Issue Type: feature
Priority: high
Description: Framework plugins now declare `[enforcement]` sections in their plugin.toml files. The compiler needs to parse these sections and apply the rules during compilation. Additionally, the compiler should parse the existing `[paths]` section (which it currently ignores) and replace the hardcoded `detect_plugins_from_path()` with manifest-driven detection.
Context: Discovered while building cleanlanguage.dev — the framework defines folder ownership and DSL blocks in plugin.toml but the compiler ignores all of it. Projects can use raw `_http_route()` calls, place files anywhere, and skip `endpoints:` blocks entirely. The framework plugins now include `[enforcement]` metadata; the compiler needs to read and act on it.

---

## New TOML Sections to Parse

### `[enforcement]` (NEW — all 5 framework plugins now include this)

```toml
[enforcement]
severity = "warn"  # "warn" = diagnostic warning, "error" = compile error

# Raw bridge functions that should not be called directly
restricted_functions = [
  { name = "_http_route", use_instead = "endpoints:", message = "Use 'endpoints:' block to define routes" },
]

# Blocks required when file is in a specific folder
required_blocks = [
  { folder = "app/backend/api/", block = "endpoints", message = "API files should use 'endpoints:' block" },
]

# Blocks that should only appear in specific folders
block_folder_rules = [
  { block = "endpoints", allowed_in = ["app/backend/", "app/server/"], message = "endpoints: block should be in app/backend/" },
]
```

### `[paths]` (EXISTS in plugin.toml — currently ignored by compiler)

```toml
[paths]
owns = ["app/backend/", "app/backend/api/"]
auto_create = true
patterns = ["*.cln"]
implicit_import = true
```

---

## Required Compiler Changes

### Change 1: Add structs to `src/plugins/plugin_abi.rs`

Add these structs and include them in `PluginManifest`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPaths {
    #[serde(default)]
    pub owns: Vec<String>,
    #[serde(default)]
    pub auto_create: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub implicit_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginEnforcement {
    #[serde(default = "default_warn")]
    pub severity: String,
    #[serde(default)]
    pub restricted_functions: Vec<RestrictedFunction>,
    #[serde(default)]
    pub required_blocks: Vec<RequiredBlock>,
    #[serde(default)]
    pub block_folder_rules: Vec<BlockFolderRule>,
}

fn default_warn() -> String { "warn".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedFunction {
    pub name: String,
    pub use_instead: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredBlock {
    pub folder: String,
    pub block: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockFolderRule {
    pub block: String,
    pub allowed_in: Vec<String>,
    #[serde(default)]
    pub message: String,
}
```

Add to `PluginManifest`:
```rust
pub struct PluginManifest {
    // ... existing fields ...
    #[serde(default)]
    pub paths: PluginPaths,
    #[serde(default)]
    pub enforcement: PluginEnforcement,
}
```

### Change 2: Replace hardcoded `detect_plugins_from_path()` in `src/lib.rs` (~line 965-1013)

The current function hardcodes path patterns like `/api/`, `/data/`, `/canvas/`. Replace with manifest-driven detection:

```rust
fn detect_plugins_from_manifests(
    file_path: &Path,
    discovered_plugins: &[PluginManifest],
) -> Vec<String> {
    let path_str = file_path.to_string_lossy();
    let mut plugins = Vec::new();

    for manifest in discovered_plugins {
        if manifest.paths.implicit_import {
            for owned_path in &manifest.paths.owns {
                if path_str.contains(owned_path) {
                    plugins.push(manifest.plugin.name.clone());
                    break;
                }
            }
        }
    }

    plugins.dedup();
    plugins
}
```

This requires loading all plugin manifests during discovery (which `PluginDiscovery` already does) and passing them to the detection function.

### Change 3: New file `src/plugins/enforcement.rs`

Create a new module that runs enforcement checks after parsing (Stage 2) but before or during plugin expansion (Stage 2.5):

```rust
use crate::ast::{Statement, Expression, Program};
use crate::plugins::plugin_abi::{PluginEnforcement, RestrictedFunction, RequiredBlock, BlockFolderRule};

pub struct EnforcementResult {
    pub warnings: Vec<EnforcementDiagnostic>,
    pub errors: Vec<EnforcementDiagnostic>,
}

pub struct EnforcementDiagnostic {
    pub message: String,
    pub suggestion: String,
    pub file_path: String,
    pub line: Option<usize>,
    pub plugin: String,
}

pub fn enforce_rules(
    program: &Program,
    file_path: &str,
    enforcement_rules: &[(String, PluginEnforcement)],  // (plugin_name, rules)
) -> EnforcementResult {
    let mut result = EnforcementResult { warnings: vec![], errors: vec![] };

    for (plugin_name, rules) in enforcement_rules {
        // 1. Check restricted function calls
        check_restricted_functions(program, file_path, plugin_name, rules, &mut result);

        // 2. Check required blocks
        check_required_blocks(program, file_path, plugin_name, rules, &mut result);

        // 3. Check block folder rules
        check_block_folder_rules(program, file_path, plugin_name, rules, &mut result);
    }

    result
}

fn check_restricted_functions(
    program: &Program,
    file_path: &str,
    plugin_name: &str,
    rules: &PluginEnforcement,
    result: &mut EnforcementResult,
) {
    // Walk AST looking for FunctionCall expressions
    // If call name matches any restricted_functions[].name, emit diagnostic
    // Use rules.severity to decide warning vs error
    for restricted in &rules.restricted_functions {
        // Walk statements and expressions looking for calls to restricted.name
        // When found:
        let diagnostic = EnforcementDiagnostic {
            message: restricted.message.clone(),
            suggestion: format!("Use {} instead", restricted.use_instead),
            file_path: file_path.to_string(),
            line: None, // Fill from AST location
            plugin: plugin_name.to_string(),
        };
        if rules.severity == "error" {
            result.errors.push(diagnostic);
        } else {
            result.warnings.push(diagnostic);
        }
    }
}

fn check_required_blocks(
    program: &Program,
    file_path: &str,
    plugin_name: &str,
    rules: &PluginEnforcement,
    result: &mut EnforcementResult,
) {
    for required in &rules.required_blocks {
        // Check if file_path contains required.folder
        if file_path.contains(&required.folder) {
            // Check if AST contains a FrameworkBlock with name == required.block
            let has_block = program.statements.iter().any(|stmt| {
                matches!(stmt, Statement::FrameworkBlock { name, .. } if name == &required.block)
            });
            if !has_block {
                let diagnostic = EnforcementDiagnostic {
                    message: required.message.clone(),
                    suggestion: format!("Add a '{}:' block to this file", required.block),
                    file_path: file_path.to_string(),
                    line: None,
                    plugin: plugin_name.to_string(),
                };
                if rules.severity == "error" {
                    result.errors.push(diagnostic);
                } else {
                    result.warnings.push(diagnostic);
                }
            }
        }
    }
}

fn check_block_folder_rules(
    program: &Program,
    file_path: &str,
    plugin_name: &str,
    rules: &PluginEnforcement,
    result: &mut EnforcementResult,
) {
    for rule in &rules.block_folder_rules {
        // Find any FrameworkBlock with name == rule.block
        let has_block = program.statements.iter().any(|stmt| {
            matches!(stmt, Statement::FrameworkBlock { name, .. } if name == &rule.block)
        });
        if has_block {
            // Check if file_path matches any of rule.allowed_in
            let in_allowed = rule.allowed_in.iter().any(|path| file_path.contains(path));
            if !in_allowed {
                let diagnostic = EnforcementDiagnostic {
                    message: rule.message.clone(),
                    suggestion: format!("Move this file to one of: {}", rule.allowed_in.join(", ")),
                    file_path: file_path.to_string(),
                    line: None,
                    plugin: plugin_name.to_string(),
                };
                if rules.severity == "error" {
                    result.errors.push(diagnostic);
                } else {
                    result.warnings.push(diagnostic);
                }
            }
        }
    }
}
```

### Change 4: Wire enforcement into compilation pipeline

In the compilation functions (e.g., `compile_with_plugins_and_opt_level` in `src/lib.rs` or `MultiFileCompiler`), after parsing and before/during plugin expansion:

```rust
// After parsing, before plugin expansion
if let Some(registry) = &plugin_registry {
    let enforcement_rules: Vec<(String, PluginEnforcement)> = registry
        .loaded_manifests()
        .iter()
        .filter(|(_, m)| !m.enforcement.restricted_functions.is_empty()
                      || !m.enforcement.required_blocks.is_empty()
                      || !m.enforcement.block_folder_rules.is_empty())
        .map(|(name, m)| (name.clone(), m.enforcement.clone()))
        .collect();

    let result = enforcement::enforce_rules(&program, file_path, &enforcement_rules);

    for warning in &result.warnings {
        eprintln!("warning[{}]: {} ({})", warning.plugin, warning.message, warning.suggestion);
    }
    for error in &result.errors {
        // Add to compilation errors
        errors.push(CompileError::enforcement(error));
    }
}
```

### Change 5: Store manifests in PluginRegistry

The `PluginRegistry` needs to retain the full `PluginManifest` for each loaded plugin (currently it only keeps handlers and bridge functions). Add:

```rust
pub struct PluginRegistry {
    // ... existing fields ...
    manifests: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn loaded_manifests(&self) -> &HashMap<String, PluginManifest> {
        &self.manifests
    }
}
```

### Change 6: Auto-create folders (low priority)

When compiling with plugins, if `paths.auto_create == true`, create the `paths.owns` directories relative to the project root:

```rust
for manifest in manifests.values() {
    if manifest.paths.auto_create {
        for owned_folder in &manifest.paths.owns {
            let folder_path = project_root.join(owned_folder);
            if !folder_path.exists() {
                fs::create_dir_all(&folder_path)?;
            }
        }
    }
}
```

### Change 7: Add `mod enforcement;` to `src/plugins/mod.rs`

```rust
pub mod enforcement;
```

---

## Files Affected

| File | Change |
|------|--------|
| `src/plugins/plugin_abi.rs` | Add `PluginPaths`, `PluginEnforcement`, and sub-structs; add fields to `PluginManifest` |
| `src/plugins/enforcement.rs` | **NEW** — Enforcement validation pass |
| `src/plugins/mod.rs` | Add `pub mod enforcement;` |
| `src/plugins/registry.rs` | Store manifests; add `loaded_manifests()` method |
| `src/lib.rs` | Replace `detect_plugins_from_path()` with manifest-driven version; wire enforcement into compilation |
| `src/compilation/multi_file_compiler.rs` | Wire enforcement into multi-file compilation |

---

## Testing Strategy

1. **Unit tests for enforcement.rs**: Create programs with restricted function calls, verify warnings/errors are emitted
2. **Integration test**: Compile a file using raw `_http_route()` with frame.server plugin loaded, verify warning is emitted
3. **Backward compat test**: Compile existing examples to verify they don't get false positives (they use `endpoints:` blocks properly)
4. **Severity test**: Set severity to "error" in a test manifest, verify compilation fails for restricted functions

---

## Migration Notes

- Phase 1 (current): All plugins use `severity = "warn"` — warnings only, no code rejected
- Phase 2 (future): Framework can change individual plugins to `severity = "error"` to enforce conventions
- The `[enforcement]` section already exists in all 5 framework plugin.toml files — this prompt is about making the compiler read and act on it
