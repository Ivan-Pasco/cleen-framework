# Frame Framework - Clean Ecosystem Integration

**Date**: November 19, 2025
**Status**: Analysis & Integration Plan
**Purpose**: Define how Frame Framework integrates with Clean ecosystem

---

## Executive Summary

After investigating the Clean Language ecosystem, I've discovered that **package management already exists** in the Clean Language compiler (`cln`). The Frame Framework should leverage this existing infrastructure rather than creating a separate system.

### Current Clean Ecosystem

```
┌─────────────────────────────────────────────┐
│          Clean Language Ecosystem            │
├─────────────────────────────────────────────┤
│                                              │
│  1. cleen (Clean Manager)                   │
│     - Compiler version management            │
│     - Similar to rustup/nvm                  │
│     - Commands: install, use, list, etc.    │
│                                              │
│  2. cln (Clean Compiler)                     │
│     - Compiles Clean → WebAssembly          │
│     - BUILT-IN package management:          │
│       • cln package init                     │
│       • cln package add <dep>                │
│       • cln package install                  │
│       • cln package publish                  │
│       • cln package search/info              │
│                                              │
│  3. package.clean.toml                       │
│     - Package manifest format                │
│     - Similar to Cargo.toml/package.json    │
│                                              │
└─────────────────────────────────────────────┘
```

---

## Discovery: Clean Already Has Package Management

### What I Found

The Clean Language compiler (`cln`) has **complete package management** built-in:

```bash
# Initialize a new package
cln package init

# Add dependencies
cln package add math-utils
cln package add http-client --version "^2.0.0"

# Install all dependencies
cln package install

# Update dependencies
cln package update

# Search for packages
cln package search http

# Get package info
cln package info math-utils

# Publish a package
cln package publish
```

### Package Manifest Format

**File**: `package.clean.toml`

```toml
[package]
name = "my-app"
version = "1.0.0"
description = "My Clean Language application"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/user/my-app"
homepage = "https://example.com"
keywords = ["web", "framework"]
categories = ["web-programming"]

[dependencies]
# Simple version
math-utils = "1.0.0"

# Caret requirement (compatible updates)
http-client = "^2.0.0"

# Detailed specification
async-runtime = { version = "1.5.0", optional = true, features = ["tokio"] }

# Git dependency
custom-lib = { git = "https://github.com/user/lib", branch = "main" }

# Local path dependency
local-utils = { path = "../utils" }

[dev_dependencies]
test-framework = "^1.0.0"

[build]
target = "wasm32-unknown-unknown"
optimization = "size"
features = ["async"]
exclude = ["tests/", "examples/"]
```

### Supported Dependency Types

1. **Registry packages**: `package = "1.0.0"`
2. **Git repositories**: `{ git = "...", branch/tag = "..." }`
3. **Local paths**: `{ path = "../local-package" }`
4. **Custom registries**: `{ version = "...", registry = "..." }`
5. **Optional dependencies**: `{ version = "...", optional = true }`
6. **Feature flags**: `{ version = "...", features = [...] }`

### Version Requirements

```toml
# Exact version
exact = "1.2.3"

# Caret (compatible within major)
caret = "^1.2.3"  # >=1.2.3, <2.0.0

# Tilde (compatible within minor)
tilde = "~1.2.3"  # >=1.2.3, <1.3.0

# Comparison operators
greater = ">1.0.0"
greater-equal = ">=1.0.0"
less = "<2.0.0"
less-equal = "<=1.9.9"

# Wildcards
wildcard-minor = "1.*"      # Any 1.x.x
wildcard-patch = "1.2.*"    # Any 1.2.x
```

---

## How Frame Framework Fits In

### Integration Strategy

Frame Framework should **extend, not replace** the existing Clean package management:

