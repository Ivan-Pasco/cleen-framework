#!/bin/bash

# Script to create independent clean-runtime project
# Run from the clean-framework directory

set -e

echo "Creating Clean Runtime independent project..."

# Define paths
RUNTIME_DIR="/Users/earcandy/Documents/Dev/Clean Language/clean-runtime"
FRAMEWORK_DIR="/Users/earcandy/Documents/Dev/Clean Language/clean-framework"

# Create directory structure
echo "Creating directory structure..."
mkdir -p "$RUNTIME_DIR/src"
mkdir -p "$RUNTIME_DIR/host-bridge/src"
mkdir -p "$RUNTIME_DIR/.github/workflows"

# Copy source files
echo "Copying runtime source files..."
cp -r "$FRAMEWORK_DIR/frame-runtime/src/"* "$RUNTIME_DIR/src/"

echo "Copying host-bridge source files..."
cp -r "$FRAMEWORK_DIR/host-bridge/src/"* "$RUNTIME_DIR/host-bridge/src/"

# Create Cargo.toml for clean-runtime
echo "Creating Cargo.toml..."
cat > "$RUNTIME_DIR/Cargo.toml" << 'EOF'
[workspace]
resolver = "2"
members = [".", "host-bridge"]

[package]
name = "clean-runtime"
version = "1.0.0"
edition = "2021"
authors = ["Ivan Pasco Lizarraga"]
license = "MIT OR Apache-2.0"
description = "Runtime execution environment for Clean Language WebAssembly applications"
repository = "https://github.com/Ivan-Pasco/clean-runtime"
homepage = "https://www.cleanlanguage.dev"
readme = "README.md"
keywords = ["webassembly", "wasm", "runtime", "clean-language", "http-server"]
categories = ["development-tools", "wasm", "web-programming::http-server"]

[[bin]]
name = "clean-runtime"
path = "src/main.rs"

[lib]
name = "clean_runtime"
path = "src/lib.rs"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"
async-trait = "0.1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# WebAssembly runtime
wasmtime = "16.0"

# HTTP server
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
bytes = "1.5"
http = "1.0"
http-body-util = "0.1"
matchit = "0.8"
parking_lot = "0.12"
dashmap = "6.0"
url = "2.5"

# Local dependencies
host-bridge = { path = "host-bridge" }

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
EOF

# Create host-bridge Cargo.toml
echo "Creating host-bridge Cargo.toml..."
cat > "$RUNTIME_DIR/host-bridge/Cargo.toml" << 'EOF'
[package]
name = "host-bridge"
version = "1.0.0"
edition = "2021"
authors = ["Ivan Pasco Lizarraga"]
license = "MIT OR Apache-2.0"
description = "Host bridge for Clean Runtime - system capabilities for WASM modules"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"
async-trait = "0.1"

# WebAssembly
wasmtime = "16.0"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "mysql", "sqlite", "json", "chrono", "uuid"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# HTTP client - force rustls-tls for cross-compilation
reqwest = { version = "0.11", default-features = false, features = ["json", "gzip", "brotli", "rustls-tls", "stream"] }

# Utilities
bytes = "1.5"
chrono = "0.4"
url = "2.5"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
tracing = "0.1"

# Crypto dependencies
bcrypt = "0.15"
argon2 = "0.5"
jsonwebtoken = "9"
base64 = "0.22"

# Filesystem dependencies
glob = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
tokio-test = "0.4"
once_cell = "1.19"
tempfile = "3.8"
EOF

# Create README.md
echo "Creating README.md..."
cat > "$RUNTIME_DIR/README.md" << 'EOF'
# Clean Runtime

Runtime execution environment for Clean Language WebAssembly applications.

## Overview

The Clean Runtime is an HTTP server that executes compiled Clean Language applications. It provides:

- **HTTP Server**: High-performance Axum-based server for handling web requests
- **WASM Execution**: Wasmtime-powered execution environment for Clean Language applications
- **Dynamic Routing**: Automatic route registration from compiled WASM modules
- **Host Bridge**: System capabilities exposed to WASM (I/O, HTTP client, database, authentication)
- **Security**: Sandboxed WASM execution with controlled system access

## Installation

### Option 1: Using cleen (Clean Package Manager)

```bash
cleen runtime install
```

### Option 2: Download from GitHub Releases

