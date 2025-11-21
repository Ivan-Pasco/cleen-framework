/// Simple Hello World server example
///
/// This example demonstrates the basic Frame Server setup without WASM,
/// showing how to create routes, add middleware, and start the server.
///
/// Run with: cargo run --example hello_world

use frame_server::{
    ServerConfig, add_system_endpoints,
};
use axum::{
    Router,
    routing::get,
    http::StatusCode,
    response::IntoResponse,
};
use anyhow::Result;

async fn hello_handler() -> impl IntoResponse {
    "Hello, Frame Server!"
}

async fn echo_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    format!("Hello, {}!", name)
}

async fn json_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        r#"{"message": "Hello from Frame Server", "version": "0.1.0"}"#,
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Frame Server Hello World example");

    // Create basic router with routes
    let router = Router::new()
        .route("/", get(hello_handler))
        .route("/echo/:name", get(echo_handler))
        .route("/json", get(json_handler));

    // Create server configuration
    let config = ServerConfig::new("0.0.0.0:3000".parse()?);

    // Add system endpoints (/health, /metrics)
    let app = add_system_endpoints(router, &config);

    println!("✓ Server configured");
    println!("  → http://localhost:3000/");
    println!("  → http://localhost:3000/echo/World");
    println!("  → http://localhost:3000/json");
    println!("  → http://localhost:3000/health");
    println!("  → http://localhost:3000/metrics");
    println!("\nPress Ctrl+C to shutdown");

    // Start the server
    frame_server::start_server(app, config).await?;

    Ok(())
}
