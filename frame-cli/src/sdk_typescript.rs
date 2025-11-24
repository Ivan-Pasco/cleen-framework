/*!
 * TypeScript SDK Generator
 *
 * Generates type-safe TypeScript client code from OpenAPI specifications.
 */

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::openapi::{OpenApiSpec, Operation, PathItem, Parameter, Schema};

/// TypeScript SDK generator
pub struct TypeScriptGenerator {
	spec: OpenApiSpec,
}

impl TypeScriptGenerator {
	/// Create a new generator from an OpenAPI spec
	pub fn new(spec: OpenApiSpec) -> Self {
		Self { spec }
	}

	/// Load from an OpenAPI spec file
	pub fn from_file(path: &Path) -> Result<Self> {
		let content = fs::read_to_string(path)
			.with_context(|| format!("Failed to read OpenAPI spec from {}", path.display()))?;

		let spec: OpenApiSpec = serde_json::from_str(&content)
			.context("Failed to parse OpenAPI spec JSON")?;

		Ok(Self::new(spec))
	}

	/// Generate TypeScript SDK code
	pub fn generate(&self) -> Result<String> {
		let mut output = String::new();

		// File header
		output.push_str(&self.generate_header());

		// Type definitions
		output.push_str(&self.generate_types());

		// API client class
		output.push_str(&self.generate_client());

		// Export default instance
		output.push_str(&self.generate_exports());

		Ok(output)
	}

	/// Write SDK to file
	pub fn write_to_file(&self, path: &Path) -> Result<()> {
		let code = self.generate()?;
		fs::write(path, code)
			.with_context(|| format!("Failed to write TypeScript SDK to {}", path.display()))?;
		Ok(())
	}

	/// Generate file header
	fn generate_header(&self) -> String {
		format!(
			r#"/**
 * {} TypeScript SDK
 * Version: {}
 *
 * Auto-generated from OpenAPI 3.1 specification.
 * Do not edit manually.
 */

"#,
			self.spec.info.title, self.spec.info.version
		)
	}

	/// Generate type definitions
	fn generate_types(&self) -> String {
		let mut output = String::new();

		output.push_str("// ============================================================================\n");
		output.push_str("// Type Definitions\n");
		output.push_str("// ============================================================================\n\n");

		// API response wrapper
		output.push_str(
			r#"export interface ApiResponse<T = any> {
  data?: T;
  error?: ApiError;
}

export interface ApiError {
  code: string;
  message: string;
  details?: any;
}

"#,
		);

