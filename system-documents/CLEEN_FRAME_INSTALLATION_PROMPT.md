# Prompt: Add Frame CLI Installation to `cleen` Version Manager

## Context

`cleen` is the version manager for Clean Language (similar to `rustup` for Rust). Currently, it manages Clean Language compiler installations. We need to extend it to also manage Clean Frame CLI installations.

**Important:** Clean Frame is the official full-stack framework for Clean Language. It depends on the compiler and provides framework-specific features via plugins.

---

## Requirements

### 1. Installation Flow

**When installing the compiler:**
```bash
$ cleen install 0.14.0
✓ Downloaded Clean Language Compiler v0.14.0
✓ Installed to ~/.cleen/versions/0.14.0/
✓ Set as active version

Would you like to install Frame CLI as well? (Y/n): Y
✓ Checking Frame CLI compatibility...
✓ Downloading Frame CLI v0.1.0 (compatible with compiler 0.14.0)
✓ Installed to ~/.cleen/versions/frame/0.1.0/
✓ Set as active Frame version
✓ Added 'frame' to PATH

Installation complete!
  cln --version  -> Clean Language Compiler 0.14.0
  frame --version -> Clean Frame CLI 0.1.0
```

**Explicit Frame installation:**
```bash
$ cleen install frame
✓ Detecting active compiler version: 0.14.0
✓ Finding compatible Frame CLI version...
✓ Downloading Frame CLI v0.1.0 (compatible with compiler 0.14.0)
✓ Installed to ~/.cleen/versions/frame/0.1.0/
✓ Set as active Frame version
```

**Specific Frame version:**
```bash
$ cleen install frame 0.1.0
✓ Checking compiler compatibility (requires >= 0.14.0)
✓ Active compiler: 0.14.0 ✓
✓ Downloading Frame CLI v0.1.0...
✓ Installed successfully
```

### 2. Storage Structure

Extend the existing `~/.cleen/` directory structure:

```
~/.cleen/
├── versions/
│   ├── 0.14.0/              # Compiler versions
│   │   ├── cln              # Compiler binary
│   │   └── lib/             # Compiler libraries
│   ├── 0.13.0/
│   │   └── cln
│   └── frame/               # Frame versions (separate namespace)
│       ├── 0.1.0/
│       │   ├── frame        # Frame CLI binary
│       │   └── lib/         # Frame libraries
│       └── 0.2.0/
│           └── frame
├── bin/
│   ├── cln -> ../versions/0.14.0/cln       # Symlink to active compiler
│   └── frame -> ../versions/frame/0.1.0/frame  # Symlink to active Frame
└── config.toml
```

### 3. Configuration Format

Update `~/.cleen/config.toml`:

```toml
# Active compiler version
active_version = "0.14.0"

# Active Frame version (optional - may not be installed)
frame_version = "0.1.0"

# Installation settings
[settings]
auto_offer_frame = true  # Offer Frame during compiler install
check_compatibility = true

# Version compatibility matrix (optional, can be fetched from remote)
[compatibility]
# Maps compiler version ranges to compatible Frame versions
"0.14.0" = "0.1.0"
"0.15.0" = "0.2.0"
```

### 4. Commands to Add/Modify

#### New Commands:

```bash
# Install Frame CLI (auto-detect compatible version)
cleen install frame

# Install specific Frame version
cleen install frame 0.1.0

# List available Frame versions
cleen list frame

# Use specific Frame version
cleen use frame 0.1.0

# Uninstall Frame version
cleen uninstall frame 0.1.0

# Show Frame installation status
cleen doctor frame
```

#### Modified Commands:

