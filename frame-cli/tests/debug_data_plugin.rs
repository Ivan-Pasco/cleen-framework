use frame_compiler_plugins::create_frame_registry;
use clean_language_compiler::compile_with_plugins;

#[test]
fn test_debug_data_compilation() {
    let source = "data:\n\tUser\n\t\tinteger id : pk, auto\n\t\tstring name\n\nfunctions:\n\tvoid start()\n\t\tprint(\"Test\")";

    let registry = create_frame_registry().expect("Failed to create registry");
    let result = compile_with_plugins(source, "test.cln", &registry);

    match result {
        Ok(_) => println!("SUCCESS: Compilation succeeded"),
        Err(errors) => {
            println!("ERRORS found:");
            for error in &errors {
                println!("  - {:?}", error);
            }
            panic!("Compilation failed with {} errors", errors.len());
        }
    }
}
