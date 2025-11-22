/*!
 * Frame Compiler - Library Integration with Clean Language Compiler
 *
 * This module provides a compiler implementation that uses the Clean Language
 * compiler as a library, with Frame DSL plugins registered.
 *
 * Unlike the standalone `cln` binary which compiles pure Clean Language,
 * the Frame compiler supports framework DSL blocks:
 * - endpoints: (HTTP routing)
 * - data: (ORM models)
 * - component: (UI components)
 */

use anyhow::{Context, Result};
use clean_language_compiler::{compile_with_plugins, VERSION as COMPILER_VERSION, MIN_PLUGIN_VERSION};
use clean_language_compiler::plugins::PluginRegistry;
use frame_compiler_plugins::create_frame_registry;
use std::path::Path;
use tracing::{debug, info, warn};

/// Frame CLI version (from Cargo.toml)
const FRAME_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Required compiler version range for this Frame version
/// Frame 0.1.x requires compiler >= 0.14.0
const REQUIRED_COMPILER_VERSION: &str = "0.14.0";

/// Check compiler version compatibility
///
/// Returns an error if the compiler version is incompatible with Frame.
/// Currently uses simple version comparison (major.minor matching).
fn check_compiler_compatibility() -> Result<()> {
    // Parse versions
    let compiler_version = COMPILER_VERSION;
    let required_version = REQUIRED_COMPILER_VERSION;

    // Simple semver check: split on '.' and compare major.minor
    let compiler_parts: Vec<&str> = compiler_version.split('.').collect();
    let required_parts: Vec<&str> = required_version.split('.').collect();

    if compiler_parts.len() < 2 || required_parts.len() < 2 {
        warn!("Unable to parse version numbers for compatibility check");
        return Ok(()); // Allow compilation if version parsing fails
    }

    let compiler_major: u32 = compiler_parts[0].parse()
        .unwrap_or_else(|_| { warn!("Invalid major version in compiler"); 0 });
    let compiler_minor: u32 = compiler_parts[1].parse()
        .unwrap_or_else(|_| { warn!("Invalid minor version in compiler"); 0 });

    let required_major: u32 = required_parts[0].parse()
        .unwrap_or_else(|_| { warn!("Invalid major version in requirement"); 0 });
    let required_minor: u32 = required_parts[1].parse()
        .unwrap_or_else(|_| { warn!("Invalid minor version in requirement"); 0 });

    // Check compatibility: compiler must be >= required version
    if compiler_major < required_major || (compiler_major == required_major && compiler_minor < required_minor) {
        anyhow::bail!(
            "Compiler version incompatibility:\n  \
            Frame {} requires Clean Language Compiler >= {}\n  \
            Found compiler version: {}\n  \
            Please upgrade the compiler or use a compatible Frame version.",
            FRAME_VERSION,
            REQUIRED_COMPILER_VERSION,
            COMPILER_VERSION
        );
    }

    debug!("Version check passed: Frame {} with Compiler {}", FRAME_VERSION, COMPILER_VERSION);
    Ok(())
}

/// Frame compiler with full DSL plugin support
pub struct FrameCompiler {
    /// Plugin registry with all Frame DSL handlers
    registry: PluginRegistry,
}

impl FrameCompiler {
    /// Create a new Frame compiler with all DSL plugins registered
    ///
    /// # Returns
    /// * `Ok(FrameCompiler)` - Compiler with WebPlugin, DataPlugin, ComponentPlugin
    /// * `Err` - If plugin registration fails or version incompatibility detected
    pub fn new() -> Result<Self> {
        info!("Initializing Frame compiler with DSL plugins");

        // Check compiler version compatibility first
        check_compiler_compatibility()
            .context("Compiler version compatibility check failed")?;

        let registry = create_frame_registry()
            .context("Failed to create Frame plugin registry")?;

        debug!("Registered plugins: {:?}", registry.registered_plugins());
        debug!("Supported DSL blocks: {:?}", registry.handled_block_types());

        Ok(Self { registry })
    }

    /// Compile a Clean Language file with Frame DSL support
    ///
    /// This compiles using the frame-compiler-plugins registry, enabling:
    /// - `endpoints:` blocks for HTTP routing
    /// - `data:` blocks for ORM models (when implemented)
    /// - `component:` blocks for UI (when implemented)
    ///
    /// # Arguments
    /// * `input` - Path to .cln source file
    /// * `output` - Path for .wasm output file
    ///
    /// # Returns
    /// * `Ok(())` - Compilation successful
    /// * `Err` - Compilation or I/O errors
    ///
    /// # Example
    /// ```ignore
    /// let compiler = FrameCompiler::new()?;
    /// compiler.compile_file("app.cln", "app.wasm")?;
    /// ```
    pub fn compile_file(&self, input: &Path, output: &Path) -> Result<()> {
        info!("Compiling {} -> {} (with Frame plugins)", input.display(), output.display());

        // Read source file
        let source = std::fs::read_to_string(input)
            .context(format!("Failed to read source file: {}", input.display()))?;

        // Compile with Frame plugins
        let file_path_str = input.to_str().unwrap_or("unknown.cln");
        let wasm_bytes = compile_with_plugins(&source, file_path_str, &self.registry)
            .map_err(|errors| {
                // Format compilation errors for user
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| format!("  - {}", e))
                    .collect();

                anyhow::anyhow!(
                    "Compilation failed with {} error(s):\n{}",
                    errors.len(),
                    error_messages.join("\n")
                )
            })?;