```bash
# Install compiler - now offers Frame
cleen install 0.14.0
  -> Interactive: "Would you like to install Frame CLI as well? (Y/n)"
  -> Flag: --with-frame (auto-install Frame)
  -> Flag: --no-frame (skip Frame prompt)

# List versions - add Frame section
cleen list
  Installed Compiler Versions:
    * 0.14.0 (active)
      0.13.0

  Installed Frame Versions:
    * 0.1.0 (active, compatible with compiler 0.14.0)

# Doctor command - check Frame compatibility
cleen doctor
  Compiler:
    ✓ Clean Language Compiler 0.14.0 installed
    ✓ Binary: ~/.cleen/bin/cln
    ✓ PATH configured correctly

  Frame:
    ✓ Frame CLI 0.1.0 installed
    ✓ Binary: ~/.cleen/bin/frame
    ⚠ Compatible with compiler: 0.14.0 ✓
    ✓ PATH configured correctly

# Uninstall - handle Frame dependency
cleen uninstall 0.14.0
  -> Warning: "Frame CLI 0.1.0 depends on this compiler version. Uninstall Frame first."
  -> Or auto-uninstall Frame if no other compatible compiler exists
```

### 5. Version Compatibility

**Compatibility Rules:**
- Frame CLI requires a specific minimum compiler version
- Frame 0.1.x requires compiler >= 0.14.0
- Frame 0.2.x requires compiler >= 0.15.0
- When installing Frame, check active compiler version
- When switching compiler versions, warn if Frame is incompatible

**Compatibility Check:**
```bash
$ cleen use 0.13.0
✓ Switched to Clean Language Compiler 0.13.0
⚠ Warning: Frame CLI 0.1.0 requires compiler >= 0.14.0
  Current compiler: 0.13.0
  Frame CLI may not work correctly.

  Options:
  - Use a newer compiler: cleen use 0.14.0
  - Downgrade Frame: cleen use frame 0.0.5
  - Continue anyway (not recommended)
```

### 6. Implementation Details

#### 6.1 Download Sources

**Compiler:**
- Currently downloads from GitHub releases
- URL: `https://github.com/cleanlang/clean-language-compiler/releases/download/v{version}/cln-{platform}.tar.gz`

**Frame CLI:**
- Download from GitHub releases
- URL: `https://github.com/cleanlang/frame/releases/download/v{version}/frame-{platform}.tar.gz`

**Compatibility Matrix:**
- Fetch from remote JSON (optional):
  ```
  https://releases.cleanlanguage.dev/compatibility.json
  ```
- Or hardcode initial version:
  ```json
  {
    "compiler": {
      "0.14.0": {
        "frame": ["0.1.0"]
      }
    }
  }
  ```

#### 6.2 Installation Functions

Add these functions to `cleen`:

```rust
// Install Frame CLI
async fn install_frame(version: Option<&str>) -> Result<()> {
    // 1. Determine version to install
    let frame_version = if let Some(v) = version {
        v.to_string()
    } else {
        // Auto-detect compatible version
        let compiler_version = get_active_compiler_version()?;
        find_compatible_frame_version(&compiler_version).await?
    };

    // 2. Check compiler compatibility
    check_frame_compatibility(&frame_version).await?;

    // 3. Download Frame binary
    download_frame(&frame_version).await?;

    // 4. Extract and install
    install_frame_binary(&frame_version)?;

    // 5. Update symlink
    update_frame_symlink(&frame_version)?;

    // 6. Update config
    update_config_frame_version(&frame_version)?;

    Ok(())
}

// Check if Frame version is compatible with active compiler
fn check_frame_compatibility(frame_version: &str) -> Result<()> {
    let compiler_version = get_active_compiler_version()?;
    let required_compiler = get_required_compiler_version(frame_version)?;

    if !is_compatible(&compiler_version, &required_compiler) {
        bail!(
            "Frame CLI {} requires compiler >= {}\nActive compiler: {}",
            frame_version,
            required_compiler,
            compiler_version
        );
    }

    Ok(())
}

// Find compatible Frame version for current compiler
async fn find_compatible_frame_version(compiler_version: &str) -> Result<String> {
    // Query compatibility matrix or use hardcoded mapping
    let compatibility = load_compatibility_matrix().await?;

    compatibility
        .get(compiler_version)
        .and_then(|versions| versions.first())
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow!("No compatible Frame version found for compiler {}", compiler_version))
}
```

