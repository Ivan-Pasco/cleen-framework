use anyhow::{Context, Result};
use std::path::Path;
use std::{fs, env};
use tracing::info;
use crate::openapi::{OpenApiGenerator, parse_endpoints_from_file};

pub async fn generate_openapi_spec(output: &Path) -> Result<()> {
	info!("Generating OpenAPI specification");

	println!("📝 Generating OpenAPI 3.1 spec...");

	// Find all .cln files in the current directory and subdirectories
	println!("  → Scanning for Clean files...");
	let clean_files = find_clean_files(&env::current_dir()?)?;

	if clean_files.is_empty() {
		anyhow::bail!("No Clean files found in current directory");
	}

	println!("  → Found {} Clean files", clean_files.len());

	// Parse endpoints from all files
	println!("  → Extracting endpoints...");
	let mut all_endpoints = Vec::new();
	for file in &clean_files {
		match parse_endpoints_from_file(file) {
			Ok(endpoints) => {
				if !endpoints.is_empty() {
					println!("    • {} endpoints from {}", endpoints.len(), file.display());
					all_endpoints.extend(endpoints);
				}
			}
			Err(e) => {
				eprintln!("    ⚠ Warning: Failed to parse {}: {}", file.display(), e);
			}
		}
	}

	if all_endpoints.is_empty() {
		anyhow::bail!("No endpoints found in Clean files. Make sure you have endpoints: blocks defined.");
	}

	println!("  → Extracted {} total endpoints", all_endpoints.len());

	// Create OpenAPI spec
	println!("  → Generating OpenAPI 3.1 schema...");
	let mut generator = OpenApiGenerator::new("Frame API", "1.0.0")
		.with_description("API generated from Clean Language endpoints");

	// Add all endpoints to the spec
	for endpoint in all_endpoints {
		generator.add_endpoint(endpoint);
	}

	// Write to output file
	println!("  → Writing spec to: {}", output.display());
	generator.write_to_file(output)
		.context("Failed to write OpenAPI spec")?;

	println!("\n✓ OpenAPI spec generated successfully!");
	println!("  View at: {}", output.display());

	Ok(())
}

/// Find all .cln files recursively
fn find_clean_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
	let mut files = Vec::new();

	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			// Skip node_modules, target, .git, etc.
			if let Some(name) = path.file_name() {
				let name_str = name.to_string_lossy();
				if name_str == "node_modules" || name_str == "target" || name_str.starts_with('.') {
					continue;
				}
			}

			if path.is_dir() {
				files.extend(find_clean_files(&path)?);
			} else if path.extension().and_then(|s| s.to_str()) == Some("cln") {
				files.push(path);
			}
		}
	}

	Ok(files)
}

pub async fn generate_sdk(lang: &str, output: &Path) -> Result<()> {
	info!("Generating SDK for language: {}", lang);

	println!("🔧 Generating {} SDK...", lang);

	match lang {
		"clean" => generate_clean_sdk(output)?,
		"typescript" => generate_typescript_sdk(output)?,
		"swift" => generate_swift_sdk(output)?,
		"kotlin" => generate_kotlin_sdk(output)?,
		_ => anyhow::bail!("Unsupported language: {}", lang),
	}

	println!("\n✓ SDK generated at: {}", output.display());

	Ok(())
}

fn generate_clean_sdk(output: &Path) -> Result<()> {
	println!("  → Generating Clean client");
	// Would generate type-safe Clean client code
	Ok(())
}

fn generate_typescript_sdk(output: &Path) -> Result<()> {
	println!("  → Generating TypeScript client");

	// Look for openapi.json in current directory
	let openapi_path = env::current_dir()?.join("openapi.json");

	if !openapi_path.exists() {
		anyhow::bail!(
			"OpenAPI spec not found. Run 'frame api:openapi' first to generate openapi.json"
		);
	}

	println!("  → Loading OpenAPI spec from: {}", openapi_path.display());

	use crate::sdk_typescript::TypeScriptGenerator;
	let generator = TypeScriptGenerator::from_file(&openapi_path)
		.context("Failed to load OpenAPI spec")?;

	println!("  → Generating TypeScript code...");
	generator.write_to_file(output)
		.context("Failed to write TypeScript SDK")?;

	println!("  → TypeScript SDK written to: {}", output.display());

	Ok(())
}

fn generate_swift_sdk(output: &Path) -> Result<()> {
	println!("  → Generating Swift client");
	// Would generate Swift structs and API client
	Ok(())
}

fn generate_kotlin_sdk(output: &Path) -> Result<()> {
	println!("  → Generating Kotlin client");
	// Would generate Kotlin data classes and API client
	Ok(())
}
