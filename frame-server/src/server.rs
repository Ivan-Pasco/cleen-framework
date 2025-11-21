use anyhow::Result;
use axum::{
	Router,
	routing::get,
	http::StatusCode,
	response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Server metrics tracking
#[derive(Debug, Clone, Serialize)]
pub struct ServerMetrics {
	/// Total number of requests handled
	pub total_requests: u64,
	/// Number of active connections
	pub active_connections: u64,
	/// Server uptime in seconds
	pub uptime_seconds: u64,
	/// Average response time in milliseconds
	pub avg_response_time_ms: f64,
	/// Server start time
	#[serde(skip)]
	pub start_time: Instant,
}

impl Default for ServerMetrics {
	fn default() -> Self {
		Self {
			total_requests: 0,
			active_connections: 0,
			uptime_seconds: 0,
			avg_response_time_ms: 0.0,
			start_time: Instant::now(),
		}
	}
}

impl ServerMetrics {
	/// Create new server metrics
	pub fn new() -> Self {
		Self::default()
	}

	/// Record a new request
	pub fn record_request(&mut self, response_time_ms: f64) {
		self.total_requests += 1;

		// Calculate running average
		let n = self.total_requests as f64;
		self.avg_response_time_ms = (self.avg_response_time_ms * (n - 1.0) + response_time_ms) / n;
	}

	/// Increment active connections
	pub fn increment_connections(&mut self) {
		self.active_connections += 1;
	}

	/// Decrement active connections
	pub fn decrement_connections(&mut self) {
		self.active_connections = self.active_connections.saturating_sub(1);
	}

	/// Update uptime
	pub fn update_uptime(&mut self) {
		self.uptime_seconds = self.start_time.elapsed().as_secs();
	}
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
	/// Address to bind to
	pub addr: SocketAddr,
	/// Enable graceful shutdown
	pub graceful_shutdown: bool,
	/// Graceful shutdown timeout in seconds
	pub shutdown_timeout_secs: u64,
	/// Enable health check endpoint
	pub enable_health_check: bool,
	/// Enable metrics endpoint
	pub enable_metrics: bool,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			addr: "0.0.0.0:3000".parse().unwrap(),
			graceful_shutdown: true,
			shutdown_timeout_secs: 30,
			enable_health_check: true,
			enable_metrics: true,
		}
	}
}

impl ServerConfig {
	/// Create a new server config with the given address
	pub fn new(addr: SocketAddr) -> Self {
		Self {
			addr,
			..Self::default()
		}
	}

	/// Set graceful shutdown timeout
	pub fn with_shutdown_timeout(mut self, timeout_secs: u64) -> Self {
		self.shutdown_timeout_secs = timeout_secs;
		self
	}

	/// Disable graceful shutdown
	pub fn without_graceful_shutdown(mut self) -> Self {
		self.graceful_shutdown = false;
		self
	}

	/// Disable health check endpoint
	pub fn without_health_check(mut self) -> Self {
		self.enable_health_check = false;
		self
	}

	/// Disable metrics endpoint
	pub fn without_metrics(mut self) -> Self {
		self.enable_metrics = false;
		self
	}
}

/// Server state shared across handlers
#[derive(Clone)]
pub struct ServerState {
	pub metrics: Arc<RwLock<ServerMetrics>>,
}

impl ServerState {
	pub fn new() -> Self {
		Self {
			metrics: Arc::new(RwLock::new(ServerMetrics::new())),
		}
	}
}

impl Default for ServerState {
	fn default() -> Self {
		Self::new()
	}
}

/// Health check handler
pub async fn health_check_handler() -> impl IntoResponse {
	(StatusCode::OK, "OK")
}

