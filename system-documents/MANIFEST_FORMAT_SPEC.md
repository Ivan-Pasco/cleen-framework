# Unified Manifest Format: package.clean.toml

**Version**: 1.0
**Date**: November 20, 2025
**Status**: Phase 2 Implementation

## Overview

The `package.clean.toml` manifest serves as the unified configuration file for Clean Language projects, including Frame Framework applications. It combines package metadata, dependencies, build configuration, and framework-specific settings in a single file.

## Design Goals

1. **Unified**: Single source of truth for all project configuration
2. **Compatible**: Works with `cleen` manager and Frame CLI
3. **Extensible**: Easy to add new sections without breaking compatibility
4. **Clear**: Human-readable with sensible defaults
5. **Standard**: Follows TOML best practices

## File Format

### Basic Structure

```toml
# package.clean.toml

[package]
name = "my-app"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"
description = "A Frame application"
license = "MIT"

[frame]
version = "0.1.0"
target = "web"

[dependencies]
# Future: External packages

[build]
# Build configuration

[dev-dependencies]
# Development-only dependencies
```

## Section Specifications

### [package] - Required

Core package metadata following Clean Language conventions.

```toml
[package]
name = "my-app"              # Package name (kebab-case, required)
version = "0.1.0"            # Semantic version (required)
authors = ["Name <email>"]   # Author list (optional)
edition = "2025"             # Clean Language edition (optional, default: current)
description = "..."          # Short description (optional)
license = "MIT"              # SPDX license identifier (optional)
repository = "https://..."   # Git repository URL (optional)
homepage = "https://..."     # Project homepage (optional)
documentation = "https://..." # Documentation URL (optional)
keywords = ["web", "api"]    # Search keywords (optional)
categories = ["web"]         # Package categories (optional)
```

**Validation Rules**:
- `name`: Must be kebab-case, 1-64 characters, alphanumeric + hyphens
- `version`: Must follow semver (major.minor.patch)
- `authors`: Each entry format: "Name" or "Name <email>"
- `edition`: Currently only "2025" supported

### [frame] - Optional

Frame Framework-specific configuration. If present, Frame CLI uses these settings.

```toml
[frame]
version = "0.1.0"            # Frame version to use (required if [frame] present)
target = "web"               # Default build target (optional, default: "web")
entry = "app.cln"            # Main entry point (optional, default: "app.cln")

# Server configuration
[frame.server]
host = "127.0.0.1"           # Dev server host (optional, default: "127.0.0.1")
port = 3000                  # Dev server port (optional, default: 3000)
graceful_shutdown = true     # Enable graceful shutdown (optional, default: true)
shutdown_timeout = 30        # Shutdown timeout in seconds (optional, default: 30)

# Database configuration
[frame.database]
url = "sqlite:./dev.db"      # Database connection string (optional)
migrations = "db/migrations" # Migrations directory (optional, default: "db/migrations")
auto_migrate = false         # Auto-run migrations (optional, default: false)

# Build options
[frame.build]
optimize = true              # Enable optimizations in release (optional, default: true)
minify = true                # Minify output (optional, default: true for release)
sourcemap = true             # Generate source maps (optional, default: true in dev)
bundle = "single"            # Bundle strategy: "single" or "split" (optional, default: "single")

# Hot reload (dev only)
[frame.dev]
hot_reload = true            # Enable hot reload (optional, default: true)
watch = ["**/*.cln"]         # File patterns to watch (optional, default: all .cln)
debounce_ms = 500            # Debounce delay in ms (optional, default: 500)
```

**Valid Targets**:
- `web` - Browser-based web application
- `pwa` - Progressive Web App
- `mobile` - Mobile app (via Capacitor)
- `desktop` - Desktop app (via Tauri)
- `server` - Server-only application
- `cli` - Command-line tool

### [dependencies] - Optional

External package dependencies (future feature).

```toml
[dependencies]
# Format: package = "version"
http-client = "1.2.0"
json-parser = "0.5.0"

# Git dependencies
my-lib = { git = "https://github.com/user/my-lib", tag = "v1.0.0" }

# Path dependencies (for local development)
shared-utils = { path = "../shared" }
```

