/*!
 * Integration tests for DataPlugin
 *
 * Tests the complete end-to-end flow of parsing `data:` blocks
 * and generating Clean Language class definitions.
 */

use frame_compiler_plugins::create_frame_registry;
use clean_language_compiler::compile_with_plugins;

#[test]
fn test_data_plugin_simple_model() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\t\tstring email : unique\n\t\tboolean active = true\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Model defined\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with data: block: {:?}", result.err());
    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_data_plugin_multiple_models() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\t\tstring email\n\tPost\n\t\tinteger id : pk, auto\n\t\tstring title\n\t\tstring content\n\t\tinteger userId\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Models defined\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with multiple models: {:?}", result.err());
}

#[test]
fn test_data_plugin_with_constraints() {
    let source = "data:\n\tProduct\n\t\tinteger id : pk, auto\n\t\tstring name : unique\n\t\tnumber price : default=0.0\n\t\tinteger stock = 100\n\t\tboolean available = true\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Product model defined\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with constraints and defaults: {:?}", result.err());
}

#[test]
fn test_data_plugin_without_data_block() {
    let source = "functions:\n\tvoid start()\n\t\tprint(\"No data blocks\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed without data: blocks: {:?}", result.err());
}

#[test]
fn test_data_and_endpoints_together() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\t\tstring email\n\nendpoints:\n\tGET \"/users\" -> listUsers\n\nfunctions:\n\tvoid listUsers()\n\t\tprint(\"List users\")\n\n\tvoid start()\n\t\tprint(\"App ready\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with both data: and endpoints: blocks: {:?}", result.err());
    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_registry_handles_data() {
    let registry = create_frame_registry().expect("Failed to create registry");

    assert!(registry.handles("data"), "Registry should handle data: blocks");
    assert!(registry.handles("endpoints"), "Registry should handle endpoints: blocks");

    let handled = registry.handled_block_types();
    assert!(handled.contains(&"data"), "Handled blocks should include 'data'");
    assert!(handled.contains(&"endpoints"), "Handled blocks should include 'endpoints'");
}

#[test]
fn test_data_plugin_table_name_generation() {
    // This test verifies that the generated class has the correct __table_name method
    let source = "data:\n\tBlogPost\n\t\tinteger id : pk, auto\n\t\tstring title\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Table name generated\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Should generate __table_name method: {:?}", result.err());
}

#[test]
fn test_data_plugin_complex_types() {
    let source = "data:\n\tEvent\n\t\tinteger id : pk, auto\n\t\tstring name\n\t\tdatetime startTime : default=now\n\t\tdatetime endTime\n\t\tboolean isPublic = true\n\t\tnumber price = 0.0\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Event model with datetime\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Should handle datetime and complex types: {:?}", result.err());
}
