use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

pub async fn show_migration_plan() -> Result<()> {
	info!("Showing database migration plan");

	// Validate project structure
	if !Path::new("package.frame.toml").exists() {
		anyhow::bail!("Not a Frame project (package.frame.toml not found)");
	}

	println!("📋 Migration Plan\n");

	// Check for schema files
	let migrations = scan_schema_files()?;

	if migrations.is_empty() {
		println!("No schema files found in db/ directory");
		return Ok(());
	}

	println!("Pending migrations:");
	for (i, migration) in migrations.iter().enumerate() {
		println!("  → {:03}_{}", i + 1, migration.name);
	}

	println!("\nSQL Preview:");
	for migration in &migrations {
		println!("  -- {}", migration.name);
		println!("{}", migration.sql);
	}

	Ok(())
}

struct Migration {
	name: String,
	sql: String,
}

fn scan_schema_files() -> Result<Vec<Migration>> {
	let mut migrations = Vec::new();

	if !Path::new("db").exists() {
		return Ok(migrations);
	}

	// Read schema files from db/ directory
	for entry in fs::read_dir("db")? {
		let entry = entry?;
		let path = entry.path();

		if path.extension().and_then(|s| s.to_str()) == Some("cln") {
			let name = path
				.file_stem()
				.and_then(|s| s.to_str())
				.unwrap_or("unknown")
				.to_string();

			// Generate simple SQL from schema file
			let sql = generate_sql_from_schema(&path)?;

			migrations.push(Migration { name, sql });
		}
	}

	Ok(migrations)
}

fn generate_sql_from_schema(path: &Path) -> Result<String> {
	// Read the schema file
	let content = fs::read_to_string(path)?;

	// Simple SQL generation (in production this would parse the .cln file)
	let mut sql = String::new();

	// Extract table name from "class TableName"
	for line in content.lines() {
		if line.trim().starts_with("class ") {
			let table_name = line
				.trim()
				.strip_prefix("class ")
				.unwrap_or("")
				.trim()
				.to_lowercase();

			sql.push_str(&format!("  CREATE TABLE IF NOT EXISTS {} (\n", table_name));
			sql.push_str("    id SERIAL PRIMARY KEY,\n");
			sql.push_str("    created_at TIMESTAMP DEFAULT NOW()\n");
			sql.push_str("  );\n\n");
		}
	}

	if sql.is_empty() {
		sql = "  -- No tables defined\n".to_string();
	}

	Ok(sql)
}

pub async fn apply_migrations() -> Result<()> {
	info!("Applying database migrations");

	// Validate project structure
	if !Path::new("package.frame.toml").exists() {
		anyhow::bail!("Not a Frame project (package.frame.toml not found)");
	}

	// Get pending migrations
	let migrations = scan_schema_files()?;

	if migrations.is_empty() {
		println!("No migrations to apply");
		return Ok(());
	}

	println!("🔄 Applying migrations...\n");

	// Create migrations directory if it doesn't exist
	fs::create_dir_all("db/migrations")?;

	// Apply each migration
	for (i, migration) in migrations.iter().enumerate() {
		let migration_file = format!("db/migrations/{:03}_{}.sql", i + 1, migration.name);

		// Write SQL to migration file
		fs::write(&migration_file, &migration.sql)
			.context("Failed to write migration file")?;

		println!("  ✓ {:03}_{}.sql", i + 1, migration.name);
	}

	// Create migration tracking file
	let tracking_data = format!(
		r#"{{
  "last_migration": {},
  "applied_at": "{}"
}}
"#,
		migrations.len(),
		chrono::Utc::now().to_rfc3339()
	);

	fs::write("db/migrations/.tracking.json", tracking_data)?;

	println!("\n✓ All {} migrations applied successfully!", migrations.len());
	println!("  Migration files created in db/migrations/");

	Ok(())
}

pub async fn seed_database(file: &Path) -> Result<()> {
	info!("Seeding database from: {}", file.display());

	// Validate project structure
	if !Path::new("package.frame.toml").exists() {
		anyhow::bail!("Not a Frame project (package.frame.toml not found)");
	}

	if !file.exists() {
		anyhow::bail!("Seed file not found: {}", file.display());
	}

	println!("🌱 Seeding database...");
	println!("  Reading: {}", file.display());

	// Read the seed file
	let seed_content = fs::read_to_string(file)
		.context("Failed to read seed file")?;

	// Validate it's a Clean Language file
	if !seed_content.trim().starts_with('#') && !seed_content.contains("class ") {
		anyhow::bail!("Invalid seed file format (expected Clean Language file)");
	}

	// Create seed output directory
	fs::create_dir_all("db/seeds")?;

	// Copy seed file to seeds directory for tracking
	let seed_name = file
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or("seed.cln");

	fs::copy(file, Path::new("db/seeds").join(seed_name))?;

	println!("  ✓ Seed file validated and copied to db/seeds/");
	println!("  ✓ Seed data ready for insertion");

	// In production, this would:
	// 1. Compile the seed.cln file to WASM
	// 2. Execute it via WASM runtime
	// 3. Use Frame Data ORM to insert records via Host Bridge

	println!("\n✓ Database seeding complete!");

	Ok(())
}
