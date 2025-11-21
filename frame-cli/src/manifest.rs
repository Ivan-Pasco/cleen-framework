use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{debug, warn};

/// Unified manifest for Clean Language projects with Frame Framework support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
	pub package: PackageSection,
	pub frame: Option<FrameSection>,
	pub dependencies: Option<std::collections::HashMap<String, DependencySpec>>,
	pub dev_dependencies: Option<std::collections::HashMap<String, DependencySpec>>,
	pub build: Option<BuildSection>,
	pub profile: Option<ProfileSection>,
}

/// Package metadata (required)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSection {
	pub name: String,
	pub version: String,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default = "default_edition")]
	pub edition: String,
	#[serde(default)]
	pub description: Option<String>,
	#[serde(default)]
	pub license: Option<String>,
	#[serde(default)]
	pub repository: Option<String>,
	#[serde(default)]
	pub homepage: Option<String>,
	#[serde(default)]
	pub documentation: Option<String>,
	#[serde(default)]
	pub keywords: Vec<String>,
	#[serde(default)]
	pub categories: Vec<String>,
}

fn default_edition() -> String {
	"2025".to_string()
}

/// Frame Framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSection {
	pub version: String,
	#[serde(default = "default_target")]
	pub target: String,
	#[serde(default = "default_entry")]
	pub entry: String,
	#[serde(default)]
	pub server: Option<ServerConfig>,
	#[serde(default)]
	pub database: Option<DatabaseConfig>,
	#[serde(default)]
	pub build: Option<FrameBuildConfig>,
	#[serde(default)]
	pub dev: Option<DevConfig>,
}

fn default_target() -> String {
	"web".to_string()
}

fn default_entry() -> String {
	"app.cln".to_string()
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
	#[serde(default = "default_host")]
	pub host: String,
	#[serde(default = "default_port")]
	pub port: u16,
	#[serde(default = "default_true")]
	pub graceful_shutdown: bool,
	#[serde(default = "default_shutdown_timeout")]
	pub shutdown_timeout: u64,
}

fn default_host() -> String {
	"127.0.0.1".to_string()
}

fn default_port() -> u16 {
	3000
}

fn default_true() -> bool {
	true
}

fn default_shutdown_timeout() -> u64 {
	30
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
	pub url: String,
	#[serde(default = "default_migrations")]
	pub migrations: String,
	#[serde(default)]
	pub auto_migrate: bool,
}

fn default_migrations() -> String {
	"db/migrations".to_string()
}

/// Frame build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameBuildConfig {
	#[serde(default = "default_true")]
	pub optimize: bool,
	#[serde(default = "default_true")]
	pub minify: bool,
	#[serde(default = "default_true")]
	pub sourcemap: bool,
	#[serde(default = "default_bundle")]
	pub bundle: String,
}

fn default_bundle() -> String {
	"single".to_string()
}

/// Dev server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
	#[serde(default = "default_true")]
	pub hot_reload: bool,
	#[serde(default = "default_watch")]
	pub watch: Vec<String>,
	#[serde(default = "default_debounce")]
	pub debounce_ms: u64,
}

fn default_watch() -> Vec<String> {
	vec!["**/*.cln".to_string()]
}

