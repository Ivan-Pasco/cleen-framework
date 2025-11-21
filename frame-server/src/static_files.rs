use axum::{
	Router,
	http::{StatusCode, header},
	response::{IntoResponse, Response},
	extract::Path,
	body::Body,
};
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;

/// Static file configuration
#[derive(Debug, Clone)]
pub struct StaticFileConfig {
	/// Directory to serve files from (default: "./public")
	pub directory: PathBuf,
	/// Enable gzip compression
	pub enable_gzip: bool,
	/// Enable cache headers
	pub enable_cache: bool,
	/// Cache max-age in seconds (default: 3600 = 1 hour)
	pub cache_max_age: u32,
	/// Fallback file for SPA routing (e.g., "index.html")
	pub fallback: Option<String>,
}

impl Default for StaticFileConfig {
	fn default() -> Self {
		Self {
			directory: PathBuf::from("./public"),
			enable_gzip: true,
			enable_cache: true,
			cache_max_age: 3600,
			fallback: None,
		}
	}
}

impl StaticFileConfig {
	/// Create a new config with the given directory
	pub fn new(directory: impl Into<PathBuf>) -> Self {
		Self {
			directory: directory.into(),
			..Self::default()
		}
	}

	/// Disable gzip compression
	pub fn without_gzip(mut self) -> Self {
		self.enable_gzip = false;
		self
	}

	/// Disable cache headers
	pub fn without_cache(mut self) -> Self {
		self.enable_cache = false;
		self
	}

	/// Set cache max-age
	pub fn with_cache_max_age(mut self, max_age: u32) -> Self {
		self.cache_max_age = max_age;
		self
	}

	/// Set SPA fallback file
	pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
		self.fallback = Some(fallback.into());
		self
	}
}

/// MIME type detection from file extension
fn get_mime_type(path: &std::path::Path) -> &'static str {
	match path.extension().and_then(|s| s.to_str()) {
		Some("html") | Some("htm") => "text/html; charset=utf-8",
		Some("css") => "text/css; charset=utf-8",
		Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
		Some("json") => "application/json",
		Some("xml") => "application/xml",
		Some("png") => "image/png",
		Some("jpg") | Some("jpeg") => "image/jpeg",
		Some("gif") => "image/gif",
		Some("svg") => "image/svg+xml",
		Some("webp") => "image/webp",
		Some("ico") => "image/x-icon",
		Some("woff") => "font/woff",
		Some("woff2") => "font/woff2",
		Some("ttf") => "font/ttf",
		Some("otf") => "font/otf",
		Some("eot") => "application/vnd.ms-fontobject",
		Some("pdf") => "application/pdf",
		Some("txt") => "text/plain; charset=utf-8",
		Some("wasm") => "application/wasm",
		_ => "application/octet-stream",
	}
}

/// Check if file type should be compressed
fn should_compress(mime_type: &str) -> bool {
	mime_type.starts_with("text/")
		|| mime_type.starts_with("application/javascript")
		|| mime_type.starts_with("application/json")
		|| mime_type.starts_with("application/xml")
		|| mime_type.starts_with("image/svg")
}

