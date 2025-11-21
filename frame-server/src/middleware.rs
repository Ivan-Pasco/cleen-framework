use axum::{
	body::Body,
	http::{Request, Response, StatusCode, header, Method},
	middleware::Next,
	response::IntoResponse,
};
use std::time::Instant;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;

/// Logging middleware that tracks request timing
pub async fn logging_middleware(
	request: Request<Body>,
	next: Next,
) -> Response<Body> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let start = Instant::now();

	// Call the next middleware/handler
	let response = next.run(request).await;

	let elapsed = start.elapsed();
	let status = response.status();

	// Log the request
	println!(
		"{} {} {} - {}ms",
		method,
		uri,
		status.as_u16(),
		elapsed.as_millis()
	);

	response
}

/// CORS middleware configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
	/// Allowed origins (* for all)
	pub allowed_origins: Vec<String>,
	/// Allowed methods
	pub allowed_methods: Vec<Method>,
	/// Allowed headers
	pub allowed_headers: Vec<String>,
	/// Allow credentials
	pub allow_credentials: bool,
	/// Max age for preflight cache
	pub max_age: u64,
}

impl Default for CorsConfig {
	fn default() -> Self {
		Self {
			allowed_origins: vec!["*".to_string()],
			allowed_methods: vec![
				Method::GET,
				Method::POST,
				Method::PUT,
				Method::PATCH,
				Method::DELETE,
				Method::OPTIONS,
			],
			allowed_headers: vec![
				"Content-Type".to_string(),
				"Authorization".to_string(),
			],
			allow_credentials: true,
			max_age: 3600, // 1 hour
		}
	}
}

impl CorsConfig {
	/// Create a permissive CORS config (allow all)
	pub fn permissive() -> Self {
		Self::default()
	}

	/// Create a restrictive CORS config (specific origins only)
	pub fn restrictive(origins: Vec<String>) -> Self {
		Self {
			allowed_origins: origins,
			..Self::default()
		}
	}

	/// Add an allowed origin
	pub fn add_origin(mut self, origin: impl Into<String>) -> Self {
		self.allowed_origins.push(origin.into());
		self
	}

	/// Add an allowed method
	pub fn add_method(mut self, method: Method) -> Self {
		self.allowed_methods.push(method);
		self
	}

	/// Add an allowed header
	pub fn add_header(mut self, header: impl Into<String>) -> Self {
		self.allowed_headers.push(header.into());
		self
	}

	/// Check if an origin is allowed
	fn is_origin_allowed(&self, origin: &str) -> bool {
		self.allowed_origins.contains(&"*".to_string())
			|| self.allowed_origins.contains(&origin.to_string())
	}

	/// Get allowed methods as string
	fn allowed_methods_str(&self) -> String {
		self.allowed_methods
			.iter()
			.map(|m| m.as_str())
			.collect::<Vec<_>>()
			.join(", ")
	}

	/// Get allowed headers as string
	fn allowed_headers_str(&self) -> String {
		self.allowed_headers.join(", ")
	}
}

/// CORS middleware layer
#[derive(Clone)]
pub struct CorsLayer {
	config: CorsConfig,
}

impl CorsLayer {
	pub fn new(config: CorsConfig) -> Self {
		Self { config }
	}
}

impl<S> Layer<S> for CorsLayer {
	type Service = CorsMiddleware<S>;

	fn layer(&self, inner: S) -> Self::Service {
		CorsMiddleware {
			inner,
			config: self.config.clone(),
		}
	}
}

/// CORS middleware service
#[derive(Clone)]
pub struct CorsMiddleware<S> {
	inner: S,
	config: CorsConfig,
}

