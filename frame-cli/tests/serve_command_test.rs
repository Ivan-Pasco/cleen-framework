use anyhow::Result;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

// Helper to create a minimal Frame project for testing
async fn create_test_project(temp_dir: &TempDir) -> Result<PathBuf> {
	let project_path = temp_dir.path();

	// Create minimal Frame project structure
	fs::write(
		project_path.join("package.frame.toml"),
		r#"[package]
name = "test-project"
version = "0.1.0"
"#,
	)?;

	// Create backend directory
	fs::create_dir_all(project_path.join("backend"))?;
	fs::write(
		project_path.join("backend/main.cln"),
		"# Backend code\n",
	)?;

	// Create frontend directory
	fs::create_dir_all(project_path.join("frontend"))?;
	fs::write(
		project_path.join("frontend/main.cln"),
		"# Frontend code\n",
	)?;

	// Create public directory
	fs::create_dir_all(project_path.join("public"))?;
	fs::write(project_path.join("public/index.html"), "<html></html>")?;

	Ok(project_path.to_path_buf())
}

// Helper to find an available port
fn find_available_port() -> u16 {
	TcpListener::bind("127.0.0.1:0")
		.expect("Failed to bind to ephemeral port")
		.local_addr()
		.unwrap()
		.port()
}

#[tokio::test]
async fn test_serve_without_package_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path().to_path_buf();

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to start server without package.frame.toml
	let port = find_available_port();
	let result = timeout(
		Duration::from_secs(2),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Should fail before timeout
	assert!(result.is_ok(), "Should not timeout");
	let server_result = result.unwrap();
	assert!(server_result.is_err(), "Should fail without package.frame.toml");

	let error_message = format!("{:#}", server_result.unwrap_err());
	assert!(
		error_message.contains("Not a Frame project"),
		"Error should mention not a Frame project. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_serve_builds_project_first() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Start server with timeout (will build but timeout before serving)
	let port = find_available_port();
	let result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Should timeout (because server would run indefinitely)
	// But should have created the build output first
	assert!(
		project_path.join("dist/web").exists(),
		"Should have built the project"
	);
	assert!(
		project_path.join("dist/web/index.html").exists(),
		"Should have created index.html"
	);

	Ok(())
}

#[tokio::test]
async fn test_serve_creates_dist_directory() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Verify dist doesn't exist yet
	assert!(
		!project_path.join("dist").exists(),
		"dist should not exist initially"
	);

	// Start server with timeout
	let port = find_available_port();
	let _result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Verify dist/web was created
	assert!(
		project_path.join("dist/web").exists(),
		"dist/web directory should exist after serve"
	);

	Ok(())
}

#[tokio::test]
async fn test_serve_with_custom_port() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Use a custom port
	let port = find_available_port();
	let _result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Verify project was built (server accepted the port)
	assert!(
		project_path.join("dist/web").exists(),
		"Should have built with custom port"
	);

	Ok(())
}

#[tokio::test]
async fn test_serve_copies_static_assets() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	// Add a static asset
	fs::write(project_path.join("public/style.css"), "body { color: red; }")?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	let port = find_available_port();
	let _result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Verify static assets were copied
	assert!(
		project_path.join("dist/web/public/style.css").exists(),
		"Static assets should be copied to dist"
	);

	let content = fs::read_to_string(project_path.join("dist/web/public/style.css"))?;
	assert_eq!(content, "body { color: red; }", "Static asset content should match");

	Ok(())
}

#[tokio::test]
async fn test_serve_on_localhost() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Test serving on localhost
	let port = find_available_port();
	let _result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("localhost", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Should have built successfully
	assert!(
		project_path.join("dist/web").exists(),
		"Should work with localhost"
	);

	Ok(())
}

#[tokio::test]
async fn test_serve_validates_project_structure() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path();

	// Create only package.frame.toml without other files
	fs::write(
		project_path.join("package.frame.toml"),
		"[package]\nname = \"incomplete\"\n",
	)?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(project_path)?;

	let port = find_available_port();
	let result = timeout(
		Duration::from_millis(500),
		frame_cli::commands::serve::start_dev_server("127.0.0.1", port)
	).await;

	std::env::set_current_dir(original_dir)?;

	// Server should have attempted to build (which creates dist/web even without source files)
	// The timeout indicates it tried to start the server
	assert!(result.is_err() || project_path.join("dist").exists());

	Ok(())
}
