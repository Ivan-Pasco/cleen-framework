use frame_server::{
    ServerConfig, add_system_endpoints,
    StaticFileConfig, add_static_routes,
};
use axum::{Router, routing::get, http::StatusCode, body::Body};
use tower::ServiceExt; // for oneshot()
use std::net::SocketAddr;

/// Test helper to create a basic router
fn create_test_router() -> Router {
    Router::new()
        .route("/test", get(|| async { "test response" }))
}

#[tokio::test]
async fn test_health_endpoint() {
    // Create router with system endpoints
    let config = ServerConfig::default();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Make request to /health
    let request = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    // Create router with system endpoints
    let config = ServerConfig::default();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Make request to /metrics
    let request = axum::http::Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Check that it's valid JSON
    assert!(body_str.contains("status"));
    assert!(body_str.contains("version"));
}

#[tokio::test]
async fn test_health_endpoint_disabled() {
    // Create router with health check disabled
    let config = ServerConfig::default().without_health_check();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Make request to /health
    let request = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 404 since health check is disabled
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_metrics_endpoint_disabled() {
    // Create router with metrics disabled
    let config = ServerConfig::default().without_metrics();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Make request to /metrics
    let request = axum::http::Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 404 since metrics is disabled
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_custom_route_still_works() {
    // Create router with system endpoints
    let config = ServerConfig::default();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Make request to custom route
    let request = axum::http::Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify custom route still works
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"test response");
}

#[tokio::test]
async fn test_server_config_builder() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let config = ServerConfig::new(addr)
        .with_shutdown_timeout(60)
        .without_health_check()
        .without_metrics();

    assert_eq!(config.addr.to_string(), "127.0.0.1:8080");
    assert_eq!(config.shutdown_timeout_secs, 60);
    assert!(!config.enable_health_check);
    assert!(!config.enable_metrics);
    assert!(config.graceful_shutdown); // Still enabled by default
}

#[tokio::test]
async fn test_server_config_without_graceful_shutdown() {
    let config = ServerConfig::default().without_graceful_shutdown();
    assert!(!config.graceful_shutdown);
}

#[tokio::test]
async fn test_static_file_config() {
    let config = StaticFileConfig::new("./public")
        .with_fallback("index.html")
        .with_cache_max_age(7200)
        .without_gzip();

    assert_eq!(config.directory.to_str().unwrap(), "./public");
    assert_eq!(config.fallback, Some("index.html".to_string()));
    assert_eq!(config.cache_max_age, 7200);
    assert!(!config.enable_gzip);
    assert!(config.enable_cache); // Still enabled by default
}

#[tokio::test]
async fn test_multiple_system_endpoints() {
    // Create router with all system endpoints enabled
    let config = ServerConfig::default();
    let router = create_test_router();
    let app = add_system_endpoints(router, &config);

    // Test /health
    let request = axum::http::Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test /metrics
    let request = axum::http::Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test custom route
    let request = axum::http::Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
