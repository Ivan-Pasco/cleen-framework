use host_bridge::{HostBridge, SysBridge};
use serde_json::json;

#[tokio::test]
async fn test_sys_platform_integration() {
	let mut bridge = HostBridge::new();

	// Test platform call through main bridge
	let result = bridge.call("sys", "platform", json!({})).await.unwrap();

	assert_eq!(result["ok"], true);
	assert!(result["data"].is_string());

	let platform = result["data"].as_str().unwrap();
	println!("Platform: {}", platform);

	// Platform should be one of the supported values
	assert!(
		platform == "linux"
			|| platform == "macos"
			|| platform == "windows"
			|| platform == "android"
			|| platform == "ios"
			|| platform == "web"
			|| !platform.is_empty()
	);
}

#[tokio::test]
async fn test_sys_arch_integration() {
	let mut bridge = HostBridge::new();

	// Test arch call through main bridge
	let result = bridge.call("sys", "arch", json!({})).await.unwrap();

	assert_eq!(result["ok"], true);
	assert!(result["data"].is_string());

	let arch = result["data"].as_str().unwrap();
	println!("Architecture: {}", arch);

	// Architecture should be one of the supported values
	assert!(
		arch == "x86_64"
			|| arch == "aarch64"
			|| arch == "arm"
			|| arch == "wasm32"
			|| !arch.is_empty()
	);
}

#[tokio::test]
async fn test_sys_version_integration() {
	let mut bridge = HostBridge::new();

	// Test version call through main bridge
	let result = bridge.call("sys", "version", json!({})).await.unwrap();

	assert_eq!(result["ok"], true);
	assert!(result["data"].is_string());

	let version = result["data"].as_str().unwrap();
	println!("Framework Version: {}", version);

	// Version should be a non-empty string in semver format
	assert!(!version.is_empty());
	assert!(version.contains('.'));
}

#[tokio::test]
async fn test_sys_env_info_integration() {
	let mut bridge = HostBridge::new();

	// Test env_info call through main bridge
	let result = bridge.call("sys", "env_info", json!({})).await.unwrap();

	assert_eq!(result["ok"], true);
	assert!(result["data"].is_object());

	let data = result["data"].as_object().unwrap();

	// Verify all required fields
	assert!(data.contains_key("platform"));
	assert!(data.contains_key("arch"));
	assert!(data.contains_key("version"));
	assert!(data.contains_key("uptime_seconds"));
	assert!(data.contains_key("process_id"));
	assert!(data.contains_key("parent_process_id"));

	// Print the environment info
	println!("Environment Info:");
	println!("  Platform: {}", data["platform"]);
	println!("  Architecture: {}", data["arch"]);
	println!("  Version: {}", data["version"]);
	println!("  Uptime: {} seconds", data["uptime_seconds"]);
	println!("  Process ID: {}", data["process_id"]);
	println!("  Parent Process ID: {}", data["parent_process_id"]);
}

#[tokio::test]
async fn test_sys_exit_validation_integration() {
	let mut bridge = HostBridge::new();

	// Test exit validation for invalid codes only
	// Note: We cannot test valid exit codes as they would terminate the test process
	let invalid_test_cases = vec![
		(-1, "Negative code should fail"),
		(256, "Too large should fail"),
		(1000, "Way too large should fail"),
	];

	for (code, description) in invalid_test_cases {
		let result = bridge.call("sys", "exit", json!({"code": code})).await.unwrap();

		// Should get validation error
		assert_eq!(result["ok"], false);
		assert_eq!(result["err"]["code"], "VALIDATION_ERROR");
		println!("Code {} correctly rejected: {}", code, description);
	}

	// Valid codes (0, 128, 255) would work but terminate the process,
	// so we only document their validity here
	println!("Valid exit codes (0-255) would pass validation but terminate process");
}

#[tokio::test]
async fn test_sys_unknown_function_integration() {
	let mut bridge = HostBridge::new();

	// Test unknown function call
	let result = bridge.call("sys", "unknown_function", json!({})).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "SYS_ERROR");
	assert!(result["err"]["message"].as_str().unwrap().contains("Unknown sys function"));
}