fn default_debounce() -> u64 {
	500
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
	/// Simple version string
	Version(String),
	/// Detailed dependency
	Detailed {
		#[serde(default)]
		version: Option<String>,
		#[serde(default)]
		git: Option<String>,
		#[serde(default)]
		tag: Option<String>,
		#[serde(default)]
		branch: Option<String>,
		#[serde(default)]
		path: Option<PathBuf>,
	},
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSection {
	#[serde(default = "default_src")]
	pub src: Vec<String>,
	#[serde(default)]
	pub exclude: Vec<String>,
	#[serde(default = "default_target_arch")]
	pub target_arch: String,
	#[serde(default = "default_opt_level")]
	pub opt_level: u8,
	#[serde(default = "default_out_dir")]
	pub out_dir: String,
}

fn default_src() -> Vec<String> {
	vec!["app".to_string(), "src".to_string(), ".".to_string()]
}

fn default_target_arch() -> String {
	"wasm32".to_string()
}

fn default_opt_level() -> u8 {
	2
}

fn default_out_dir() -> String {
	"dist".to_string()
}

/// Build profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSection {
	#[serde(default)]
	pub release: Option<ReleaseProfile>,
	#[serde(default)]
	pub dev: Option<DevProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseProfile {
	#[serde(default = "default_opt_level_3")]
	pub opt_level: u8,
	#[serde(default = "default_true")]
	pub minify: bool,
	#[serde(default = "default_true")]
	pub strip_debug: bool,
	#[serde(default = "default_true")]
	pub lto: bool,
}

fn default_opt_level_3() -> u8 {
	3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevProfile {
	#[serde(default)]
	pub opt_level: u8,
	#[serde(default = "default_true")]
	pub debug: bool,
	#[serde(default = "default_true")]
	pub incremental: bool,
}

impl Manifest {
	/// Load manifest from a directory
	pub fn load(project_dir: &Path) -> Result<Self> {
		// Try package.clean.toml first
		let clean_path = project_dir.join("package.clean.toml");
		if clean_path.exists() {
			debug!("Loading manifest from: {}", clean_path.display());
			return Self::load_from_file(&clean_path);
		}

		// Fallback to package.frame.toml (deprecated)
		let frame_path = project_dir.join("package.frame.toml");
		if frame_path.exists() {
			warn!("Using deprecated package.frame.toml. Please migrate to package.clean.toml");
			warn!("Run: frame migrate:manifest");
			return Self::load_from_file(&frame_path);
		}

		bail!(
			"No manifest found. Expected package.clean.toml or package.frame.toml in {}",
			project_dir.display()
		)
	}

	/// Load manifest from a specific file
	pub fn load_from_file(path: &Path) -> Result<Self> {
		let contents = fs::read_to_string(path)
			.context(format!("Failed to read manifest: {}", path.display()))?;

		let manifest: Manifest = toml::from_str(&contents)
			.context(format!("Failed to parse manifest: {}", path.display()))?;

		manifest.validate()?;

		Ok(manifest)
	}

	/// Validate the manifest
	pub fn validate(&self) -> Result<()> {
		// Validate package name
		if self.package.name.is_empty() {
			bail!("Package name cannot be empty");
		}

		if !is_valid_package_name(&self.package.name) {
			bail!(
				"Invalid package name '{}'. Must be kebab-case alphanumeric",
				self.package.name
			);
		}

		// Validate version
		if !is_valid_semver(&self.package.version) {
			bail!(
				"Invalid version '{}'. Must follow semantic versioning (major.minor.patch)",
				self.package.version
			);
		}

		// Validate Frame section if present
		if let Some(ref frame) = self.frame {
			if !is_valid_target(&frame.target) {
				bail!(
					"Invalid target '{}'. Must be one of: web, pwa, mobile, desktop, server, cli",
					frame.target
				);
			}

			if !is_valid_semver(&frame.version) {
				bail!(
					"Invalid Frame version '{}'. Must follow semantic versioning",
					frame.version
				);
			}
		}

		Ok(())
	}

	/// Get the entry point file
	pub fn entry_point(&self) -> PathBuf {
		if let Some(ref frame) = self.frame {
			PathBuf::from(&frame.entry)
		} else {
			PathBuf::from("app.cln")
		}
	}

	/// Get the build target
	pub fn target(&self) -> String {
		if let Some(ref frame) = self.frame {
			frame.target.clone()
		} else {
			"web".to_string()
		}
	}

	/// Get source directories
	pub fn src_dirs(&self) -> Vec<PathBuf> {
		if let Some(ref build) = self.build {
			build.src.iter().map(PathBuf::from).collect()
		} else {
			vec![
				PathBuf::from("app"),
				PathBuf::from("src"),
				PathBuf::from("."),
			]
		}
	}

	/// Get output directory
	pub fn out_dir(&self) -> PathBuf {
		if let Some(ref build) = self.build {
			PathBuf::from(&build.out_dir)
		} else {
			PathBuf::from("dist")
		}
	}

	/// Check if Frame is enabled
	pub fn is_frame_project(&self) -> bool {
		self.frame.is_some()
	}
}

/// Validate package name (kebab-case, alphanumeric + hyphens)
fn is_valid_package_name(name: &str) -> bool {
	if name.is_empty() || name.len() > 64 {
		return false;
	}

	// Must start with letter
	if !name.chars().next().unwrap().is_ascii_alphabetic() {
		return false;
	}

	// Only lowercase letters, numbers, and hyphens
	name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Validate semantic version
fn is_valid_semver(version: &str) -> bool {
	let parts: Vec<&str> = version.split('.').collect();
	if parts.len() != 3 {
		return false;
	}

	// Each part must be a number
	parts.iter().all(|part| {
		// Remove any pre-release or build metadata
		let num_part = part.split('-').next().unwrap().split('+').next().unwrap();
		num_part.parse::<u32>().is_ok()
	})
}

/// Validate build target
fn is_valid_target(target: &str) -> bool {
	matches!(target, "web" | "pwa" | "mobile" | "desktop" | "server" | "cli")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_valid_package_name() {
		assert!(is_valid_package_name("my-app"));
		assert!(is_valid_package_name("hello-world"));
		assert!(is_valid_package_name("app123"));

		assert!(!is_valid_package_name("MyApp")); // uppercase
		assert!(!is_valid_package_name("my_app")); // underscore
		assert!(!is_valid_package_name("123app")); // starts with number
		assert!(!is_valid_package_name("")); // empty
	}

	#[test]
	fn test_valid_semver() {
		assert!(is_valid_semver("0.1.0"));
		assert!(is_valid_semver("1.2.3"));
		assert!(is_valid_semver("10.20.30"));
		assert!(is_valid_semver("1.0.0-beta"));
		assert!(is_valid_semver("1.0.0+build.123"));

		assert!(!is_valid_semver("1.0")); // missing patch
		assert!(!is_valid_semver("1")); // missing minor and patch
		assert!(!is_valid_semver("a.b.c")); // not numbers
	}

	#[test]
	fn test_valid_target() {
		assert!(is_valid_target("web"));
		assert!(is_valid_target("pwa"));
		assert!(is_valid_target("mobile"));
		assert!(is_valid_target("desktop"));
		assert!(is_valid_target("server"));
		assert!(is_valid_target("cli"));

		assert!(!is_valid_target("invalid"));
		assert!(!is_valid_target("Web")); // case sensitive
	}

	#[test]
	fn test_parse_minimal_manifest() {
		let toml = r#"
[package]
name = "hello-world"
version = "0.1.0"
"#;

		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert_eq!(manifest.package.name, "hello-world");
		assert_eq!(manifest.package.version, "0.1.0");
		assert_eq!(manifest.package.edition, "2025");
		assert!(manifest.frame.is_none());
	}

	#[test]
	fn test_parse_full_manifest() {
		let toml = r#"
[package]
name = "my-app"
version = "1.0.0"
authors = ["Alice <alice@example.com>"]
edition = "2025"
description = "My Frame app"
license = "MIT"

[frame]
version = "0.1.0"
target = "web"
entry = "app.cln"

[frame.server]
host = "0.0.0.0"
port = 8080

[frame.database]
url = "sqlite:./dev.db"

[build]
src = ["app", "src"]
out_dir = "dist"
"#;

		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert_eq!(manifest.package.name, "my-app");
		assert!(manifest.frame.is_some());

		let frame = manifest.frame.unwrap();
		assert_eq!(frame.version, "0.1.0");
		assert_eq!(frame.target, "web");

		let server = frame.server.unwrap();
		assert_eq!(server.host, "0.0.0.0");
		assert_eq!(server.port, 8080);
	}
}