#### 6.3 Interactive Prompt

During compiler installation:

```rust
async fn install_compiler(version: &str, opts: InstallOptions) -> Result<()> {
    // Install compiler
    download_and_install_compiler(version).await?;

    // Offer Frame installation
    if !opts.no_frame && config.settings.auto_offer_frame {
        if opts.with_frame || prompt_install_frame()? {
            match install_frame(None).await {
                Ok(_) => println!("✓ Frame CLI installed successfully"),
                Err(e) => eprintln!("⚠ Failed to install Frame CLI: {}", e),
            }
        }
    }

    Ok(())
}

fn prompt_install_frame() -> Result<bool> {
    use dialoguer::Confirm;

    Confirm::new()
        .with_prompt("Would you like to install Frame CLI as well?")
        .default(true)
        .interact()
        .map_err(Into::into)
}
```

### 7. Error Handling

**Common Errors:**

1. **Incompatible compiler version:**
   ```
   Error: Frame CLI 0.1.0 requires Clean Language Compiler >= 0.14.0

   Current compiler: 0.13.0

   To fix:
   - Upgrade compiler: cleen install 0.14.0
   - Use compatible Frame: cleen install frame 0.0.5
   ```

2. **No compiler installed:**
   ```
   Error: Cannot install Frame CLI without a compiler

   Please install a compiler first:
     cleen install 0.14.0
   ```

3. **Download failure:**
   ```
   Error: Failed to download Frame CLI v0.1.0

   Tried URLs:
   - https://github.com/cleanlang/frame/releases/download/v0.1.0/frame-macos.tar.gz (404)

   Please check:
   - Version exists: https://github.com/cleanlang/frame/releases
   - Network connection
   ```

4. **Dependency conflict:**
   ```
   Error: Cannot uninstall compiler 0.14.0

   Reason: Frame CLI 0.1.0 depends on this version

   Options:
   - Uninstall Frame first: cleen uninstall frame 0.1.0
   - Force uninstall: cleen uninstall 0.14.0 --force (may break Frame)
   ```

### 8. Testing Requirements

**Unit Tests:**
```rust
#[test]
fn test_version_compatibility() {
    assert!(is_compatible("0.14.0", "0.14.0"));
    assert!(is_compatible("0.15.0", "0.14.0"));
    assert!(!is_compatible("0.13.0", "0.14.0"));
}

#[test]
fn test_find_compatible_frame_version() {
    let frame_version = find_compatible_frame_version("0.14.0").unwrap();
    assert_eq!(frame_version, "0.1.0");
}

#[test]
fn test_frame_installation_path() {
    let path = get_frame_binary_path("0.1.0");
    assert_eq!(path, home_dir().join(".cleen/versions/frame/0.1.0/frame"));
}
```

**Integration Tests:**
```bash
# Test compiler-only installation
$ cleen install 0.14.0 --no-frame
# Verify Frame not installed

# Test compiler + Frame installation
$ cleen install 0.14.0 --with-frame
# Verify both installed

# Test explicit Frame installation
$ cleen install frame
# Verify Frame installed with compatible version

# Test version switching
$ cleen use 0.13.0
# Verify compatibility warning shown

# Test uninstall
$ cleen uninstall 0.14.0
# Verify dependency check
```

### 9. Documentation

Add to `cleen --help`:

```
USAGE:
    cleen <COMMAND>

COMMANDS:
    install      Install compiler or Frame CLI
    list         List installed versions
    use          Switch active version
    uninstall    Remove installed version
    doctor       Check installation health
    update       Update cleen itself

EXAMPLES:
    # Install compiler
    cleen install 0.14.0

    # Install compiler with Frame CLI
    cleen install 0.14.0 --with-frame

    # Install Frame CLI (auto-detect version)
    cleen install frame

    # Install specific Frame version
    cleen install frame 0.1.0

    # List all versions
    cleen list

    # Switch Frame version
    cleen use frame 0.1.0

    # Check compatibility
    cleen doctor
```

### 10. Migration for Existing Users

**First Run After Update:**
```bash
$ cleen list
✓ Migrated to new config format

Installed Compiler Versions:
  * 0.14.0 (active)

Frame CLI is now available!
  Install with: cleen install frame
  Learn more: https://docs.cleanlanguage.dev/frame
```

**Config Migration:**
```rust
fn migrate_config() -> Result<()> {
    let config = load_config()?;

    // Add Frame fields if missing
    if !config.contains_key("frame_version") {
        config.insert("frame_version", None);
    }

    if !config.contains_key("settings") {
        config.insert("settings", Settings::default());
    }

    save_config(&config)?;
    Ok(())
}
```

---

## Implementation Checklist

- [ ] Add Frame installation function
- [ ] Implement version compatibility checking
- [ ] Add interactive Frame prompt during compiler install
- [ ] Add `cleen install frame` command
- [ ] Add `cleen list frame` support
- [ ] Add `cleen use frame <version>` command
- [ ] Add `cleen uninstall frame <version>` command
- [ ] Update `cleen doctor` to check Frame
- [ ] Add Frame symlink management
- [ ] Update config.toml structure
- [ ] Add dependency checks (prevent removing required compiler)
- [ ] Implement compatibility warnings
- [ ] Add `--with-frame` and `--no-frame` flags
- [ ] Add download URLs for Frame releases
- [ ] Update help text and documentation
- [ ] Add unit tests for compatibility logic
- [ ] Add integration tests for installation flow
- [ ] Handle config migration for existing users
- [ ] Add error messages with helpful suggestions

---

## Expected File Changes

**Files to Modify:**
- `src/commands/install.rs` - Add Frame installation logic
- `src/commands/list.rs` - Add Frame version listing
- `src/commands/use.rs` - Add Frame version switching
- `src/commands/uninstall.rs` - Add Frame uninstall + dependency check
- `src/commands/doctor.rs` - Add Frame health checks
- `src/config.rs` - Extend config structure for Frame
- `src/version.rs` - Add compatibility checking functions
- `src/download.rs` - Add Frame download URLs
- `src/main.rs` - Add Frame subcommands to CLI
- `README.md` - Document Frame installation

**Files to Create:**
- `src/frame.rs` - Frame-specific installation logic
- `src/compatibility.rs` - Version compatibility matrix and checking

---

## Platform Support

Frame CLI should support the same platforms as the compiler:
- macOS (Intel and Apple Silicon)
- Linux (x64)
- Windows (x64)

Download URL format:
```
https://github.com/cleanlang/frame/releases/download/v{version}/frame-{platform}-{arch}.tar.gz

Platforms:
- frame-macos-x64.tar.gz
- frame-macos-arm64.tar.gz
- frame-linux-x64.tar.gz
- frame-windows-x64.zip
```

---

## Success Criteria

1. ✅ User can install compiler and Frame in one command
2. ✅ User can install Frame separately
3. ✅ Version compatibility is automatically checked
4. ✅ Clear error messages when versions are incompatible
5. ✅ `cleen doctor` shows Frame status
6. ✅ `cleen list` shows both compiler and Frame versions
7. ✅ Uninstalling compiler checks Frame dependencies
8. ✅ Existing cleen installations can upgrade without issues
9. ✅ All tests pass
10. ✅ Documentation is clear and comprehensive

---

**Start with:** Implementing the basic `cleen install frame` command and version compatibility checking.

**Priority:** High - Frame CLI is the official framework and should have first-class version manager support.
