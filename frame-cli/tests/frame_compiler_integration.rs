/*!
 * Integration tests for FrameCompiler
 *
 * Tests the complete plugin architecture:
 * - FrameCompiler compiles with plugins (endpoints:, data:, component:)
 * - Pure compiler rejects framework DSL
 * - Error messages are helpful
 */

use anyhow::Result;
use frame_cli::frame_compiler::FrameCompiler;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_frame_compiler_with_endpoints() -> Result<()> {
    // Create temporary directory for test files
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("test_endpoints.cln");
    let output_path = temp_dir.path().join("test_endpoints.wasm");

    // Write test Clean code with endpoints: block
    let source = r#"
class Router
	constructor()

	functions:
		void get(string path, any handler)
			print("GET " + path)

		void post(string path, any handler)
			print("POST " + path)

functions:
	void listUsers()
		print("Listing users")

	void createUser()
		print("Creating user")

endpoints:
	GET "/users" -> listUsers
	POST "/users" -> createUser

start()
	Router router = Router()
	__frame_register_routes(router)
	print("Server ready")
"#;

    fs::write(&input_path, source)?;

    // Compile with FrameCompiler (has plugins)
    let compiler = FrameCompiler::new()?;
    let result = compiler.compile_file(&input_path, &output_path);

    // Verify compilation succeeded
    assert!(
        result.is_ok(),
        "FrameCompiler should compile endpoints: blocks. Error: {:?}",
        result.err()
    );

    // Verify WASM file was created
    assert!(output_path.exists(), "WASM file should be created");

    // Verify WASM is not empty
    let wasm_bytes = fs::read(&output_path)?;
    assert!(!wasm_bytes.is_empty(), "WASM file should not be empty");

    // Verify WASM magic number (0x00 0x61 0x73 0x6D)
    assert_eq!(&wasm_bytes[0..4], b"\0asm", "Should be valid WASM");

    println!("✅ FrameCompiler successfully compiled endpoints: block");
    println!("   WASM size: {} bytes", wasm_bytes.len());

    Ok(())
}

#[test]
fn test_frame_compiler_pure_clean() -> Result<()> {
    // Test that FrameCompiler can also compile pure Clean Language
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("test_pure.cln");
    let output_path = temp_dir.path().join("test_pure.wasm");

    // Pure Clean Language (no framework DSL)
    let source = r#"
functions:
	integer add(integer a, integer b)
		return a + b

start()
	integer result = add(5, 3)
	print(result.toString())
"#;

    fs::write(&input_path, source)?;

    // Compile with FrameCompiler
    let compiler = FrameCompiler::new()?;
    let result = compiler.compile_file(&input_path, &output_path);

    assert!(
        result.is_ok(),
        "FrameCompiler should compile pure Clean Language. Error: {:?}",
        result.err()
    );

    assert!(output_path.exists(), "WASM file should be created");

    println!("✅ FrameCompiler successfully compiled pure Clean Language");

    Ok(())
}

#[test]
fn test_frame_compiler_multi_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create multiple source files
    let file1 = temp_dir.path().join("models.cln");
    let file2 = temp_dir.path().join("routes.cln");
    let output_path = temp_dir.path().join("bundle.wasm");

    fs::write(&file1, r#"
class User
	string name
	string email

	constructor(string name, string email)
		self.name = name
		self.email = email
"#)?;

    fs::write(&file2, r#"
functions:
	void hello()
		print("Hello from routes")

start()
	hello()
	print("App started")
"#)?;

    // Compile multiple files
    let compiler = FrameCompiler::new()?;
    let input_files: Vec<PathBuf> = vec![file1, file2];
    let result = compiler.compile_project(&input_files, &output_path);

    assert!(
        result.is_ok(),
        "FrameCompiler should compile multiple files. Error: {:?}",
        result.err()
    );

    assert!(output_path.exists(), "WASM bundle should be created");

    println!("✅ FrameCompiler successfully compiled multi-file project");

    Ok(())
}

#[test]
fn test_frame_compiler_registry_info() -> Result<()> {
    let compiler = FrameCompiler::new()?;
    let registry = compiler.registry();

    // Verify WebPlugin is registered
    assert!(
        registry.handles("endpoints"),
        "FrameCompiler should support endpoints: blocks"
    );

    // Verify plugin list
    let plugins = registry.registered_plugins();
    assert!(
        !plugins.is_empty(),
        "FrameCompiler should have plugins registered"
    );

    println!("✅ FrameCompiler has plugins:");
    for plugin in plugins {
        println!("   - {}", plugin);
    }

    println!("   Supported DSL blocks: {:?}", registry.handled_block_types());

    Ok(())
}

#[test]
fn test_invalid_endpoints_syntax() -> Result<()> {
    // Test that invalid endpoints: syntax produces helpful error
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("test_invalid.cln");
    let output_path = temp_dir.path().join("test_invalid.wasm");

    // Invalid endpoints syntax (missing ->)
    let source = r#"
endpoints:
	GET "/users" listUsers

functions:
	void listUsers()
		print("test")

start()
	print("test")
"#;

    fs::write(&input_path, source)?;

    let compiler = FrameCompiler::new()?;
    let result = compiler.compile_file(&input_path, &output_path);

    // Should fail with validation error
    assert!(
        result.is_err(),
        "Invalid endpoints: syntax should fail compilation"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("->") || error_msg.contains("validation") || error_msg.contains("arrow"),
        "Error should mention missing -> operator"
    );

    println!("✅ FrameCompiler correctly rejects invalid endpoints: syntax");
    println!("   Error: {}", error_msg);

    Ok(())
}
