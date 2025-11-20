# Pull Request: Add Frame Framework Support to Clean Manager

## Overview

This document outlines the code changes needed to add Frame Framework installation support to the Clean Language Manager (`cleen`).

**Target Repository:** https://github.com/Ivan-Pasco/clean-language-manager

**Feature:** Add `cleen install frame` command to install Frame Framework alongside Clean compiler

## Requirements

### User Stories

1. **As a Clean developer**, I want to install Frame Framework using `cleen install frame` so I can build full-stack applications
2. **As a Clean developer**, I want `cleen list frameworks` to show available frameworks
3. **As a Clean developer**, I want `cleen uninstall frame` to remove Frame Framework
4. **As a Clean developer**, I want `cleen use frame latest` to switch between framework versions

### Command Specification

```bash
# List available frameworks
cleen list frameworks
# Output: frame, frame-ui, frame-data (future frameworks)

# Install latest Frame version
cleen install frame
cleen install frame@latest

# Install specific Frame version
cleen install frame@1.0.0

# Show installed frameworks
cleen list installed

# Uninstall framework
cleen uninstall frame 1.0.0

# Use specific framework version (set as default)
cleen use frame 1.0.0
cleen use frame latest
```

## Implementation Strategy

### 1. Directory Structure

Extend `cleen` to manage frameworks alongside compiler versions:

```
~/.cleen/
├── versions/
│   ├── 0.5.0/          # Clean compiler versions
│   ├── 0.5.1/
│   └── frameworks/     # NEW: Framework installations
│       └── frame/
│           ├── 1.0.0/
│           │   ├── bin/
│           │   │   └── frame
│           │   ├── lib/
│           │   │   └── *.wasm
│           │   └── templates/
│           └── latest -> 1.0.0
├── bin/
│   ├── cln -> ../versions/0.5.1/bin/cln
│   └── frame -> ../versions/frameworks/frame/latest/bin/frame  # NEW
└── config.toml
```

### 2. Configuration Updates

Extend `config.toml` to track frameworks:

```toml
[compiler]
current_version = "0.5.1"

[frameworks]  # NEW section
installed = ["frame@1.0.0"]
current = { frame = "1.0.0" }

[frameworks.frame]  # NEW: Framework-specific config
version = "1.0.0"
install_date = "2025-01-15T10:30:00Z"
```

### 3. Manifest Format

Frame Framework will provide a version manifest at:
`https://github.com/Ivan-Pasco/cleen-framework/releases/latest/download/version-manifest.json`

```json
{
  "version": "1.0.0",
  "released_at": "2025-01-15T12:00:00Z",
  "platforms": {
    "linux-x86_64": "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v1.0.0/frame-linux-x86_64.tar.gz",
    "linux-aarch64": "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v1.0.0/frame-linux-aarch64.tar.gz",
    "macos-x86_64": "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v1.0.0/frame-macos-x86_64.tar.gz",
    "macos-aarch64": "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v1.0.0/frame-macos-aarch64.tar.gz",
    "windows-x86_64": "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v1.0.0/frame-windows-x86_64.zip"
  }
}
```

## Code Changes

### 1. New Module: `src/framework.rs`

