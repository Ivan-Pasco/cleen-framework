use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// HTTP response representation for Clean Language applications
///
/// This type is returned from WASM functions and represents the HTTP response
/// to be sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
	/// HTTP status code (200, 404, 500, etc.)
	pub status: u16,

	/// Response headers
	pub headers: HashMap<String, String>,

	/// Response body as string
	pub body: String,
}

impl Response {
	/// Create a new response with the given status code
	pub fn new(status: u16) -> Self {
		Self {
			status,
			headers: HashMap::new(),
			body: String::new(),
		}
	}

	/// Set a header
	pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.insert(key.into(), value.into());
		self
	}

	/// Set the body
	pub fn body(mut self, body: impl Into<String>) -> Self {
		self.body = body.into();
		self
	}

	/// Set content type header
	pub fn content_type(self, content_type: impl Into<String>) -> Self {
		self.header("Content-Type", content_type)
	}
}

/// Response builder helpers

/// Create a JSON response (200 OK)
pub fn json<T: Serialize>(data: T) -> Response {
	let body = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

	Response::new(200)
		.content_type("application/json")
		.body(body)
}

/// Create a JSON response with custom status code
pub fn json_with_status<T: Serialize>(status: u16, data: T) -> Response {
	let body = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

	Response::new(status)
		.content_type("application/json")
		.body(body)
}

/// Create an HTML response (200 OK)
pub fn html(content: impl Into<String>) -> Response {
	Response::new(200)
		.content_type("text/html; charset=utf-8")
		.body(content)
}

/// Create a plain text response (200 OK)
pub fn text(content: impl Into<String>) -> Response {
	Response::new(200)
		.content_type("text/plain; charset=utf-8")
		.body(content)
}

/// Create a redirect response (302 Found)
pub fn redirect(url: impl Into<String>) -> Response {
	Response::new(302)
		.header("Location", url)
		.body("")
}

/// Create a permanent redirect response (301 Moved Permanently)
pub fn redirect_permanent(url: impl Into<String>) -> Response {
	Response::new(301)
		.header("Location", url)
		.body("")
}

/// Create a 404 Not Found response
pub fn not_found() -> Response {
	json_with_status(404, serde_json::json!({
		"error": "Not Found",
		"message": "The requested resource was not found"
	}))
}

/// Create a 404 Not Found response with custom message
pub fn not_found_with_message(message: impl Into<String>) -> Response {
	json_with_status(404, serde_json::json!({
		"error": "Not Found",
		"message": message.into()
	}))
}

/// Create a 400 Bad Request response
pub fn bad_request(message: impl Into<String>) -> Response {
	json_with_status(400, serde_json::json!({
		"error": "Bad Request",
		"message": message.into()
	}))
}

/// Create a 401 Unauthorized response
pub fn unauthorized() -> Response {
	json_with_status(401, serde_json::json!({
		"error": "Unauthorized",
		"message": "Authentication required"
	}))
}

/// Create a 401 Unauthorized response with custom message
pub fn unauthorized_with_message(message: impl Into<String>) -> Response {
	json_with_status(401, serde_json::json!({
		"error": "Unauthorized",
		"message": message.into()
	}))
}

/// Create a 403 Forbidden response
pub fn forbidden() -> Response {
	json_with_status(403, serde_json::json!({
		"error": "Forbidden",
		"message": "You do not have permission to access this resource"
	}))
}

/// Create a 403 Forbidden response with custom message
pub fn forbidden_with_message(message: impl Into<String>) -> Response {
	json_with_status(403, serde_json::json!({
		"error": "Forbidden",
		"message": message.into()
	}))
}

/// Create a 500 Internal Server Error response
pub fn internal_error(message: impl Into<String>) -> Response {
	json_with_status(500, serde_json::json!({
		"error": "Internal Server Error",
		"message": message.into()
	}))
}

/// Create a 422 Unprocessable Entity response (validation errors)
pub fn validation_error(errors: Value) -> Response {
	json_with_status(422, serde_json::json!({
		"error": "Validation Error",
		"errors": errors
	}))
}

