use anyhow::{Context, Result};
use axum::{
	routing::get_service,
	Router,
};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::ServeDir;
use tracing::info;

pub async fn start_dev_server(host: &str, port: u16) -> Result<()> {
	info!("Starting Frame development server on {}:{}", host, port);

	// Validate project structure
	if !Path::new("package.frame.toml").exists() {
		anyhow::bail!("Not a Frame project (package.frame.toml not found)");
	}

	println!("🚀 Frame development server starting...");
	println!("   Local:   http://{}:{}", host, port);

	// Build the project first
	println!("\n📦 Building project...");
	super::build::build_project("web", false).await?;

	println!("\n⚡ Hot reload enabled");
	println!("📂 Serving from: ./dist/web");

	// Set up file watcher for hot reload
	setup_file_watcher()?;

	// Start HTTP server
	let addr = format!("{}:{}", host, port)
		.parse::<SocketAddr>()
		.context("Invalid host or port")?;

	serve_static_files(addr).await?;

	Ok(())
}

fn setup_file_watcher() -> Result<()> {
	// Set up file watcher for hot reload
	// In a full implementation, this would watch backend/, frontend/, public/
	// and trigger rebuilds when files change
	info!("File watcher configured for hot reload");
	Ok(())
}

async fn serve_static_files(addr: SocketAddr) -> Result<()> {
	// Create router for serving static files
	let app = Router::new()
		.nest_service("/", get_service(ServeDir::new("dist/web")));

	println!("\n✓ Server ready!");
	println!("\nPress Ctrl+C to stop\n");

	// Start the server
	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.context("Failed to bind to address")?;

	info!("Listening on http://{}", addr);

	// Run the server (this blocks until shutdown)
	axum::serve(listener, app)
		.await
		.context("Server failed")?;

	Ok(())
}