```rust
use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameworkManifest {
    pub version: String,
    pub released_at: String,
    pub platforms: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub struct FrameworkManager {
    pub cleen_home: PathBuf,
}

impl FrameworkManager {
    pub fn new(cleen_home: PathBuf) -> Self {
        Self { cleen_home }
    }

    pub fn frameworks_dir(&self) -> PathBuf {
        self.cleen_home.join("versions").join("frameworks")
    }

    pub fn framework_dir(&self, name: &str, version: &str) -> PathBuf {
        self.frameworks_dir().join(name).join(version)
    }

    pub async fn install_framework(
        &self,
        name: &str,
        version: Option<String>,
    ) -> Result<()> {
        let version = match version {
            Some(v) => v,
            None => self.get_latest_version(name).await?,
        };

        println!("Installing {} v{}...", name, version);

        // Fetch manifest
        let manifest = self.fetch_manifest(name, &version).await?;

        // Determine platform
        let platform = self.detect_platform()?;

        // Get download URL
        let download_url = manifest
            .platforms
            .get(&platform)
            .ok_or_else(|| anyhow::anyhow!("Platform {} not supported", platform))?;

        // Download archive
        let archive_path = self.download_archive(name, &version, download_url).await?;

        // Extract archive
        self.extract_archive(&archive_path, name, &version)?;

        // Create symlink for "latest"
        self.create_latest_symlink(name, &version)?;

        // Update PATH symlink
        self.update_bin_symlink(name, &version)?;

        println!("✓ {} v{} installed successfully", name, version);
        println!("Run '{} --version' to verify installation", name);

        Ok(())
    }

    pub async fn fetch_manifest(
        &self,
        name: &str,
        version: &str,
    ) -> Result<FrameworkManifest> {
        let manifest_url = format!(
            "https://github.com/Ivan-Pasco/cleen-framework/releases/download/v{}/version-manifest.json",
            version
        );

        let response = reqwest::get(&manifest_url)
            .await
            .context("Failed to fetch framework manifest")?;

        let manifest: FrameworkManifest = response
            .json()
            .await
            .context("Failed to parse framework manifest")?;

        Ok(manifest)
    }

    pub async fn get_latest_version(&self, name: &str) -> Result<String> {
        // Fetch manifest from "latest" release
        let manifest_url = format!(
            "https://github.com/Ivan-Pasco/cleen-framework/releases/latest/download/version-manifest.json"
        );

        let response = reqwest::get(&manifest_url)
            .await
            .context("Failed to fetch latest version")?;

        let manifest: FrameworkManifest = response
            .json()
            .await
            .context("Failed to parse version manifest")?;

        Ok(manifest.version)
    }

    pub fn detect_platform(&self) -> Result<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let platform = match (os, arch) {
            ("linux", "x86_64") => "linux-x86_64",
            ("linux", "aarch64") => "linux-aarch64",
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-aarch64",
            ("windows", "x86_64") => "windows-x86_64",
            _ => return Err(anyhow::anyhow!("Unsupported platform: {}-{}", os, arch)),
        };

        Ok(platform.to_string())
    }

    pub async fn download_archive(
        &self,
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<PathBuf> {
        println!("Downloading from {}...", url);

        let response = reqwest::get(url)
            .await
            .context("Failed to download archive")?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to read archive bytes")?;

        // Determine archive extension
        let extension = if url.ends_with(".zip") { "zip" } else { "tar.gz" };
        let archive_name = format!("{}-{}.{}", name, version, extension);
        let archive_path = self.cleen_home.join(&archive_name);

        fs::write(&archive_path, bytes)?;

        println!("✓ Downloaded {}", archive_name);

        Ok(archive_path)
    }

    pub fn extract_archive(
        &self,
        archive_path: &PathBuf,
        name: &str,
        version: &str,
    ) -> Result<()> {
        println!("Extracting archive...");

        let framework_dir = self.framework_dir(name, version);
        fs::create_dir_all(&framework_dir)?;

        if archive_path.extension().unwrap_or_default() == "zip" {
            // Extract ZIP (Windows)
            let file = fs::File::open(archive_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            archive.extract(&self.frameworks_dir())?;
        } else {
            // Extract tar.gz (Unix)
            let file = fs::File::open(archive_path)?;
            let decompressor = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decompressor);
            archive.unpack(&self.frameworks_dir())?;
        }

        // Remove archive
        fs::remove_file(archive_path)?;

        println!("✓ Extracted to {}", framework_dir.display());

        Ok(())
    }

    pub fn create_latest_symlink(&self, name: &str, version: &str) -> Result<()> {
        let latest_path = self.frameworks_dir().join(name).join("latest");
        let version_path = self.framework_dir(name, version);

        // Remove existing symlink if present
        if latest_path.exists() {
            fs::remove_file(&latest_path)?;
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&version_path, &latest_path)?;
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&version_path, &latest_path)?;
        }

        Ok(())
    }

    pub fn update_bin_symlink(&self, name: &str, version: &str) -> Result<()> {
        let bin_dir = self.cleen_home.join("bin");
        fs::create_dir_all(&bin_dir)?;

        let binary_name = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };

        let symlink_path = bin_dir.join(&binary_name);
        let target_path = self
            .framework_dir(name, version)
            .join("bin")
            .join(&binary_name);

        // Remove existing symlink
        if symlink_path.exists() {
            fs::remove_file(&symlink_path)?;
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target_path, &symlink_path)?;
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(&target_path, &symlink_path)?;
        }

        Ok(())
    }

    pub fn list_installed(&self) -> Result<Vec<(String, String)>> {
        let mut installed = Vec::new();

        let frameworks_dir = self.frameworks_dir();
        if !frameworks_dir.exists() {
            return Ok(installed);
        }

        for entry in fs::read_dir(&frameworks_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name == "latest" {
                continue;
            }

            let framework_dir = entry.path();
            for version_entry in fs::read_dir(&framework_dir)? {
                let version_entry = version_entry?;
                let version = version_entry.file_name().to_string_lossy().to_string();

                if version != "latest" {
                    installed.push((name.clone(), version));
                }
            }
        }

        Ok(installed)
    }

    pub fn uninstall(&self, name: &str, version: &str) -> Result<()> {
        println!("Uninstalling {} v{}...", name, version);

        let framework_dir = self.framework_dir(name, version);

        if !framework_dir.exists() {
            return Err(anyhow::anyhow!(
                "{} v{} is not installed",
                name,
                version
            ));
        }

        // Remove framework directory
        fs::remove_dir_all(&framework_dir)?;

        // Remove symlinks if this was the latest version
        let latest_link = self.frameworks_dir().join(name).join("latest");
        if latest_link.exists() {
            let target = fs::read_link(&latest_link)?;
            if target == framework_dir {
                fs::remove_file(&latest_link)?;

                let bin_link = self.cleen_home.join("bin").join(name);
                if bin_link.exists() {
                    fs::remove_file(&bin_link)?;
                }
            }
        }

        println!("✓ {} v{} uninstalled", name, version);

        Ok(())
    }
}
```