```
┌─────────────────────────────────────────────┐
│     Frame Framework Integration              │
├─────────────────────────────────────────────┤
│                                              │
│  1. Frame as a Package                       │
│     - Published to Clean registry            │
│     - Installed via: cln package add frame   │
│     - Or global: cln package add --global    │
│                                              │
│  2. Frame CLI Tool                           │
│     - Scaffolding: frame new <app>           │
│     - Development: frame serve               │
│     - Building: frame build                  │
│     - Delegates to cln for compilation       │
│                                              │
│  3. Frame Components as Packages             │
│     - frame-runtime (core runtime)           │
│     - frame-ui (UI components)               │
│     - frame-data (ORM)                       │
│     - frame-auth (authentication)            │
│     - frame-server (server runtime)          │
│     - Auto-added by `frame new`              │
│                                              │
└─────────────────────────────────────────────┘
```

---

## Recommended Installation Flow

### 1. Install Clean Manager (One-Time)

```bash
# Install Clean Manager (version manager)
curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash

# Initialize environment
cleen init

# Install latest Clean compiler
cleen install latest
cleen use latest
```

### 2. Install Frame Framework

**Option A: Global CLI Installation** (Recommended)
```bash
# Install Frame CLI as a global tool
cln package add --global frame

# Or via Clean Manager if it supports global packages
cleen install-package frame

# Now 'frame' command is available
frame --version
```

**Option B: Per-Project Installation**
```bash
# Create a new directory
mkdir my-app
cd my-app

# Initialize with Frame
frame new .

# This creates package.clean.toml with Frame dependencies
# and runs 'cln package install' automatically
```

### 3. Use Frame Framework

```bash
# Create new project (Frame CLI does this)
frame new my-blog

cd my-blog

# Auto-generated package.clean.toml:
# [dependencies]
# frame-runtime = "1.0.0"
# frame-ui = "1.0.0"
# frame-data = "1.0.0"
# frame-server = "1.0.0"

# Install dependencies (done automatically by 'frame new')
cln package install

# Start development server
frame serve

# Build for production
frame build --target=server
```

---

## Frame Package Structure

### Published Packages

Frame should be split into multiple packages on the Clean registry:

#### 1. `frame` (CLI Tool)
```toml
[package]
name = "frame"
version = "1.0.0"
description = "Full-stack WebAssembly framework CLI for Clean Language"
license = "MIT"

[bin]
frame = "src/main.cln"  # Or compiled Rust binary

[dependencies]
frame-runtime = "1.0.0"
```

#### 2. `frame-runtime` (Core Runtime)
```toml
[package]
name = "frame-runtime"
version = "1.0.0"
description = "Frame Framework core runtime - WASM execution and host bridge"

[dependencies]
# No external dependencies - pure Clean/Rust
```

#### 3. `frame-ui` (UI Components)
```toml
[package]
name = "frame-ui"
version = "1.0.0"
description = "Frame Framework UI components with SSR and islands architecture"

[dependencies]
frame-runtime = "1.0.0"
```

#### 4. `frame-data` (ORM)
```toml
[package]
name = "frame-data"
version = "1.0.0"
description = "Frame Framework ORM with type-safe queries and migrations"

[dependencies]
frame-runtime = "1.0.0"
```

#### 5. `frame-server` (Server Runtime)
```toml
[package]
name = "frame-server"
version = "1.0.0"
description = "Frame Framework server runtime with routing and SSR"

[dependencies]
frame-runtime = "1.0.0"
frame-ui = "1.0.0"
```

#### 6. `frame-auth` (Authentication)
```toml
[package]
name = "frame-auth"
version = "1.0.0"
description = "Frame Framework authentication and authorization"

[dependencies]
frame-runtime = "1.0.0"
frame-server = "1.0.0"
```

### Meta-Package: `frame-full`

For convenience, a meta-package that includes everything:

```toml
[package]
name = "frame-full"
version = "1.0.0"
description = "Frame Framework - complete bundle with all components"

[dependencies]
frame-runtime = "1.0.0"
frame-server = "1.0.0"
frame-ui = "1.0.0"
frame-data = "1.0.0"
frame-auth = "1.0.0"
```

---

## Generated Project Structure

When running `frame new my-app`, it should create:

```
my-app/
├── package.clean.toml          # Package manifest (auto-generated)
├── .gitignore
├── README.md
├── app/
│   ├── api/
│   │   └── users.cln          # API endpoints
│   ├── pages/
│   │   └── index.cln          # UI pages
│   └── layout.cln             # Layout component
├── config/
│   ├── database.cln           # DB configuration
│   ├── ui.cln                 # UI theming
│   └── roles.cln              # Auth roles
├── data/
│   └── schema.cln             # Data models
└── public/
    └── favicon.ico            # Static assets
```

**Auto-generated `package.clean.toml`:**
```toml
[package]
name = "my-app"
version = "0.1.0"
description = "A Frame Framework application"

[dependencies]
frame-runtime = "^1.0.0"
frame-server = "^1.0.0"
frame-ui = "^1.0.0"
frame-data = "^1.0.0"
frame-auth = "^1.0.0"

[build]
target = "wasm32-unknown-unknown"
optimization = "size"
```

---

## Frame CLI Commands

### Project Management

```bash
# Create new project
frame new my-app
frame new my-app --template=blog
frame new my-app --template=api-only

# Serve development server
frame serve
frame serve --port 8080

# Build for production
frame build
frame build --target=server
frame build --target=web
frame build --target=mobile
```

### Database Commands

```bash
# Generate migration
frame db:generate AddUserTable

# Run migrations
frame db:migrate
frame db:migrate --to=20231119_001

# Rollback migration
frame db:rollback
frame db:rollback --steps=2

# Seed database
frame db:seed

# Database REPL
frame db:console
```

### Package Commands (Delegate to cln)

```bash
# Add Frame plugin
cln package add frame-plugin-seo
cln package add frame-plugin-analytics

# Update Frame components
cln package update frame-ui
cln package update  # Update all
```

---

## Implementation Plan

### Phase 1: Package Manifest Files
- [ ] Create `package.clean.toml` for each Frame module
- [ ] Define dependencies between modules
- [ ] Set version numbers (start at 1.0.0)
- [ ] Add metadata (authors, license, repository, etc.)

### Phase 2: Frame CLI as Package
- [ ] Decide: Rust binary or Clean source?
  - **Recommendation**: Rust binary (faster, standalone)
- [ ] Create publishable package structure
- [ ] Configure as global binary package
- [ ] Test installation via `cln package add --global frame`

### Phase 3: Registry Publication
- [ ] Investigate Clean package registry
  - Where is it hosted?
  - How to publish?
  - Authentication required?
- [ ] Publish all Frame modules to registry
- [ ] Set up CI/CD for automated publishing

### Phase 4: Integration Testing
- [ ] Test full workflow:
  1. Install Clean Manager
  2. Install Clean compiler
  3. Install Frame CLI
  4. Create new project
  5. Build and run
- [ ] Verify all dependencies resolve correctly
- [ ] Test version upgrades

### Phase 5: Documentation
- [ ] Update installation docs
- [ ] Add package.clean.toml examples
- [ ] Document integration with Clean ecosystem
- [ ] Create migration guide from standalone to ecosystem

---

## Questions to Resolve

### 1. Package Registry
**Question**: Where is the Clean package registry hosted?
- GitHub releases?
- Custom registry server?
- Decentralized (Git only)?

**Impact**: Affects publication workflow

### 2. Global Binary Packages
**Question**: Does `cln package add --global` support binaries?
- If yes: Frame CLI can be installed globally
- If no: Need alternative (maybe via cleen)

**Impact**: Installation UX

### 3. Binary vs Source Distribution
**Question**: Should Frame CLI be:
- A) Compiled Rust binary (faster, platform-specific)
- B) Clean source code (portable, slower startup)

**Recommendation**: Rust binary
- Faster execution
- No compilation needed on install
- Can still use Clean for templates/scaffolding

### 4. Monorepo vs Multi-Repo
**Question**: Should Frame modules be:
- A) Single monorepo (easier versioning)
- B) Separate repositories (independent versions)

**Recommendation**: Monorepo with workspace
- Unified versioning
- Easier development
- Publish separately to registry

---

## Proposed File Structure for Publishing

