/*! * Integration tests for ComponentPlugin
 *
 * Tests the complete end-to-end flow of parsing `component:` blocks
 * and generating Clean Language class definitions.
 */

use frame_compiler_plugins::create_frame_registry;
use clean_language_compiler::compile_with_plugins;

#[test]
fn test_component_plugin_simple() {
    let source = "component:\n\tUserCard(user: User)\n\t\tdiv class=\"card\"\n\t\t\th2 {user.name}\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Component defined\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with component: block: {:?}", result.err());
    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_component_plugin_multiple_params() {
    let source = "component:\n\tButton(text: string, onClick: string)\n\t\tbutton onClick={onClick} {text}\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Button component\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with multiple parameters: {:?}", result.err());
}

#[test]
fn test_component_plugin_multiple_components() {
    let source = "component:\n\tUserCard(user: User)\n\t\tdiv class=\"card\"\n\t\t\th2 {user.name}\n\tButton(text: string)\n\t\tbutton {text}\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Multiple components\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with multiple components: {:?}", result.err());
}

#[test]
fn test_component_plugin_with_attributes() {
    let source = "component:\n\tCard(title: string, content: string)\n\t\tdiv class=\"card-wrapper\"\n\t\t\th3 class=\"title\" {title}\n\t\t\tp class=\"content\" {content}\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Card with attributes\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with HTML attributes: {:?}", result.err());
}

#[test]
fn test_component_and_data_together() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\ncomponent:\n\tUserCard(user: User)\n\t\tdiv class=\"user\"\n\t\t\th2 {user.name}\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Component and data\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with both component: and data: blocks: {:?}", result.err());
    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_component_endpoints_data_together() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\ncomponent:\n\tUserCard(user: User)\n\t\tdiv class=\"user-card\"\n\t\t\th2 {user.name}\n\nendpoints:\n\tGET \"/users\" -> listUsers\n\nfunctions:\n\tvoid listUsers()\n\t\tprint(\"List users\")\n\n\tvoid start()\n\t\tprint(\"All features together\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with all plugin blocks: {:?}", result.err());
    let wasm = result.unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_component_plugin_without_component_block() {
    let source = "functions:\n\tvoid start()\n\t\tprint(\"No component blocks\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed without component: blocks: {:?}", result.err());
}

#[test]
fn test_registry_handles_component() {
    let registry = create_frame_registry().expect("Failed to create registry");

    assert!(registry.handles("component"), "Registry should handle component: blocks");
    assert!(registry.handles("data"), "Registry should handle data: blocks");
    assert!(registry.handles("endpoints"), "Registry should handle endpoints: blocks");

    let handled = registry.handled_block_types();
    assert!(handled.contains(&"component"), "Handled blocks should include 'component'");
    assert!(handled.contains(&"data"), "Handled blocks should include 'data'");
    assert!(handled.contains(&"endpoints"), "Handled blocks should include 'endpoints'");
}

#[test]
fn test_component_no_params() {
    let source = "component:\n\tHeader()\n\t\theader class=\"app-header\"\n\t\t\th1 \"My App\"\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Header component\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    assert!(result.is_ok(), "Compilation should succeed with component without parameters: {:?}", result.err());
}