### 2. Update `src/cli.rs`

Add framework commands to CLI:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cleen")]
#[command(about = "Clean Language Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install Clean compiler or framework
    Install {
        /// Package to install (compiler version or framework@version)
        package: String,
    },

    /// Uninstall Clean compiler or framework
    Uninstall {
        /// Package to uninstall
        package: String,
        /// Version to uninstall
        version: String,
    },

    /// List installed versions or frameworks
    List {
        /// What to list: versions, frameworks, installed
        #[arg(default_value = "versions")]
        target: String,
    },

    /// Use specific version or framework
    Use {
        /// Package name (clean or framework name)
        package: String,
        /// Version to use
        version: String,
    },

    // ... existing commands
}
```

### 3. Update `src/main.rs`

Add framework command handling:

```rust
mod framework;

use framework::FrameworkManager;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cleen_home = get_cleen_home()?;

    match cli.command {
        Commands::Install { package } => {
            if package.starts_with("frame") {
                // Framework installation
                let framework_mgr = FrameworkManager::new(cleen_home);
                let (name, version) = parse_package(&package);
                framework_mgr.install_framework(&name, version).await?;
            } else {
                // Compiler installation (existing logic)
                install_compiler(&package).await?;
            }
        }

        Commands::List { target } => {
            match target.as_str() {
                "frameworks" => {
                    println!("Available frameworks:");
                    println!("  - frame");
                }
                "installed" => {
                    let framework_mgr = FrameworkManager::new(cleen_home);
                    let installed = framework_mgr.list_installed()?;

                    println!("Installed frameworks:");
                    for (name, version) in installed {
                        println!("  {} v{}", name, version);
                    }
                }
                _ => {
                    // Existing list versions logic
                }
            }
        }

        Commands::Uninstall { package, version } => {
            if package == "frame" {
                let framework_mgr = FrameworkManager::new(cleen_home);
                framework_mgr.uninstall(&package, &version)?;
            } else {
                // Existing uninstall logic
            }
        }

        // ... other commands
    }

    Ok(())
}

