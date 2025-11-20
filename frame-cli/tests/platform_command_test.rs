use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper to create a test directory for platform commands
fn create_test_dir(temp_dir: &TempDir) -> Result<PathBuf> {
	let project_path = temp_dir.path();
	Ok(project_path.to_path_buf())
}

// Helper to run command in test directory
async fn run_in_dir<F, Fut>(project_path: &PathBuf, f: F) -> Result<()>
where
	F: FnOnce() -> Fut,
	Fut: std::future::Future<Output = Result<()>>,
{
	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(project_path)?;

	let result = f().await;

	std::env::set_current_dir(original_dir)?;

	result
}

#[tokio::test]
async fn test_pwa_init_creates_manifest() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_pwa().await
	})
	.await?;

	// Verify public directory was created
	assert!(
		project_path.join("public").exists(),
		"public directory should exist"
	);

	// Verify manifest.json was created
	assert!(
		project_path.join("public/manifest.json").exists(),
		"manifest.json should exist"
	);

	// Verify manifest content
	let manifest_content = fs::read_to_string(project_path.join("public/manifest.json"))?;
	assert!(
		manifest_content.contains("Frame App"),
		"Manifest should have app name"
	);
	assert!(
		manifest_content.contains("standalone"),
		"Manifest should have display mode"
	);
	assert!(
		manifest_content.contains("icons"),
		"Manifest should have icons array"
	);

	Ok(())
}

#[tokio::test]
async fn test_pwa_init_creates_service_worker() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_pwa().await
	})
	.await?;

	// Verify service worker was created
	assert!(
		project_path.join("public/sw.js").exists(),
		"sw.js should exist"
	);

	// Verify service worker content
	let sw_content = fs::read_to_string(project_path.join("public/sw.js"))?;
	assert!(
		sw_content.contains("addEventListener"),
		"Service worker should have event listeners"
	);
	assert!(
		sw_content.contains("install"),
		"Service worker should handle install event"
	);
	assert!(
		sw_content.contains("fetch"),
		"Service worker should handle fetch event"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_init_creates_capacitor_config() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_mobile().await
	})
	.await?;

	// Verify dist/mobile/capacitor directory was created
	assert!(
		project_path.join("dist/mobile/capacitor").exists(),
		"capacitor directory should exist"
	);

	// Verify capacitor.config.ts was created
	assert!(
		project_path
			.join("dist/mobile/capacitor/capacitor.config.ts")
			.exists(),
		"capacitor.config.ts should exist"
	);

	// Verify config content
	let config_content =
		fs::read_to_string(project_path.join("dist/mobile/capacitor/capacitor.config.ts"))?;
	assert!(
		config_content.contains("CapacitorConfig"),
		"Config should import CapacitorConfig"
	);
	assert!(
		config_content.contains("appId"),
		"Config should have appId"
	);
	assert!(
		config_content.contains("webDir"),
		"Config should have webDir"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_init_creates_package_json() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_mobile().await
	})
	.await?;

	// Verify package.json was created
	assert!(
		project_path
			.join("dist/mobile/capacitor/package.json")
			.exists(),
		"package.json should exist"
	);

	// Verify package.json content
	let package_content =
		fs::read_to_string(project_path.join("dist/mobile/capacitor/package.json"))?;
	assert!(
		package_content.contains("@capacitor/core"),
		"Package should include @capacitor/core"
	);
	assert!(
		package_content.contains("@capacitor/android"),
		"Package should include @capacitor/android"
	);
	assert!(
		package_content.contains("@capacitor/ios"),
		"Package should include @capacitor/ios"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_plugin_camera() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("camera").await
	})
	.await?;

	// Verify bridge stub was created
	assert!(
		project_path.join("bridge/mobile/camera.cln").exists(),
		"camera bridge stub should exist"
	);

	// Verify bridge content
	let bridge_content = fs::read_to_string(project_path.join("bridge/mobile/camera.cln"))?;
	assert!(
		bridge_content.contains("bridge camera"),
		"Bridge should define camera bridge"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_plugin_filesystem() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("filesystem").await
	})
	.await?;

	// Verify bridge stub was created
	assert!(
		project_path.join("bridge/mobile/filesystem.cln").exists(),
		"filesystem bridge stub should exist"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_plugin_invalid() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	let result = run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("invalid-plugin").await
	})
	.await;

	assert!(result.is_err(), "Should fail with unknown plugin");

	let error_message = format!("{:#}", result.unwrap_err());
	assert!(
		error_message.contains("Unknown plugin"),
		"Error should mention unknown plugin. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_desktop_init_creates_tauri_config() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_desktop().await
	})
	.await?;

	// Verify dist/desktop/tauri directory was created
	assert!(
		project_path.join("dist/desktop/tauri/src-tauri").exists(),
		"tauri src-tauri directory should exist"
	);

	// Verify tauri.conf.json was created
	assert!(
		project_path
			.join("dist/desktop/tauri/src-tauri/tauri.conf.json")
			.exists(),
		"tauri.conf.json should exist"
	);

	// Verify config content
	let config_content =
		fs::read_to_string(project_path.join("dist/desktop/tauri/src-tauri/tauri.conf.json"))?;
	assert!(
		config_content.contains("productName"),
		"Config should have productName"
	);
	assert!(
		config_content.contains("Frame App"),
		"Config should have Frame App name"
	);

	Ok(())
}

