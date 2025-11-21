use anyhow::Result;
use axum::{
	Router as AxumRouter,
	routing::{get, post, put, patch, delete as axum_delete, MethodRouter},
	extract::{Path, Query, State},
	http::{HeaderMap, Method, StatusCode},
	body::Body,
	response::IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;

use crate::runtime::WasmRuntime;
use crate::request::{Request, RequestBuilder};
use crate::response::Response;

/// Frame router that connects HTTP requests to WASM functions
pub struct FrameRouter {
	runtime: Arc<WasmRuntime>,
	routes: Vec<Route>,
}

/// Route definition
#[derive(Debug, Clone)]
pub struct Route {
	pub method: HttpMethod,
	pub path: String,
	pub handler_name: String,
}

/// HTTP methods supported by Frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
	Get,
	Post,
	Put,
	Patch,
	Delete,
	Options,
	Head,
}

impl HttpMethod {
	pub fn as_str(&self) -> &str {
		match self {
			HttpMethod::Get => "GET",
			HttpMethod::Post => "POST",
			HttpMethod::Put => "PUT",
			HttpMethod::Patch => "PATCH",
			HttpMethod::Delete => "DELETE",
			HttpMethod::Options => "OPTIONS",
			HttpMethod::Head => "HEAD",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_uppercase().as_str() {
			"GET" => Some(HttpMethod::Get),
			"POST" => Some(HttpMethod::Post),
			"PUT" => Some(HttpMethod::Put),
			"PATCH" => Some(HttpMethod::Patch),
			"DELETE" => Some(HttpMethod::Delete),
			"OPTIONS" => Some(HttpMethod::Options),
			"HEAD" => Some(HttpMethod::Head),
			_ => None,
		}
	}
}

impl FrameRouter {
	/// Create a new router with the given WASM runtime
	pub fn new(runtime: WasmRuntime) -> Self {
		Self {
			runtime: Arc::new(runtime),
			routes: Vec::new(),
		}
	}

	/// Add a route
	pub fn route(mut self, method: HttpMethod, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.routes.push(Route {
			method,
			path: path.into(),
			handler_name: handler_name.into(),
		});
		self
	}

	/// Add a GET route
	pub fn get(self, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.route(HttpMethod::Get, path, handler_name)
	}

	/// Add a POST route
	pub fn post(self, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.route(HttpMethod::Post, path, handler_name)
	}

	/// Add a PUT route
	pub fn put(self, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.route(HttpMethod::Put, path, handler_name)
	}

	/// Add a PATCH route
	pub fn patch(self, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.route(HttpMethod::Patch, path, handler_name)
	}

	/// Add a DELETE route
	pub fn delete(self, path: impl Into<String>, handler_name: impl Into<String>) -> Self {
		self.route(HttpMethod::Delete, path, handler_name)
	}

