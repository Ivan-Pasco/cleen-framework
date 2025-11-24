/*!
 * OpenAPI 3.1 Specification Generator
 *
 * This module generates OpenAPI specifications from Clean Language
 * `endpoints:` blocks, enabling API documentation and SDK generation.
 */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// OpenAPI 3.1 Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
	pub openapi: String,
	pub info: Info,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub servers: Vec<Server>,
	pub paths: HashMap<String, PathItem>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Components>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
	pub title: String,
	pub version: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub contact: Option<Contact>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
	pub url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub get: Option<Operation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub post: Option<Operation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub put: Option<Operation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub patch: Option<Operation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delete: Option<Operation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parameters: Option<Vec<Parameter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub summary: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub operation_id: Option<String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub parameters: Vec<Parameter>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub request_body: Option<RequestBody>,
	pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
	pub name: String,
	#[serde(rename = "in")]
	pub location: String, // "path", "query", "header", "cookie"
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	pub required: bool,
	pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	pub required: bool,
	pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
	pub description: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
	pub schema: Schema,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "type")]
	pub type_name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub format: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub properties: Option<HashMap<String, Box<Schema>>>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub required: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<Box<Schema>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "$ref")]
	pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub schemas: HashMap<String, Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}

/// Endpoint definition extracted from Clean code
#[derive(Debug, Clone)]
pub struct Endpoint {
	pub method: String,
	pub path: String,
	pub handler: String,
	pub summary: Option<String>,
	pub description: Option<String>,
	pub tags: Vec<String>,
}

/// OpenAPI generator
pub struct OpenApiGenerator {
	spec: OpenApiSpec,
}

impl OpenApiGenerator {
	/// Create a new generator with default info
	pub fn new(title: &str, version: &str) -> Self {
		Self {
			spec: OpenApiSpec {
				openapi: "3.1.0".to_string(),
				info: Info {
					title: title.to_string(),
					version: version.to_string(),
					description: Some("API generated from Clean Language endpoints".to_string()),
					contact: None,
					license: None,
				},
				servers: vec![
					Server {
						url: "http://localhost:3000".to_string(),
						description: Some("Development server".to_string()),
					},
				],
				paths: HashMap::new(),
				components: Some(Components {
					schemas: HashMap::new(),
				}),
				tags: Vec::new(),
			},
		}
	}

	/// Set API description
	pub fn with_description(mut self, description: &str) -> Self {
		self.spec.info.description = Some(description.to_string());
		self
	}

	/// Add a server
	pub fn add_server(mut self, url: &str, description: Option<&str>) -> Self {
		self.spec.servers.push(Server {
			url: url.to_string(),
			description: description.map(|s| s.to_string()),
		});
		self
	}

	/// Add an endpoint to the spec
	pub fn add_endpoint(&mut self, endpoint: Endpoint) {
		// Extract path parameters from the path (e.g., /users/{id})
		let path_params = extract_path_parameters(&endpoint.path);

		// Create operation
		let operation = Operation {
			summary: endpoint.summary,
			description: endpoint.description,
			operation_id: Some(endpoint.handler.clone()),
			tags: endpoint.tags,
			parameters: path_params,
			request_body: create_request_body(&endpoint.method),
			responses: create_default_responses(&endpoint.method),
		};

		// Get or create path item
		let path_item = self.spec.paths.entry(endpoint.path.clone()).or_insert(PathItem {
			get: None,
			post: None,
			put: None,
			patch: None,
			delete: None,
			parameters: None,
		});

		// Set the appropriate method
		match endpoint.method.to_uppercase().as_str() {
			"GET" => path_item.get = Some(operation),
			"POST" => path_item.post = Some(operation),
			"PUT" => path_item.put = Some(operation),
			"PATCH" => path_item.patch = Some(operation),
			"DELETE" => path_item.delete = Some(operation),
			_ => {}
		}
	}

	/// Generate the OpenAPI spec as JSON
	pub fn generate(&self) -> Result<String> {
		serde_json::to_string_pretty(&self.spec)
			.context("Failed to serialize OpenAPI spec to JSON")
	}

	/// Write the spec to a file
	pub fn write_to_file(&self, path: &Path) -> Result<()> {
		let json = self.generate()?;
		fs::write(path, json)
			.with_context(|| format!("Failed to write OpenAPI spec to {}", path.display()))?;
		Ok(())
	}
}