#[tokio::test]
async fn test_desktop_init_creates_cargo_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_desktop().await
	})
	.await?;

	// Verify Cargo.toml was created
	assert!(
		project_path
			.join("dist/desktop/tauri/src-tauri/Cargo.toml")
			.exists(),
		"Cargo.toml should exist"
	);

	// Verify Cargo.toml content
	let cargo_content =
		fs::read_to_string(project_path.join("dist/desktop/tauri/src-tauri/Cargo.toml"))?;
	assert!(
		cargo_content.contains("[package]"),
		"Cargo.toml should have package section"
	);
	assert!(
		cargo_content.contains("tauri ="),
		"Cargo.toml should have tauri dependency"
	);

	Ok(())
}

#[tokio::test]
async fn test_desktop_adapter() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_desktop_adapter("notifications").await
	})
	.await?;

	// Verify adapter stub was created
	assert!(
		project_path.join("bridge/desktop/notifications.cln").exists(),
		"notifications adapter stub should exist"
	);

	// Verify adapter content
	let adapter_content =
		fs::read_to_string(project_path.join("bridge/desktop/notifications.cln"))?;
	assert!(
		adapter_content.contains("bridge notifications"),
		"Adapter should define notifications bridge"
	);

	Ok(())
}

#[tokio::test]
async fn test_server_init_creates_dockerfile() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_server().await
	})
	.await?;

	// Verify Dockerfile was created
	assert!(
		project_path.join("Dockerfile").exists(),
		"Dockerfile should exist"
	);

	// Verify Dockerfile content
	let dockerfile_content = fs::read_to_string(project_path.join("Dockerfile"))?;
	assert!(
		dockerfile_content.contains("FROM node:20-alpine"),
		"Dockerfile should have Node.js base image"
	);
	assert!(
		dockerfile_content.contains("EXPOSE 8080"),
		"Dockerfile should expose port 8080"
	);

	Ok(())
}

#[tokio::test]
async fn test_server_init_creates_docker_compose() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_server().await
	})
	.await?;

	// Verify docker-compose.yml was created
	assert!(
		project_path.join("docker-compose.yml").exists(),
		"docker-compose.yml should exist"
	);

	// Verify docker-compose content
	let compose_content = fs::read_to_string(project_path.join("docker-compose.yml"))?;
	assert!(
		compose_content.contains("services:"),
		"Compose should have services section"
	);
	assert!(
		compose_content.contains("app:"),
		"Compose should have app service"
	);
	assert!(
		compose_content.contains("db:"),
		"Compose should have db service"
	);
	assert!(
		compose_content.contains("postgres"),
		"Compose should use PostgreSQL"
	);

	Ok(())
}

#[tokio::test]
async fn test_server_init_creates_health_check() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_server().await
	})
	.await?;

	// Verify health check endpoint was created
	assert!(
		project_path.join("backend/health.cln").exists(),
		"health.cln should exist"
	);

	// Verify health check content
	let health_content = fs::read_to_string(project_path.join("backend/health.cln"))?;
	assert!(
		health_content.contains("healthCheck"),
		"Health check should define healthCheck function"
	);
	assert!(
		health_content.contains("status"),
		"Health check should return status"
	);

	Ok(())
}

#[tokio::test]
async fn test_multiple_platform_inits() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	// Initialize multiple platforms
	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_pwa().await
	})
	.await?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_mobile().await
	})
	.await?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::init_desktop().await
	})
	.await?;

	// Verify all platform files exist
	assert!(
		project_path.join("public/manifest.json").exists(),
		"PWA manifest should exist"
	);
	assert!(
		project_path
			.join("dist/mobile/capacitor/capacitor.config.ts")
			.exists(),
		"Mobile config should exist"
	);
	assert!(
		project_path
			.join("dist/desktop/tauri/src-tauri/tauri.conf.json")
			.exists(),
		"Desktop config should exist"
	);

	Ok(())
}

#[tokio::test]
async fn test_mobile_multiple_plugins() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_dir(&temp_dir)?;

	// Add multiple plugins
	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("camera").await
	})
	.await?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("filesystem").await
	})
	.await?;

	run_in_dir(&project_path, || async {
		frame_cli::commands::platform::add_mobile_plugin("push").await
	})
	.await?;

	// Verify all plugin bridges exist
	assert!(
		project_path.join("bridge/mobile/camera.cln").exists(),
		"camera bridge should exist"
	);
	assert!(
		project_path.join("bridge/mobile/filesystem.cln").exists(),
		"filesystem bridge should exist"
	);
	assert!(
		project_path.join("bridge/mobile/push.cln").exists(),
		"push bridge should exist"
	);

	Ok(())
}
