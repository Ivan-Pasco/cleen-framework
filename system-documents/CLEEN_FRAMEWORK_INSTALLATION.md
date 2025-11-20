# Frame Framework Installation via Clean Manager (cleen)

**Date**: November 19, 2025
**Status**: Proposed Implementation
**Strategy**: Extend `cleen` to manage Frame Framework installations

---

## Concept Overview

**Leverage existing `cleen` (Clean Manager) to install Frame Framework** alongside the Clean compiler.

### Current `cleen` Capabilities

```bash
cleen install 0.5.0        # Install Clean compiler v0.5.0
cleen use 0.5.0            # Activate compiler version
cleen list                 # List installed compiler versions
cleen available            # Show available compiler versions
```

### Proposed Extension for Frame

```bash
cleen install frame                    # Install latest Frame Framework
cleen install frame@1.0.0             # Install specific Frame version
cleen install frame@latest            # Install latest Frame version
cleen list frameworks                  # List installed frameworks
cleen use frame 1.0.0                 # Switch Frame version
cleen uninstall frame 1.0.0           # Remove Frame version
```

---

## Installation Architecture

### Directory Structure

Extend existing `~/.cleen/` structure:

```
~/.cleen/
├── bin/
│   ├── cln                           # Clean compiler shim (existing)
│   └── frame                         # Frame CLI shim (new)
├── versions/
│   ├── compiler/                     # Clean compiler versions (existing)
│   │   ├── 0.4.1/cln
│   │   └── 0.5.0/cln
│   └── frameworks/                   # Framework versions (new)
│       └── frame/
│           ├── 1.0.0/
│           │   ├── frame             # Frame CLI binary
│           │   ├── runtime.wasm      # Bundled runtime components
│           │   ├── server.wasm
│           │   ├── ui.wasm
│           │   ├── data.wasm
│           │   └── auth.wasm
│           └── 1.1.0/
│               └── ...
└── config.json                       # Extended config (existing)
```

### Extended Config Format

```json
{
  "active_compiler_version": "0.5.0",
  "active_framework_version": "1.0.0",
  "compiler_versions": ["0.4.1", "0.5.0"],
  "framework_versions": {
    "frame": ["1.0.0", "1.1.0"]
  }
}
```

---

## User Workflow

### Initial Setup

```bash
# 1. Install Clean Manager (one-time)
curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash

# 2. Initialize environment
cleen init

# 3. Install Clean compiler
cleen install 0.5.0
cleen use 0.5.0

# 4. Install Frame Framework
cleen install frame
# Downloads from: https://github.com/clean-lang/frame-framework/releases/latest

# 5. Verify installations
cln --version          # Clean Language v0.5.0
frame --version        # Frame Framework v1.0.0
```

### Version Management

```bash
# Install specific Frame version
cleen install frame@1.0.0

# List installed Frame versions
cleen list frameworks
# Output:
# Installed frameworks:
#   frame:
#     * 1.0.0 (active)
#       1.1.0

# Switch Frame version
cleen use frame 1.1.0

# Uninstall old version
cleen uninstall frame 1.0.0

# Check for updates
cleen available frameworks
# Output:
# Available frame versions:
#   1.2.0 (latest)
#   1.1.0
#   1.0.0 (installed)
```

---

## Implementation Plan

### Phase 1: Frame Framework GitHub Repository

**Repository Structure**:
```
clean-lang/frame-framework/
├── .github/
│   └── workflows/
│       └── release.yml              # Auto-build and release
├── frame-cli/                       # CLI source
├── frame-runtime/                   # Runtime source
├── frame-server/                    # Server source
├── frame-ui/                        # UI source
├── frame-data/                      # Data source
├── frame-auth/                      # Auth source
├── host-bridge/                     # Host bridge (already done!)
├── docs/                            # Documentation
├── examples/                        # Example apps
├── README.md
├── LICENSE
├── Cargo.toml                       # Rust workspace
└── build-release.sh                 # Build script for releases
```

**GitHub Releases Format**:
```
Release: v1.0.0
Assets:
  - frame-linux-x86_64.tar.gz        # Linux binary + runtime WASMs
  - frame-macos-x86_64.tar.gz        # macOS Intel binary + WASMs
  - frame-macos-aarch64.tar.gz       # macOS Apple Silicon + WASMs
  - frame-windows-x86_64.zip         # Windows binary + WASMs
  - checksums.txt                    # SHA256 checksums
```

