# Frame Framework - Clean Ecosystem Integration (REVISED)

**Date**: November 19, 2025
**Status**: Integration Strategy - Revised Understanding
**Purpose**: Define Frame's role in the evolving Clean ecosystem

---

## Corrected Understanding of Clean Ecosystem

### What Exists Today ✅

**1. Clean Manager (`cleen`) - Fully Implemented**
```bash
cleen install 0.5.0        # Downloads Clean compiler from GitHub
cleen use 0.5.0            # Activates version
cleen list                 # Lists installed versions
cleen available            # Shows available versions on GitHub
```

**How it works**:
- Downloads pre-built binaries from GitHub releases
- Stores in `~/.cleen/versions/<version>/`
- Creates shim in `~/.cleen/bin/cln`
- Manages **compiler versions only**, not packages

**2. Clean Compiler (`cln`) - Partial Implementation**
```bash
cln package init           # Command exists
cln package add <name>     # Command exists
cln package install        # Command exists
# But... full registry/download system not implemented yet
```

**What's defined but not fully working**:
- CLI command structure ✅
- `package.clean.toml` format ✅
- Package manifest parsing ✅
- Dependency resolution ❌ (partial)
- Package registry ❌ (not built)
- Package download/install ❌ (not built)
- Package publishing ❌ (not built)

### What Doesn't Exist Yet ⚠️

**Package Registry**:
- No central package repository
- No package search/discovery
- No package hosting infrastructure

**Package Download/Install**:
- No automated dependency installation
- No version resolution
- No transitive dependency handling

---

## Revised Strategy: Frame's Role in the Ecosystem

### Frame Framework as Ecosystem Pioneer

Since Clean's package management is still evolving, Frame has a unique opportunity:

**1. Define Best Practices** - Show how packages should work
**2. Drive Requirements** - Help shape the package system
**3. Provide Templates** - Demonstrate ideal package structure
**4. Bootstrap Ecosystem** - Be the first major framework

---

## Short-Term Solution (Today - Next 3 Months)

### Frame as Standalone Installation

Until Clean's package system is ready, Frame should:

**1. Direct Binary Distribution**
```bash
# Install Frame CLI directly (similar to cleen)
curl -sSL https://frame-framework.org/install.sh | bash

# Or via GitHub releases
wget https://github.com/clean-lang/frame/releases/latest/download/frame-<platform>.tar.gz
```

**2. Manual Dependency Management**
```bash
# Frame CLI creates projects with dependencies documented
frame new my-app

# Creates package.clean.toml (ready for when package manager works)
# But also includes README with manual setup instructions
```

**3. Bundled Runtime Components**
- Include all Frame modules in the CLI distribution
- No external downloads needed initially
- Self-contained framework

### Implementation Approach

```
Frame Framework Installation (Current)
├── frame CLI binary (standalone)
├── Includes bundled:
│   ├── frame-runtime (embedded)
│   ├── frame-server (embedded)
│   ├── frame-ui (embedded)
│   ├── frame-data (embedded)
│   └── frame-auth (embedded)
└── Generates package.clean.toml (for future compatibility)
```

**User Experience**:
```bash
# 1. Install Clean compiler
cleen install latest
cleen use latest

# 2. Install Frame framework
curl -sSL https://frame-framework.org/install.sh | bash
# Or: brew install frame-framework
# Or: Download binary from GitHub releases

# 3. Use Frame
frame new my-blog
cd my-blog
frame serve
```

---

## Medium-Term Solution (3-6 Months)

### Help Build Clean Package Management

Frame team can contribute to Clean Language by:

**1. Package Registry Design**
- Collaborate on registry architecture
- Propose registry API specification
- Help implement package hosting

**2. Package Manager Implementation**
- Contribute to `cln package` command implementation
- Implement dependency resolution
- Add download/install functionality

**3. Registry Infrastructure**
```
Proposed Clean Package Registry
├── Registry Server (API)
│   ├── Package search endpoint
│   ├── Package metadata endpoint
│   ├── Package download endpoint
│   └── Package publish endpoint
├── Package Storage
│   ├── GitHub releases (initial)
│   └── Dedicated CDN (future)
└── Package Index
    └── JSON/TOML index file
```

---

## Long-Term Vision (6+ Months)

### Frame Fully Integrated with Clean Ecosystem

Once Clean's package system is mature:

**1. Frame CLI as Package**
```bash
# Install via Clean package manager
cln package add --global frame
```

**2. Frame Components as Packages**
```toml
# package.clean.toml
[dependencies]
frame-runtime = "^1.0.0"
frame-ui = "^1.0.0"
frame-data = "^1.0.0"
```

**3. Automatic Dependency Resolution**
```bash
frame new my-app
cd my-app
cln package install  # Automatically downloads all Frame dependencies
```