Download the pre-built binary for your platform from [GitHub Releases](https://github.com/Ivan-Pasco/clean-runtime/releases):

- **Linux x86_64**: `clean-runtime-linux-x86_64`
- **Linux ARM64**: `clean-runtime-linux-aarch64`
- **macOS x86_64**: `clean-runtime-macos-x86_64`
- **macOS ARM64**: `clean-runtime-macos-aarch64`
- **Windows x86_64**: `clean-runtime-windows-x86_64.exe`

Make the binary executable and add it to your PATH:

```bash
# Linux/macOS
chmod +x clean-runtime-*
sudo mv clean-runtime-* /usr/local/bin/clean-runtime

# Windows
# Move clean-runtime-windows-x86_64.exe to a directory in your PATH
```

### Option 3: Build from Source

```bash
git clone https://github.com/Ivan-Pasco/clean-runtime.git
cd clean-runtime
cargo build --release
sudo mv target/release/clean-runtime /usr/local/bin/
```

### Option 4: Install with Cargo

```bash
cargo install clean-runtime
```

## Usage

### Basic Usage

```bash
# Run a compiled Clean Language application
clean-runtime app.wasm

# Run on a custom port
clean-runtime app.wasm --port 8080

# Run with custom host
clean-runtime app.wasm --host 127.0.0.1 --port 3000

# Enable verbose logging
clean-runtime app.wasm --verbose

# Disable CORS (use with caution)
clean-runtime app.wasm --no-cors

# Set request body size limit (in MB)
clean-runtime app.wasm --body-limit 20
```

### Command Line Options

```
Usage: clean-runtime [OPTIONS] <WASM_FILE>

Arguments:
  <WASM_FILE>  Path to the WASM file to execute

Options:
  -p, --port <PORT>              Port to listen on [default: 3000]
      --host <HOST>              Host address to bind to [default: 0.0.0.0]
  -v, --verbose                  Enable verbose logging
      --no-cors                  Disable CORS
      --body-limit <BODY_LIMIT>  Request body size limit in MB [default: 10]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Example

Compile and run a Clean Language application:

```bash
# Compile Clean Language app
cln compile app.cln

# Run with Clean Runtime
clean-runtime app.wasm
```

The server will start on `http://0.0.0.0:3000` by default.

## Architecture

```text
HTTP Request
     │
     ▼
┌─────────────┐
│ Axum Server │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Router    │──► Match route by method + path
└──────┬──────┘
       │
       ▼
┌─────────────┐
│WASM Instance│──► Call handler function
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Host Bridge │──► System I/O, HTTP, DB, Auth
└─────────────┘
```

## Host Bridge Capabilities

The runtime exposes these system capabilities to WASM modules through the Host Bridge:

### HTTP Client (`bridge:http`)
- `http_get`, `http_post`, `http_put`, `http_delete`, `http_patch`
- Full HTTP client functionality for making external API requests

### Database (`bridge:db`)
- `db_query`, `db_execute`, `db_transaction`
- Support for PostgreSQL, MySQL, and SQLite

### Environment (`bridge:env`)
- `env_get`, `env_set`
- Access to environment variables

### Time (`bridge:time`)
- `time_now`, `time_format`, `time_parse`
- Time and date operations

### Crypto (`bridge:crypto`)
- `crypto_hash`, `crypto_random`, `crypto_jwt_sign`, `crypto_jwt_verify`
- Cryptographic operations for security

### Logging (`bridge:log`)
- `log_info`, `log_warn`, `log_error`, `log_debug`
- Structured logging support

### Filesystem (`bridge:fs`) *(Desktop/CLI only)*
- `fs_read`, `fs_write`, `fs_exists`, `fs_delete`
- File system operations (restricted in web environments)

### System (`bridge:sys`)
- `sys_info`, `sys_platform`
- System information queries

## Security

### WASM Sandboxing

All application code runs in an isolated WebAssembly sandbox with:
- No direct access to system resources
- Memory isolation
- Controlled execution environment

### Host Bridge Permissions

System access is controlled through:
- Explicit Host Bridge function calls
- Allowlisting of capabilities
- Request validation and sanitization

### Production Security

For production deployments:
- Use HTTPS (terminate SSL at reverse proxy or load balancer)
- Configure CORS properly (don't use `--no-cors` in production)
- Set appropriate body size limits
- Use environment variables for secrets (never hardcode)
- Keep runtime updated to latest version

## Performance

### Targets

- **First Request (SSR)**: < 50ms
- **API Latency (Simple)**: < 10ms (p95)
- **API Latency (Database)**: < 100ms (p95)
- **Throughput**: > 10k req/sec (simple endpoints)

### Optimization Tips

1. Use production builds: `cargo build --release` or download pre-built binaries
2. Configure appropriate body size limits
3. Enable HTTP/2 at reverse proxy
4. Use connection pooling for databases
5. Implement caching strategies in your application

## Compatibility

### WASM Module Requirements

WASM modules must:

1. Export an initialization function: `main`, `_start`, `start`, or `init`
2. Register routes using `_http_route(method, path, handler_index)`
3. Export handler functions that return string pointers
4. Use Clean Language standard memory format (length-prefixed strings)

### Supported Platforms

- Linux x86_64 / ARM64
- macOS x86_64 / ARM64 (Apple Silicon)
- Windows x86_64

## Troubleshooting

### Common Issues

**WASM file not found**
```
Error: WASM file not found: app.wasm
```
Solution: Ensure the WASM file path is correct and the file exists.

**Port already in use**
```
Error: Address already in use (os error 98)
```
Solution: Use a different port with `--port <PORT>` or stop the process using that port.

**Invalid WASM file**
```
Error: Failed to load WASM module
```
Solution: Ensure the file is a valid WASM module compiled by the Clean Language compiler.

### Enable Debug Logging

```bash
clean-runtime app.wasm --verbose
```

This will show detailed logs of request handling, route matching, and Host Bridge calls.

## Related Projects

- **[Clean Language Compiler](https://github.com/Ivan-Pasco/clean-language-compiler)** - Compiles Clean Language to WASM
- **[cleen](https://github.com/Ivan-Pasco/clean-manager)** - Clean Language package manager and CLI
- **[Frame Framework](https://github.com/clean-language/clean-framework)** - Full-stack framework for Clean Language

## Development

### Building

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- app.wasm
```

### Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Author

**Ivan Pasco Lizarraga**

- Website: https://www.cleanlanguage.dev
- GitHub: https://github.com/Ivan-Pasco

## Support

- **Documentation**: https://www.cleanlanguage.dev/docs
- **Issues**: https://github.com/Ivan-Pasco/clean-runtime/issues
- **Discussions**: https://github.com/Ivan-Pasco/clean-runtime/discussions
EOF

# Create GitHub Actions workflow
echo "Creating GitHub Actions workflow..."
cat > "$RUNTIME_DIR/.github/workflows/release.yml" << 'EOF'
name: Release Clean Runtime

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      tag:
        description: 'Release tag'
        required: true
        default: 'v1.0.0'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: clean-runtime-linux-x86_64
            archive: tar.gz
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest-arm64
            name: clean-runtime-linux-aarch64
            archive: tar.gz
          - target: x86_64-apple-darwin
            os: macos-latest
            name: clean-runtime-macos-x86_64
            archive: tar.gz
          - target: aarch64-apple-darwin
            os: macos-latest
            name: clean-runtime-macos-aarch64
            archive: tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            name: clean-runtime-windows-x86_64
            archive: zip

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}

    - name: Build runtime binary
      run: cargo build --release --bin clean-runtime --target ${{ matrix.target }}
      env:
        OPENSSL_STATIC: 1
        OPENSSL_VENDOR: 1

    - name: Strip binary (Unix)
      if: matrix.os != 'windows-latest'
      run: strip target/${{ matrix.target }}/release/clean-runtime || true

    - name: Create archive (Unix)
      if: matrix.os != 'windows-latest'
      run: |
        mkdir -p release-package
        cp target/${{ matrix.target }}/release/clean-runtime release-package/
        tar -czf ${{ matrix.name }}.tar.gz -C release-package .
        # Also copy direct binary for cleanmanager compatibility
        cp target/${{ matrix.target }}/release/clean-runtime ${{ matrix.name }}

    - name: Create archive (Windows)
      if: matrix.os == 'windows-latest'
      shell: pwsh
      run: |
        Set-Location "target/${{ matrix.target }}/release"
        7z a "../../../${{ matrix.name }}.zip" clean-runtime.exe
        # Also copy direct binary for cleanmanager compatibility
        Copy-Item clean-runtime.exe "../../../${{ matrix.name }}.exe"

    - name: Upload binary artifact
      uses: actions/upload-artifact@v4
      with:
        name: ${{ matrix.name }}
        path: |
          ${{ matrix.name }}.${{ matrix.archive }}
          ${{ matrix.name }}${{ matrix.os == 'windows-latest' && '.exe' || '' }}

  release:
    name: Create Release
    runs-on: ubuntu-latest
    needs: build
    permissions:
      contents: write
    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Download all artifacts
      uses: actions/download-artifact@v4
      with:
        path: artifacts

    - name: Move artifacts to root
      run: |
        # Move archives
        find artifacts -name "*.tar.gz" -o -name "*.zip" | while read file; do
          mv "$file" .
        done
        # Move direct binaries for cleanmanager compatibility
        find artifacts -name "clean-runtime-linux-*" -type f | while read file; do
          mv "$file" .
        done
        find artifacts -name "clean-runtime-macos-*" -type f | while read file; do
          mv "$file" .
        done
        find artifacts -name "clean-runtime-windows-*.exe" -type f | while read file; do
          mv "$file" .
        done
        # List what we have
        ls -la *.tar.gz *.zip clean-runtime-* 2>/dev/null || echo "Some files may be missing"

    - name: Create Release
      uses: softprops/action-gh-release@v2
      with:
        files: |
          *.tar.gz
          *.zip
          clean-runtime-linux-x86_64
          clean-runtime-linux-aarch64
          clean-runtime-macos-x86_64
          clean-runtime-macos-aarch64
          clean-runtime-windows-x86_64.exe
        body: |
          ## Clean Runtime ${{ github.ref_name }}

          **Author:** Ivan Pasco Lizarraga
          **Website:** https://www.cleanlanguage.dev

          Runtime execution environment for Clean Language WebAssembly applications.

          ### Installation

          Download the appropriate binary for your platform and add it to your PATH.

          ### Platforms

          Choose between direct binaries (for cleanmanager) or archives:

          - **Linux x86_64**: `clean-runtime-linux-x86_64` or `clean-runtime-linux-x86_64.tar.gz`
          - **Linux ARM64**: `clean-runtime-linux-aarch64` or `clean-runtime-linux-aarch64.tar.gz`
          - **macOS x86_64**: `clean-runtime-macos-x86_64` or `clean-runtime-macos-x86_64.tar.gz`
          - **macOS ARM64**: `clean-runtime-macos-aarch64` or `clean-runtime-macos-aarch64.tar.gz`
          - **Windows x86_64**: `clean-runtime-windows-x86_64.exe` or `clean-runtime-windows-x86_64.zip`

          ### Usage

          ```bash
          # Run a Clean Language application
          clean-runtime app.wasm

          # Run on custom port
          clean-runtime app.wasm --port 8080

          # Check version
          clean-runtime --version
          ```

          For more information, visit: https://www.cleanlanguage.dev
        draft: false
        prerelease: false
EOF

# Create .gitignore
echo "Creating .gitignore..."
cat > "$RUNTIME_DIR/.gitignore" << 'EOF'
# Rust
/target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDEs
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Test artifacts
*.wasm
*.wat

# Logs
*.log
EOF

# Initialize git repository
echo "Initializing git repository..."
cd "$RUNTIME_DIR"
git init
git add .
git commit -m "feat: initial release of Clean Runtime v1.0.0

- HTTP server runtime for Clean Language WASM applications
- Wasmtime-powered execution environment
- Host Bridge for system capabilities (I/O, HTTP, DB, Auth)
- Dynamic route registration from WASM modules
- Multi-platform support (Linux, macOS, Windows)
- Production-ready with security sandboxing"

echo ""
echo "✓ Clean Runtime project created successfully!"
echo ""
echo "Next steps:"
echo "  1. cd '$RUNTIME_DIR'"
echo "  2. cargo build --release"
echo "  3. cargo test"
echo "  4. Create GitHub repository: https://github.com/new"
echo "  5. git remote add origin https://github.com/Ivan-Pasco/clean-runtime.git"
echo "  6. git push -u origin main"
echo "  7. Create release tag: git tag v1.0.0 && git push origin v1.0.0"
echo ""
