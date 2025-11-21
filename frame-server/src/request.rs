use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Context};

/// HTTP request representation for Clean Language applications
///
/// This type is passed to WASM functions and represents all aspects
/// of an incoming HTTP request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
	/// HTTP method (GET, POST, PUT, PATCH, DELETE, etc.)
	pub method: String,

	/// Full path including query string
	pub path: String,

	/// Path parameters extracted from the route
	/// Example: /users/:id -> {"id": "123"}
	pub params: HashMap<String, String>,

	/// Query string parameters
	/// Example: ?page=1&limit=10 -> {"page": "1", "limit": "10"}
	pub query: HashMap<String, String>,

	/// Request headers
	pub headers: HashMap<String, String>,

	/// Parsed JSON body (if Content-Type: application/json)
	pub body: Option<Value>,

	/// Raw body as string (for non-JSON content)
	pub raw_body: Option<String>,

	/// Authenticated user (populated by auth middleware)
	pub user: Option<User>,

	/// Request ID for tracing
	pub request_id: String,
}

/// Authenticated user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: i64,
	pub email: String,
	pub roles: Vec<String>,
	pub metadata: HashMap<String, Value>,
}

impl Request {
	/// Create a new request
	pub fn new(method: String, path: String) -> Self {
		Self {
			method,
			path,
			params: HashMap::new(),
			query: HashMap::new(),
			headers: HashMap::new(),
			body: None,
			raw_body: None,
			user: None,
			request_id: uuid::Uuid::new_v4().to_string(),
		}
	}

	/// Get a path parameter by name
	pub fn param(&self, name: &str) -> Option<&String> {
		self.params.get(name)
	}

	/// Get a query parameter by name
	pub fn query_param(&self, name: &str) -> Option<&String> {
		self.query.get(name)
	}

	/// Get a header by name (case-insensitive)
	pub fn header(&self, name: &str) -> Option<&String> {
		let name_lower = name.to_lowercase();
		self.headers
			.iter()
			.find(|(k, _)| k.to_lowercase() == name_lower)
			.map(|(_, v)| v)
	}

	/// Parse JSON body into a specific type
	pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
		let body = self.body.as_ref()
			.ok_or_else(|| anyhow::anyhow!("No JSON body in request"))?;

		serde_json::from_value(body.clone())
			.context("Failed to deserialize request body")
	}

	/// Check if user is authenticated
	pub fn is_authenticated(&self) -> bool {
		self.user.is_some()
	}

	/// Check if user has a specific role
	pub fn has_role(&self, role: &str) -> bool {
		self.user.as_ref()
			.map(|u| u.roles.contains(&role.to_string()))
			.unwrap_or(false)
	}

	/// Get content type
	pub fn content_type(&self) -> Option<&String> {
		self.header("content-type")
	}

	/// Check if request accepts JSON
	pub fn accepts_json(&self) -> bool {
		self.header("accept")
			.map(|accept| accept.contains("application/json"))
			.unwrap_or(false)
	}
}

/// Builder for constructing Request objects
pub struct RequestBuilder {
	request: Request,
}

impl RequestBuilder {
	pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
		Self {
			request: Request::new(method.into(), path.into()),
		}
	}

	pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.request.params.insert(key.into(), value.into());
		self
	}

	pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.request.query.insert(key.into(), value.into());
		self
	}

	pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.request.headers.insert(key.into(), value.into());
		self
	}

	pub fn body(mut self, body: Value) -> Self {
		self.request.body = Some(body);
		self
	}

	pub fn raw_body(mut self, body: String) -> Self {
		self.request.raw_body = Some(body);
		self
	}

	pub fn user(mut self, user: User) -> Self {
		self.request.user = Some(user);
		self
	}

	pub fn build(self) -> Request {
		self.request
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_request_creation() {
		let req = Request::new("GET".to_string(), "/users".to_string());
		assert_eq!(req.method, "GET");
		assert_eq!(req.path, "/users");
		assert!(!req.is_authenticated());
	}

	#[test]
	fn test_request_builder() {
		let req = RequestBuilder::new("POST", "/users")
			.param("id", "123")
			.query("page", "1")
			.header("Content-Type", "application/json")
			.body(serde_json::json!({"name": "Alice"}))
			.build();

		assert_eq!(req.method, "POST");
		assert_eq!(req.param("id"), Some(&"123".to_string()));
		assert_eq!(req.query_param("page"), Some(&"1".to_string()));
		assert_eq!(req.header("content-type"), Some(&"application/json".to_string()));
		assert!(req.body.is_some());
	}

	#[test]
	fn test_json_deserialization() {
		#[derive(Deserialize)]
		struct TestBody {
			name: String,
		}

		let req = RequestBuilder::new("POST", "/users")
			.body(serde_json::json!({"name": "Alice"}))
			.build();

		let body: TestBody = req.json().unwrap();
		assert_eq!(body.name, "Alice");
	}

	#[test]
	fn test_user_authentication() {
		let user = User {
			id: 1,
			email: "alice@example.com".to_string(),
			roles: vec!["admin".to_string(), "user".to_string()],
			metadata: HashMap::new(),
		};

		let req = RequestBuilder::new("GET", "/admin")
			.user(user)
			.build();

		assert!(req.is_authenticated());
		assert!(req.has_role("admin"));
		assert!(req.has_role("user"));
		assert!(!req.has_role("moderator"));
	}

	#[test]
	fn test_header_case_insensitive() {
		let req = RequestBuilder::new("GET", "/")
			.header("Content-Type", "application/json")
			.build();

		assert_eq!(req.header("content-type"), Some(&"application/json".to_string()));
		assert_eq!(req.header("Content-Type"), Some(&"application/json".to_string()));
		assert_eq!(req.header("CONTENT-TYPE"), Some(&"application/json".to_string()));
	}
}
