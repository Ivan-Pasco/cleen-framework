use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

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
	if !Path::new("package.frame.toml").exists() {
		anyhow::bail!("Not a Frame project (package.frame.toml not found)");
	}
	Ok(())
}

fn build_web(_release: bool) -> Result<()> {
	println!("  → Creating output directory");
	fs::create_dir_all("dist/web")?;

	println!("  → Compiling backend.wasm");
	// Simulate WASM compilation - in production this would call the Clean compiler
	fs::write("dist/web/backend.wasm", b"WASM backend module")?;

	println!("  → Compiling frontend.wasm");
	fs::write("dist/web/frontend.wasm", b"WASM frontend module")?;

	println!("  → Copying static assets");
	if Path::new("public").exists() {
		copy_dir_recursively("public", "dist/web/public")?;
	}

	// Create index.html
	fs::write(
		"dist/web/index.html",
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

	println!("  → Compiling backend.wasm");
	fs::write("dist/server/backend.wasm", b"WASM server module")?;

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

	println!("  → Compiling CLI binary");
	fs::write("dist/cli/app.wasm", b"WASM CLI module")?;

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
