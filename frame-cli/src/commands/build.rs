use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::compiler::CompilerInvoker;
use crate::manifest::Manifest;

pub async fn build_project(target: &str, release: bool) -> Result<()> {
	info!("Building Frame project for target: {}", target);

	// Validate project structure
	validate_project_structure()?;

	let mode = if release { "production" } else { "development" };
	println!("📦 Building Frame project ({} mode)...", mode);
	println!("🎯 Target: {}", target);

	match target {
		"web" => build_web(release)?,
		"pwa" => build_pwa(release)?,
		"mobile" => build_mobile(release)?,
		"desktop" => build_desktop(release)?,
		"server" => build_server(release)?,
		"cli" => build_cli(release)?,
		_ => anyhow::bail!("Unknown target: {}", target),
	}

	println!("✓ Build complete!");
	println!("\nOutput directory: ./dist/{}", target);

	Ok(())
}

fn validate_project_structure() -> Result<()> {
	// Check if we're in a Frame project directory
	// Try package.clean.toml first, then fall back to package.frame.toml
	if !Path::new("package.clean.toml").exists() && !Path::new("package.frame.toml").exists() {
		anyhow::bail!(
			"Not a Clean Language project. Expected package.clean.toml or package.frame.toml.\n\
			 Create one with: frame new <name>"
		);
	}
	Ok(())
}

fn build_web(_release: bool) -> Result<()> {
	// Load manifest
	let manifest = Manifest::load(Path::new("."))
		.context("Failed to load manifest")?;

	// Get output directory from manifest
	let out_dir = manifest.out_dir().join("web");
	println!("  → Creating output directory: {}", out_dir.display());
	fs::create_dir_all(&out_dir)?;

	// Detect Clean compiler
	println!("  → Detecting Clean compiler");
	let compiler = CompilerInvoker::detect()
		.context("Failed to detect Clean compiler. Please install via: cleen install latest")?;

	println!("  → Found Clean compiler v{}", compiler.version());

	// Find all Clean Language files in source directories
	println!("  → Finding Clean Language files");
	let mut cln_files = Vec::new();
	for src_dir in manifest.src_dirs() {
		if src_dir.exists() {
			let mut files = CompilerInvoker::find_cln_files(&src_dir)?;
			cln_files.append(&mut files);
		}
	}

	// Remove duplicates and sort
	cln_files.sort();
	cln_files.dedup();

	if cln_files.is_empty() {
		anyhow::bail!("No .cln files found in project source directories");
	}

	println!("  → Found {} .cln file(s)", cln_files.len());
	for file in &cln_files {
		println!("     - {}", file.display());
	}

	// Compile backend.wasm
	println!("  → Compiling backend.wasm");
	let backend_wasm = out_dir.join("backend.wasm");
	compiler.compile_project(&cln_files, &backend_wasm)
		.context("Failed to compile backend WASM module")?;

	println!("  → Compiling frontend.wasm");
	// For Phase 1, we use the same WASM for both frontend and backend
	// Phase 2+ will separate these based on client: vs server: blocks
	let frontend_wasm = out_dir.join("frontend.wasm");
	fs::copy(&backend_wasm, &frontend_wasm)
		.context("Failed to copy WASM module for frontend")?;

	println!("  → Copying static assets");
	if Path::new("public").exists() {
		let public_out = out_dir.join("public");
		copy_dir_recursively("public", &public_out)?;
	}

	// Create index.html
	let index_html = out_dir.join("index.html");
	fs::write(
		&index_html,
		r#"<!DOCTYPE html>
<html>
<head>
	<meta charset="UTF-8">
	<title>Frame App</title>
	<script type="module" src="/frontend.js"></script>
</head>
<body>
	<div id="app"></div>
</body>
</html>
"#,
	)?;

	Ok(())
}

