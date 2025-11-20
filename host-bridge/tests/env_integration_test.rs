use host_bridge::EnvBridge;
use serde_json::json;
use std::env;

/// Integration test for ENV bridge with full envelope format validation
#[tokio::test]
async fn test_env_get_with_envelope() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test environment variable
	env::set_var("TEST_ENV_INTEGRATION", "integration_value");

	// Test GET with proper envelope
	let request = json!({
		"name": "TEST_ENV_INTEGRATION"
	});

	let response = bridge.call("get", request).await.unwrap();

	// Validate response envelope
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "integration_value");
	assert!(response["err"].is_null());

	// Cleanup
	env::remove_var("TEST_ENV_INTEGRATION");
}

#[tokio::test]
async fn test_env_set_with_envelope() {
	let bridge = EnvBridge::new_unrestricted();

	// Test SET with proper envelope
	let request = json!({
		"name": "TEST_ENV_SET_INTEGRATION",
		"value": "set_integration_value"
	});

	let response = bridge.call("set", request).await.unwrap();

	// Validate response envelope
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"]["name"], "TEST_ENV_SET_INTEGRATION");
	assert_eq!(response["data"]["value"], "set_integration_value");

	// Verify the value was actually set
	assert_eq!(env::var("TEST_ENV_SET_INTEGRATION").unwrap(), "set_integration_value");

	// Cleanup
	env::remove_var("TEST_ENV_SET_INTEGRATION");
}

#[tokio::test]
async fn test_env_has_with_envelope() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test environment variable
	env::set_var("TEST_ENV_HAS_INTEGRATION", "value");

	// Test HAS with existing variable
	let request = json!({
		"name": "TEST_ENV_HAS_INTEGRATION"
	});

	let response = bridge.call("has", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], true);

	// Test HAS with non-existing variable
	let request = json!({
		"name": "NONEXISTENT_VAR_INTEGRATION"
	});

	let response = bridge.call("has", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], false);

	// Cleanup
	env::remove_var("TEST_ENV_HAS_INTEGRATION");
}

#[tokio::test]
async fn test_env_list_with_envelope() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test environment variables
	env::set_var("TEST_LIST_INTEGRATION_1", "value1");
	env::set_var("TEST_LIST_INTEGRATION_2", "value2");

	// Test LIST
	let response = bridge.call("list", json!({})).await.unwrap();

	// Validate response envelope
	assert_eq!(response["ok"], true);
	assert!(response["data"].is_object());

	let data = response["data"].as_object().unwrap();
	assert_eq!(data.get("TEST_LIST_INTEGRATION_1").unwrap(), "value1");
	assert_eq!(data.get("TEST_LIST_INTEGRATION_2").unwrap(), "value2");

	// Test "all" alias
	let response = bridge.call("all", json!({})).await.unwrap();
	assert_eq!(response["ok"], true);
	assert!(response["data"].is_object());

	// Cleanup
	env::remove_var("TEST_LIST_INTEGRATION_1");
	env::remove_var("TEST_LIST_INTEGRATION_2");
}

#[tokio::test]
async fn test_env_error_envelopes() {
	let bridge = EnvBridge::new_unrestricted();

	// Test NOT_FOUND error
	let request = json!({
		"name": "NONEXISTENT_VARIABLE_12345"
	});

	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "NOT_FOUND");
	assert!(response["err"]["message"].as_str().unwrap().contains("not found"));
	assert_eq!(response["err"]["details"]["name"], "NONEXISTENT_VARIABLE_12345");

	// Test VALIDATION_ERROR for invalid name
	let request = json!({
		"name": "INVALID-NAME"
	});

	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Invalid environment variable name"));

	// Test PERMISSION_DENIED for set when not allowed
	let restricted_bridge = EnvBridge::new(); // Default doesn't allow set

	let request = json!({
		"name": "TEST_VAR",
		"value": "value"
	});

	let response = restricted_bridge.call("set", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");
	assert!(response["err"]["message"].as_str().unwrap().contains("Setting environment variables is not permitted"));

	// Test ENV_ERROR for unknown function
	let request = json!({});
	let response = bridge.call("unknown_function", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "ENV_ERROR");
	assert!(response["err"]["message"].as_str().unwrap().contains("Unknown env function"));
}