/// Metrics handler (simplified - returns basic metrics)
pub async fn metrics_handler() -> impl IntoResponse {
	// For now, return simple metrics
	// TODO: Integrate with metrics middleware to track actual metrics
	let metrics = serde_json::json!({
		"status": "healthy",
		"version": env!("CARGO_PKG_VERSION"),
	});

	(StatusCode::OK, serde_json::to_string(&metrics).unwrap())
}

/// Add system endpoints (health, metrics) to the router
pub fn add_system_endpoints(router: Router, config: &ServerConfig) -> Router {
	let mut router = router;

	// Add health check endpoint
	if config.enable_health_check {
		router = router.route("/health", get(health_check_handler));
	}

	// Add metrics endpoint
	if config.enable_metrics {
		router = router.route("/metrics", get(metrics_handler));
	}

	router
}

/// Start the server with the given configuration
pub async fn start_server(
	app: Router,
	config: ServerConfig,
) -> Result<()> {
	println!("🚀 Frame Server starting on {}", config.addr);

	// Create TCP listener
	let listener = tokio::net::TcpListener::bind(config.addr).await?;

	if config.graceful_shutdown {
		// Start server with graceful shutdown
		println!("✓ Graceful shutdown enabled (timeout: {}s)", config.shutdown_timeout_secs);

		axum::serve(listener, app.into_make_service())
			.with_graceful_shutdown(shutdown_signal())
			.await?;
	} else {
		// Start server without graceful shutdown
		axum::serve(listener, app.into_make_service()).await?;
	}

	Ok(())
}

/// Shutdown signal handler
async fn shutdown_signal() {
	// Wait for Ctrl+C
	tokio::signal::ctrl_c()
		.await
		.expect("Failed to install CTRL+C signal handler");

	println!("\n🛑 Shutdown signal received, draining connections...");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_config_default() {
		let config = ServerConfig::default();
		assert_eq!(config.addr.to_string(), "0.0.0.0:3000");
		assert!(config.graceful_shutdown);
		assert_eq!(config.shutdown_timeout_secs, 30);
		assert!(config.enable_health_check);
		assert!(config.enable_metrics);
	}

	#[test]
	fn test_server_config_builder() {
		let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
		let config = ServerConfig::new(addr)
			.with_shutdown_timeout(60)
			.without_health_check();

		assert_eq!(config.addr.to_string(), "127.0.0.1:8080");
		assert_eq!(config.shutdown_timeout_secs, 60);
		assert!(!config.enable_health_check);
		assert!(config.enable_metrics);
	}

	#[test]
	fn test_server_metrics_default() {
		let metrics = ServerMetrics::default();
		assert_eq!(metrics.total_requests, 0);
		assert_eq!(metrics.active_connections, 0);
		assert_eq!(metrics.uptime_seconds, 0);
		assert_eq!(metrics.avg_response_time_ms, 0.0);
	}

	#[test]
	fn test_server_metrics_record_request() {
		let mut metrics = ServerMetrics::new();

		// Record first request
		metrics.record_request(100.0);
		assert_eq!(metrics.total_requests, 1);
		assert_eq!(metrics.avg_response_time_ms, 100.0);

		// Record second request
		metrics.record_request(200.0);
		assert_eq!(metrics.total_requests, 2);
		assert_eq!(metrics.avg_response_time_ms, 150.0); // Average of 100 and 200
	}

	#[test]
	fn test_server_metrics_connections() {
		let mut metrics = ServerMetrics::new();

		metrics.increment_connections();
		assert_eq!(metrics.active_connections, 1);

		metrics.increment_connections();
		assert_eq!(metrics.active_connections, 2);

		metrics.decrement_connections();
		assert_eq!(metrics.active_connections, 1);

		// Test underflow protection
		metrics.decrement_connections();
		metrics.decrement_connections();
		assert_eq!(metrics.active_connections, 0);
	}

	#[test]
	fn test_server_state_creation() {
		let state = ServerState::new();
		// Just verify it was created successfully
		assert!(Arc::strong_count(&state.metrics) == 1);
	}
}