**Archive Contents** (e.g., `frame-linux-x86_64.tar.gz`):
```
frame-1.0.0/
├── bin/
│   └── frame                        # CLI binary
├── lib/
│   ├── runtime.wasm
│   ├── server.wasm
│   ├── ui.wasm
│   ├── data.wasm
│   └── auth.wasm
├── templates/                       # Project templates
└── README.txt
```

### Phase 2: Extend Clean Manager (`cleen`)

**New Commands to Add**:

#### 1. `cleen install frame[@version]`

**Implementation** (`src/commands/install_framework.rs`):
```rust
pub async fn install_framework(
    name: &str,           // "frame"
    version: Option<&str> // None, "latest", or "1.0.0"
) -> Result<()> {
    // 1. Resolve version (if "latest", fetch from GitHub API)
    let version = resolve_framework_version(name, version).await?;

    // 2. Check if already installed
    if is_framework_installed(name, &version)? {
        println!("✓ Frame Framework v{} is already installed", version);
        return Ok(());
    }

    // 3. Download from GitHub releases
    let download_url = format!(
        "https://github.com/clean-lang/frame-framework/releases/download/v{}/frame-{}-{}.{}",
        version,
        get_platform(),
        get_arch(),
        get_extension()
    );

    println!("📥 Downloading Frame Framework v{}...", version);
    let archive_path = download_file(&download_url).await?;

    // 4. Verify checksum
    verify_checksum(&archive_path, &version).await?;

    // 5. Extract to ~/.cleen/versions/frameworks/frame/{version}/
    let install_dir = get_framework_dir(name, &version);
    extract_archive(&archive_path, &install_dir)?;

    // 6. Create/update shim in ~/.cleen/bin/frame
    create_framework_shim(name, &version)?;

    // 7. Update config.json
    update_config_framework(name, &version)?;

    println!("✅ Frame Framework v{} installed successfully!", version);
    println!("Run 'frame --version' to verify installation");

    Ok(())
}
```

#### 2. `cleen list frameworks`

**Implementation**:
```rust
pub fn list_frameworks() -> Result<()> {
    let frameworks_dir = get_frameworks_dir();

    println!("Installed frameworks:\n");

    for framework in ["frame"] {  // Extensible for future frameworks
        let framework_dir = frameworks_dir.join(framework);

        if !framework_dir.exists() {
            continue;
        }

        println!("  {}:", framework);

        // List versions
        let versions = get_installed_framework_versions(framework)?;
        let active_version = get_active_framework_version(framework)?;

        for version in versions {
            let marker = if Some(&version) == active_version.as_ref() {
                "* "
            } else {
                "  "
            };
            println!("    {}{}", marker, version);
        }
        println!();
    }

    Ok(())
}
```

#### 3. `cleen use frame <version>`

**Implementation**:
```rust
pub fn use_framework(name: &str, version: &str) -> Result<()> {
    // 1. Verify version is installed
    if !is_framework_installed(name, version)? {
        return Err(anyhow!(
            "Frame Framework v{} is not installed. Run 'cleen install frame@{}'",
            version, version
        ));
    }

    // 2. Update shim to point to this version
    update_framework_shim(name, version)?;

    // 3. Update config.json
    update_active_framework(name, version)?;

    println!("✅ Switched to Frame Framework v{}", version);

    Ok(())
}
```

#### 4. `cleen available frameworks`

**Implementation**:
```rust
pub async fn available_frameworks() -> Result<()> {
    println!("Fetching available Frame Framework versions from GitHub...\n");

    let releases = fetch_github_releases("clean-lang/frame-framework").await?;
    let installed = get_installed_framework_versions("frame")?;

    println!("Available frame versions:");

    for release in releases {
        let version = release.tag_name.trim_start_matches('v');
        let installed_marker = if installed.contains(&version.to_string()) {
            " (installed)"
        } else {
            ""
        };
        let latest_marker = if release.is_latest {
            " (latest)"
        } else {
            ""
        };

        println!("  {}{}{}", version, installed_marker, latest_marker);
    }

    Ok(())
}
```

### Phase 3: Shim Creation

**Create `~/.cleen/bin/frame` shim**:

**Unix/macOS** (`~/.cleen/bin/frame`):
```bash
#!/bin/bash
# Frame Framework shim - managed by cleen

CLEEN_DIR="$HOME/.cleen"
CONFIG_FILE="$CLEEN_DIR/config.json"

# Read active version from config
ACTIVE_VERSION=$(cat "$CONFIG_FILE" | grep -o '"active_framework_version": *"[^"]*"' | cut -d'"' -f4)

if [ -z "$ACTIVE_VERSION" ]; then
    echo "Error: No active Frame Framework version"
    echo "Run: cleen install frame"
    exit 1
fi

# Execute the active version
FRAME_BIN="$CLEEN_DIR/versions/frameworks/frame/$ACTIVE_VERSION/bin/frame"

if [ ! -f "$FRAME_BIN" ]; then
    echo "Error: Frame Framework v$ACTIVE_VERSION not found"
    echo "Run: cleen install frame@$ACTIVE_VERSION"
    exit 1
fi

exec "$FRAME_BIN" "$@"
```

**Windows** (`~/.cleen/bin/frame.bat`):
```batch
@echo off
REM Frame Framework shim - managed by cleen

set CLEEN_DIR=%USERPROFILE%\.cleen
set CONFIG_FILE=%CLEEN_DIR%\config.json

REM Read active version (simplified - use PowerShell for JSON parsing)
for /f "tokens=*" %%a in ('powershell -Command "(Get-Content '%CONFIG_FILE%' | ConvertFrom-Json).active_framework_version"') do set ACTIVE_VERSION=%%a

if "%ACTIVE_VERSION%"=="" (
    echo Error: No active Frame Framework version
    echo Run: cleen install frame
    exit /b 1
)

set FRAME_BIN=%CLEEN_DIR%\versions\frameworks\frame\%ACTIVE_VERSION%\bin\frame.exe

if not exist "%FRAME_BIN%" (
    echo Error: Frame Framework v%ACTIVE_VERSION% not found
    echo Run: cleen install frame@%ACTIVE_VERSION%
    exit /b 1
)

"%FRAME_BIN%" %*
```

---

## Frame Framework GitHub Release Automation

### CI/CD Workflow (`.github/workflows/release.yml`)

```yaml
name: Release Frame Framework

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-release:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: windows-x86_64

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true

      - name: Build Frame CLI
        run: |
          cargo build --release --target ${{ matrix.target }} --bin frame

      - name: Build Runtime WASMs
        run: |
          # Compile all runtime components to WASM
          ./scripts/build-runtime.sh

      - name: Package Release
        run: |
          mkdir -p frame-${{ github.ref_name }}/bin
          mkdir -p frame-${{ github.ref_name }}/lib

          # Copy binary
          cp target/${{ matrix.target }}/release/frame* frame-${{ github.ref_name }}/bin/

          # Copy runtime WASMs
          cp target/wasm32-unknown-unknown/release/*.wasm frame-${{ github.ref_name }}/lib/

          # Copy templates
          cp -r templates frame-${{ github.ref_name }}/

          # Create archive
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            7z a frame-${{ matrix.name }}.zip frame-${{ github.ref_name }}/*
          else
            tar czf frame-${{ matrix.name }}.tar.gz frame-${{ github.ref_name }}
          fi

      - name: Generate Checksum
        run: |
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            certutil -hashfile frame-${{ matrix.name }}.zip SHA256 > frame-${{ matrix.name }}.sha256
          else
            shasum -a 256 frame-${{ matrix.name }}.tar.gz > frame-${{ matrix.name }}.sha256
          fi

      - name: Upload Release Assets
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ steps.create_release.outputs.upload_url }}
          asset_path: ./frame-${{ matrix.name }}.${{ matrix.os == 'windows-latest' && 'zip' || 'tar.gz' }}
          asset_name: frame-${{ matrix.name }}.${{ matrix.os == 'windows-latest' && 'zip' || 'tar.gz' }}
          asset_content_type: application/octet-stream
```

---

## Integration with Clean Manager Repository

### Files to Modify in `clean-manager`

**1. Add new command modules**:
```
clean-manager/src/commands/
├── install_framework.rs     # NEW
├── list_frameworks.rs       # NEW
├── use_framework.rs         # NEW
├── available_frameworks.rs  # NEW
└── mod.rs                   # UPDATE
```

