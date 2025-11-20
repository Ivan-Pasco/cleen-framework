/// Integration tests for the LOG bridge module
/// Tests the complete workflow through the bridge envelope system

use host_bridge::{HostBridge, LogBridge, LogLevel, LogConfig};
use serde_json::json;

#[tokio::test]
async fn test_log_through_host_bridge() {
	let mut bridge = HostBridge::new();

	// Test debug log
	let result = bridge.call("log", "debug", json!({
		"message": "Debug message through host bridge"
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], json!(null));

	// Test info log
	let result = bridge.call("log", "info", json!({
		"message": "Info message through host bridge"
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Test warn log
	let result = bridge.call("log", "warn", json!({
		"message": "Warning message through host bridge"
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Test error log
	let result = bridge.call("log", "error", json!({
		"message": "Error message through host bridge"
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_with_structured_data() {
	let mut bridge = HostBridge::new();

	let result = bridge.call("log", "info", json!({
		"message": "User action logged",
		"data": {
			"userId": 12345,
			"action": "login",
			"timestamp": 1700000000,
			"ip": "192.168.1.1",
			"userAgent": "Mozilla/5.0"
		}
	})).await;

	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_error_envelopes() {
	let mut bridge = HostBridge::new();

	// Test empty message error
	let result = bridge.call("log", "info", json!({
		"message": ""
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test invalid params
	let result = bridge.call("log", "info", json!(123)).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test unknown function
	let result = bridge.call("log", "unknown", json!({
		"message": "test"
	})).await;
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "LOG_ERROR");
}

#[tokio::test]
async fn test_log_level_configuration() {
	let bridge = LogBridge::new();

	// Default should allow all levels
	assert_eq!(bridge.get_min_level(), LogLevel::Debug);

	// Change to only warn and above
	bridge.set_min_level(LogLevel::Warn);
	assert_eq!(bridge.get_min_level(), LogLevel::Warn);

	// Debug and info should still succeed but won't output
	let result = bridge.call("debug", json!({
		"message": "This won't be logged"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	let result = bridge.call("info", json!({
		"message": "This won't be logged either"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Warn and error should output
	let result = bridge.call("warn", json!({
		"message": "This will be logged"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_direct_api() {
	let bridge = LogBridge::new();

	// Test direct methods (these should not panic)
	bridge.debug("Direct debug message");
	bridge.info("Direct info message");
	bridge.warn("Direct warning message");
	bridge.error("Direct error message");

	// Test with data
	bridge.debug_with_data("Debug with data", json!({"key": "value"}));
	bridge.info_with_data("Info with data", json!({"key": "value"}));
	bridge.warn_with_data("Warning with data", json!({"key": "value"}));
	bridge.error_with_data("Error with data", json!({"key": "value"}));
}

#[tokio::test]
async fn test_log_configuration_options() {
	let config = LogConfig {
		min_level: LogLevel::Info,
		json_output: true,
		include_timestamp: true,
		max_message_size: 1024,
	};

	let bridge = LogBridge::with_config(config);

	// Verify config was applied
	assert_eq!(bridge.get_min_level(), LogLevel::Info);

	// Test that debug is filtered
	let result = bridge.call("debug", json!({
		"message": "Filtered debug message"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Test that info passes through
	let result = bridge.call("info", json!({
		"message": "Info message"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_message_size_limits() {
	let bridge = LogBridge::new();
	bridge.set_max_message_size(100);

	// Within limit
	let result = bridge.call("info", json!({
		"message": "Short message"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Exceeds limit
	let long_message = "a".repeat(200);
	let result = bridge.call("info", json!({
		"message": long_message
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("exceeds maximum size"));
}

#[tokio::test]
async fn test_log_legacy_format_compatibility() {
	let bridge = LogBridge::new();

	// Old format: just a string
	let result = bridge.call("info", json!("Simple string message"));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// New format: object with message field
	let result = bridge.call("info", json!({
		"message": "Object format message"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_complex_data_structures() {
	let bridge = LogBridge::new();

	let result = bridge.call("info", json!({
		"message": "Complex nested data",
		"data": {
			"user": {
				"id": 123,
				"profile": {
					"name": "John Doe",
					"email": "john@example.com",
					"roles": ["admin", "user"],
					"permissions": {
						"read": true,
						"write": true,
						"delete": false
					}
				}
			},
			"request": {
				"method": "POST",
				"path": "/api/users",
				"headers": {
					"Content-Type": "application/json",
					"Authorization": "Bearer token"
				}
			},
			"metadata": {
				"timestamp": 1700000000,
				"duration_ms": 45,
				"success": true
			}
		}
	}));

	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_unicode_and_special_chars() {
	let bridge = LogBridge::new();

	// Unicode characters
	let result = bridge.call("info", json!({
		"message": "Hello 世界 🌍 Привет мир",
		"data": {
			"emoji": "🚀🔥💡",
			"unicode": "日本語 한글 中文"
		}
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);

	// Special characters
	let result = bridge.call("info", json!({
		"message": "Special: \"quotes\" 'apostrophes' \n newlines \t tabs"
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], true);
}

#[tokio::test]
async fn test_log_concurrent_operations() {
	use std::sync::Arc;
	use tokio::task;

	let bridge = Arc::new(LogBridge::new());
	let mut handles = vec![];

	// Spawn multiple concurrent logging tasks
	for i in 0..20 {
		let bridge_clone = Arc::clone(&bridge);
		let handle = task::spawn(async move {
			let result = bridge_clone.call("info", json!({
				"message": format!("Concurrent log {}", i),
				"data": {
					"thread_id": i,
					"timestamp": chrono::Utc::now().timestamp()
				}
			}));
			assert!(result.is_ok());
		});
		handles.push(handle);
	}

	// Wait for all tasks to complete
	for handle in handles {
		handle.await.unwrap();
	}
}

#[tokio::test]
async fn test_log_all_levels_with_data() {
	let bridge = LogBridge::new();
	let test_data = json!({
		"requestId": "req-12345",
		"userId": 789,
		"action": "test"
	});

	// Debug
	let result = bridge.call("debug", json!({
		"message": "Debug level log",
		"data": test_data.clone()
	}));
	assert!(result.is_ok());
	assert_eq!(result.unwrap()["ok"], true);

	// Info
	let result = bridge.call("info", json!({
		"message": "Info level log",
		"data": test_data.clone()
	}));
	assert!(result.is_ok());
	assert_eq!(result.unwrap()["ok"], true);

	// Warn
	let result = bridge.call("warn", json!({
		"message": "Warning level log",
		"data": test_data.clone()
	}));
	assert!(result.is_ok());
	assert_eq!(result.unwrap()["ok"], true);

	// Error
	let result = bridge.call("error", json!({
		"message": "Error level log",
		"data": test_data.clone()
	}));
	assert!(result.is_ok());
	assert_eq!(result.unwrap()["ok"], true);
}

#[tokio::test]
async fn test_log_validation_edge_cases() {
	let bridge = LogBridge::new();

	// Missing message field
	let result = bridge.call("info", json!({
		"data": {"key": "value"}
	}));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Null params
	let result = bridge.call("info", json!(null));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);

	// Array params
	let result = bridge.call("info", json!(["array", "params"]));
	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(response["ok"], false);
}

#[tokio::test]
async fn test_log_runtime_configuration_changes() {
	let bridge = LogBridge::new();

	// Start with JSON output
	bridge.set_json_output(true);
	let result = bridge.call("info", json!({"message": "JSON output"}));
	assert!(result.is_ok());

	// Switch to plain text
	bridge.set_json_output(false);
	let result = bridge.call("info", json!({"message": "Plain text output"}));
	assert!(result.is_ok());

	// Toggle timestamp
	bridge.set_include_timestamp(false);
	let result = bridge.call("info", json!({"message": "No timestamp"}));
	assert!(result.is_ok());

	bridge.set_include_timestamp(true);
	let result = bridge.call("info", json!({"message": "With timestamp"}));
	assert!(result.is_ok());
}

#[test]
fn test_log_level_conversion() {
	// Test from_str
	assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
	assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
	assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
	assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
	assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
	assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warn));
	assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
	assert_eq!(LogLevel::from_str("INVALID"), None);

	// Test as_str
	assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
	assert_eq!(LogLevel::Info.as_str(), "INFO");
	assert_eq!(LogLevel::Warn.as_str(), "WARN");
	assert_eq!(LogLevel::Error.as_str(), "ERROR");

	// Test ordering
	assert!(LogLevel::Debug < LogLevel::Info);
	assert!(LogLevel::Info < LogLevel::Warn);
	assert!(LogLevel::Warn < LogLevel::Error);
}
