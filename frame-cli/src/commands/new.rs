use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

pub async fn create_project(name: &str, template: &str) -> Result<()> {
	info!("Creating new Frame project: {}", name);
	info!("Using template: {}", template);

	let project_path = Path::new(name);

	if project_path.exists() {
		anyhow::bail!("Directory '{}' already exists", name);
	}

	// Create project structure
	create_project_structure(project_path, template)
		.context("Failed to create project structure")?;

	println!("✓ Created Frame project: {}", name);
	println!("\nNext steps:");
	println!("  cd {}", name);
	println!("  frame db:migrate");
	println!("  frame serve");

	Ok(())
}

fn create_project_structure(path: &Path, template: &str) -> Result<()> {
	match template {
		"full-stack" => create_fullstack_template(path),
		"api" => create_api_template(path),
		"ui" => create_ui_template(path),
		_ => anyhow::bail!("Unknown template: {}", template),
	}
}

fn create_fullstack_template(path: &Path) -> Result<()> {
	// Create directory structure
	let dirs = [
		"backend",
		"frontend",
		"db",
		"config",
		"public",
		"public/css",
		"public/js",
		"public/icons",
	];

	for dir in &dirs {
		fs::create_dir_all(path.join(dir))?;
	}

	// Create backend/main.cln
	fs::write(
		path.join("backend/main.cln"),
		r#"# Backend Entry Point

import Data from "frame/data"
import Server from "frame/server"

functions:
	integer main()
		Server.start(port: 3000)
		return 0
"#,
	)?;

	// Create frontend/main.cln
	fs::write(
		path.join("frontend/main.cln"),
		r#"# Frontend Entry Point

import UI from "frame/ui"

functions:
	Widget render()
		return (
			<ui-page title="Welcome to Frame">
				<ui-section>
					<ui-card>
						<h1>Hello, Frame!</h1>
						<p>Your full-stack Clean Language application is ready.</p>
					</ui-card>
				</ui-section>
			</ui-page>
		)
"#,
	)?;

	// Create db/schema.cln
	fs::write(
		path.join("db/schema.cln"),
		r#"# Database Schema

class User
	integer id : pk, auto
	string name : min=1, max=80
	string email : email, unique
	boolean active = true
	datetime createdAt : default=now
"#,
	)?;

	// Create config/app.cln
	fs::write(
		path.join("config/app.cln"),
		r#"# Application Configuration

app:
	name = "My Frame App"
	env = "development"
	port = 3000

db:
	driver = "postgres"
	host = "localhost"
	port = 5432
	database = "frameapp"
	username = env("DB_USER")
	password = env("DB_PASSWORD")
"#,
	)?;

	// Create .env
	fs::write(
		path.join(".env"),
		r#"DB_USER=postgres
DB_PASSWORD=secret
JWT_SECRET=change-this-secret-in-production
"#,
	)?;

	// Create .gitignore
	fs::write(
		path.join(".gitignore"),
		r#"/target
/dist
.env.local
*.wasm
node_modules/
"#,
	)?;

	// Create package.frame.toml
	fs::write(
		path.join("package.frame.toml"),
		format!(
			r#"[package]
name = "{}"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#,
			path.file_name().unwrap().to_str().unwrap()
		),
	)?;

	Ok(())
}

fn create_api_template(path: &Path) -> Result<()> {
	fs::create_dir_all(path.join("backend"))?;
	fs::create_dir_all(path.join("db"))?;
	fs::create_dir_all(path.join("config"))?;

	fs::write(
		path.join("backend/main.cln"),
		r#"# API Backend

import Server from "frame/server"

functions:
	integer main()
		Server.start(port: 8080)
		return 0
"#,
	)?;

	Ok(())
}

fn create_ui_template(path: &Path) -> Result<()> {
	fs::create_dir_all(path.join("frontend"))?;
	fs::create_dir_all(path.join("public"))?;

	fs::write(
		path.join("frontend/main.cln"),
		r#"# UI Application

import UI from "frame/ui"

functions:
	Widget render()
		return <ui-page title="App"><h1>Hello!</h1></ui-page>
"#,
	)?;

	Ok(())
}