/// Create a 204 No Content response
pub fn no_content() -> Response {
	Response::new(204)
}

/// Create a custom response with status code and JSON body
pub fn custom<T: Serialize>(status: u16, data: T) -> Response {
	json_with_status(status, data)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_json_response() {
		let resp = json(serde_json::json!({"message": "Hello"}));
		assert_eq!(resp.status, 200);
		assert_eq!(resp.headers.get("Content-Type"), Some(&"application/json".to_string()));
		assert!(resp.body.contains("Hello"));
	}

	#[test]
	fn test_json_with_status() {
		let resp = json_with_status(201, serde_json::json!({"created": true}));
		assert_eq!(resp.status, 201);
		assert_eq!(resp.headers.get("Content-Type"), Some(&"application/json".to_string()));
	}

	#[test]
	fn test_html_response() {
		let resp = html("<h1>Hello</h1>");
		assert_eq!(resp.status, 200);
		assert_eq!(resp.headers.get("Content-Type"), Some(&"text/html; charset=utf-8".to_string()));
		assert_eq!(resp.body, "<h1>Hello</h1>");
	}

	#[test]
	fn test_text_response() {
		let resp = text("Plain text");
		assert_eq!(resp.status, 200);
		assert_eq!(resp.headers.get("Content-Type"), Some(&"text/plain; charset=utf-8".to_string()));
		assert_eq!(resp.body, "Plain text");
	}

	#[test]
	fn test_redirect() {
		let resp = redirect("/login");
		assert_eq!(resp.status, 302);
		assert_eq!(resp.headers.get("Location"), Some(&"/login".to_string()));
	}

	#[test]
	fn test_redirect_permanent() {
		let resp = redirect_permanent("/new-page");
		assert_eq!(resp.status, 301);
		assert_eq!(resp.headers.get("Location"), Some(&"/new-page".to_string()));
	}

	#[test]
	fn test_not_found() {
		let resp = not_found();
		assert_eq!(resp.status, 404);
		assert!(resp.body.contains("Not Found"));
	}

	#[test]
	fn test_not_found_with_message() {
		let resp = not_found_with_message("User not found");
		assert_eq!(resp.status, 404);
		assert!(resp.body.contains("User not found"));
	}

	#[test]
	fn test_bad_request() {
		let resp = bad_request("Invalid input");
		assert_eq!(resp.status, 400);
		assert!(resp.body.contains("Invalid input"));
	}

	#[test]
	fn test_unauthorized() {
		let resp = unauthorized();
		assert_eq!(resp.status, 401);
		assert!(resp.body.contains("Unauthorized"));
	}

	#[test]
	fn test_forbidden() {
		let resp = forbidden();
		assert_eq!(resp.status, 403);
		assert!(resp.body.contains("Forbidden"));
	}

	#[test]
	fn test_internal_error() {
		let resp = internal_error("Database connection failed");
		assert_eq!(resp.status, 500);
		assert!(resp.body.contains("Database connection failed"));
	}

	#[test]
	fn test_validation_error() {
		let errors = serde_json::json!({
			"email": ["Email is required"],
			"password": ["Password must be at least 8 characters"]
		});
		let resp = validation_error(errors);
		assert_eq!(resp.status, 422);
		assert!(resp.body.contains("Validation Error"));
	}

	#[test]
	fn test_no_content() {
		let resp = no_content();
		assert_eq!(resp.status, 204);
		assert!(resp.body.is_empty());
	}

	#[test]
	fn test_custom_response() {
		let resp = custom(418, serde_json::json!({"message": "I'm a teapot"}));
		assert_eq!(resp.status, 418);
	}

	#[test]
	fn test_response_builder() {
		let resp = Response::new(200)
			.header("X-Custom", "value")
			.content_type("application/json")
			.body(r#"{"test": true}"#);

		assert_eq!(resp.status, 200);
		assert_eq!(resp.headers.get("X-Custom"), Some(&"value".to_string()));
		assert_eq!(resp.headers.get("Content-Type"), Some(&"application/json".to_string()));
	}
}
