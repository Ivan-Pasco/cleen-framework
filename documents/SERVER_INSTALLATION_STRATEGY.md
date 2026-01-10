# Frame Server Installation Strategy

**Date:** January 2025  
**Status:** Analysis & Recommendations  
**Purpose:** Determine installation strategy for Frame Runtime (server service)

---

## Executive Summary

The **Frame Runtime** (`frame-runtime`) is the HTTP server service that executes compiled Clean Framework WASM applications. Currently, it's part of the framework repository but **not independently installable**. This document analyzes whether it should be:

1. **Part of the framework** (installed with `cleen install frame`)
2. **Standalone service** (installable independently for production)
3. **Both** (framework includes it, but also available standalone)

**Recommendation:** **Both** - Include in framework installation AND provide standalone installation option.

---

## Current State Analysis

### What is Frame Runtime?

**Frame Runtime** (`frame-runtime`) is a Rust binary that:

- **HTTP Server**: Uses Axum to handle HTTP requests
- **WASM Execution**: Uses Wasmtime to execute compiled Clean Language WASM modules
- **Route Management**: Dynamically registers routes from WASM modules
- **Host Bridge Integration**: Provides system capabilities (HTTP, DB, Crypto, etc.) to WASM

**Architecture:**
```
HTTP Request → Axum Server → Router → WASM Instance → Host Bridge → System Resources
```

**Current Location:**
- Source: `frame-runtime/` directory in clean-framework repository
- Binary: `frame-runtime` (built from Rust)
- Purpose: Run compiled WASM applications as HTTP servers

### Current Installation Status

**❌ NOT Currently Standalone Installable**

1. **Framework Installation**: 
   - `cleen install frame` is documented but not fully implemented
   - Would install framework components but unclear if runtime is included

2. **Manual Installation**:
   - Must build from source: `cargo build --release --bin frame-runtime`
   - Binary must be manually copied to PATH
   - No package manager support

3. **Production Deployment**:
   - Currently requires building from source or bundling in Docker
   - No standalone binary distribution

---

## Use Cases Analysis

### Use Case 1: Development (Framework Installation)

**Scenario:** Developer wants to build Frame applications

**Needs:**
- Compiler (`cln`)
- Plugins (`frame.web`, `frame.data`, etc.)
- Runtime (`frame-runtime`) to test locally
- CLI tools for project management

**Current Solution:** Build from source or install via `cleen install frame` (when implemented)

**Recommendation:** ✅ Include in `cleen install frame`

### Use Case 2: Production Deployment (Standalone Service)

**Scenario:** DevOps engineer deploys Frame application to production

**Needs:**
- Only the runtime server (`frame-runtime`)
- Does NOT need compiler or plugins
- Minimal dependencies
- Easy installation on production servers

**Current Solution:** ❌ Must build from source or bundle in Docker

**Recommendation:** ✅ Provide standalone installation option

### Use Case 3: CI/CD Pipelines

**Scenario:** Automated testing and deployment

**Needs:**
- Runtime available in CI environment
- Fast installation
- Version pinning

**Current Solution:** ❌ Build from source or use Docker

**Recommendation:** ✅ Standalone installation with version support

### Use Case 4: Multiple Applications on Same Server

**Scenario:** Hosting multiple Frame apps on one server

**Needs:**
- Single runtime installation
- Multiple WASM files
- Version management

**Current Solution:** ❌ Must build/install runtime per application

**Recommendation:** ✅ Standalone installation shared across applications

---

## Recommended Strategy: Dual Installation Model

### Strategy Overview

**Both Framework and Standalone Installation**

1. **Framework Installation** (`cleen install frame`):
   - Includes runtime as part of framework bundle
   - For developers building applications
   - Includes compiler, plugins, runtime, and CLI tools

2. **Standalone Installation** (`cleen install frame-runtime`):
   - Runtime only, minimal dependencies
   - For production deployments
   - Can be installed independently

### Implementation Plan

#### 1. Framework Installation (`cleen install frame`)

**What Gets Installed:**
```
~/.cleen/versions/frameworks/frame/<version>/
├── bin/
│   ├── frame              # Framework CLI (if exists)
│   └── frame-runtime       # Runtime server
├── plugins/                # Framework plugins
│   ├── frame.web/
│   ├── frame.data/
│   ├── frame.auth/
│   └── frame.ui/
└── lib/                    # Shared libraries
```

**Commands:**
```bash
cleen install frame                    # Install latest framework
cleen install frame@1.0.0            # Install specific version
cleen use frame 1.0.0                # Activate framework version
```

**Use Case:** Development, building applications

#### 2. Standalone Runtime Installation (`cleen install frame-runtime`)

**What Gets Installed:**
```
~/.cleen/versions/runtimes/frame-runtime/<version>/
├── bin/
│   └── frame-runtime       # Runtime server only
└── lib/                    # Runtime dependencies (minimal)
```

**Commands:**
```bash
cleen install frame-runtime           # Install latest runtime
cleen install frame-runtime@1.0.0   # Install specific version
cleen use frame-runtime 1.0.0       # Activate runtime version
```

**Use Case:** Production deployment, CI/CD, shared hosting

#### 3. Binary Distribution

**Release Artifacts:**
- `frame-runtime-<platform>-<arch>.tar.gz` - Standalone runtime
- `frame-framework-<platform>-<arch>.tar.gz` - Full framework (includes runtime)

**Platforms:**
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

---

## Clean Manager Integration

### Extended `cleen` Commands