**4. Plugin Ecosystem**
```bash
cln package add frame-plugin-seo
cln package add frame-plugin-analytics
cln package search frame-plugin
```

---

## Immediate Action Plan

### Phase 1: Frame as Standalone (Now - Week 5)

**Deliverables**:
1. Frame CLI as binary distribution
2. Bundled runtime components
3. Installer script (install.sh, install.ps1)
4. GitHub releases automation
5. Documentation for manual installation

**Distribution Channels**:
- GitHub Releases (primary)
- Direct download scripts
- Homebrew (macOS) - future
- Chocolatey (Windows) - future
- APT/YUM repos (Linux) - future

**File Structure**:
```
frame-cli/
├── Cargo.toml
├── build.rs                  # Build script for bundling
├── src/
│   ├── main.rs              # CLI entry point
│   ├── embedded/            # Bundled components
│   │   ├── runtime.wasm
│   │   ├── server.wasm
│   │   └── ui.wasm
│   └── templates/           # Project templates
└── install.sh               # Installation script
```

### Phase 2: Prepare for Package System (Week 6-12)

**Deliverables**:
1. Create `package.clean.toml` for each Frame module
2. Organize code for future package split
3. Define package dependencies clearly
4. Document package structure

**Module Package Manifests**:

**frame-runtime/package.clean.toml**:
```toml
[package]
name = "frame-runtime"
version = "1.0.0"
description = "Frame Framework core runtime - WASM execution and host bridge"
authors = ["Frame Framework Team"]
license = "MIT"
repository = "https://github.com/clean-lang/frame-framework"

[dependencies]
# Pure runtime, no dependencies

[build]
target = "wasm32-unknown-unknown"
optimization = "size"
```

**frame-ui/package.clean.toml**:
```toml
[package]
name = "frame-ui"
version = "1.0.0"
description = "Frame Framework UI components with SSR and islands"
authors = ["Frame Framework Team"]
license = "MIT"

[dependencies]
frame-runtime = "^1.0.0"

[build]
target = "wasm32-unknown-unknown"
features = ["ssr", "islands"]
```

**frame-data/package.clean.toml**:
```toml
[package]
name = "frame-data"
version = "1.0.0"
description = "Frame Framework ORM with type-safe queries"
authors = ["Frame Framework Team"]
license = "MIT"

[dependencies]
frame-runtime = "^1.0.0"

[build]
target = "wasm32-unknown-unknown"
```

### Phase 3: Contribute to Clean Package System (Week 13+)

**Collaboration Areas**:

**1. Registry Specification**
```markdown
# Proposed Package Registry API

## Endpoints

GET /packages                    # List all packages
GET /packages/{name}             # Package metadata
GET /packages/{name}/{version}   # Version-specific metadata
GET /packages/{name}/{version}/download  # Download package
POST /packages                   # Publish package (authenticated)

## Package Metadata Format

{
  "name": "frame-ui",
  "version": "1.0.0",
  "description": "...",
  "authors": ["..."],
  "license": "MIT",
  "repository": "https://github.com/...",
  "download_url": "https://registry.clean-lang.org/packages/frame-ui/1.0.0/download",
  "checksum": "sha256:...",
  "dependencies": {
    "frame-runtime": "^1.0.0"
  },
  "published_at": "2025-11-19T00:00:00Z"
}
```

**2. Package Manager Implementation**

Help implement in Clean compiler:
- Dependency resolution algorithm
- Package download and extraction
- Lock file generation (`package.clean.lock`)
- Transitive dependency handling
- Version conflict resolution

**3. Registry Infrastructure**

Options for hosting:
- **GitHub Releases** (simple, free)
- **Custom Registry Server** (full control)
- **Cloudflare Workers** (edge distribution)
- **AWS S3 + CloudFront** (scalable)

---

## Package.clean.toml Best Practices

Based on Frame's needs, here are recommendations for the package format:

### Complete Example

```toml
[package]
name = "frame-ui"
version = "1.0.0"
description = "Frame Framework UI components with SSR and islands architecture"
authors = ["Frame Framework Team <team@frame-framework.org>"]
license = "MIT"
repository = "https://github.com/clean-lang/frame-framework"
homepage = "https://frame-framework.org"
documentation = "https://frame-framework.org/docs"
readme = "README.md"
keywords = ["web", "framework", "ui", "ssr", "islands"]
categories = ["web-programming", "wasm-bindings"]

# Minimum Clean compiler version
clean_version = ">=0.5.0"

[dependencies]
# Required dependencies
frame-runtime = "^1.0.0"

# Optional dependencies (feature-gated)
frame-animations = { version = "^1.0.0", optional = true }

[dev_dependencies]
test-framework = "^1.0.0"
benchmark-utils = "^0.5.0"

[features]
# Feature flags
default = ["ssr"]
ssr = []
islands = ["ssr"]
animations = ["dep:frame-animations"]
full = ["ssr", "islands", "animations"]

[build]
target = "wasm32-unknown-unknown"
optimization = "size"
features = ["ssr", "islands"]
exclude = [
    "tests/",
    "examples/",
    "benches/",
    ".github/",
]

[scripts]
# Custom build scripts
prebuild = "cln run scripts/prebuild.cln"
postbuild = "cln run scripts/optimize-wasm.cln"
test = "cln test --all-features"

[package.metadata.frame]
# Custom metadata for Frame tooling
component_style = "islands"
default_theme = "clean"
```