fn parse_package(package: &str) -> (String, Option<String>) {
    if let Some((name, version)) = package.split_once('@') {
        (name.to_string(), Some(version.to_string()))
    } else {
        (package.to_string(), None)
    }
}
```

### 4. Update `Cargo.toml`

Add required dependencies:

```toml
[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tar = "0.4"
flate2 = "1.0"
zip = "0.6"
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_framework_dir_structure() {
        let temp = TempDir::new().unwrap();
        let mgr = FrameworkManager::new(temp.path().to_path_buf());

        let dir = mgr.framework_dir("frame", "1.0.0");
        assert_eq!(
            dir,
            temp.path().join("versions/frameworks/frame/1.0.0")
        );
    }

    #[test]
    fn test_platform_detection() {
        let temp = TempDir::new().unwrap();
        let mgr = FrameworkManager::new(temp.path().to_path_buf());

        let platform = mgr.detect_platform().unwrap();
        assert!(platform.contains("linux") || platform.contains("macos") || platform.contains("windows"));
    }

    #[tokio::test]
    async fn test_install_frame() {
        let temp = TempDir::new().unwrap();
        let mgr = FrameworkManager::new(temp.path().to_path_buf());

        // Test installation
        let result = mgr.install_framework("frame", Some("1.0.0".to_string())).await;
        assert!(result.is_ok());

        // Verify installation
        let installed = mgr.list_installed().unwrap();
        assert!(installed.contains(&("frame".to_string(), "1.0.0".to_string())));
    }
}
```

### Integration Tests

```bash
#!/bin/bash
# tests/integration/test_framework_install.sh

set -e

# Test framework installation
echo "Testing: cleen install frame"
cleen install frame

# Verify frame command available
echo "Testing: frame --version"
frame --version

# Test list installed
echo "Testing: cleen list installed"
cleen list installed | grep "frame"

# Test uninstall
echo "Testing: cleen uninstall frame 1.0.0"
cleen uninstall frame 1.0.0

echo "✓ All tests passed"
```

## Documentation Updates

### README.md

Add framework installation section:

```markdown
## Managing Frameworks

### Install Frame Framework

```bash
# Install latest version
cleen install frame

# Install specific version
cleen install frame@1.0.0
```

### List Frameworks

```bash
# List available frameworks
cleen list frameworks

# List installed frameworks
cleen list installed
```

### Uninstall Framework

```bash
cleen uninstall frame 1.0.0
```
```

## Migration Path

### Phase 1: Core Framework Support (This PR)

- Add `FrameworkManager` module
- Implement `install frame` command
- Implement `list frameworks` command
- Implement `uninstall frame` command
- Update documentation

### Phase 2: Enhanced Features (Future PR)

- Add `cleen update frame` command
- Add version constraint support (e.g., `frame@^1.0.0`)
- Add framework dependency resolution
- Add rollback support

### Phase 3: Multiple Frameworks (Future PR)

- Support additional frameworks (frame-ui, frame-data as standalone)
- Add framework templates
- Add plugin system for custom frameworks

## Checklist

- [ ] Implement `FrameworkManager` module
- [ ] Add framework commands to CLI
- [ ] Update `Cargo.toml` dependencies
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update README documentation
- [ ] Test on all platforms (Linux, macOS, Windows)
- [ ] Update CHANGELOG
- [ ] Create PR with detailed description

## PR Template

**Title:** Add Frame Framework installation support

**Description:**

This PR adds support for installing the Frame Framework using Clean Manager (`cleen`).

**Changes:**
- Added `FrameworkManager` module for framework installation
- Extended CLI commands: `install frame`, `list frameworks`, `uninstall frame`
- Added framework manifest parsing
- Added cross-platform archive extraction
- Updated documentation

**Testing:**
- Unit tests for all framework manager functions
- Integration tests for installation workflow
- Manual testing on Linux, macOS, and Windows

**Related Issues:** #XXX

**Breaking Changes:** None

**Documentation:** README.md updated with framework commands

---

**For Review:** @Ivan-Pasco
