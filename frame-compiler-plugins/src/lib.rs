/*!
 * Clean Frame Compiler Plugins
 *
 * This crate contains all DSL plugins for the Clean Frame framework.
 * These plugins transform framework-specific syntax (like `endpoints:`,
 * `data:`, `component:`) into standard Clean Language code during compilation.
 *
 * ## Architecture
 *
 * - **Compiler**: Provides plugin infrastructure (`FrameworkPlugin` trait)
 * - **This Crate**: Implements Frame-specific DSL plugins
 * - **frame-cli**: Registers these plugins during compilation
 *
 * ## Available Plugins
 *
 * - `WebPlugin`: Handles `endpoints:` blocks for HTTP routing
 * - `DataPlugin`: Handles `data:` blocks for model definitions (TODO)
 * - `ComponentPlugin`: Handles `component:` blocks for UI (TODO)
 */

// Re-export plugin infrastructure from compiler
pub use clean_language_compiler::plugins::{
    FrameworkBlock, FrameworkPlugin, PluginError, PluginResult, PluginRegistry,
};

// Frame DSL plugins
mod web;
mod data;

pub use web::WebPlugin;
pub use data::DataPlugin;

// TODO: Add component plugin
// mod component;
// pub use component::ComponentPlugin;

/// Create a plugin registry with all Frame plugins registered
///
/// This is a convenience function for frame-cli to get a pre-configured
/// registry with all Frame DSL plugins.
///
/// Uses the immutable builder pattern to ensure plugins cannot be
/// injected mid-compilation.
///
/// # Example
///
/// ```ignore
/// use frame_compiler_plugins::create_frame_registry;
///
/// let registry = create_frame_registry()?;
/// // Use registry in compilation pipeline
/// ```
pub fn create_frame_registry() -> Result<PluginRegistry, PluginError> {
    PluginRegistry::builder()
        .add(WebPlugin::new())
        .add(DataPlugin::new())
        // TODO: Add ComponentPlugin when implemented
        // .add(ComponentPlugin::new())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_registry() {
        let registry = create_frame_registry().expect("Failed to create registry");

        // Verify WebPlugin is registered
        assert!(registry.handles("endpoints"));

        // Verify DataPlugin is registered
        assert!(registry.handles("data"));

        // TODO: Add check for ComponentPlugin when implemented
        // assert!(registry.handles("component"));
    }

    #[test]
    fn test_builder_pattern() {
        // Test that builder pattern works
        let registry = PluginRegistry::builder()
            .add(WebPlugin::new())
            .build()
            .expect("Failed to build registry");

        assert!(registry.handles("endpoints"));
        assert_eq!(registry.handled_block_types(), vec!["endpoints"]);
    }
}