**Version Specifiers**:
- `"1.2.3"` - Exact version
- `"^1.2.0"` - Compatible with 1.2.0 (>= 1.2.0, < 2.0.0)
- `"~1.2.0"` - Patch compatible (>= 1.2.0, < 1.3.0)
- `"*"` - Any version (not recommended)

### [dev-dependencies] - Optional

Dependencies only needed during development.

```toml
[dev-dependencies]
test-framework = "2.0.0"
mock-server = "1.5.0"
```

### [build] - Optional

Build system configuration.

```toml
[build]
# Source directories
src = ["src", "app"]         # Source directories (optional, default: ["app", "src", "."])
exclude = ["**/*.test.cln"]  # Exclude patterns (optional, default: [])

# Compilation
target_arch = "wasm32"       # Target architecture (optional, default: "wasm32")
opt_level = 2                # Optimization level 0-3 (optional, default: 2 for release)

# Output
out_dir = "dist"             # Output directory (optional, default: "dist")
```

### [profile.release] - Optional

Release build profile.

```toml
[profile.release]
opt_level = 3                # Max optimization
minify = true                # Minify output
strip_debug = true           # Remove debug info
lto = true                   # Link-time optimization
```

### [profile.dev] - Optional

Development build profile.

```toml
[profile.dev]
opt_level = 0                # No optimization for faster builds
debug = true                 # Include debug symbols
incremental = true           # Incremental compilation
```

## Complete Example

```toml
# package.clean.toml - Frame Hello World

[package]
name = "hello-world"
version = "0.1.0"
authors = ["Frame Framework"]
edition = "2025"
description = "Hello World example for Frame Framework"
license = "MIT"

[frame]
version = "0.1.0"
target = "web"
entry = "app.cln"

[frame.server]
host = "127.0.0.1"
port = 3000

[frame.database]
url = "sqlite:./dev.db"
migrations = "db/migrations"

[frame.build]
optimize = true
bundle = "single"

[frame.dev]
hot_reload = true
debounce_ms = 500

[build]
src = ["app"]
exclude = ["**/*.test.cln"]
out_dir = "dist"
```

## Migration from package.frame.toml

For existing Frame projects using `package.frame.toml`, migration is straightforward:

**Before (package.frame.toml)**:
```toml
[frame]
name = "my-app"
version = "0.1.0"
```

**After (package.clean.toml)**:
```toml
[package]
name = "my-app"
version = "0.1.0"

[frame]
version = "0.1.0"
```

The CLI will support both formats during a transition period, with a warning to migrate.

## Compatibility

### Backward Compatibility

- Frame CLI v0.1.x will support both `package.frame.toml` and `package.clean.toml`
- Warning displayed when using deprecated `package.frame.toml`
- Auto-migration command: `frame migrate:manifest`

### Forward Compatibility

- Unknown sections are ignored (allows future extensions)
- Unknown keys in known sections log warnings
- Strict parsing can be enabled via `--strict` flag

## Validation

The CLI validates manifests on every command:

```bash
frame validate
```

**Checks**:
1. Required fields present (`package.name`, `package.version`)
2. Valid TOML syntax
3. Semantic version format
4. Valid target names
5. Path dependencies exist
6. No circular dependencies

**Exit Codes**:
- `0` - Valid manifest
- `1` - Validation errors
- `2` - Warnings (valid but suboptimal)

## Schema

JSON Schema for editor autocompletion available at:
```
https://frame.clean.dev/schemas/package.clean.toml.json
```

## Future Enhancements

Planned for future versions:

1. **Workspace support**: Multi-package projects
2. **Feature flags**: Conditional dependencies
3. **Platform-specific deps**: Different deps per target
4. **Build scripts**: Custom build steps
5. **Plugins**: Framework plugins configuration
6. **Assets**: Static asset management

## References

- Clean Language Specification
- Frame CLI Specification (02_frame_cli.md)
- TOML v1.0.0 Specification
- Cargo.toml (Rust) - inspiration
- package.json (Node.js) - inspiration

---

**Status**: Ready for implementation in Phase 2
