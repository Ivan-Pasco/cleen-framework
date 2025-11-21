/// Full server example with middleware and static files
///
/// This example demonstrates:
/// - Custom routes
/// - Middleware (logging, CORS, request ID)
/// - Static file serving
/// - System endpoints (health, metrics)
/// - Graceful shutdown
///
/// Run with: cargo run --example full_server

use frame_server::{
    ServerConfig, add_system_endpoints,
    CorsConfig, CorsLayer,
    logging_middleware, request_id_middleware,
    StaticFileConfig, add_static_routes,
};
use axum::{
    Router,
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;

// Example JSON request/response types
#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

// API Handlers
async fn list_users() -> impl IntoResponse {
    let users = vec![
        User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
        User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    ];
    Json(users)
}

async fn get_user(
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    let user = User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    };
    Json(user)
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let user = User {
        id: 3,
        name: payload.name,
        email: payload.email,
    };
    (StatusCode::CREATED, Json(user))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Frame Server Full Example");

    // Create API router
    let api_router = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user));

    // Create main router
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(|| async { "Frame Server Full Example" }));

    // Add middleware
    println!("✓ Configuring middleware...");
    let router = router
        .layer(tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn(logging_middleware))
            .layer(axum::middleware::from_fn(request_id_middleware))
            .layer(CorsLayer::new(CorsConfig::permissive()))
        );

    // Add static file serving (optional - requires ./public directory)
    // let static_config = StaticFileConfig::new("./public")
    //     .with_fallback("index.html");
    // let router = add_static_routes(router, static_config);

    // Add system endpoints
    let config = ServerConfig::new("0.0.0.0:3000".parse()?);
    let app = add_system_endpoints(router, &config);

    println!("✓ Server configured");
    println!("\n📍 Available endpoints:");
    println!("  API:");
    println!("    → GET  http://localhost:3000/api/users");
    println!("    → POST http://localhost:3000/api/users");
    println!("    → GET  http://localhost:3000/api/users/:id");
    println!("  System:");
    println!("    → GET  http://localhost:3000/health");
    println!("    → GET  http://localhost:3000/metrics");
    println!("\n🔧 Features enabled:");
    println!("  ✓ Logging middleware");
    println!("  ✓ Request ID middleware");
    println!("  ✓ CORS (permissive)");
    println!("  ✓ Graceful shutdown");
    println!("\nPress Ctrl+C to shutdown gracefully");

    // Start the server
    frame_server::start_server(app, config).await?;

    Ok(())
}
