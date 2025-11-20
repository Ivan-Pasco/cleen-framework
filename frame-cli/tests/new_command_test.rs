use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper to run the new command
async fn create_project(temp_dir: &TempDir, name: &str, template: &str) -> Result<PathBuf> {
	// Change to temp directory
	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(temp_dir.path())?;

	// Create project (will create in current dir = temp_dir)
	let result = frame_cli::commands::new::create_project(name, template).await;

	// Restore original directory
	std::env::set_current_dir(original_dir)?;

	// Return result
	result?;
	Ok(temp_dir.path().join(name))
}

#[tokio::test]
async fn test_new_fullstack_template() -> Result<()> {
	let temp_dir = TempDir::new()?;

	// Create project
	let project_path = create_project(&temp_dir, "test-fullstack", "full-stack").await?;

	// Verify directory structure
	assert!(project_path.exists(), "Project directory should exist");
	assert!(project_path.join("backend").exists(), "backend/ should exist");
	assert!(project_path.join("frontend").exists(), "frontend/ should exist");
	assert!(project_path.join("db").exists(), "db/ should exist");
	assert!(project_path.join("config").exists(), "config/ should exist");
	assert!(project_path.join("public").exists(), "public/ should exist");
	assert!(project_path.join("public/css").exists(), "public/css/ should exist");
	assert!(project_path.join("public/js").exists(), "public/js/ should exist");
	assert!(project_path.join("public/icons").exists(), "public/icons/ should exist");

	// Verify files exist
	assert!(project_path.join("backend/main.cln").exists(), "backend/main.cln should exist");
	assert!(project_path.join("frontend/main.cln").exists(), "frontend/main.cln should exist");
	assert!(project_path.join("db/schema.cln").exists(), "db/schema.cln should exist");
	assert!(project_path.join("config/app.cln").exists(), "config/app.cln should exist");
	assert!(project_path.join(".env").exists(), ".env should exist");
	assert!(project_path.join(".gitignore").exists(), ".gitignore should exist");
	assert!(project_path.join("package.frame.toml").exists(), "package.frame.toml should exist");

	// Verify backend/main.cln content
	let backend_content = fs::read_to_string(project_path.join("backend/main.cln"))?;
	assert!(backend_content.contains("import Data from \"frame/data\""), "Backend should import Data");
	assert!(backend_content.contains("import Server from \"frame/server\""), "Backend should import Server");
	assert!(backend_content.contains("Server.start(port: 3000)"), "Backend should start server");

	// Verify frontend/main.cln content
	let frontend_content = fs::read_to_string(project_path.join("frontend/main.cln"))?;
	assert!(frontend_content.contains("import UI from \"frame/ui\""), "Frontend should import UI");
	assert!(frontend_content.contains("Widget render()"), "Frontend should have render function");
	assert!(frontend_content.contains("<ui-page"), "Frontend should use ui-page component");

	// Verify db/schema.cln content
	let schema_content = fs::read_to_string(project_path.join("db/schema.cln"))?;
	assert!(schema_content.contains("class User"), "Schema should define User class");
	assert!(schema_content.contains("integer id : pk, auto"), "Schema should have id field");
	assert!(schema_content.contains("string email : email, unique"), "Schema should have email field");

	// Verify config/app.cln content
	let config_content = fs::read_to_string(project_path.join("config/app.cln"))?;
	assert!(config_content.contains("app:"), "Config should have app block");
	assert!(config_content.contains("db:"), "Config should have db block");
	assert!(config_content.contains("driver = \"postgres\""), "Config should specify postgres driver");

	// Verify .env content
	let env_content = fs::read_to_string(project_path.join(".env"))?;
	assert!(env_content.contains("DB_USER="), "Env should have DB_USER");
	assert!(env_content.contains("DB_PASSWORD="), "Env should have DB_PASSWORD");
	assert!(env_content.contains("JWT_SECRET="), "Env should have JWT_SECRET");

	// Verify .gitignore content
	let gitignore_content = fs::read_to_string(project_path.join(".gitignore"))?;
	assert!(gitignore_content.contains("/target"), "Gitignore should ignore /target");
	assert!(gitignore_content.contains("/dist"), "Gitignore should ignore /dist");
	assert!(gitignore_content.contains(".env.local"), "Gitignore should ignore .env.local");

	// Verify package.frame.toml content
	let package_content = fs::read_to_string(project_path.join("package.frame.toml"))?;
	assert!(package_content.contains("[package]"), "Package should have package block");
	assert!(package_content.contains("name = \"test-fullstack\""), "Package should have correct name");
	assert!(package_content.contains("version = \"0.1.0\""), "Package should have version");

	// Restore original directory

	Ok(())
}