/// Extract path parameters from a path template
fn extract_path_parameters(path: &str) -> Vec<Parameter> {
	let mut params = Vec::new();

	// Find all {param} patterns
	let mut chars = path.chars().peekable();
	while let Some(c) = chars.next() {
		if c == '{' {
			let mut param_name = String::new();
			while let Some(&next_char) = chars.peek() {
				if next_char == '}' {
					chars.next(); // consume '}'
					break;
				}
				param_name.push(chars.next().unwrap());
			}

			if !param_name.is_empty() {
				params.push(Parameter {
					name: param_name.clone(),
					location: "path".to_string(),
					description: Some(format!("Path parameter: {}", param_name)),
					required: true,
					schema: Schema {
						type_name: Some("string".to_string()),
						format: None,
						properties: None,
						required: Vec::new(),
						items: None,
						reference: None,
					},
				});
			}
		}
	}

	params
}

/// Create default request body for methods that typically have one
fn create_request_body(method: &str) -> Option<RequestBody> {
	match method.to_uppercase().as_str() {
		"POST" | "PUT" | "PATCH" => {
			let mut content = HashMap::new();
			content.insert(
				"application/json".to_string(),
				MediaType {
					schema: Schema {
						type_name: Some("object".to_string()),
						format: None,
						properties: None,
						required: Vec::new(),
						items: None,
						reference: None,
					},
					example: None,
				},
			);

			Some(RequestBody {
				description: Some("Request payload".to_string()),
				required: true,
				content,
			})
		}
		_ => None,
	}
}

/// Create default responses for an endpoint
fn create_default_responses(method: &str) -> HashMap<String, Response> {
	let mut responses = HashMap::new();

	// Success response
	let success_code = match method.to_uppercase().as_str() {
		"POST" => "201",
		"DELETE" => "204",
		_ => "200",
	};

	let mut success_content = HashMap::new();
	if success_code != "204" {
		success_content.insert(
			"application/json".to_string(),
			MediaType {
				schema: Schema {
					type_name: Some("object".to_string()),
					format: None,
					properties: None,
					required: Vec::new(),
					items: None,
					reference: None,
				},
				example: None,
			},
		);
	}

	responses.insert(
		success_code.to_string(),
		Response {
			description: "Successful response".to_string(),
			content: if success_content.is_empty() {
				None
			} else {
				Some(success_content)
			},
		},
	);

	// Error responses
	responses.insert(
		"400".to_string(),
		Response {
			description: "Bad request".to_string(),
			content: None,
		},
	);

	responses.insert(
		"500".to_string(),
		Response {
			description: "Internal server error".to_string(),
			content: None,
		},
	);

	responses
}

/// Parse Clean code file for endpoints
pub fn parse_endpoints_from_file(file_path: &Path) -> Result<Vec<Endpoint>> {
	let content = fs::read_to_string(file_path)
		.with_context(|| format!("Failed to read file: {}", file_path.display()))?;

	parse_endpoints_from_content(&content)
}

/// Parse endpoints from Clean code content
pub fn parse_endpoints_from_content(content: &str) -> Result<Vec<Endpoint>> {
	let mut endpoints = Vec::new();
	let lines: Vec<&str> = content.lines().collect();

	let mut i = 0;
	while i < lines.len() {
		let line = lines[i].trim();

		// Look for endpoints: block
		if line == "endpoints:" {
			i += 1;

			// Parse endpoint definitions
			while i < lines.len() {
				let endpoint_line = lines[i];

				// Check if we've left the block (no leading whitespace)
				if !endpoint_line.starts_with('\t') && !endpoint_line.starts_with("    ") {
					break;
				}

				let trimmed = endpoint_line.trim();

				// Skip empty lines and comments
				if trimmed.is_empty() || trimmed.starts_with("//") {
					i += 1;
					continue;
				}

				// Parse endpoint line: METHOD "path" -> handler
				if let Some(endpoint) = parse_endpoint_line(trimmed) {
					endpoints.push(endpoint);
				}

				i += 1;
			}
		}

		i += 1;
	}

	Ok(endpoints)
}

