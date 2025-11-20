use host_bridge::HostBridge;
use serde_json::json;

#[tokio::test]
async fn test_http_bridge_creation() {
	let mut bridge = HostBridge::new();

	// Verify the HTTP bridge is accessible
	let http = bridge.http();
	assert!(std::ptr::addr_of!(*http) as usize != 0);
}

#[tokio::test]
async fn test_http_invalid_url_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"method": "GET",
		"url": "not-a-valid-url"
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "INVALID_URL");
	assert!(result["err"]["message"].as_str().unwrap().contains("Invalid URL"));
}

#[tokio::test]
async fn test_http_invalid_method_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"method": "INVALID_METHOD",
		"url": "https://example.com"
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");
	assert!(result["err"]["message"].as_str().unwrap().contains("Unsupported HTTP method"));
}

#[tokio::test]
async fn test_http_private_ip_blocked_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"method": "GET",
		"url": "http://127.0.0.1/"
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "NETWORK_FAIL");
	assert!(result["err"]["message"].as_str().unwrap().contains("private IP"));
}

#[tokio::test]
async fn test_http_unsupported_scheme_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"method": "GET",
		"url": "ftp://example.com/file.txt"
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "INVALID_URL");
	assert!(result["err"]["message"].as_str().unwrap().contains("Unsupported URL scheme"));
}

#[tokio::test]
async fn test_http_timeout_validation_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"method": "GET",
		"url": "https://example.com",
		"timeout": 400000 // Exceeds max of 300000ms
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");
	assert!(result["err"]["message"].as_str().unwrap().contains("Timeout exceeds maximum"));
}

#[tokio::test]
async fn test_http_respond_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"status": 200,
		"headers": {
			"Content-Type": "application/json"
		},
		"body": "{\"message\": \"ok\"}"
	});

	let result = bridge.call("http", "respond", params).await.unwrap();

	assert_eq!(result["ok"], true);
	assert_eq!(result["data"]["status"], 200);
	assert_eq!(result["data"]["headers"]["Content-Type"], "application/json");
}

#[tokio::test]
async fn test_http_respond_invalid_status_integration() {
	let mut bridge = HostBridge::new();

	let params = json!({
		"status": 999,
		"headers": {},
		"body": ""
	});

	let result = bridge.call("http", "respond", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");
	assert!(result["err"]["message"].as_str().unwrap().contains("Invalid HTTP status code"));
}

#[tokio::test]
async fn test_http_unknown_function_integration() {
	let mut bridge = HostBridge::new();

	let result = bridge.call("http", "unknown_function", json!({})).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "HTTP_ERROR");
	assert!(result["err"]["message"].as_str().unwrap().contains("Unknown http function"));
}

#[tokio::test]
async fn test_http_validation_error_format_integration() {
	let mut bridge = HostBridge::new();

	// Missing required fields
	let params = json!({
		"invalid": "params"
	});

	let result = bridge.call("http", "request", params).await.unwrap();

	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");
	assert!(result["err"]["message"].is_string());
	assert!(result["err"]["details"].is_object());
}
