use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

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

	// Create public directory with a test file
	fs::create_dir_all(project_path.join("public"))?;
	fs::write(project_path.join("public/test.txt"), "test content")?;

	Ok(project_path.to_path_buf())
}

// Helper to run build command
async fn build_project(project_path: &PathBuf, target: &str, release: bool) -> Result<()> {
	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(project_path)?;

	let result = frame_cli::commands::build::build_project(target, release).await;

	std::env::set_current_dir(original_dir)?;

	result
}

#[tokio::test]
async fn test_build_web_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "web", false).await?;

	// Verify dist/web directory structure
	assert!(
		project_path.join("dist/web").exists(),
		"dist/web directory should exist"
	);
	assert!(
		project_path.join("dist/web/backend.wasm").exists(),
		"backend.wasm should exist"
	);
	assert!(
		project_path.join("dist/web/frontend.wasm").exists(),
		"frontend.wasm should exist"
	);
	assert!(
		project_path.join("dist/web/index.html").exists(),
		"index.html should exist"
	);

	// Verify static assets were copied
	assert!(
		project_path.join("dist/web/public/test.txt").exists(),
		"Static assets should be copied"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_pwa_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "pwa", false).await?;

	// Verify dist/pwa directory structure
	assert!(
		project_path.join("dist/pwa").exists(),
		"dist/pwa directory should exist"
	);
	assert!(
		project_path.join("dist/pwa/manifest.json").exists(),
		"manifest.json should exist"
	);
	assert!(
		project_path.join("dist/pwa/sw.js").exists(),
		"service worker should exist"
	);

	// Verify manifest.json content
	let manifest_content = fs::read_to_string(project_path.join("dist/pwa/manifest.json"))?;
	assert!(
		manifest_content.contains("Frame PWA"),
		"Manifest should have correct app name"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_mobile_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "mobile", false).await?;

	// Verify dist/mobile directory structure
	assert!(
		project_path.join("dist/mobile").exists(),
		"dist/mobile directory should exist"
	);
	assert!(
		project_path.join("dist/mobile/www").exists(),
		"www directory should exist"
	);
	assert!(
		project_path.join("dist/mobile/www/backend.wasm").exists(),
		"WASM modules should be in www"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_desktop_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "desktop", false).await?;

	// Verify dist/desktop directory structure
	assert!(
		project_path.join("dist/desktop").exists(),
		"dist/desktop directory should exist"
	);
	assert!(
		project_path.join("dist/desktop/www").exists(),
		"www directory should exist"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_server_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "server", false).await?;

	// Verify dist/server directory structure
	assert!(
		project_path.join("dist/server").exists(),
		"dist/server directory should exist"
	);
	assert!(
		project_path.join("dist/server/backend.wasm").exists(),
		"backend.wasm should exist"
	);
	assert!(
		project_path.join("dist/server/server.js").exists(),
		"server.js should exist"
	);
	assert!(
		project_path.join("dist/server/Dockerfile").exists(),
		"Dockerfile should exist"
	);

	// Verify Dockerfile content
	let dockerfile_content = fs::read_to_string(project_path.join("dist/server/Dockerfile"))?;
	assert!(
		dockerfile_content.contains("FROM node:20-alpine"),
		"Dockerfile should have correct base image"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_cli_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	build_project(&project_path, "cli", false).await?;

	// Verify dist/cli directory structure
	assert!(
		project_path.join("dist/cli").exists(),
		"dist/cli directory should exist"
	);
	assert!(
		project_path.join("dist/cli/app.wasm").exists(),
		"app.wasm should exist"
	);
	assert!(
		project_path.join("dist/cli/runtime.js").exists(),
		"runtime.js should exist"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_without_package_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path().to_path_buf();

	// Try to build without package.frame.toml
	let result = build_project(&project_path, "web", false).await;

	assert!(result.is_err(), "Should fail without package.frame.toml");

	let error_message = format!("{:#}", result.unwrap_err());
	assert!(
		error_message.contains("Not a Frame project"),
		"Error should mention not a Frame project. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_build_invalid_target() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	// Try to build with invalid target
	let result = build_project(&project_path, "invalid-target", false).await;

	assert!(result.is_err(), "Should fail with invalid target");

	let error_message = format!("{:#}", result.unwrap_err());
	assert!(
		error_message.contains("Unknown target"),
		"Error should mention unknown target. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_build_release_mode() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	// Build in release mode
	build_project(&project_path, "web", true).await?;

	// Verify output still created correctly
	assert!(
		project_path.join("dist/web/backend.wasm").exists(),
		"Release build should create output"
	);

	Ok(())
}

#[tokio::test]
async fn test_build_multiple_targets() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir).await?;

	// Build multiple targets
	build_project(&project_path, "web", false).await?;
	build_project(&project_path, "server", false).await?;

	// Verify both exist
	assert!(
		project_path.join("dist/web").exists(),
		"web build should exist"
	);
	assert!(
		project_path.join("dist/server").exists(),
		"server build should exist"
	);

	Ok(())
}