**2. Update `src/commands/mod.rs`**:
```rust
pub mod install_framework;
pub mod list_frameworks;
pub mod use_framework;
pub mod available_frameworks;
```

**3. Update `src/main.rs` CLI**:
```rust
#[derive(Subcommand)]
enum Commands {
    // Existing compiler commands
    Install { version: String },
    Use { version: String },
    List,
    Available,

    // New framework commands
    #[command(name = "install-framework")]
    InstallFramework {
        name: String,
        #[arg(short, long)]
        version: Option<String>,
    },

    #[command(name = "list-frameworks")]
    ListFrameworks,

    #[command(name = "use-framework")]
    UseFramework {
        name: String,
        version: String,
    },

    #[command(name = "available-frameworks")]
    AvailableFrameworks,
}
```

**4. Update `src/core/config.rs`**:
```rust
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub active_compiler_version: Option<String>,
    pub active_framework_version: Option<String>,  // NEW
    pub compiler_versions: Vec<String>,
    pub framework_versions: HashMap<String, Vec<String>>,  // NEW
}
```

---

## Benefits of This Approach

### For Users ✅

1. **Familiar workflow** - Same tool for compiler and framework
2. **Version management** - Easy switching between Frame versions
3. **Unified installation** - One command for everything
4. **Proven reliability** - Leverages existing `cleen` infrastructure

### For Development ✅

1. **No new infrastructure** - Use existing GitHub releases
2. **Automatic updates** - GitHub Actions handles builds
3. **Cross-platform** - Works on all platforms `cleen` supports
4. **Simple distribution** - Just upload to GitHub releases

### For Clean Ecosystem ✅

1. **Ecosystem coherence** - Everything managed through `cleen`
2. **Extensible** - Can add more frameworks later
3. **Community driven** - Contributors can add frameworks
4. **Professional** - Production-grade tooling

---

## Example Complete Workflow

```bash
# Install Clean ecosystem
curl -sSL https://clean-lang.org/install.sh | bash
cleen init

# Install Clean compiler
cleen install 0.5.0
cleen use 0.5.0

# Install Frame Framework
cleen install frame
# Downloads from GitHub: clean-lang/frame-framework releases

# Verify installations
cln --version        # Clean Language v0.5.0
frame --version      # Frame Framework v1.0.0

# Create new app
frame new my-blog
cd my-blog

# Build and run
frame serve

# Later: upgrade Frame
cleen install frame@1.1.0
cleen use frame 1.1.0

# Now using Frame v1.1.0
frame --version      # Frame Framework v1.1.0
```

---

## Timeline

### Week 1: Prepare Frame Repository
- [ ] Organize Frame code for release
- [ ] Create build scripts
- [ ] Set up GitHub repository structure
- [ ] Configure GitHub Actions for releases

### Week 2: Extend Clean Manager
- [ ] Add framework installation commands to `cleen`
- [ ] Implement download and extraction logic
- [ ] Create shim system for frameworks
- [ ] Test on all platforms

### Week 3: Integration Testing
- [ ] Test full installation workflow
- [ ] Verify version switching
- [ ] Test updates and uninstalls
- [ ] Cross-platform testing

### Week 4: Release
- [ ] Release Frame v1.0.0 on GitHub
- [ ] Release updated `cleen` with framework support
- [ ] Update documentation
- [ ] Announce to community

---

## Coordination with Clean Manager Team

**Proposed Collaboration**:

1. **Fork clean-manager** and implement framework support
2. **Submit PR** with new framework commands
3. **Coordinate release** of updated `cleen` with Frame v1.0.0
4. **Maintain compatibility** with existing `cleen` features

**Alternative** (if team is busy):
- Frame team maintains a fork: `cleen-frameworks`
- Users can use either version
- Merge back to main `cleen` when ready

---

## Conclusion

**Using `cleen` to install Frame Framework is the ideal solution** because:

✅ Leverages existing infrastructure
✅ Familiar user experience
✅ Simple implementation
✅ Professional distribution
✅ Extensible for future frameworks

**Next Steps**:
1. Set up Frame Framework GitHub repository
2. Create release builds and GitHub Actions
3. Extend `cleen` with framework commands
4. Test and release!

---

**Report Prepared By**: Development Team
**Report Date**: November 19, 2025
**Version**: 1.0
**Status**: Implementation Plan Ready
