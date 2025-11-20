# Frame Framework Installation Guide

Complete installation instructions for the Frame Framework.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
  - [Via Clean Manager (Recommended)](#via-clean-manager-recommended)
  - [From Binary Release](#from-binary-release)
  - [From Source](#from-source)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

---

## Prerequisites

### Required

1. **Operating System**: Linux, macOS, or Windows
2. **Clean Language Compiler**: v0.5.0 or higher

### Optional (for building from source)

- **Rust**: 1.70 or higher
- **Git**: For cloning the repository

---

## Installation Methods

### Via Clean Manager (Recommended)

This is the easiest and recommended way to install Frame Framework.

#### Step 1: Install Clean Manager

If you haven't already, install Clean Manager (`cleen`):

**Unix/Linux/macOS**:
```bash
curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash
```

**Windows PowerShell**:
```powershell
iwr https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.ps1 | iex
```

#### Step 2: Initialize Clean Manager

```bash
cleen init
```

Follow the prompts to configure your shell.

#### Step 3: Install Clean Compiler

```bash
cleen install latest
cleen use latest
```

Verify installation:
```bash
cln --version
# Should output: Clean Language v0.5.0 (or higher)
```

#### Step 4: Install Frame Framework

```bash
cleen install frame
```

This will:
- Download the latest Frame Framework release
- Extract it to `~/.cleen/versions/frameworks/frame/`
- Create a `frame` command in your PATH

#### Step 5: Verify Frame Installation

```bash
frame --version
# Should output: Frame Framework v1.0.0
```

---

### From Binary Release

If you prefer manual installation, you can download pre-built binaries.

#### Step 1: Download Binary

Go to the [Releases page](https://github.com/Ivan-Pasco/cleen-framework/releases/latest) and download the appropriate archive for your platform:

- **Linux (x86_64)**: `frame-linux-x86_64.tar.gz`
- **Linux (ARM64)**: `frame-linux-aarch64.tar.gz`
- **macOS (Intel)**: `frame-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `frame-macos-aarch64.tar.gz`
- **Windows (x86_64)**: `frame-windows-x86_64.zip`

#### Step 2: Verify Checksum (Recommended)

Download the corresponding `.sha256` file and verify:

**Unix/Linux/macOS**:
```bash
shasum -a 256 -c frame-<platform>.tar.gz.sha256
```

**Windows PowerShell**:
```powershell
$hash = (Get-FileHash frame-windows-x86_64.zip -Algorithm SHA256).Hash
$expected = Get-Content frame-windows-x86_64.zip.sha256
if ($hash -eq $expected) { Write-Host "✓ Checksum verified" } else { Write-Host "✗ Checksum mismatch" }
```

#### Step 3: Extract Archive

**Unix/Linux/macOS**:
```bash
tar -xzf frame-<platform>.tar.gz
cd frame-1.0.0
```

**Windows**:
```powershell
Expand-Archive frame-windows-x86_64.zip -DestinationPath .
cd frame-1.0.0
```

#### Step 4: Install Binary

**Unix/Linux/macOS** (system-wide):
```bash
sudo mv bin/frame /usr/local/bin/
```

**Unix/Linux/macOS** (user-only):
```bash
mkdir -p ~/.local/bin
mv bin/frame ~/.local/bin/
# Add ~/.local/bin to PATH if not already
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Windows**:
```powershell
# Move to a directory in your PATH, e.g.:
Move-Item bin\frame.exe C:\Windows\System32\
# Or create a local bin directory and add to PATH
```

#### Step 5: Verify Installation

```bash
frame --version
```

---

### From Source

For developers or those who want the latest development version.

#### Step 1: Install Prerequisites

**Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Git**:
- Most systems have Git pre-installed
- Otherwise, install from [git-scm.com](https://git-scm.com/)

#### Step 2: Clone Repository

```bash
git clone https://github.com/Ivan-Pasco/cleen-framework.git
cd cleen-framework
```

#### Step 3: Build Frame CLI

```bash
cargo build --release --bin frame
```

The binary will be at: `target/release/frame`

#### Step 4: Build Runtime Components (Optional)

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build runtime components
cargo build --release --target wasm32-unknown-unknown -p host-bridge
```

#### Step 5: Install

**Unix/Linux/macOS**:
```bash
sudo cp target/release/frame /usr/local/bin/
# Or for user-only:
mkdir -p ~/.local/bin
cp target/release/frame ~/.local/bin/
```

**Windows**:
```powershell
Copy-Item target\release\frame.exe C:\Windows\System32\
```

#### Step 6: Verify Installation

```bash
frame --version
```

---

## Verification

After installation, verify everything is working:

```bash
# Check Frame version
frame --version

# Check Clean compiler version
cln --version

# Create a test project
frame new test-app

# Navigate to project
cd test-app

# Verify project structure
ls -la

# Try to serve (will show message about Phase 2 implementation)
frame serve
```

Expected output:
```
Frame Framework v1.0.0
Clean Language v0.5.0 (or higher)
```

---

## Troubleshooting

### Command Not Found: `frame`

**Problem**: Running `frame` gives "command not found"

**Solutions**:

1. **Check if binary is in PATH**:
   ```bash
   echo $PATH
   ```

2. **For Clean Manager installation**:
   ```bash
   # Verify cleen installation
   cleen list frameworks

   # Reinstall if needed
   cleen install frame
   ```

3. **For manual installation**:
   ```bash
   # Find where you installed it
   which frame

   # Add to PATH if needed
   export PATH="/path/to/frame/bin:$PATH"
   ```

4. **Restart your terminal** after installation

### Clean Compiler Not Found

**Problem**: Frame requires Clean compiler but can't find it

**Solution**:
```bash
# Install Clean compiler
cleen install latest
cleen use latest

# Verify
cln --version
```

### Permission Denied

**Problem**: Permission errors when running `frame`

**Solutions**:

1. **Make binary executable** (Unix/Linux/macOS):
   ```bash
   chmod +x /path/to/frame
   ```

2. **Don't use sudo** for user installations:
   ```bash
   # Use ~/.local/bin instead of /usr/local/bin
   mkdir -p ~/.local/bin
   mv frame ~/.local/bin/
   ```

3. **On Windows**, run PowerShell as Administrator

### Version Mismatch

**Problem**: Frame and Clean compiler versions incompatible

**Solution**:
```bash
# Update Clean compiler
cleen install latest
cleen use latest

# Update Frame Framework
cleen install frame@latest
cleen use frame latest
```

### macOS "Unidentified Developer" Warning

**Problem**: macOS blocks the binary

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /path/to/frame

# Or allow in System Preferences
# System Preferences > Security & Privacy > General > "Allow Anyway"
```

---

## Uninstallation

### Via Clean Manager

```bash
cleen uninstall frame 1.0.0
```

### Manual Uninstallation

**Unix/Linux/macOS**:
```bash
# Remove binary
sudo rm /usr/local/bin/frame
# Or if installed to user directory
rm ~/.local/bin/frame

# Remove runtime files (if installed manually)
rm -rf ~/.frame
```

**Windows**:
```powershell
Remove-Item C:\Windows\System32\frame.exe
```

---

## Next Steps

After successful installation:

1. **Read the Quick Start Guide**: [README.md](./README.md#quick-start)
2. **Create your first project**: `frame new my-app`
3. **Explore documentation**: [./documents/](./documents/)
4. **Join the community**: [GitHub Discussions](https://github.com/Ivan-Pasco/cleen-framework/discussions)

---

## Support

- **Issues**: [GitHub Issues](https://github.com/Ivan-Pasco/cleen-framework/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Ivan-Pasco/cleen-framework/discussions)
- **Documentation**: [./documents/](./documents/)

---

**Frame Framework** - Full-stack WebAssembly framework for Clean Language