        // Ensure output directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        // Write WASM output
        let wasm_size = wasm_bytes.len();
        std::fs::write(output, wasm_bytes)
            .context(format!("Failed to write output file: {}", output.display()))?;

        info!("✅ Compilation successful: {} bytes", wasm_size);
        Ok(())
    }

    /// Compile multiple .cln files into a single WASM bundle
    ///
    /// Files are concatenated in source order before compilation.
    /// This enables multi-file Frame applications.
    ///
    /// # Arguments
    /// * `input_files` - List of .cln source files
    /// * `output` - Path for .wasm output file
    ///
    /// # Returns
    /// * `Ok(())` - Compilation successful
    /// * `Err` - Compilation or I/O errors
    pub fn compile_project(&self, input_files: &[impl AsRef<Path>], output: &Path) -> Result<()> {
        if input_files.is_empty() {
            anyhow::bail!("No input files provided");
        }

        info!("Compiling {} files into {}", input_files.len(), output.display());

        // Single file: direct compilation
        if input_files.len() == 1 {
            return self.compile_file(input_files[0].as_ref(), output);
        }

        // Multi-file: bundle sources
        let mut bundled_source = String::new();
        bundled_source.push_str("// Frame multi-file bundle\n");
        bundled_source.push_str(&format!("// Files: {}\n\n", input_files.len()));

        for (idx, file) in input_files.iter().enumerate() {
            let file_path = file.as_ref();
            debug!("  Bundling {}/{}: {}", idx + 1, input_files.len(), file_path.display());

            bundled_source.push_str(&format!("// ===== File {}: {} =====\n", idx + 1, file_path.display()));

            let content = std::fs::read_to_string(file_path)
                .context(format!("Failed to read {}", file_path.display()))?;

            bundled_source.push_str(&content);
            bundled_source.push_str("\n\n");
        }

        // Compile bundled source
        let wasm_bytes = compile_with_plugins(&bundled_source, "bundle.cln", &self.registry)
            .map_err(|errors| {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| format!("  - {}", e))
                    .collect();

                anyhow::anyhow!(
                    "Compilation failed with {} error(s):\n{}",
                    errors.len(),
                    error_messages.join("\n")
                )
            })?;

        // Ensure output directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        // Write WASM output
        let wasm_size = wasm_bytes.len();
        std::fs::write(output, wasm_bytes)
            .context(format!("Failed to write output file: {}", output.display()))?;

        info!("✅ Multi-file compilation successful: {} bytes", wasm_size);
        Ok(())
    }

    /// Get the plugin registry (for inspection/testing)
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_compiler_creation() {
        let compiler = FrameCompiler::new().expect("Failed to create compiler");

        // Verify Frame plugins are registered
        assert!(compiler.registry().handles("endpoints"));

        // TODO: When DataPlugin and ComponentPlugin are implemented
        // assert!(compiler.registry().handles("data"));
        // assert!(compiler.registry().handles("component"));
    }

    #[test]
    fn test_plugin_registry() {
        let compiler = FrameCompiler::new().expect("Failed to create compiler");
        let registry = compiler.registry();

        // Should have at least WebPlugin
        assert!(!registry.registered_plugins().is_empty());
        assert!(registry.handled_block_types().contains(&"endpoints"));
    }

    #[test]
    fn test_version_compatibility_check() {
        // This test verifies that version checking is enabled
        // The actual check happens at FrameCompiler::new()
        // Since we're using the real compiler version, this should pass
        let result = check_compiler_compatibility();
        assert!(result.is_ok(), "Version compatibility check should pass with current compiler version");
    }

    #[test]
    fn test_version_info_available() {
        // Verify version constants are available
        assert!(!FRAME_VERSION.is_empty());
        assert!(!COMPILER_VERSION.is_empty());
        assert!(!REQUIRED_COMPILER_VERSION.is_empty());

        // Verify versions are valid semver format
        assert!(FRAME_VERSION.split('.').count() >= 2);
        assert!(COMPILER_VERSION.split('.').count() >= 2);
        assert!(REQUIRED_COMPILER_VERSION.split('.').count() >= 2);
    }
}
