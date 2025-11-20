/// Additional tests to achieve 100% coverage for ENV bridge
/// This test file specifically targets uncovered code paths identified through coverage analysis

use host_bridge::EnvBridge;
use serde_json::json;
use std::env;

#[tokio::test]
async fn test_get_env_legacy_string_format() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test variable
	env::set_var("LEGACY_FORMAT_TEST", "legacy_value");

	// Test with legacy string format (direct "name" field)
	let params = json!({
		"name": "LEGACY_FORMAT_TEST"
	});

	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "legacy_value");

	// Cleanup
	env::remove_var("LEGACY_FORMAT_TEST");
}

#[tokio::test]
async fn test_get_env_invalid_json_format() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with completely invalid JSON format (no "name" field)
	let params = json!({
		"invalid_field": "value"
	});

	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Invalid request format"));
}

#[tokio::test]
async fn test_has_env_legacy_string_format() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test variable
	env::set_var("HAS_LEGACY_FORMAT_TEST", "value");

	// Test with legacy string format
	let params = json!({
		"name": "HAS_LEGACY_FORMAT_TEST"
	});

	let response = bridge.call("has", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], true);

	// Cleanup
	env::remove_var("HAS_LEGACY_FORMAT_TEST");
}

#[tokio::test]
async fn test_has_env_invalid_json_format() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with invalid JSON format (no "name" field)
	let params = json!({
		"wrong_field": "value"
	});

	let response = bridge.call("has", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Invalid request format"));
}

#[tokio::test]
async fn test_set_env_invalid_json_format() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with missing "value" field
	let params = json!({
		"name": "TEST_VAR"
	});

	let response = bridge.call("set", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Invalid request format"));
}

