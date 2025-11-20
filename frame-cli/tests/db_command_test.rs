use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper to create a minimal Frame project for testing
async fn create_test_project(temp_dir: &TempDir, with_schema: bool) -> Result<PathBuf> {
	let project_path = temp_dir.path();

	// Create minimal Frame project structure
	fs::write(
		project_path.join("package.frame.toml"),
		r#"[package]
name = "test-project"
version = "0.1.0"
"#,
	)?;

	if with_schema {
		// Create db directory with schema file
		fs::create_dir_all(project_path.join("db"))?;
		fs::write(
			project_path.join("db/schema.cln"),
			r#"# Database Schema

class User
	integer id : pk, auto
	string name : min=1, max=80
	string email : email, unique
	boolean active = true
	datetime createdAt : default=now
"#,
		)?;
	}

	Ok(project_path.to_path_buf())
}

#[tokio::test]
async fn test_migration_plan_without_package_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path().to_path_buf();

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to show migration plan without package.frame.toml
	let result = frame_cli::commands::db::show_migration_plan().await;

	std::env::set_current_dir(original_dir)?;

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
async fn test_migration_plan_with_schema() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, true).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Show migration plan
	let result = frame_cli::commands::db::show_migration_plan().await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_ok(), "Should succeed with schema file");

	Ok(())
}

#[tokio::test]
async fn test_migration_plan_without_schema() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, false).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Show migration plan without schema (should succeed but show nothing)
	let result = frame_cli::commands::db::show_migration_plan().await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_ok(), "Should succeed even without schema");

	Ok(())
}

#[tokio::test]
async fn test_apply_migrations_without_package_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path().to_path_buf();

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to apply migrations without package.frame.toml
	let result = frame_cli::commands::db::apply_migrations().await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_err(), "Should fail without package.frame.toml");

	Ok(())
}

#[tokio::test]
async fn test_apply_migrations_creates_migration_files() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, true).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Apply migrations
	let result = frame_cli::commands::db::apply_migrations().await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_ok(), "Should succeed with schema file");

	// Verify migrations directory was created
	assert!(
		project_path.join("db/migrations").exists(),
		"db/migrations directory should exist"
	);

	// Verify migration file was created
	assert!(
		project_path.join("db/migrations/001_schema.sql").exists(),
		"Migration SQL file should exist"
	);

	// Verify tracking file was created
	assert!(
		project_path.join("db/migrations/.tracking.json").exists(),
		"Migration tracking file should exist"
	);

	// Verify tracking file content
	let tracking_content = fs::read_to_string(project_path.join("db/migrations/.tracking.json"))?;
	assert!(
		tracking_content.contains("last_migration"),
		"Tracking file should contain last_migration"
	);
	assert!(
		tracking_content.contains("applied_at"),
		"Tracking file should contain applied_at timestamp"
	);

	Ok(())
}

#[tokio::test]
async fn test_apply_migrations_without_schema() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, false).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Apply migrations without schema (should succeed but do nothing)
	let result = frame_cli::commands::db::apply_migrations().await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_ok(), "Should succeed even without schema");

	// Verify no migrations directory was created
	assert!(
		!project_path.join("db/migrations").exists(),
		"Should not create migrations directory without schema"
	);

	Ok(())
}

#[tokio::test]
async fn test_seed_database_without_package_toml() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = temp_dir.path().to_path_buf();

	// Create a seed file
	let seed_file = project_path.join("seed.cln");
	fs::write(&seed_file, "# Seed data\nclass User")?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to seed without package.frame.toml
	let result = frame_cli::commands::db::seed_database(&seed_file).await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_err(), "Should fail without package.frame.toml");

	Ok(())
}

#[tokio::test]
async fn test_seed_database_with_valid_file() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, false).await?;

	// Create a seed file
	let seed_file = project_path.join("seed.cln");
	fs::write(
		&seed_file,
		r#"# Seed data

class User
	integer id
	string name
"#,
	)?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Seed database
	let result = frame_cli::commands::db::seed_database(&seed_file).await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_ok(), "Should succeed with valid seed file");

	// Verify seeds directory was created
	assert!(
		project_path.join("db/seeds").exists(),
		"db/seeds directory should exist"
	);

	// Verify seed file was copied
	assert!(
		project_path.join("db/seeds/seed.cln").exists(),
		"Seed file should be copied to db/seeds/"
	);

	Ok(())
}

#[tokio::test]
async fn test_seed_database_with_missing_file() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, false).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to seed with non-existent file
	let seed_file = project_path.join("nonexistent.cln");
	let result = frame_cli::commands::db::seed_database(&seed_file).await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_err(), "Should fail with missing seed file");

	let error_message = format!("{:#}", result.unwrap_err());
	assert!(
		error_message.contains("Seed file not found"),
		"Error should mention seed file not found. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_seed_database_with_invalid_format() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, false).await?;

	// Create an invalid seed file (not Clean Language)
	let seed_file = project_path.join("seed.txt");
	fs::write(&seed_file, "This is not a Clean Language file")?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Try to seed with invalid file
	let result = frame_cli::commands::db::seed_database(&seed_file).await;

	std::env::set_current_dir(original_dir)?;

	assert!(result.is_err(), "Should fail with invalid seed file format");

	let error_message = format!("{:#}", result.unwrap_err());
	assert!(
		error_message.contains("Invalid seed file format"),
		"Error should mention invalid format. Got: {}",
		error_message
	);

	Ok(())
}

#[tokio::test]
async fn test_migration_sql_generation() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, true).await?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Apply migrations
	frame_cli::commands::db::apply_migrations().await?;

	std::env::set_current_dir(original_dir)?;

	// Read generated SQL
	let sql_content = fs::read_to_string(project_path.join("db/migrations/001_schema.sql"))?;

	// Verify SQL contains table creation
	assert!(
		sql_content.contains("CREATE TABLE"),
		"SQL should contain CREATE TABLE statement"
	);
	assert!(
		sql_content.contains("user"),
		"SQL should reference the user table"
	);

	Ok(())
}

#[tokio::test]
async fn test_multiple_schema_files() -> Result<()> {
	let temp_dir = TempDir::new()?;
	let project_path = create_test_project(&temp_dir, true).await?;

	// Create additional schema file
	fs::write(
		project_path.join("db/posts.cln"),
		r#"# Posts Schema

class Post
	integer id : pk, auto
	string title
	text content
"#,
	)?;

	let original_dir = std::env::current_dir()?;
	std::env::set_current_dir(&project_path)?;

	// Apply migrations
	frame_cli::commands::db::apply_migrations().await?;

	std::env::set_current_dir(original_dir)?;

	// Verify both migration files were created
	assert!(
		project_path.join("db/migrations/001_schema.sql").exists(),
		"First migration should exist"
	);
	assert!(
		project_path.join("db/migrations/002_posts.sql").exists(),
		"Second migration should exist"
	);

	Ok(())
}