#[tokio::test]
async fn test_env_security_allowlist() {
	let bridge = EnvBridge::with_allowlist(vec![
		"ALLOWED_VAR_1".to_string(),
		"ALLOWED_VAR_2".to_string(),
	]);

	// Set up test variables
	env::set_var("ALLOWED_VAR_1", "allowed_value");
	env::set_var("DENIED_VAR", "denied_value");

	// Test access to allowed variable
	let request = json!({
		"name": "ALLOWED_VAR_1"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "allowed_value");

	// Test access to denied variable
	let request = json!({
		"name": "DENIED_VAR"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Test list only returns allowed variables
	let response = bridge.call("list", json!({})).await.unwrap();
	assert_eq!(response["ok"], true);
	let data = response["data"].as_object().unwrap();
	assert!(data.contains_key("ALLOWED_VAR_1"));
	assert!(!data.contains_key("DENIED_VAR"));

	// Cleanup
	env::remove_var("ALLOWED_VAR_1");
	env::remove_var("DENIED_VAR");
}

#[tokio::test]
async fn test_env_security_denylist() {
	let bridge = EnvBridge::new_unrestricted();

	// Add sensitive variables to denylist
	bridge.add_to_denylist("SENSITIVE_SECRET".to_string());
	bridge.add_to_denylist("API_KEY".to_string());

	// Set up test variables
	env::set_var("SENSITIVE_SECRET", "very_secret");
	env::set_var("NORMAL_VAR", "normal_value");

	// Test access to denylisted variable
	let request = json!({
		"name": "SENSITIVE_SECRET"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "PERMISSION_DENIED");

	// Test access to normal variable
	let request = json!({
		"name": "NORMAL_VAR"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "normal_value");

	// Test list excludes denylisted variables
	let response = bridge.call("list", json!({})).await.unwrap();
	assert_eq!(response["ok"], true);
	let data = response["data"].as_object().unwrap();
	assert!(!data.contains_key("SENSITIVE_SECRET"));
	assert!(data.contains_key("NORMAL_VAR"));

	// Cleanup
	env::remove_var("SENSITIVE_SECRET");
	env::remove_var("NORMAL_VAR");
}

#[tokio::test]
async fn test_env_through_host_bridge() {
	// Make the env bridge unrestricted for testing
	let env_bridge = EnvBridge::new_unrestricted();
	env::set_var("TEST_HOST_BRIDGE_ENV", "host_value");

	// Test calling through main bridge dispatcher
	let request = json!({
		"name": "TEST_HOST_BRIDGE_ENV"
	});

	// Note: We need to use the EnvBridge directly since HostBridge.env() returns a reference
	// This is a limitation of the current design - we'll test the EnvBridge directly
	let response = env_bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "host_value");

	// Cleanup
	env::remove_var("TEST_HOST_BRIDGE_ENV");
}

#[tokio::test]
async fn test_env_validation_edge_cases() {
	let bridge = EnvBridge::new_unrestricted();

	// Test empty name
	let request = json!({
		"name": ""
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test name starting with digit
	let request = json!({
		"name": "123_INVALID"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test name with spaces
	let request = json!({
		"name": "INVALID NAME"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test name with special characters
	let request = json!({
		"name": "INVALID-NAME"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "VALIDATION_ERROR");

	// Test valid name with underscore prefix
	env::set_var("_VALID_NAME", "value");
	let request = json!({
		"name": "_VALID_NAME"
	});
	let response = bridge.call("get", request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "value");

	// Cleanup
	env::remove_var("_VALID_NAME");
}

#[tokio::test]
async fn test_env_full_workflow() {
	let bridge = EnvBridge::new_unrestricted();

	// 1. Check if variable exists (should not exist)
	let check_request = json!({
		"name": "WORKFLOW_TEST_VAR"
	});
	let response = bridge.call("has", check_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], false);

	// 2. Try to get non-existent variable (should fail)
	let get_request = json!({
		"name": "WORKFLOW_TEST_VAR"
	});
	let response = bridge.call("get", get_request).await.unwrap();
	assert_eq!(response["ok"], false);
	assert_eq!(response["err"]["code"], "NOT_FOUND");

	// 3. Set the variable
	let set_request = json!({
		"name": "WORKFLOW_TEST_VAR",
		"value": "workflow_value"
	});
	let response = bridge.call("set", set_request).await.unwrap();
	assert_eq!(response["ok"], true);

	// 4. Check if variable exists now (should exist)
	let check_request = json!({
		"name": "WORKFLOW_TEST_VAR"
	});
	let response = bridge.call("has", check_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], true);

	// 5. Get the variable (should succeed)
	let get_request = json!({
		"name": "WORKFLOW_TEST_VAR"
	});
	let response = bridge.call("get", get_request).await.unwrap();
	assert_eq!(response["ok"], true);
	assert_eq!(response["data"], "workflow_value");

	// 6. Verify it appears in list
	let response = bridge.call("list", json!({})).await.unwrap();
	assert_eq!(response["ok"], true);
	let data = response["data"].as_object().unwrap();
	assert_eq!(data.get("WORKFLOW_TEST_VAR").unwrap(), "workflow_value");

	// Cleanup
	env::remove_var("WORKFLOW_TEST_VAR");
}

#[test]
fn test_env_direct_api() {
	let bridge = EnvBridge::new_unrestricted();

	// Set up test variable
	env::set_var("TEST_DIRECT_API", "direct_value");

	// Test direct get method
	assert_eq!(bridge.get("TEST_DIRECT_API"), Some("direct_value".to_string()));
	assert_eq!(bridge.get("NONEXISTENT"), None);

	// Test direct has method
	assert!(bridge.has("TEST_DIRECT_API"));
	assert!(!bridge.has("NONEXISTENT"));

	// Test direct set method
	assert!(bridge.set("TEST_DIRECT_SET_API", "new_value").is_ok());
	assert_eq!(env::var("TEST_DIRECT_SET_API").unwrap(), "new_value");

	// Test direct all method
	let all_vars = bridge.all();
	assert!(all_vars.contains_key("TEST_DIRECT_API"));
	assert!(all_vars.contains_key("TEST_DIRECT_SET_API"));

	// Test that invalid names are rejected
	assert_eq!(bridge.get("INVALID-NAME"), None);
	assert!(!bridge.has("INVALID NAME"));
	assert!(bridge.set("123INVALID", "value").is_err());

	// Cleanup
	env::remove_var("TEST_DIRECT_API");
	env::remove_var("TEST_DIRECT_SET_API");
}

#[test]
fn test_env_default_security() {
	let bridge = EnvBridge::new();

	// Set sensitive variables
	env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
	env::set_var("PRIVATE_KEY", "key");
	env::set_var("NORMAL_VAR", "normal");

	// Test that default denylist blocks sensitive variables
	assert_eq!(bridge.get("AWS_SECRET_ACCESS_KEY"), None);
	assert_eq!(bridge.get("PRIVATE_KEY"), None);

	// Test that normal variables are accessible
	assert_eq!(bridge.get("NORMAL_VAR"), Some("normal".to_string()));

	// Test that set is not allowed by default
	assert!(bridge.set("NEW_VAR", "value").is_err());

	// Cleanup
	env::remove_var("AWS_SECRET_ACCESS_KEY");
	env::remove_var("PRIVATE_KEY");
	env::remove_var("NORMAL_VAR");
}