		// Request options
		output.push_str(
			r#"export interface RequestOptions {
  headers?: Record<string, string>;
  timeout?: number;
  signal?: AbortSignal;
}

"#,
		);

		output
	}

	/// Generate API client class
	fn generate_client(&self) -> String {
		let mut output = String::new();

		output.push_str("// ============================================================================\n");
		output.push_str("// API Client\n");
		output.push_str("// ============================================================================\n\n");

		// Client configuration interface
		output.push_str(
			r#"export interface ApiClientConfig {
  baseUrl: string;
  headers?: Record<string, string>;
  timeout?: number;
}

"#,
		);

		// Client class
		output.push_str(
			r#"export class ApiClient {
  private config: ApiClientConfig;

  constructor(config: ApiClientConfig) {
    this.config = config;
  }

  /**
   * Make an HTTP request
   */
  private async request<T = any>(
    method: string,
    path: string,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.config.baseUrl}${path}`;
    const headers = {
      'Content-Type': 'application/json',
      ...this.config.headers,
      ...options.headers,
    };

    const timeout = options.timeout || this.config.timeout || 30000;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    try {
      const response = await fetch(url, {
        method,
        headers,
        signal: options.signal || controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error: ApiError = {
          code: `HTTP_${response.status}`,
          message: response.statusText,
        };

        try {
          const errorData = await response.json();
          error.details = errorData;
        } catch {
          // Response is not JSON
        }

        return { error };
      }

      // Handle 204 No Content
      if (response.status === 204) {
        return { data: undefined as T };
      }

      const data = await response.json();
      return { data };
    } catch (error) {
      return {
        error: {
          code: 'NETWORK_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error',
        },
      };
    }
  }

  /**
   * Make a GET request
   */
  private async get<T = any>(path: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>('GET', path, options);
  }

  /**
   * Make a POST request
   */
  private async post<T = any>(
    path: string,
    body?: any,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>('POST', path, {
      ...options,
      headers: {
        ...options?.headers,
        'Content-Type': 'application/json',
      },
      // Note: Fetch API doesn't support body in the same way
      // This would need to be handled in the request method
    });
  }

  /**
   * Make a PUT request
   */
  private async put<T = any>(
    path: string,
    body?: any,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>('PUT', path, options);
  }

  /**
   * Make a PATCH request
   */
  private async patch<T = any>(
    path: string,
    body?: any,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>('PATCH', path, options);
  }

  /**
   * Make a DELETE request
   */
  private async delete<T = any>(path: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>('DELETE', path, options);
  }

"#,
		);

		// Generate endpoint methods
		for (path, path_item) in &self.spec.paths {
			output.push_str(&self.generate_endpoint_methods(path, path_item));
		}

		output.push_str("}\n\n");

		output
	}

	/// Generate methods for a path item
	fn generate_endpoint_methods(&self, path: &str, path_item: &PathItem) -> String {
		let mut output = String::new();

		if let Some(op) = &path_item.get {
			output.push_str(&self.generate_method("GET", path, op));
		}

		if let Some(op) = &path_item.post {
			output.push_str(&self.generate_method("POST", path, op));
		}

		if let Some(op) = &path_item.put {
			output.push_str(&self.generate_method("PUT", path, op));
		}

		if let Some(op) = &path_item.patch {
			output.push_str(&self.generate_method("PATCH", path, op));
		}

		if let Some(op) = &path_item.delete {
			output.push_str(&self.generate_method("DELETE", path, op));
		}

		output
	}

	/// Generate a single method
	fn generate_method(&self, http_method: &str, path: &str, operation: &Operation) -> String {
		let method_name = operation
			.operation_id
			.as_ref()
			.map(|s| to_camel_case(s))
			.unwrap_or_else(|| generate_method_name(http_method, path));

		let description = operation
			.summary
			.as_ref()
			.or(operation.description.as_ref())
			.map(|s| s.as_str())
			.unwrap_or("");

		// Extract path parameters
		let path_params: Vec<&Parameter> = operation
			.parameters
			.iter()
			.filter(|p| p.location == "path")
			.collect();

		// Build parameter list
		let mut params = Vec::new();
		for param in &path_params {
			params.push(format!("{}: {}", param.name, schema_to_ts_type(&param.schema)));
		}

		// Add request body for POST/PUT/PATCH
		let has_body = matches!(http_method, "POST" | "PUT" | "PATCH");
		if has_body {
			params.push("body?: any".to_string());
		}

		params.push("options?: RequestOptions".to_string());

		let params_str = params.join(", ");

		// Build path template
		let path_template = build_path_template(path, &path_params);

		// Determine HTTP method call
		let http_call = match http_method {
			"GET" => format!("this.get(`{}`", path_template),
			"POST" => format!("this.post(`{}`, body", path_template),
			"PUT" => format!("this.put(`{}`, body", path_template),
			"PATCH" => format!("this.patch(`{}`, body", path_template),
			"DELETE" => format!("this.delete(`{}`", path_template),
			_ => format!("this.get(`{}`", path_template),
		};

		format!(
			r#"  /**
   * {}
   */
  async {}({}): Promise<ApiResponse> {{
    return {}, options);
  }}

"#,
			description, method_name, params_str, http_call
		)
	}

	/// Generate exports
	fn generate_exports(&self) -> String {
		let base_url = self
			.spec
			.servers
			.first()
			.map(|s| s.url.as_str())
			.unwrap_or("http://localhost:3000");

		format!(
			r#"// ============================================================================
// Default Export
// ============================================================================

export default new ApiClient({{
  baseUrl: '{}',
}});
"#,
			base_url
		)
	}
}

/// Convert a string to camelCase
fn to_camel_case(s: &str) -> String {
	let mut result = String::new();
	let mut capitalize_next = false;

	for (i, ch) in s.chars().enumerate() {
		if ch == '_' || ch == '-' {
			capitalize_next = true;
		} else if i == 0 {
			result.push(ch.to_lowercase().next().unwrap());
		} else if capitalize_next {
			result.push(ch.to_uppercase().next().unwrap());
			capitalize_next = false;
		} else {
			result.push(ch);
		}
	}

	result
}

/// Generate method name from HTTP method and path
fn generate_method_name(http_method: &str, path: &str) -> String {
	// Remove leading slash and parameters
	let clean_path = path
		.trim_start_matches('/')
		.replace("{", "")
		.replace("}", "")
		.replace("/", "_");

	let prefix = match http_method {
		"GET" => "get",
		"POST" => "create",
		"PUT" => "update",
		"PATCH" => "patch",
		"DELETE" => "delete",
		_ => "request",
	};

	format!("{}_{}", prefix, clean_path)
}

/// Convert OpenAPI schema to TypeScript type
fn schema_to_ts_type(schema: &Schema) -> String {
	if let Some(type_name) = &schema.type_name {
		match type_name.as_str() {
			"string" => "string".to_string(),
			"number" | "integer" => "number".to_string(),
			"boolean" => "boolean".to_string(),
			"array" => {
				if let Some(items) = &schema.items {
					format!("{}[]", schema_to_ts_type(items))
				} else {
					"any[]".to_string()
				}
			}
			"object" => "any".to_string(),
			_ => "any".to_string(),
		}
	} else {
		"any".to_string()
	}
}

/// Build path template with parameter interpolation
fn build_path_template(path: &str, params: &[&Parameter]) -> String {
	let mut result = path.to_string();

	for param in params {
		result = result.replace(
			&format!("{{{}}}", param.name),
			&format!("${{{}}}", param.name),
		);
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_to_camel_case() {
		assert_eq!(to_camel_case("hello_world"), "helloWorld");
		assert_eq!(to_camel_case("get_user"), "getUser");
		assert_eq!(to_camel_case("create_user_account"), "createUserAccount");
		assert_eq!(to_camel_case("list-items"), "listItems");
	}

	#[test]
	fn test_generate_method_name() {
		assert_eq!(generate_method_name("GET", "/users"), "get_users");
		assert_eq!(generate_method_name("POST", "/users"), "create_users");
		assert_eq!(generate_method_name("GET", "/users/{id}"), "get_users_id");
		assert_eq!(
			generate_method_name("PUT", "/users/{id}"),
			"update_users_id"
		);
		assert_eq!(
			generate_method_name("DELETE", "/users/{id}"),
			"delete_users_id"
		);
	}

	#[test]
	fn test_build_path_template() {
		let param = Parameter {
			name: "id".to_string(),
			location: "path".to_string(),
			description: None,
			required: true,
			schema: Schema {
				type_name: Some("string".to_string()),
				format: None,
				properties: None,
				required: Vec::new(),
				items: None,
				reference: None,
			},
		};

		let params = vec![&param];
		assert_eq!(build_path_template("/users/{id}", &params), "/users/${id}");
		assert_eq!(build_path_template("/users", &[]), "/users");
	}
}
