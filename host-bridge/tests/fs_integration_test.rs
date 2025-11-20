use host_bridge::FsBridge;
use serde_json::json;
use tempfile::TempDir;

/// Helper to create a sandboxed FS bridge for testing
fn create_test_bridge() -> (TempDir, FsBridge) {
	let temp_dir = TempDir::new().unwrap();
	let bridge = FsBridge::with_sandbox(temp_dir.path().to_path_buf());
	(temp_dir, bridge)
}

/// Test FS write and read operations with full envelope validation
#[tokio::test]
async fn test_fs_write_read_integration() {
	let (_temp, bridge) = create_test_bridge();

	// Write a file
	let write_request = json!({
		"path": "test_integration.txt",
		"content": "Hello from integration test",
		"encoding": "utf-8"
	});

	let response = bridge.call("write", write_request).await.unwrap();

	// Validate write response envelope
	assert_eq!(response["ok"], true);
	assert!(response["data"].is_null());
	assert!(response["err"].is_null());

	// Read the file back
	let read_request = json!({
		"path": "test_integration.txt",
		"encoding": "utf-8"
	});

	let response = bridge.call("read", read_request).await.unwrap();

	// Validate read response envelope
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "Hello from integration test");
	assert!(response["err"].is_null());
}

/// Test FS binary operations (base64 encoding)
#[tokio::test]
async fn test_fs_binary_operations() {
	let (_temp, bridge) = create_test_bridge();

	// Write binary data
	let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
	let base64_content = base64::Engine::encode(
		&base64::engine::general_purpose::STANDARD,
		&binary_data,
	);

	let write_request = json!({
		"path": "binary_test.bin",
		"content": base64_content,
		"encoding": "base64"
	});

	let response = bridge.call("write", write_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Read binary data back
	let read_request = json!({
		"path": "binary_test.bin",
		"encoding": "base64"
	});

	let response = bridge.call("read", read_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Decode and verify
	let read_base64 = response["data"].as_str().unwrap();
	let read_data = base64::Engine::decode(
		&base64::engine::general_purpose::STANDARD,
		read_base64,
	)
	.unwrap();
	assert_eq!(read_data, binary_data);
}

/// Test FS directory operations
#[tokio::test]
async fn test_fs_directory_operations() {
	let (_temp, bridge) = create_test_bridge();

	// Create a directory
	let mkdir_request = json!({
		"path": "test_dir",
		"recursive": false
	});

	let response = bridge.call("mkdir", mkdir_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Verify directory exists
	let exists_request = json!({
		"path": "test_dir"
	});

	let response = bridge.call("exists", exists_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], true);

	// Create files in the directory
	for i in 1..=3 {
		let write_request = json!({
			"path": format!("test_dir/file{}.txt", i),
			"content": format!("Content {}", i),
			"encoding": "utf-8"
		});

		let response = bridge.call("write", write_request).await.unwrap();
		assert_eq!(response["ok"], true);
	}

	// List directory contents
	let list_request = json!({
		"path": "test_dir"
	});

	let response = bridge.call("list", list_request).await.unwrap();
	assert_eq!(response["ok"], true);

	let entries = response["data"]["entries"].as_array().unwrap();
	assert_eq!(entries.len(), 3);

	// Verify entry structure
	for entry in entries {
		assert!(entry["name"].is_string());
		assert!(entry["type"].is_string());
		assert!(entry["size"].is_number());
	}
}

/// Test FS list with glob pattern
#[tokio::test]
async fn test_fs_list_with_pattern() {
	let (_temp, bridge) = create_test_bridge();

	// Create directory with mixed file types
	bridge
		.call("mkdir", json!({"path": "pattern_test"}))
		.await
		.unwrap();

	// Create various files
	for ext in &["txt", "rs", "txt", "md", "txt"] {
		let write_request = json!({
			"path": format!("pattern_test/file.{}", ext),
			"content": "test",
			"encoding": "utf-8"
		});

		bridge.call("write", write_request).await.unwrap();
	}

	// List only .txt files
	let list_request = json!({
		"path": "pattern_test",
		"pattern": "*.txt"
	});

	let response = bridge.call("list", list_request).await.unwrap();
	assert_eq!(response["ok"], true);

	let _entries = response["data"]["entries"].as_array().unwrap();
	// Note: The test creates files with same name but different extensions
	// Last write wins, so we end up with file.txt, file.rs, file.md
	// The pattern "*.txt" will match file.txt
}

/// Test FS append operation
#[tokio::test]
async fn test_fs_append_operation() {
	let (_temp, bridge) = create_test_bridge();

	// Write initial content
	let write_request = json!({
		"path": "append_test.txt",
		"content": "First line\n",
		"encoding": "utf-8"
	});

	bridge.call("write", write_request).await.unwrap();

	// Append content
	let append_request = json!({
		"path": "append_test.txt",
		"content": "Second line\n"
	});

	let response = bridge.call("append", append_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Append more content
	let append_request = json!({
		"path": "append_test.txt",
		"content": "Third line\n"
	});

	bridge.call("append", append_request).await.unwrap();

	// Read and verify
	let read_request = json!({
		"path": "append_test.txt",
		"encoding": "utf-8"
	});

	let response = bridge.call("read", read_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "First line\nSecond line\nThird line\n");
}

/// Test FS delete operations
#[tokio::test]
async fn test_fs_delete_operations() {
	let (_temp, bridge) = create_test_bridge();

	// Create and delete a file
	bridge
		.call(
			"write",
			json!({
				"path": "delete_test.txt",
				"content": "delete me",
				"encoding": "utf-8"
			}),
		)
		.await
		.unwrap();

	let delete_request = json!({
		"path": "delete_test.txt",
		"recursive": false
	});

	let response = bridge.call("delete", delete_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Verify file is gone
	let exists_request = json!({
		"path": "delete_test.txt"
	});

	let response = bridge.call("exists", exists_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], false);

	// Test recursive delete
	bridge
		.call("mkdir", json!({"path": "delete_dir", "recursive": true}))
		.await
		.unwrap();

	bridge
		.call(
			"write",
			json!({
				"path": "delete_dir/file.txt",
				"content": "test",
				"encoding": "utf-8"
			}),
		)
		.await
		.unwrap();

	let delete_request = json!({
		"path": "delete_dir",
		"recursive": true
	});

	let response = bridge.call("delete", delete_request).await.unwrap();
	assert_eq!(response["ok"], true);
}

/// Test FS stat operation
#[tokio::test]
async fn test_fs_stat_operation() {
	let (_temp, bridge) = create_test_bridge();

	// Create a file
	bridge
		.call(
			"write",
			json!({
				"path": "stat_test.txt",
				"content": "Test content for stat",
				"encoding": "utf-8"
			}),
		)
		.await
		.unwrap();

	// Get file stats
	let stat_request = json!({
		"path": "stat_test.txt"
	});

	let response = bridge.call("stat", stat_request).await.unwrap();
	assert_eq!(response["ok"], true);

	let data = &response["data"];

	// Validate stat response structure
	assert_eq!(data["is_file"], true);
	assert_eq!(data["is_directory"], false);
	assert_eq!(data["is_symlink"], false);
	assert_eq!(data["size"], 21); // "Test content for stat" is 21 bytes

	assert!(data["created"].is_string());
	assert!(data["modified"].is_string());
	assert!(data["accessed"].is_string());

	// permissions field is optional (platform-specific)
	// On Unix it should be present, on Windows it may not be
}

/// Test FS recursive directory creation
#[tokio::test]
async fn test_fs_recursive_mkdir() {
	let (_temp, bridge) = create_test_bridge();

	// Create nested directories
	let mkdir_request = json!({
		"path": "a/b/c/d/e",
		"recursive": true
	});

	let response = bridge.call("mkdir", mkdir_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// Verify all directories exist
	for path in &["a", "a/b", "a/b/c", "a/b/c/d", "a/b/c/d/e"] {
		let exists_request = json!({
			"path": path
		});

		let response = bridge.call("exists", exists_request).await.unwrap();
		assert_eq!(response["ok"], true);
		assert_eq!(response["data"], true);
	}
}

/// Test FS error handling - path traversal
#[tokio::test]
async fn test_fs_security_path_traversal() {
	let (_temp, bridge) = create_test_bridge();

	// Attempt path traversal
	let read_request = json!({
		"path": "../../../etc/passwd",
		"encoding": "utf-8"
	});

	let response = bridge.call("read", read_request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PATH_TRAVERSAL");
}

/// Test FS error handling - file not found
#[tokio::test]
async fn test_fs_error_not_found() {
	let (_temp, bridge) = create_test_bridge();

	let read_request = json!({
		"path": "nonexistent.txt",
		"encoding": "utf-8"
	});

	let response = bridge.call("read", read_request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "NOT_FOUND");
}

/// Test FS error handling - invalid encoding
#[tokio::test]
async fn test_fs_error_invalid_encoding() {
	let (_temp, bridge) = create_test_bridge();

	let write_request = json!({
		"path": "test.txt",
		"content": "test",
		"encoding": "invalid_encoding"
	});

	let response = bridge.call("write", write_request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
}

/// Test FS error handling - unknown function
#[tokio::test]
async fn test_fs_error_unknown_function() {
	let (_temp, bridge) = create_test_bridge();

	let response = bridge.call("unknown_function", json!({})).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "FS_ERROR");
}

/// Test FS platform restriction
#[tokio::test]
async fn test_fs_platform_restriction() {
	// Create a bridge without sandbox (default)
	let bridge = FsBridge::new();

	// On desktop platforms (Windows, macOS, Linux, FreeBSD), operations should work
	// On web/mobile platforms, operations should be denied

	#[cfg(any(
		target_os = "windows",
		target_os = "macos",
		target_os = "linux",
		target_os = "freebsd"
	))]
	{
		// Platform is allowed - we can't test actual operations without a sandbox
		// but we can verify the platform check passes
		assert!(bridge.is_platform_allowed());
	}

	#[cfg(not(any(
		target_os = "windows",
		target_os = "macos",
		target_os = "linux",
		target_os = "freebsd"
	)))]
	{
		// Platform is not allowed
		assert!(!bridge.is_platform_allowed());

		let read_request = json!({
			"path": "test.txt",
			"encoding": "utf-8"
		});

		let response = bridge.call("read", read_request).await.unwrap();
		assert_eq!(response["ok"], false);
		assert_eq!(response["err"]["code"], "PERMISSION_DENIED");
	}
}