/// Static file handler
pub async fn serve_static_file(
	Path(path): Path<String>,
	config: StaticFileConfig,
) -> Result<Response, StatusCode> {
	// Prevent directory traversal attacks
	let path = path.trim_start_matches('/');
	if path.contains("..") {
		return Err(StatusCode::FORBIDDEN);
	}

	// Build file path
	let file_path = config.directory.join(path);

	// Try to read the file
	match fs::read(&file_path).await {
		Ok(contents) => {
			let mime_type = get_mime_type(&file_path);

			// Build response
			let mut response = Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, mime_type);

			// Add cache headers if enabled
			if config.enable_cache {
				response = response
					.header(header::CACHE_CONTROL, format!("public, max-age={}", config.cache_max_age));

				// Add ETag (simple hash of content length + modified time would be better)
				let etag = format!("\"{}\"", contents.len());
				response = response.header(header::ETAG, etag);
			}

			// TODO: Add gzip compression if enabled and supported
			// This would require checking Accept-Encoding header and compressing content

			response
				.body(Body::from(contents))
				.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
		}
		Err(_) => {
			// If file not found and fallback is set, try to serve fallback
			if let Some(fallback) = config.fallback {
				let fallback_path = config.directory.join(fallback);
				match fs::read(&fallback_path).await {
					Ok(contents) => {
						let mime_type = get_mime_type(&fallback_path);
						Response::builder()
							.status(StatusCode::OK)
							.header(header::CONTENT_TYPE, mime_type)
							.body(Body::from(contents))
							.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
					}
					Err(_) => Err(StatusCode::NOT_FOUND),
				}
			} else {
				Err(StatusCode::NOT_FOUND)
			}
		}
	}
}

/// Add static file routes to the router
pub fn add_static_routes(router: Router, config: StaticFileConfig) -> Router {
	// Clone config for closure
	let config_clone = config.clone();

	router.fallback(move |path: Path<String>| {
		let config = config_clone.clone();
		async move { serve_static_file(path, config).await }
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_static_file_config_default() {
		let config = StaticFileConfig::default();
		assert_eq!(config.directory, PathBuf::from("./public"));
		assert!(config.enable_gzip);
		assert!(config.enable_cache);
		assert_eq!(config.cache_max_age, 3600);
		assert_eq!(config.fallback, None);
	}

	#[test]
	fn test_static_file_config_builder() {
		let config = StaticFileConfig::new("./dist")
			.without_gzip()
			.with_cache_max_age(7200)
			.with_fallback("index.html");

		assert_eq!(config.directory, PathBuf::from("./dist"));
		assert!(!config.enable_gzip);
		assert!(config.enable_cache);
		assert_eq!(config.cache_max_age, 7200);
		assert_eq!(config.fallback, Some("index.html".to_string()));
	}

	#[test]
	fn test_get_mime_type_html() {
		let path = std::path::Path::new("index.html");
		assert_eq!(get_mime_type(path), "text/html; charset=utf-8");
	}

	#[test]
	fn test_get_mime_type_css() {
		let path = std::path::Path::new("styles.css");
		assert_eq!(get_mime_type(path), "text/css; charset=utf-8");
	}

	#[test]
	fn test_get_mime_type_js() {
		let path = std::path::Path::new("app.js");
		assert_eq!(get_mime_type(path), "application/javascript; charset=utf-8");
	}

	#[test]
	fn test_get_mime_type_json() {
		let path = std::path::Path::new("data.json");
		assert_eq!(get_mime_type(path), "application/json");
	}

	#[test]
	fn test_get_mime_type_png() {
		let path = std::path::Path::new("image.png");
		assert_eq!(get_mime_type(path), "image/png");
	}

	#[test]
	fn test_get_mime_type_wasm() {
		let path = std::path::Path::new("app.wasm");
		assert_eq!(get_mime_type(path), "application/wasm");
	}

	#[test]
	fn test_get_mime_type_unknown() {
		let path = std::path::Path::new("file.xyz");
		assert_eq!(get_mime_type(path), "application/octet-stream");
	}

	#[test]
	fn test_should_compress_text() {
		assert!(should_compress("text/html"));
		assert!(should_compress("text/css"));
		assert!(should_compress("text/plain"));
	}

	#[test]
	fn test_should_compress_javascript() {
		assert!(should_compress("application/javascript"));
	}

	#[test]
	fn test_should_compress_json() {
		assert!(should_compress("application/json"));
	}

	#[test]
	fn test_should_not_compress_image() {
		assert!(!should_compress("image/png"));
		assert!(!should_compress("image/jpeg"));
	}

	#[test]
	fn test_should_not_compress_binary() {
		assert!(!should_compress("application/octet-stream"));
	}
}