```bash
# Framework installation (includes runtime)
cleen install frame
cleen install frame@1.0.0
cleen list frameworks

# Standalone runtime installation
cleen install frame-runtime
cleen install frame-runtime@1.0.0
cleen list runtimes

# Version management
cleen use frame 1.0.0              # Use framework version
cleen use frame-runtime 1.0.0      # Use runtime version

# Uninstallation
cleen uninstall frame 1.0.0
cleen uninstall frame-runtime 1.0.0
```

### Directory Structure

```
~/.cleen/
├── bin/
│   ├── cln                      # Compiler shim
│   ├── frame                    # Framework CLI shim (if exists)
│   └── frame-runtime            # Runtime shim
├── versions/
│   ├── compiler/                # Clean compiler versions
│   │   └── 0.5.0/
│   ├── frameworks/              # Framework installations
│   │   └── frame/
│   │       └── 1.0.0/
│   │           ├── bin/
│   │           │   ├── frame
│   │           │   └── frame-runtime
│   │           └── plugins/
│   └── runtimes/                # Standalone runtime installations
│       └── frame-runtime/
│           └── 1.0.0/
│               └── bin/
│                   └── frame-runtime
└── config.json
```

---

## Benefits of Dual Installation Model

### 1. **Flexibility**
- Developers get everything they need with `cleen install frame`
- Production servers only install what's needed with `cleen install frame-runtime`

### 2. **Size Optimization**
- Framework installation: ~50-100 MB (includes compiler, plugins, runtime)
- Runtime installation: ~10-20 MB (runtime only)

### 3. **Version Independence**
- Framework and runtime can have different version numbers
- Production can pin runtime version independently
- Framework updates don't force runtime updates

### 4. **Security**
- Production servers don't need compiler or plugins
- Reduced attack surface
- Minimal dependencies

### 5. **CI/CD Friendly**
- Fast runtime installation for testing
- No need to install full framework in CI

---

## Implementation Requirements

### 1. Build System Changes

**Create separate release targets:**

```toml
# Cargo.toml workspace
[workspace]
members = [
    "host-bridge",
    "frame-runtime",      # Standalone runtime
]

# Build scripts
scripts/build-runtime.sh    # Build runtime only
scripts/build-framework.sh # Build framework bundle
```

### 2. Release Process

**GitHub Actions Workflow:**
```yaml
# Build runtime standalone
- name: Build Runtime
  run: cargo build --release --bin frame-runtime

# Build framework bundle
- name: Build Framework
  run: |
    cargo build --release --bin frame-runtime
    cargo build --release --bin frame  # If CLI exists
    ./scripts/build-plugins.sh

# Create release artifacts
- name: Package Runtime
  run: tar czf frame-runtime-${{ matrix.platform }}.tar.gz frame-runtime

- name: Package Framework
  run: tar czf frame-framework-${{ matrix.platform }}.tar.gz frame-runtime frame plugins
```

### 3. Clean Manager Extensions

**Add to `cleen`:**

```rust
// New commands
cleen install frame-runtime
cleen list runtimes
cleen use frame-runtime <version>
cleen uninstall frame-runtime <version>
```

**Registry Support:**
- Runtime releases: `https://github.com/clean-language/frame/releases/tag/frame-runtime-v1.0.0`
- Framework releases: `https://github.com/clean-language/frame/releases/tag/frame-v1.0.0`

---

## Migration Path

### Phase 1: Current State (Now)
- ✅ Runtime exists in repository
- ❌ Not standalone installable
- ❌ Framework installation incomplete

### Phase 2: Framework Installation (Short-term)
- ✅ Implement `cleen install frame`
- ✅ Include runtime in framework bundle
- ✅ Test with developers

### Phase 3: Standalone Runtime (Medium-term)
- ✅ Add `cleen install frame-runtime`
- ✅ Create separate release artifacts
- ✅ Document production deployment

### Phase 4: Full Integration (Long-term)
- ✅ Version management for both
- ✅ CI/CD integration
- ✅ Production deployment guides

---

## Recommendations

### ✅ **RECOMMENDED: Dual Installation Model**

1. **Framework Installation** (`cleen install frame`):
   - Include runtime as part of framework
   - For developers building applications
   - One command installs everything

2. **Standalone Runtime** (`cleen install frame-runtime`):
   - Runtime only, minimal dependencies
   - For production deployments
   - Smaller, faster installation

### Implementation Priority

1. **High Priority**: Framework installation with runtime included
   - Enables developer workflow
   - Most common use case

2. **Medium Priority**: Standalone runtime installation
   - Enables production deployments
   - CI/CD integration

3. **Low Priority**: Advanced features
   - Version pinning
   - Multiple runtime versions
   - Runtime plugins/extensions

---

## Questions for Decision

1. **Should runtime be included in `cleen install frame`?**
   - ✅ **YES** - Developers need it to test applications

2. **Should runtime be independently installable?**
   - ✅ **YES** - Production servers don't need full framework

3. **Should runtime have separate versioning?**
   - ⚠️ **MAYBE** - Can start same as framework, split later if needed

4. **Should runtime be installable via system package managers?**
   - ⚠️ **FUTURE** - Start with `cleen`, add system packages later

---

## Conclusion

**Frame Runtime should be BOTH:**
- ✅ Part of framework installation (`cleen install frame`)
- ✅ Independently installable (`cleen install frame-runtime`)

This provides:
- **Developer convenience**: One command installs everything
- **Production efficiency**: Install only what's needed
- **Flexibility**: Choose installation method based on use case
- **Future-proof**: Can evolve independently if needed

**Next Steps:**
1. Implement framework installation in `cleen`
2. Add standalone runtime installation option
3. Create release artifacts for both
4. Update documentation

---

**End of Document**




