	/// Build an Axum router
	pub fn build(self) -> AxumRouter {
		let runtime = Arc::clone(&self.runtime);
		let mut axum_router = AxumRouter::new();

		// Add each route to the Axum router
		for route in self.routes {
			let runtime_clone = Arc::clone(&runtime);
			let handler_name = route.handler_name.clone();

			let method_router = match route.method {
				HttpMethod::Get => get(move |path, query, headers, body| {
					handle_request(runtime_clone, handler_name, Method::GET, path, query, headers, body)
				}),
				HttpMethod::Post => post(move |path, query, headers, body| {
					handle_request(runtime_clone, handler_name, Method::POST, path, query, headers, body)
				}),
				HttpMethod::Put => put(move |path, query, headers, body| {
					handle_request(runtime_clone, handler_name, Method::PUT, path, query, headers, body)
				}),
				HttpMethod::Patch => patch(move |path, query, headers, body| {
					handle_request(runtime_clone, handler_name, Method::PATCH, path, query, headers, body)
				}),
				HttpMethod::Delete => axum_delete(move |path, query, headers, body| {
					handle_request(runtime_clone, handler_name, Method::DELETE, path, query, headers, body)
				}),
				HttpMethod::Options => {
					// TODO: Implement OPTIONS handler
					continue;
				}
				HttpMethod::Head => {
					// TODO: Implement HEAD handler
					continue;
				}
			};

			axum_router = axum_router.route(&route.path, method_router);
		}

		axum_router
	}
}

/// Handle an HTTP request by calling the WASM function
async fn handle_request(
	runtime: Arc<WasmRuntime>,
	handler_name: String,
	method: Method,
	path: Path<HashMap<String, String>>,
	query: Query<HashMap<String, String>>,
	headers: HeaderMap,
	body: String,
) -> impl IntoResponse {
	// Build Request object
	let mut request_builder = RequestBuilder::new(
		method.as_str(),
		format!("/{}", path.iter().map(|(_, v)| v.as_str()).collect::<Vec<_>>().join("/")),
	);

	// Add path parameters
	for (key, value) in path.0 {
		request_builder = request_builder.param(key, value);
	}

	// Add query parameters
	for (key, value) in query.0 {
		request_builder = request_builder.query(key, value);
	}

	// Add headers
	for (key, value) in headers.iter() {
		if let Ok(value_str) = value.to_str() {
			request_builder = request_builder.header(key.as_str(), value_str);
		}
	}

	// Parse body as JSON if content-type is application/json
	if let Some(content_type) = headers.get("content-type") {
		if let Ok(ct_str) = content_type.to_str() {
			if ct_str.contains("application/json") {
				if let Ok(json_value) = serde_json::from_str::<Value>(&body) {
					request_builder = request_builder.body(json_value);
				}
			}
		}
	}

	// If body wasn't parsed as JSON, store as raw body
	if !body.is_empty() {
		request_builder = request_builder.raw_body(body);
	}

	let request = request_builder.build();

	// Serialize request to JSON
	let request_json = match serde_json::to_value(&request) {
		Ok(v) => v,
		Err(e) => {
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Failed to serialize request: {}", e),
			).into_response();
		}
	};

	// Call WASM function
	let response_json = match runtime.call_function(&handler_name, request_json).await {
		Ok(v) => v,
		Err(e) => {
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("WASM function error: {}", e),
			).into_response();
		}
	};

	// Deserialize response
	let response: Response = match serde_json::from_value(response_json) {
		Ok(r) => r,
		Err(e) => {
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Failed to deserialize response: {}", e),
			).into_response();
		}
	};

	// Convert to Axum response
	let mut axum_response = axum::http::Response::builder()
		.status(response.status);

	// Add headers
	for (key, value) in response.headers {
		axum_response = axum_response.header(key, value);
	}

	// Build final response
	match axum_response.body(Body::from(response.body)) {
		Ok(resp) => resp.into_response(),
		Err(e) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Failed to build response: {}", e),
		).into_response(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_http_method_from_str() {
		assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::Get));
		assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::Post));
		assert_eq!(HttpMethod::from_str("PUT"), Some(HttpMethod::Put));
		assert_eq!(HttpMethod::from_str("invalid"), None);
	}

	#[test]
	fn test_http_method_as_str() {
		assert_eq!(HttpMethod::Get.as_str(), "GET");
		assert_eq!(HttpMethod::Post.as_str(), "POST");
		assert_eq!(HttpMethod::Put.as_str(), "PUT");
	}

	#[test]
	fn test_route_creation() {
		let route = Route {
			method: HttpMethod::Get,
			path: "/users".to_string(),
			handler_name: "get_users".to_string(),
		};

		assert_eq!(route.method, HttpMethod::Get);
		assert_eq!(route.path, "/users");
		assert_eq!(route.handler_name, "get_users");
	}

	// Note: Full integration tests for the router require a WASM module
	// These will be added once we have test fixtures
}