---

## Transition Plan

### Today (No Package System)

```bash
# Install Frame
curl -sSL https://frame-framework.org/install.sh | bash

# Create project
frame new my-app

# Generated package.clean.toml (documented but not functional)
# Project works without package manager
```

### When Package System is Ready

```bash
# Install Frame (now via package manager)
cln package add --global frame

# Create project
frame new my-app

# Install dependencies (now works!)
cln package install

# Add packages
cln package add frame-plugin-seo
```

**Migration Path**: Zero breaking changes!
- Projects already have `package.clean.toml`
- Just start working when `cln package install` is ready
- Existing projects get package management for free

---

## Benefits of This Approach

### For Frame Users (Short-Term)

✅ **Works Today** - No waiting for package system
✅ **Simple Install** - One script, everything bundled
✅ **Self-Contained** - No dependency hell
✅ **Fast Start** - Get building immediately

### For Clean Language (Medium-Term)

✅ **Drives Requirements** - Frame's needs guide package system design
✅ **Real-World Testing** - Frame tests package format in production
✅ **Flagship Framework** - Shows what Clean can do
✅ **Ecosystem Growth** - Frame attracts developers to Clean

### For Future (Long-Term)

✅ **Seamless Transition** - package.clean.toml already there
✅ **Modular Architecture** - Can split into packages when ready
✅ **Plugin Ecosystem** - Packages enable community extensions
✅ **Version Management** - Proper dependency resolution

---

## Collaboration with Clean Team

### Recommended Discussion Topics

**1. Package System Timeline**
- When is package management planned?
- Can Frame team help implement it?
- What features are highest priority?

**2. Package Format**
- Is current `package.clean.toml` format final?
- Should we add any fields based on Frame's needs?
- How to handle native dependencies (Rust crates)?

**3. Registry Design**
- Centralized or decentralized?
- GitHub releases or custom server?
- How to handle package namespacing?
- Authentication for publishing?

**4. Version Management**
- How does `cleen` relate to package versions?
- Per-project compiler versions?
- Lock file format?

---

## Immediate Next Steps

### This Week (Week 5)

1. **Create Package Manifests**
   - [ ] `frame-runtime/package.clean.toml`
   - [ ] `frame-server/package.clean.toml`
   - [ ] `frame-ui/package.clean.toml`
   - [ ] `frame-data/package.clean.toml`
   - [ ] `frame-auth/package.clean.toml`
   - [ ] `frame-cli/package.clean.toml`

2. **Design Standalone Distribution**
   - [ ] Bundling strategy (embed runtime WASMs)
   - [ ] Install script (Unix/Windows)
   - [ ] GitHub release automation
   - [ ] Version management

3. **Start Frame CLI Development** (Phase 2)
   - [ ] CLI argument parsing (clap)
   - [ ] `frame new` command
   - [ ] Project template generation
   - [ ] Integration with `cln` compiler

### Next 2 Weeks (Week 6-7)

4. **Build Distribution System**
   - [ ] CI/CD for binary builds
   - [ ] Cross-platform compilation
   - [ ] Release automation
   - [ ] Version tagging

5. **Reach Out to Clean Team**
   - [ ] Share Frame's package requirements
   - [ ] Offer to help build package system
   - [ ] Discuss registry architecture
   - [ ] Coordinate timeline

---

## Conclusion

**Frame Framework will:**

1. **Start standalone** - Work without package system (bundled distribution)
2. **Be package-ready** - Include `package.clean.toml` from day one
3. **Help build ecosystem** - Contribute to Clean package management
4. **Transition seamlessly** - Migrate to packages when system is ready

This approach:
- ✅ Unblocks Frame development now
- ✅ Prepares for future package system
- ✅ Helps Clean Language mature
- ✅ Provides best user experience at each stage

**Frame doesn't need to wait for Clean's package system to be useful!**

---

**Report Prepared By**: Development Team
**Report Date**: November 19, 2025
**Version**: 2.0 (Revised)
**Status**: Revised Strategy - Ready to Implement