/// Parse a single endpoint line
fn parse_endpoint_line(line: &str) -> Option<Endpoint> {
	// Expected format: METHOD "path" -> handler
	// or: METHOD 'path' -> handler

	let parts: Vec<&str> = line.split_whitespace().collect();
	if parts.len() < 4 {
		return None;
	}

	let method = parts[0];
	let path_with_quotes = parts[1];
	let arrow = parts[2];
	let handler = parts[3];

	if arrow != "->" {
		return None;
	}

	// Remove quotes from path
	let path = path_with_quotes.trim_matches('"').trim_matches('\'');

	Some(Endpoint {
		method: method.to_string(),
		path: path.to_string(),
		handler: handler.to_string(),
		summary: None,
		description: None,
		tags: Vec::new(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_path_parameters() {
		let params = extract_path_parameters("/users/{id}");
		assert_eq!(params.len(), 1);
		assert_eq!(params[0].name, "id");
		assert_eq!(params[0].location, "path");
		assert_eq!(params[0].required, true);

		let params = extract_path_parameters("/users/{userId}/posts/{postId}");
		assert_eq!(params.len(), 2);
		assert_eq!(params[0].name, "userId");
		assert_eq!(params[1].name, "postId");

		let params = extract_path_parameters("/users");
		assert_eq!(params.len(), 0);
	}

	#[test]
	fn test_parse_endpoint_line() {
		let endpoint = parse_endpoint_line(r#"GET "/users" -> listUsers"#).unwrap();
		assert_eq!(endpoint.method, "GET");
		assert_eq!(endpoint.path, "/users");
		assert_eq!(endpoint.handler, "listUsers");

		let endpoint = parse_endpoint_line(r#"POST "/users" -> createUser"#).unwrap();
		assert_eq!(endpoint.method, "POST");
		assert_eq!(endpoint.path, "/users");
		assert_eq!(endpoint.handler, "createUser");

		let endpoint = parse_endpoint_line(r#"GET "/users/{id}" -> getUser"#).unwrap();
		assert_eq!(endpoint.method, "GET");
		assert_eq!(endpoint.path, "/users/{id}");
		assert_eq!(endpoint.handler, "getUser");
	}

	#[test]
	fn test_parse_endpoints_from_content() {
		let content = r#"
endpoints:
	GET "/users" -> listUsers
	POST "/users" -> createUser
	GET "/users/{id}" -> getUser
	PUT "/users/{id}" -> updateUser
	DELETE "/users/{id}" -> deleteUser

functions:
	void listUsers()
		// Implementation
"#;

		let endpoints = parse_endpoints_from_content(content).unwrap();
		assert_eq!(endpoints.len(), 5);
		assert_eq!(endpoints[0].method, "GET");
		assert_eq!(endpoints[0].path, "/users");
		assert_eq!(endpoints[1].method, "POST");
		assert_eq!(endpoints[2].path, "/users/{id}");
	}

	#[test]
	fn test_openapi_generator() {
		let mut generator = OpenApiGenerator::new("Test API", "1.0.0");

		generator.add_endpoint(Endpoint {
			method: "GET".to_string(),
			path: "/users".to_string(),
			handler: "listUsers".to_string(),
			summary: Some("List all users".to_string()),
			description: None,
			tags: vec!["users".to_string()],
		});

		generator.add_endpoint(Endpoint {
			method: "GET".to_string(),
			path: "/users/{id}".to_string(),
			handler: "getUser".to_string(),
			summary: Some("Get user by ID".to_string()),
			description: None,
			tags: vec!["users".to_string()],
		});

		let json = generator.generate().unwrap();
		assert!(json.contains("\"openapi\": \"3.1.0\""));
		assert!(json.contains("\"title\": \"Test API\""));
		assert!(json.contains("\"/users\""));
		assert!(json.contains("\"/users/{id}\""));
	}

	#[test]
	fn test_create_request_body() {
		let body = create_request_body("POST");
		assert!(body.is_some());
		assert_eq!(body.unwrap().required, true);

		let body = create_request_body("GET");
		assert!(body.is_none());
	}

	#[test]
	fn test_create_default_responses() {
		let responses = create_default_responses("GET");
		assert!(responses.contains_key("200"));
		assert!(responses.contains_key("400"));
		assert!(responses.contains_key("500"));

		let responses = create_default_responses("POST");
		assert!(responses.contains_key("201"));

		let responses = create_default_responses("DELETE");
		assert!(responses.contains_key("204"));
	}
}