```
clean-framework/
├── Cargo.toml                  # Rust workspace
├── package.clean.toml          # Root manifest (workspace)
├── frame-cli/
│   ├── Cargo.toml             # Rust build
│   ├── package.clean.toml     # Package manifest
│   └── src/
├── frame-runtime/
│   ├── package.clean.toml
│   └── src/
├── frame-server/
│   ├── package.clean.toml
│   └── src/
├── frame-ui/
│   ├── package.clean.toml
│   └── src/
├── frame-data/
│   ├── package.clean.toml
│   └── src/
└── frame-auth/
    ├── package.clean.toml
    └── src/
```

**Root `package.clean.toml` (Workspace)**:
```toml
[workspace]
members = [
    "frame-cli",
    "frame-runtime",
    "frame-server",
    "frame-ui",
    "frame-data",
    "frame-auth"
]

[workspace.package]
version = "1.0.0"
authors = ["Frame Framework Team"]
license = "MIT"
repository = "https://github.com/clean-lang/frame-framework"
```

---

## Recommended User Journey

### First-Time Setup (5 minutes)

```bash
# 1. Install Clean Manager
curl -sSL https://clean-lang.org/install.sh | bash

# 2. Initialize and install compiler
cleen init
cleen install latest
cleen use latest

# 3. Install Frame CLI globally
cln package add --global frame

# 4. Verify setup
cln --version      # Clean Language v0.5.0
frame --version    # Frame Framework v1.0.0
```

### Creating a Project (1 minute)

```bash
# 1. Create new Frame app
frame new my-blog-app

# 2. Enter directory
cd my-blog-app

# 3. Dependencies already installed by 'frame new'
# (it runs 'cln package install' automatically)

# 4. Start development
frame serve
```

### Adding Packages (30 seconds)

```bash
# Add a Frame plugin
cln package add frame-plugin-seo

# Add any Clean package
cln package add date-utils
cln package add markdown-parser

# Install
cln package install
```

---

## Benefits of This Approach

### For Users

1. **Single Ecosystem**: One package manager for everything (Clean compiler manages packages)
2. **Familiar Workflow**: Similar to Rust/Cargo, Go modules, npm
3. **Automatic Dependencies**: `frame new` sets up everything
4. **Version Management**: Both compiler and packages versioned
5. **Unified Tooling**: `cln` for packages, `cleen` for compiler versions

### For Frame Framework

1. **Leverage Existing Infrastructure**: Don't reinvent package management
2. **Clean Integration**: Frame is "just another package"
3. **Ecosystem Compatibility**: Works with all Clean packages
4. **Simple Distribution**: Publish to existing registry
5. **Modular Architecture**: Users can choose which Frame modules to use

### For Clean Language

1. **Flagship Framework**: Frame showcases Clean's capabilities
2. **Ecosystem Growth**: Encourages more Clean packages
3. **Real-World Usage**: Frame provides production use cases
4. **Community Building**: Active framework drives adoption

---

## Next Steps

**Immediate (This Week)**:
1. ✅ Research Clean ecosystem (DONE)
2. [ ] Create `package.clean.toml` for all Frame modules
3. [ ] Test publishing to Clean registry (if available)

**Short-Term (Next 2 Weeks)**:
4. [ ] Build Frame CLI as publishable binary
5. [ ] Set up CI/CD for package publishing
6. [ ] Create installation documentation

**Medium-Term (Next Month)**:
7. [ ] Coordinate with Clean Language team on registry
8. [ ] Publish Frame v1.0.0 to registry
9. [ ] Update all documentation with ecosystem integration

---

## Conclusion

**Frame Framework should fully embrace the Clean Language ecosystem** by:

1. Using `cln package` commands for dependency management
2. Publishing Frame modules to the Clean registry
3. Providing Frame CLI as a global tool
4. Generating `package.clean.toml` in new projects
5. Leveraging existing infrastructure instead of building custom solutions

This approach provides the best user experience, integrates seamlessly with the Clean ecosystem, and positions Frame as the premier framework for Clean Language development.

---

**Report Prepared By**: Development Team
**Report Date**: November 19, 2025
**Version**: 1.0
**Status**: Analysis Complete - Ready for Implementation
