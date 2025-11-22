use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use wasmtime::*;

mod runtime;
mod router;
mod middleware;
mod bridge;
mod request;
mod response;
mod server;
mod static_files;
mod ssr;

pub use runtime::{WasmRuntime, RuntimeConfig};
pub use router::{FrameRouter, Route, HttpMethod};
pub use bridge::{BridgeLinker, BridgeState};
pub use request::{Request, RequestBuilder, User};
pub use response::{
	Response, json, json_with_status, html, text, redirect, redirect_permanent,
	not_found, not_found_with_message, bad_request, unauthorized, unauthorized_with_message,
	forbidden, forbidden_with_message, internal_error, validation_error, no_content, custom,
};
pub use middleware::{
	logging_middleware, error_handling_middleware, request_id_middleware, auth_middleware,
	CorsConfig, CorsLayer, CorsMiddleware,
};
pub use server::{
	ServerConfig, ServerMetrics, ServerState,
	health_check_handler, metrics_handler, add_system_endpoints, start_server,
};
pub use static_files::{
	StaticFileConfig, add_static_routes, serve_static_file,
};
pub use ssr::{
	SsrEngine, SsrConfig,
};

/// Frame server that executes WASM modules
pub struct FrameServer {
	runtime: WasmRuntime,
	router: Router,
	addr: SocketAddr,
}

impl FrameServer {
	pub fn new(wasm_path: &str, addr: SocketAddr) -> Result<Self> {
		let runtime = WasmRuntime::new(wasm_path)?;
		let router = Router::new();

		Ok(Self {
			runtime,
			router,
			addr,
		})
	}

	pub async fn start(&self) -> Result<()> {
		// Create server configuration
		let config = ServerConfig::new(self.addr);

		// Add system endpoints (health, metrics) to the router
		let app = add_system_endpoints(self.router.clone(), &config);

		// Start the server
		start_server(app, config).await
	}

	pub fn router(&self) -> &Router {
		&self.router
	}
}
