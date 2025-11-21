use anyhow::{Context, Result};
use axum::{
	routing::get_service,
	Router,
};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing::{info, warn};

use crate::manifest::Manifest;

pub async fn start_dev_server(host: &str, port: u16) -> Result<()> {
	// Load manifest to get configuration
	let manifest = Manifest::load(Path::new("."))
		.context("Failed to load manifest")?;

	// Use manifest config if available, otherwise use CLI args
	let actual_host = if let Some(ref frame) = manifest.frame {
		if let Some(ref server) = frame.server {
			server.host.clone()
		} else {
			host.to_string()
		}
	} else {
		host.to_string()
	};

	let actual_port = if let Some(ref frame) = manifest.frame {
		if let Some(ref server) = frame.server {
			server.port
		} else {
			port
		}
	} else {
		port
	};

	info!("Starting Frame development server on {}:{}", actual_host, actual_port);

	println!("🚀 Frame development server starting...");
	println!("   Local:   http://{}:{}", actual_host, actual_port);

	// Build the project first
	println!("\n📦 Building project...");
	let target = manifest.target();
	super::build::build_project(&target, false).await?;

	// Determine serve directory from manifest
	let serve_dir = manifest.out_dir().join(&target);

	println!("\n⚡ Hot reload enabled");
	println!("📂 Serving from: {}", serve_dir.display());
	println!("👀 Watching: .cln files");

	// Set up file watcher for hot reload
	let rebuilding = Arc::new(AtomicBool::new(false));
	setup_file_watcher(rebuilding.clone(), target.clone())?;

	// Start HTTP server
	let addr = format!("{}:{}", actual_host, actual_port)
		.parse::<SocketAddr>()
		.context("Invalid host or port")?;

	serve_static_files(addr, serve_dir).await?;

	Ok(())
}

fn setup_file_watcher(rebuilding: Arc<AtomicBool>, target: String) -> Result<()> {
	info!("Setting up file watcher for .cln files");

	// Create a channel for receiving file system events
	let (tx, rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();

	// Create a watcher
	let mut watcher = notify::recommended_watcher(tx)
		.context("Failed to create file watcher")?;

	// Watch the current directory recursively for .cln files
	watcher.watch(Path::new("."), RecursiveMode::Recursive)
		.context("Failed to watch current directory")?;

	// Spawn a background task to handle file change events
	std::thread::spawn(move || {
		// Keep watcher alive
		let _watcher = watcher;

		// Debounce timer to avoid rebuilding on rapid file changes
		let mut last_rebuild = std::time::Instant::now();
		let debounce_duration = Duration::from_millis(500);

		loop {
			match rx.recv() {
				Ok(Ok(event)) => {
					// Check if this is a file modification event
					if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
						// Check if any .cln files were modified
						let has_cln_changes = event.paths.iter().any(|path| {
							path.extension().and_then(|ext| ext.to_str()) == Some("cln")
						});

						if has_cln_changes {
							// Debounce: only rebuild if enough time has passed
							let now = std::time::Instant::now();
							if now.duration_since(last_rebuild) < debounce_duration {
								continue;
							}

							// Check if already rebuilding
							if rebuilding.swap(true, Ordering::SeqCst) {
								continue; // Already rebuilding, skip
							}

							last_rebuild = now;

							println!("\n🔄 Changes detected, rebuilding...");

							// Trigger rebuild with the target from manifest
							let target_clone = target.clone();
							let rebuild_result = tokio::runtime::Handle::try_current()
								.map(|handle| {
									handle.block_on(async {
										super::build::build_project(&target_clone, false).await
									})
								})
								.unwrap_or_else(|_| {
									// If no tokio runtime, create a new one
									tokio::runtime::Runtime::new()
										.unwrap()
										.block_on(async {
											super::build::build_project(&target_clone, false).await
										})
								});

							match rebuild_result {
								Ok(_) => {
									println!("✓ Rebuild complete\n");
								}
								Err(e) => {
									warn!("❌ Rebuild failed: {}", e);
									eprintln!("❌ Rebuild failed: {}\n", e);
								}
							}

							rebuilding.store(false, Ordering::SeqCst);
						}
					}
				}
				Ok(Err(e)) => {
					warn!("File watcher error: {}", e);
				}
				Err(e) => {
					warn!("Channel error: {}", e);
					break;
				}
			}
		}
	});

	info!("File watcher started successfully");
	Ok(())
}

async fn serve_static_files(addr: SocketAddr, serve_dir: PathBuf) -> Result<()> {
	// Create router for serving static files
	let app = Router::new()
		.nest_service("/", get_service(ServeDir::new(serve_dir)));

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