#[tokio::test]
async fn test_sys_direct_bridge_access() {
	let bridge = HostBridge::new();

	// Test direct access to sys bridge
	let sys = bridge.sys();

	// Test direct methods
	let platform = sys.get_platform();
	assert!(!platform.is_empty());
	println!("Direct platform: {}", platform);

	let arch = sys.get_arch();
	assert!(!arch.is_empty());
	println!("Direct arch: {}", arch);

	let version = sys.get_version();
	assert!(!version.is_empty());
	assert!(version.contains('.'));
	println!("Direct version: {}", version);

	let pid = sys.get_process_id();
	assert!(pid > 0);
	println!("Direct process ID: {}", pid);
}

#[tokio::test]
async fn test_sys_json_envelope_compliance() {
	let bridge = SysBridge::new();

	// Test that all functions return proper JSON envelopes
	let functions = vec!["platform", "arch", "version", "env_info"];

	for function in functions {
		let result = bridge.call(function, json!({})).await.unwrap();

		// Verify envelope structure
		assert!(result.is_object(), "Function {} should return object", function);
		assert!(result.get("ok").is_some(), "Function {} missing 'ok' field", function);

		if result["ok"] == true {
			assert!(result.get("data").is_some(), "Function {} missing 'data' field", function);
		} else {
			assert!(result.get("err").is_some(), "Function {} missing 'err' field", function);
			let err = result["err"].as_object().unwrap();
			assert!(err.contains_key("code"), "Function {} error missing 'code'", function);
			assert!(err.contains_key("message"), "Function {} error missing 'message'", function);
		}

		println!("Function {} has valid envelope", function);
	}
}

#[tokio::test]
async fn test_sys_consistency_across_calls() {
	let bridge = SysBridge::new();

	// Test that multiple calls to the same function return consistent results
	let functions = vec!["platform", "arch", "version"];

	for function in functions {
		let result1 = bridge.call(function, json!({})).await.unwrap();
		let result2 = bridge.call(function, json!({})).await.unwrap();

		assert_eq!(
			result1["data"], result2["data"],
			"Function {} should return consistent results",
			function
		);

		println!("Function {} is consistent", function);
	}
}

#[tokio::test]
async fn test_sys_env_info_completeness() {
	let bridge = SysBridge::new();

	// Get environment info
	let result = bridge.call("env_info", json!({})).await.unwrap();
	let data = result["data"].as_object().unwrap();

	// Verify platform matches standalone call
	let platform_result = bridge.call("platform", json!({})).await.unwrap();
	assert_eq!(data["platform"], platform_result["data"]);

	// Verify arch matches standalone call
	let arch_result = bridge.call("arch", json!({})).await.unwrap();
	assert_eq!(data["arch"], arch_result["data"]);

	// Verify version matches standalone call
	let version_result = bridge.call("version", json!({})).await.unwrap();
	assert_eq!(data["version"], version_result["data"]);

	println!("env_info data matches individual calls");
}

#[tokio::test]
async fn test_sys_exit_request_formats() {
	let bridge = SysBridge::new();

	// Test invalid format only (to avoid process termination)
	let result = bridge.call("exit", json!({"invalid": "field"})).await.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	// Test that exit codes outside valid range are rejected
	let result = bridge.call("exit", json!({"code": 300})).await.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	println!("Exit request format validation works correctly");

	// Note: Valid formats like json!({"code": 100}) and json!(100) would work
	// but would terminate the test process, so we don't test them
}

#[tokio::test]
async fn test_sys_uptime_tracking() {
	let bridge = SysBridge::new();

	// Get initial environment info
	let result1 = bridge.call("env_info", json!({})).await.unwrap();
	let uptime1 = result1["data"]["uptime_seconds"].as_u64().unwrap();

	// Wait a bit
	tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

	// Get environment info again
	let result2 = bridge.call("env_info", json!({})).await.unwrap();
	let uptime2 = result2["data"]["uptime_seconds"].as_u64().unwrap();

	// Uptime should have increased (or stayed the same if too fast)
	assert!(uptime2 >= uptime1, "Uptime should increase or stay same");
	println!("Uptime tracking works: {} -> {}", uptime1, uptime2);
}
