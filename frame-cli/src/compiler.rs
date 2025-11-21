use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{info, warn, debug};

/// Compiler invoker for Clean Language
pub struct CompilerInvoker {
    cln_path: PathBuf,
    version: String,
}

impl CompilerInvoker {
    /// Detect and initialize compiler
    pub fn detect() -> Result<Self> {
        // Try to find `cln` in PATH
        let cln_path = which::which("cln")
            .context("Clean compiler (cln) not found in PATH. Please install via: cleen install latest")?;

        info!("Found Clean compiler at: {}", cln_path.display());

        // Get version
        let version = Self::get_version(&cln_path)?;
        info!("Clean compiler version: {}", version);

        // Validate version (require >=0.13.0)
        Self::validate_version(&version)?;

        Ok(Self {
            cln_path,
            version,
        })
    }

    /// Get compiler version
    fn get_version(cln_path: &Path) -> Result<String> {
        let output = Command::new(cln_path)
            .arg("--version")
            .output()
            .context("Failed to execute cln --version")?;

        if !output.status.success() {
            bail!("Failed to get compiler version");
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        // Extract version number (e.g., "cln 0.13.0" -> "0.13.0")
        let version = version_str
            .split_whitespace()
            .nth(1)
            .unwrap_or("unknown")
            .trim()
            .to_string();

        Ok(version)
    }

    /// Validate compiler version compatibility
    fn validate_version(version: &str) -> Result<()> {
        // Parse version
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 {
            warn!("Could not parse compiler version: {}", version);
            return Ok(()); // Allow unknown versions
        }

        let major: u32 = parts[0].parse().unwrap_or(0);
        let minor: u32 = parts[1].parse().unwrap_or(0);

        // Require >=0.13.0
        if major == 0 && minor < 13 {
            bail!(
                "Clean compiler version {} is too old. Frame requires >=0.13.0.\n\
                 Please update: cleen install latest",
                version
            );
        }

        // Warn if version is very new (might be incompatible)
        if major > 0 || minor > 20 {
            warn!(
                "Clean compiler version {} is newer than tested versions. \
                 Frame was tested with 0.13.x - 0.15.x. \
                 Compilation may fail or produce unexpected results.",
                version
            );
        }

        Ok(())
    }

    /// Compile a single Clean Language file to WASM
    pub fn compile_file(&self, input: &Path, output: &Path) -> Result<()> {
        debug!("Compiling {} -> {}", input.display(), output.display());

        // Ensure output directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        // Build command: cln compile -i input.cln -o output.wasm
        let mut cmd = Command::new(&self.cln_path);
        cmd.arg("compile")
            .arg("-i")
            .arg(input)
            .arg("-o")
            .arg(output)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Running: {:?}", cmd);

        // Execute compilation
        let output_result = cmd.output()
            .context("Failed to execute Clean compiler")?;

        // Check status
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            let stdout = String::from_utf8_lossy(&output_result.stdout);

            eprintln!("❌ Compilation failed for {}", input.display());
            eprintln!("\nCompiler output:");
            if !stdout.is_empty() {
                eprintln!("{}", stdout);
            }
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }

            bail!("Clean compilation failed");
        }

        // Success
        let stdout = String::from_utf8_lossy(&output_result.stdout);
        if !stdout.is_empty() {
            debug!("Compiler output: {}", stdout);
        }

        Ok(())
    }

    /// Compile multiple files to a single WASM bundle
    pub fn compile_project(&self, input_files: &[PathBuf], output: &Path) -> Result<()> {
        info!("Compiling {} files to {}", input_files.len(), output.display());

        // For Phase 1: Compile all files individually, then combine
        // TODO Phase 2: Use --project mode when compiler supports it

        if input_files.is_empty() {
            bail!("No Clean Language files found to compile");
        }

        // For now, compile the first file as the main entry point
        // Multi-file bundling will be implemented in Phase 2
        if input_files.len() == 1 {
            self.compile_file(&input_files[0], output)?;
        } else {
            // For multiple files, compile the first one and warn about others
            warn!(
                "Multi-file compilation not yet supported. Compiling only: {}",
                input_files[0].display()
            );
            warn!("Other files will be ignored (Phase 2 feature)");

            self.compile_file(&input_files[0], output)?;
        }

        Ok(())
    }

    /// Find all .cln files in a directory
    pub fn find_cln_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !dir.exists() {
            return Ok(files);
        }

        find_cln_files_recursive(dir, &mut files)?;

        // Sort for deterministic order
        files.sort();

        Ok(files)
    }

    /// Get compiler version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get compiler path
    pub fn path(&self) -> &Path {
        &self.cln_path
    }
}

/// Recursively find .cln files
fn find_cln_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and build output
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "dist" || name == "target" {
                    continue;
                }
            }
            find_cln_files_recursive(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("cln") {
            files.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_validation() {
        // Valid versions
        assert!(CompilerInvoker::validate_version("0.13.0").is_ok());
        assert!(CompilerInvoker::validate_version("0.14.5").is_ok());
        assert!(CompilerInvoker::validate_version("0.15.0").is_ok());

        // Too old - should fail
        assert!(CompilerInvoker::validate_version("0.12.0").is_err());
        assert!(CompilerInvoker::validate_version("0.10.5").is_err());
    }

    #[test]
    fn test_version_parsing() {
        // Should handle various version formats gracefully
        assert!(CompilerInvoker::validate_version("0.13.0-beta").is_ok());
        assert!(CompilerInvoker::validate_version("0.13").is_ok());
    }
}