#[tokio::test]
async fn test_new_api_template() -> Result<()> {
	let temp_dir = TempDir::new()?;

	let project_path = create_project(&temp_dir, "test-api", "api").await?;

	// Verify API-only structure
	assert!(project_path.exists(), "Project directory should exist");
	assert!(project_path.join("backend").exists(), "backend/ should exist");
	assert!(project_path.join("db").exists(), "db/ should exist");
	assert!(project_path.join("config").exists(), "config/ should exist");

	// Should NOT have frontend
	assert!(!project_path.join("frontend").exists(), "frontend/ should NOT exist for API template");

	// Verify backend file exists
	assert!(project_path.join("backend/main.cln").exists(), "backend/main.cln should exist");

	// Verify backend content
	let backend_content = fs::read_to_string(project_path.join("backend/main.cln"))?;
	assert!(backend_content.contains("import Server from \"frame/server\""), "API backend should import Server");
	assert!(backend_content.contains("Server.start(port: 8080)"), "API backend should start on port 8080");

	Ok(())
}

#[tokio::test]
async fn test_new_ui_template() -> Result<()> {
	let temp_dir = TempDir::new()?;

	let project_path = create_project(&temp_dir, "test-ui", "ui").await?;

	// Verify UI-only structure
	assert!(project_path.exists(), "Project directory should exist");
	assert!(project_path.join("frontend").exists(), "frontend/ should exist");
	assert!(project_path.join("public").exists(), "public/ should exist");

	// Should NOT have backend or db
	assert!(!project_path.join("backend").exists(), "backend/ should NOT exist for UI template");
	assert!(!project_path.join("db").exists(), "db/ should NOT exist for UI template");

	// Verify frontend file exists
	assert!(project_path.join("frontend/main.cln").exists(), "frontend/main.cln should exist");

	// Verify frontend content
	let frontend_content = fs::read_to_string(project_path.join("frontend/main.cln"))?;
	assert!(frontend_content.contains("import UI from \"frame/ui\""), "UI frontend should import UI");
	assert!(frontend_content.contains("<ui-page"), "UI frontend should use ui-page");

	Ok(())
}

#[tokio::test]
async fn test_new_rejects_existing_directory() -> Result<()> {
	let temp_dir = TempDir::new()?;

	// Create project first time
	create_project(&temp_dir, "test-existing", "full-stack").await?;

	// Try to create again - should fail
	let result = create_project(&temp_dir, "test-existing", "full-stack").await;
	assert!(result.is_err(), "Should reject existing directory");

	let error_message = result.unwrap_err().to_string();
	assert!(
		error_message.contains("already exists"),
		"Error should mention directory already exists"
	);

	Ok(())
}

#[tokio::test]
async fn test_new_rejects_invalid_template() -> Result<()> {
	let temp_dir = TempDir::new()?;

	// Try to create with invalid template
	let result = create_project(&temp_dir, "test-invalid", "invalid-template").await;
	assert!(result.is_err(), "Should reject invalid template");

	let error = result.unwrap_err();
	// Use {:#} to get full error chain
	let error_message = format!("{:#}", error);
	assert!(
		error_message.contains("Unknown template"),
		"Error should mention unknown template. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_new_creates_valid_package_name() -> Result<()> {
	let temp_dir = TempDir::new()?;

	let project_path = create_project(&temp_dir, "my-awesome-app", "full-stack").await?;

	// Verify package name is correct in package.frame.toml
	let package_content = fs::read_to_string(project_path.join("package.frame.toml"))?;
	assert!(
		package_content.contains("name = \"my-awesome-app\""),
		"Package name should match project name"
	);

	Ok(())
}

#[tokio::test]
async fn test_new_creates_all_required_directories() -> Result<()> {
	let temp_dir = TempDir::new()?;

	let project_path = create_project(&temp_dir, "test-dirs", "full-stack").await?;

	// Verify all required directories exist
	let required_dirs = vec![
		"backend",
		"frontend",
		"db",
		"config",
		"public",
		"public/css",
		"public/js",
		"public/icons",
	];

	for dir in required_dirs {
		let dir_path = project_path.join(dir);
		assert!(
			dir_path.exists() && dir_path.is_dir(),
			"Directory {} should exist",
			dir
		);
	}

	Ok(())
}

#[tokio::test]
async fn test_new_creates_all_required_files() -> Result<()> {
	let temp_dir = TempDir::new()?;

	let project_path = create_project(&temp_dir, "test-files", "full-stack").await?;

	// Verify all required files exist
	let required_files = vec![
		"backend/main.cln",
		"frontend/main.cln",
		"db/schema.cln",
		"config/app.cln",
		".env",
		".gitignore",
		"package.frame.toml",
	];

	for file in required_files {
		let file_path = project_path.join(file);
		assert!(
			file_path.exists() && file_path.is_file(),
			"File {} should exist",
			file
		);

		// Verify file is not empty
		let content = fs::read_to_string(&file_path)?;
		assert!(!content.is_empty(), "File {} should not be empty", file);
	}

	Ok(())
}