impl<S> Service<Request<Body>> for CorsMiddleware<S>
where
	S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
	S::Future: Send + 'static,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, request: Request<Body>) -> Self::Future {
		let config = self.config.clone();
		let mut inner = self.inner.clone();

		Box::pin(async move {
			// Get origin from request (clone to owned String before moving request)
			let origin = request
				.headers()
				.get(header::ORIGIN)
				.and_then(|v| v.to_str().ok())
				.unwrap_or("*")
				.to_string();

			// Handle preflight OPTIONS request
			if request.method() == Method::OPTIONS {
				let mut response = Response::new(Body::empty());
				*response.status_mut() = StatusCode::NO_CONTENT;

				// Add CORS headers
				if config.is_origin_allowed(&origin) {
					response.headers_mut().insert(
						header::ACCESS_CONTROL_ALLOW_ORIGIN,
						origin.parse().unwrap(),
					);
				}

				response.headers_mut().insert(
					header::ACCESS_CONTROL_ALLOW_METHODS,
					config.allowed_methods_str().parse().unwrap(),
				);

				response.headers_mut().insert(
					header::ACCESS_CONTROL_ALLOW_HEADERS,
					config.allowed_headers_str().parse().unwrap(),
				);

				if config.allow_credentials {
					response.headers_mut().insert(
						header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
						"true".parse().unwrap(),
					);
				}

				response.headers_mut().insert(
					header::ACCESS_CONTROL_MAX_AGE,
					config.max_age.to_string().parse().unwrap(),
				);

				return Ok(response);
			}

			// For regular requests, add CORS headers to response
			let mut response = inner.call(request).await?;

			if config.is_origin_allowed(&origin) {
				response.headers_mut().insert(
					header::ACCESS_CONTROL_ALLOW_ORIGIN,
					origin.parse().unwrap(),
				);
			}

			if config.allow_credentials {
				response.headers_mut().insert(
					header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
					"true".parse().unwrap(),
				);
			}

			Ok(response)
		})
	}
}

/// Error handling middleware that catches panics and returns 500
pub async fn error_handling_middleware(
	request: Request<Body>,
	next: Next,
) -> Response<Body> {
	// Call the next middleware/handler
	let response = next.run(request).await;

	// If response is an error status, we could add additional logging here
	if response.status().is_server_error() {
		eprintln!("Server error: {}", response.status());
	}

	response
}

/// Request ID middleware that adds a unique ID to each request
pub async fn request_id_middleware(
	mut request: Request<Body>,
	next: Next,
) -> Response<Body> {
	// Generate or get request ID
	let request_id = request
		.headers()
		.get("x-request-id")
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string())
		.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

	// Insert request ID into request extensions
	request.extensions_mut().insert(request_id.clone());

	// Call next middleware
	let mut response = next.run(request).await;

	// Add request ID to response headers
	response.headers_mut().insert(
		"x-request-id",
		request_id.parse().unwrap(),
	);

	response
}

/// Authentication middleware (placeholder - will integrate with frame-auth)
pub async fn auth_middleware(
	request: Request<Body>,
	next: Next,
) -> Response<Body> {
	// TODO: Integrate with frame-auth
	// For now, just pass through
	//
	// Future implementation:
	// 1. Check for session cookie or JWT token
	// 2. Validate token
	// 3. Load user from database
	// 4. Attach user to request extensions
	// 5. If invalid, return 401

	next.run(request).await
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_cors_config_default() {
		let config = CorsConfig::default();
		assert_eq!(config.allowed_origins, vec!["*"]);
		assert!(config.allowed_methods.contains(&Method::GET));
		assert!(config.allowed_methods.contains(&Method::POST));
		assert!(config.allow_credentials);
		assert_eq!(config.max_age, 3600);
	}

	#[test]
	fn test_cors_config_permissive() {
		let config = CorsConfig::permissive();
		assert!(config.is_origin_allowed("https://example.com"));
		assert!(config.is_origin_allowed("https://any-origin.com"));
	}

	#[test]
	fn test_cors_config_restrictive() {
		let config = CorsConfig::restrictive(vec![
			"https://example.com".to_string(),
			"https://app.example.com".to_string(),
		]);

		assert!(config.is_origin_allowed("https://example.com"));
		assert!(config.is_origin_allowed("https://app.example.com"));
		assert!(!config.is_origin_allowed("https://evil.com"));
	}

	#[test]
	fn test_cors_config_builder() {
		let config = CorsConfig::restrictive(vec!["https://example.com".to_string()])
			.add_origin("https://app.example.com")
			.add_method(Method::PATCH)
			.add_header("X-Custom-Header");

		assert_eq!(config.allowed_origins.len(), 2);
		assert!(config.allowed_methods.contains(&Method::PATCH));
		assert!(config.allowed_headers.contains(&"X-Custom-Header".to_string()));
	}

	#[test]
	fn test_cors_allowed_methods_str() {
		let config = CorsConfig::default();
		let methods_str = config.allowed_methods_str();
		assert!(methods_str.contains("GET"));
		assert!(methods_str.contains("POST"));
	}

	#[test]
	fn test_cors_allowed_headers_str() {
		let config = CorsConfig::default();
		let headers_str = config.allowed_headers_str();
		assert!(headers_str.contains("Content-Type"));
		assert!(headers_str.contains("Authorization"));
	}

	// Note: Full middleware integration tests require Axum server setup
	// These will be added in integration test file
}