#[tokio::test]
async fn test_set_allowlist_method() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test variables
	env::set_var("ALLOWED_VAR_1", "value1");
	env::set_var("ALLOWED_VAR_2", "value2");
	env::set_var("DENIED_VAR_1", "denied");

	// Dynamically set allowlist
	bridge.set_allowlist(vec![
		"ALLOWED_VAR_1".to_string(),
		"ALLOWED_VAR_2".to_string(),
	]);

	// Test allowed variable
	let params = json!({ "name": "ALLOWED_VAR_1" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "value1");

	// Test denied variable
	let params = json!({ "name": "DENIED_VAR_1" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Cleanup
	env::remove_var("ALLOWED_VAR_1");
	env::remove_var("ALLOWED_VAR_2");
	env::remove_var("DENIED_VAR_1");
}

#[tokio::test]
async fn test_clear_allowlist_method() {
	let bridge = EnvBridge::with_allowlist(vec!["ONLY_THIS".to_string()]);

	// Set up test variables
	env::set_var("ONLY_THIS", "allowed");
	env::set_var("NOT_THIS", "denied");

	// Initially, only ONLY_THIS is accessible
	let params = json!({ "name": "NOT_THIS" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Clear allowlist - now all should be accessible (except denylisted)
	bridge.clear_allowlist();

	let params = json!({ "name": "NOT_THIS" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "denied");

	// Cleanup
	env::remove_var("ONLY_THIS");
	env::remove_var("NOT_THIS");
}

#[tokio::test]
async fn test_add_and_remove_from_denylist() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test variable
	env::set_var("SENSITIVE_DATA", "secret");

	// Initially accessible
	let params = json!({ "name": "SENSITIVE_DATA" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "secret");

	// Add to denylist
	bridge.add_to_denylist("SENSITIVE_DATA".to_string());

	// Now should be denied
	let params = json!({ "name": "SENSITIVE_DATA" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Remove from denylist
	bridge.remove_from_denylist("SENSITIVE_DATA");

	// Now accessible again
	let params = json!({ "name": "SENSITIVE_DATA" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "secret");

	// Cleanup
	env::remove_var("SENSITIVE_DATA");
}

#[tokio::test]
async fn test_clear_denylist_method() {
	let bridge = EnvBridge::new(); // Has default denylist

	// Set up sensitive variable
	env::set_var("AWS_SECRET_ACCESS_KEY", "sensitive");

	// Should be denied by default
	let params = json!({ "name": "AWS_SECRET_ACCESS_KEY" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Clear denylist
	bridge.clear_denylist();

	// Now should be accessible
	let params = json!({ "name": "AWS_SECRET_ACCESS_KEY" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "sensitive");

	// Cleanup
	env::remove_var("AWS_SECRET_ACCESS_KEY");
}

#[test]
fn test_direct_get_with_invalid_name() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with invalid names
	assert_eq!(bridge.get(""), None);
	assert_eq!(bridge.get("123INVALID"), None);
	assert_eq!(bridge.get("INVALID-NAME"), None);
	assert_eq!(bridge.get("INVALID NAME"), None);
}

#[test]
fn test_direct_get_with_denied_var() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_DIRECT".to_string());

	env::set_var("DENIED_DIRECT", "value");

	// Should return None for denied variable
	assert_eq!(bridge.get("DENIED_DIRECT"), None);

	// Cleanup
	env::remove_var("DENIED_DIRECT");
}

#[test]
fn test_direct_has_with_invalid_name() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with invalid names
	assert!(!bridge.has(""));
	assert!(!bridge.has("123INVALID"));
	assert!(!bridge.has("INVALID-NAME"));
}

#[test]
fn test_direct_has_with_denied_var() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_HAS".to_string());

	env::set_var("DENIED_HAS", "value");

	// Should return false for denied variable
	assert!(!bridge.has("DENIED_HAS"));

	// Cleanup
	env::remove_var("DENIED_HAS");
}

#[test]
fn test_direct_set_with_invalid_name() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with invalid names
	assert!(bridge.set("", "value").is_err());
	assert!(bridge.set("123INVALID", "value").is_err());
	assert!(bridge.set("INVALID-NAME", "value").is_err());
}

#[test]
fn test_direct_set_with_denied_var() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_SET".to_string());

	// Should fail to set denied variable
	assert!(bridge.set("DENIED_SET", "value").is_err());
}

#[test]
fn test_direct_set_permission_denied() {
	let bridge = EnvBridge::new(); // Default doesn't allow set

	// Should fail because set is not allowed
	let result = bridge.set("TEST_VAR", "value");
	assert!(result.is_err());
	assert!(result.unwrap_err().to_string().contains("not permitted"));
}

#[test]
fn test_direct_all_with_allowlist() {
	let bridge = EnvBridge::with_allowlist(vec![
		"ALLOWED_ALL_1".to_string(),
		"ALLOWED_ALL_2".to_string(),
	]);

	// Set up test variables
	env::set_var("ALLOWED_ALL_1", "value1");
	env::set_var("ALLOWED_ALL_2", "value2");
	env::set_var("DENIED_ALL", "denied");

	let all_vars = bridge.all();

	// Should contain allowed variables
	assert!(all_vars.contains_key("ALLOWED_ALL_1"));
	assert!(all_vars.contains_key("ALLOWED_ALL_2"));

	// Should NOT contain denied variables
	assert!(!all_vars.contains_key("DENIED_ALL"));

	// Cleanup
	env::remove_var("ALLOWED_ALL_1");
	env::remove_var("ALLOWED_ALL_2");
	env::remove_var("DENIED_ALL");
}

#[test]
fn test_direct_all_with_denylist() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_IN_ALL".to_string());

	// Set up test variables
	env::set_var("NORMAL_VAR_ALL", "value");
	env::set_var("DENIED_IN_ALL", "secret");

	let all_vars = bridge.all();

	// Should contain normal variables
	assert!(all_vars.contains_key("NORMAL_VAR_ALL"));

	// Should NOT contain denied variables
	assert!(!all_vars.contains_key("DENIED_IN_ALL"));

	// Cleanup
	env::remove_var("NORMAL_VAR_ALL");
	env::remove_var("DENIED_IN_ALL");
}

#[tokio::test]
async fn test_set_env_with_invalid_name() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with invalid name
	let params = json!({
		"name": "123INVALID",
		"value": "test"
	});

	let response = bridge.call("set", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Invalid environment variable name"));
}

#[tokio::test]
async fn test_set_env_with_denied_var() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_SET_VAR".to_string());

	// Try to set denied variable
	let params = json!({
		"name": "DENIED_SET_VAR",
		"value": "test"
	});

	let response = bridge.call("set", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");
}

#[tokio::test]
async fn test_has_env_with_invalid_name() {
	let bridge = EnvBridge::new_unrestricted();

	// Test with empty name
	let params = json!({
		"name": ""
	});

	let response = bridge.call("has", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_has_env_with_denied_var() {
	let bridge = EnvBridge::new_unrestricted();
	bridge.add_to_denylist("DENIED_HAS_VAR".to_string());

	env::set_var("DENIED_HAS_VAR", "value");

	// Try to check denied variable
	let params = json!({
		"name": "DENIED_HAS_VAR"
	});

	let response = bridge.call("has", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Cleanup
	env::remove_var("DENIED_HAS_VAR");
}

#[tokio::test]
async fn test_list_with_mixed_permissions() {
	let bridge = EnvBridge::with_allowlist(vec![
		"LIST_ALLOWED_1".to_string(),
		"LIST_ALLOWED_2".to_string(),
	]);

	// Set up various test variables
	env::set_var("LIST_ALLOWED_1", "value1");
	env::set_var("LIST_ALLOWED_2", "value2");
	env::set_var("LIST_DENIED_1", "denied1");
	env::set_var("LIST_DENIED_2", "denied2");

	// Test list
	let response = bridge.call("list", json!({})).await.unwrap();
	assert_eq!(response["ok"], true);

	let data = response["data"].as_object().unwrap();

	// Should only contain allowed variables
	assert!(data.contains_key("LIST_ALLOWED_1"));
	assert!(data.contains_key("LIST_ALLOWED_2"));
	assert!(!data.contains_key("LIST_DENIED_1"));
	assert!(!data.contains_key("LIST_DENIED_2"));

	// Cleanup
	env::remove_var("LIST_ALLOWED_1");
	env::remove_var("LIST_ALLOWED_2");
	env::remove_var("LIST_DENIED_1");
	env::remove_var("LIST_DENIED_2");
}

#[test]
fn test_default_denylist_coverage() {
	let bridge = EnvBridge::new();

	// Test all default denylisted variables
	let sensitive_vars = vec![
		"AWS_SECRET_ACCESS_KEY",
		"AWS_SESSION_TOKEN",
		"PRIVATE_KEY",
		"ENCRYPTION_KEY",
		"MASTER_KEY",
		"SSH_AUTH_SOCK",
		"GPG_AGENT_INFO",
	];

	for var in sensitive_vars {
		env::set_var(var, "sensitive");

		// Should be denied
		assert_eq!(bridge.get(var), None);
		assert!(!bridge.has(var));

		// Cleanup
		env::remove_var(var);
	}
}

#[test]
fn test_default_trait_implementation() {
	let bridge = EnvBridge::default();

	// Should behave like EnvBridge::new()
	// Test that it has default denylist
	env::set_var("AWS_SECRET_ACCESS_KEY", "secret");

	assert_eq!(bridge.get("AWS_SECRET_ACCESS_KEY"), None);

	// Cleanup
	env::remove_var("AWS_SECRET_ACCESS_KEY");
}

#[tokio::test]
async fn test_all_function_aliases() {
	let bridge = EnvBridge::new_unrestricted();

	env::set_var("TEST_ALIAS_VAR", "alias_value");

	// Test "list" function
	let response1 = bridge.call("list", json!({})).await.unwrap();
	assert_eq!(response1["ok"], true);

	// Test "all" function (should behave the same)
	let response2 = bridge.call("all", json!({})).await.unwrap();
	assert_eq!(response2["ok"], true);

	// Both should return the same data structure
	assert!(response1["data"].is_object());
	assert!(response2["data"].is_object());

	// Both should contain our test variable
	assert_eq!(response1["data"]["TEST_ALIAS_VAR"], "alias_value");
	assert_eq!(response2["data"]["TEST_ALIAS_VAR"], "alias_value");

	// Cleanup
	env::remove_var("TEST_ALIAS_VAR");
}

#[test]
fn test_valid_names_comprehensive() {
	let bridge = EnvBridge::new_unrestricted();

	// Test various valid name patterns by attempting to set them
	env::set_var("A", "value");
	assert!(bridge.get("A").is_some());

	env::set_var("ABC", "value");
	assert!(bridge.get("ABC").is_some());

	env::set_var("_", "value");
	assert!(bridge.get("_").is_some());

	env::set_var("_ABC", "value");
	assert!(bridge.get("_ABC").is_some());

	env::set_var("__", "value");
	assert!(bridge.get("__").is_some());

	env::set_var("A_B_C", "value");
	assert!(bridge.get("A_B_C").is_some());

	env::set_var("ABC123", "value");
	assert!(bridge.get("ABC123").is_some());

	env::set_var("_123", "value");
	assert!(bridge.get("_123").is_some());

	env::set_var("VAR_NAME_123", "value");
	assert!(bridge.get("VAR_NAME_123").is_some());

	// Test invalid name patterns - should return None even if env var exists
	// These are tested through the API which validates names
	assert_eq!(bridge.get(""), None);
	assert_eq!(bridge.get("1"), None);
	assert_eq!(bridge.get("123"), None);
	assert_eq!(bridge.get("1ABC"), None);
	assert_eq!(bridge.get("ABC-DEF"), None);
	assert_eq!(bridge.get("ABC DEF"), None);
	assert_eq!(bridge.get("ABC.DEF"), None);
	assert_eq!(bridge.get("ABC@DEF"), None);
	assert_eq!(bridge.get("ABC$DEF"), None);
	assert_eq!(bridge.get("ABC#DEF"), None);

	// Cleanup
	env::remove_var("A");
	env::remove_var("ABC");
	env::remove_var("_");
	env::remove_var("_ABC");
	env::remove_var("__");
	env::remove_var("A_B_C");
	env::remove_var("ABC123");
	env::remove_var("_123");
	env::remove_var("VAR_NAME_123");
}

#[tokio::test]
async fn test_concurrent_access() {
	use std::sync::Arc;
	use tokio::task;

	let bridge = Arc::new(EnvBridge::new_unrestricted());

	// Set up test variable
	env::set_var("CONCURRENT_TEST", "concurrent_value");

	// Spawn multiple concurrent tasks
	let mut handles = vec![];

	for i in 0..10 {
		let bridge_clone = Arc::clone(&bridge);
		let handle = task::spawn(async move {
			let params = json!({ "name": "CONCURRENT_TEST" });
			let response = bridge_clone.call("get", params).await.unwrap();
			assert_eq!(response["ok"], true);
			assert_eq!(response["data"], "concurrent_value");
			i
		});
		handles.push(handle);
	}

	// Wait for all tasks to complete
	for handle in handles {
		handle.await.unwrap();
	}

	// Cleanup
	env::remove_var("CONCURRENT_TEST");
}

#[tokio::test]
async fn test_concurrent_allowlist_modification() {
	use std::sync::Arc;
	use tokio::task;

	let bridge = Arc::new(EnvBridge::new_unrestricted());

	// Test concurrent modifications to allowlist
	let mut handles = vec![];

	for i in 0..5 {
		let bridge_clone = Arc::clone(&bridge);
		let handle = task::spawn(async move {
			bridge_clone.set_allowlist(vec![
				format!("VAR_{}", i),
				format!("VAR_{}_2", i),
			]);
		});
		handles.push(handle);
	}

	// Wait for all tasks to complete
	for handle in handles {
		handle.await.unwrap();
	}

	// Bridge should still be functional
	bridge.clear_allowlist();
	env::set_var("TEST_AFTER_CONCURRENT", "value");
	let params = json!({ "name": "TEST_AFTER_CONCURRENT" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);

	// Cleanup
	env::remove_var("TEST_AFTER_CONCURRENT");
}

#[tokio::test]
async fn test_get_with_allowlist_and_denylist() {
	let bridge = EnvBridge::with_allowlist(vec![
		"ALLOWED_1".to_string(),
		"ALLOWED_2".to_string(),
		"BLOCKED".to_string(),
	]);

	// Add one of the allowed vars to denylist (denylist takes precedence)
	bridge.add_to_denylist("BLOCKED".to_string());

	env::set_var("ALLOWED_1", "value1");
	env::set_var("ALLOWED_2", "value2");
	env::set_var("BLOCKED", "blocked_value");

	// ALLOWED_1 should work
	let params = json!({ "name": "ALLOWED_1" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], true);

	// BLOCKED should be denied (denylist takes precedence)
	let params = json!({ "name": "BLOCKED" });
	let response = bridge.call("get", params).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Cleanup
	env::remove_var("ALLOWED_1");
	env::remove_var("ALLOWED_2");
	env::remove_var("BLOCKED");
}
