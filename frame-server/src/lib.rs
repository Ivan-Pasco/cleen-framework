use anyhow::Result;
use axum::{Router, Server};
use std::net::SocketAddr;
use wasmtime::*;

mod runtime;
mod router;
mod middleware;

pub use runtime::WasmRuntime;
pub use router::FrameRouter;

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
		println!("🚀 Frame server starting on {}", self.addr);

		// TODO: Implement server startup
		// Reference: TASKS.md Phase 3 - Frame Server
		// Spec: documents/specification/03_frame_server.md
		//
		// Implementation steps:
		// 1. Load and compile WASM modules (server.wasm, ui.wasm)
		// 2. Initialize Host Bridge for this runtime
		// 3. Parse manifest.json for route mappings
		// 4. Set up Axum routes using file-based routing
		// 5. Configure middleware (logging, CORS, etc.)
		// 6. Start HTTP server on self.addr
		// 7. Handle graceful shutdown
		//
		// Each request flow:
		// - Parse HTTP request → Clean types
		// - Execute WASM function in isolated context
		// - Bridge calls go through HostBridge
		// - Convert result → HTTP response

		unimplemented!("FrameServer::start - See TASKS.md Phase 3")
	}

	pub fn router(&self) -> &Router {
		&self.router
	}
}