fn build_pwa(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/pwa")?;

	println!("  → Building web assets");
	// First build as web
	build_web(false)?;

	// Copy web build to pwa
	copy_dir_recursively("dist/web", "dist/pwa")?;

	println!("  → Generating manifest.json");
	if !Path::new("dist/pwa/manifest.json").exists() {
		fs::write(
			"dist/pwa/manifest.json",
			r##"{
  "name": "Frame PWA",
  "short_name": "FramePWA",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#2563eb",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}
"##,
		)?;
	}

	println!("  → Creating service worker");
	fs::write(
		"dist/pwa/sw.js",
		r#"self.addEventListener('install', e => self.skipWaiting());
self.addEventListener('activate', e => clients.claim());
self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request).then(response => {
      return response || fetch(event.request);
    })
  );
});
"#,
	)?;

	Ok(())
}

fn build_mobile(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/mobile")?;

	println!("  → Building Capacitor wrapper");
	// Build web assets first
	build_web(false)?;

	// Copy to mobile dist
	copy_dir_recursively("dist/web", "dist/mobile/www")?;

	println!("  → Compiling WASM modules");
	// WASM modules already compiled in build_web

	Ok(())
}

fn build_desktop(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/desktop")?;

	println!("  → Building Tauri wrapper");
	// Build web assets first
	build_web(false)?;

	// Copy to desktop dist
	copy_dir_recursively("dist/web", "dist/desktop/www")?;

	println!("  → Compiling WASM modules");
	// WASM modules already compiled in build_web

	Ok(())
}

fn build_server(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/server")?;

	// Detect Clean compiler
	println!("  → Detecting Clean compiler");
	let compiler = CompilerInvoker::detect()
		.context("Failed to detect Clean compiler. Please install via: cleen install latest")?;

	// Find all Clean Language files
	println!("  → Finding Clean Language files");
	let cln_files = CompilerInvoker::find_cln_files(Path::new("."))?;

	if cln_files.is_empty() {
		anyhow::bail!("No .cln files found in project directory");
	}

	println!("  → Compiling backend.wasm");
	compiler.compile_project(&cln_files, Path::new("dist/server/backend.wasm"))
		.context("Failed to compile server WASM module")?;

	println!("  → Creating server runtime");
	fs::write(
		"dist/server/server.js",
		r#"// Frame server runtime
const fs = require('fs');
const wasmBinary = fs.readFileSync('./backend.wasm');
console.log('Server starting...');
"#,
	)?;

	println!("  → Generating Dockerfile");
	fs::write(
		"dist/server/Dockerfile",
		r#"FROM node:20-alpine
WORKDIR /app
COPY . .
EXPOSE 8080
CMD ["node", "server.js"]
"#,
	)?;

	Ok(())
}

fn build_cli(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/cli")?;

	// Detect Clean compiler
	println!("  → Detecting Clean compiler");
	let compiler = CompilerInvoker::detect()
		.context("Failed to detect Clean compiler. Please install via: cleen install latest")?;

	// Find all Clean Language files
	println!("  → Finding Clean Language files");
	let cln_files = CompilerInvoker::find_cln_files(Path::new("."))?;

	if cln_files.is_empty() {
		anyhow::bail!("No .cln files found in project directory");
	}

	println!("  → Compiling CLI binary");
	compiler.compile_project(&cln_files, Path::new("dist/cli/app.wasm"))
		.context("Failed to compile CLI WASM module")?;

	println!("  → Bundling WASM runtime");
	fs::write(
		"dist/cli/runtime.js",
		r#"// Frame CLI runtime
const fs = require('fs');
const wasmBinary = fs.readFileSync('./app.wasm');
console.log('CLI app starting...');
"#,
	)?;

	Ok(())
}

fn copy_dir_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
	fs::create_dir_all(&dst)?;
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let ty = entry.file_type()?;
		if ty.is_dir() {
			copy_dir_recursively(entry.path(), dst.as_ref().join(entry.file_name()))?;
		} else {
			fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}
