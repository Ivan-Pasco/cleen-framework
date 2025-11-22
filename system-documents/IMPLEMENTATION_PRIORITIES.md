# Frame Framework - Implementation Priorities

**Date:** 2025-11-22
**Current Completion:** ~80%

## ✅ Production-Ready Components

These components are complete and ready for production use:

### 1. Compiler & Plugins (100%)
- **Status:** Fully production-ready
- **Tests:** 38/38 passing
- **Features:**
  - WebPlugin: `endpoints:` DSL for HTTP routing
  - DataPlugin: `data:` DSL for ORM models
  - ComponentPlugin: `component:` DSL for UI components
- **Next:** No action needed, fully complete

### 2. Host Bridge (100%)
- **Status:** Fully production-ready
- **Tests:** 160/160 passing
- **Features:**
  - 8 bridge namespaces (http, db, env, time, crypto, log, fs, sys)
  - Complete JSON envelope API
  - Async support throughout
- **Next:** No action needed, fully complete

### 3. Frame CLI (90%)
- **Status:** Production-ready for basic use
- **Tests:** All commands working
- **Features:**
  - `frame new` - Project scaffolding (3 templates)
  - `frame serve` - Dev server with hot reload
  - `frame build` - Multi-platform builds (6 targets)
  - `frame db:*` - Database commands
  - Platform init commands
- **Missing (10%):**
  - OpenAPI spec generation (stub exists)
  - SDK generation (stub exists)
- **Next:** Implement OpenAPI parser and SDK generators

### 4. Frame Server (95%)
- **Status:** Production-ready for HTTP applications
- **Tests:** 9/9 passing
- **Features:**
  - WASM runtime with resource limits
  - HTTP server with graceful shutdown
  - Request routing and WASM invocation
  - Complete middleware stack
  - Static file serving
  - Health/metrics endpoints
- **Missing (5%):**
  - File-based routing
  - SSR integration
  - WebSocket support
  - HTTP/2 support
- **Next:** File-based routing is highest priority

### 5. Frame Data ORM (85%)
- **Status:** Core features production-ready
- **Tests:** 11/11 passing
- **Features:**
  - Connection management via Host Bridge
  - Type-safe query builder
  - Transactions with nested support
  - Model trait with CRUD operations
  - Schema management
- **Missing (15%):**
  - Migration execution engine
  - Seed data loading
  - Many-to-many helpers
  - Eager loading optimization
  - Query plan caching
- **Next:** Migration execution engine

---

## 🔨 Components Needing Work

### 1. Frame UI (30% Complete) - 🔴 HIGH PRIORITY
**Current State:** ComponentPlugin exists, runtime missing

**What's Missing:**
- Server-side rendering engine (CRITICAL)
- Islands manifest generation
- Client hydration loader
- Event handling runtime
- Theme system implementation

**Priority Tasks:**
1. Implement SSR engine to render components server-side
2. Generate islands manifest for selective hydration
3. Build client-side hydration system
4. Add event handling (onClick, onSubmit, etc.)
5. Implement theme system

**Estimated Effort:** 2-3 weeks

### 2. Frame Auth (20% Complete) - 🔴 HIGH PRIORITY
**Current State:** Specification complete, no runtime implementation

**What's Missing:**
- Session management (CRITICAL)
- JWT generation/validation (CRITICAL)
- Password hashing integration
- RBAC enforcement
- CSRF protection
- Auth middleware

**Priority Tasks:**
1. Implement session storage via Host Bridge
2. Add JWT signing/verification using crypto bridge
3. Integrate bcrypt/argon2 password hashing
4. Build RBAC permission checking
5. Add CSRF token generation/validation
6. Create auth middleware for Frame Server

**Estimated Effort:** 1-2 weeks

### 3. Frame CLI - API Generation (10% Complete) - 🟡 MEDIUM PRIORITY
**Current State:** Stubs exist, no actual generation

**What's Missing:**
- OpenAPI 3.1 spec generation from `endpoints:` blocks
- SDK generation for 4 languages (Clean, TypeScript, Swift, Kotlin)

**Priority Tasks:**
1. Parse `endpoints:` blocks to extract routes, params, responses
2. Generate OpenAPI 3.1 JSON schema
3. Implement TypeScript SDK generator (highest demand)
4. Implement Clean SDK generator
5. Implement Swift SDK generator
6. Implement Kotlin SDK generator

**Estimated Effort:** 1 week

---

## 📋 Recommended Implementation Order

Based on dependencies and impact:

### Phase 1: Complete Core Infrastructure (1-2 weeks)
1. **Frame Server File-based Routing** (3 days)
   - Parse filesystem to discover routes
   - Map files to route handlers
   - Integrate with existing router

2. **Frame Data Migration Engine** (2 days)
   - Execute SQL migrations via db bridge
   - Track applied migrations
   - Support up/down migrations

3. **Frame Auth Session/JWT** (5 days)
   - Session storage and retrieval
   - JWT signing and validation
   - Integration with Frame Server middleware

### Phase 2: UI Runtime (2-3 weeks)
4. **Frame UI SSR Engine** (1 week)
   - Parse component templates
   - Generate HTML from components
   - Handle component composition

5. **Frame UI Hydration System** (1 week)
   - Generate islands manifest
   - Client-side hydration loader
   - Progressive enhancement

6. **Frame UI Event Handling** (3 days)
   - Event binding system
   - WASM function callbacks
   - State management

### Phase 3: Advanced Features (1 week)
7. **Frame CLI API Generation** (5 days)
   - OpenAPI spec generation
   - TypeScript SDK generator
   - Other SDK generators

8. **Frame Server Advanced** (2 days)
   - WebSocket support
   - HTTP/2 support
   - SSR integration with Frame UI

---

## 🎯 Quick Wins (Can Be Done Anytime)

These tasks are independent and can be done in parallel:

- **Tests:** Add more integration tests for Frame Server router
- **Docs:** Update user-facing documentation
- **Examples:** Create example applications
- **Performance:** Profile and optimize hot paths
- **Error Messages:** Improve error messages with codes and suggestions

---

## 📊 Success Criteria for v1.0 Release

- ✅ All compiler plugins working
- ✅ All Host Bridge functions working
- ✅ Frame CLI can create and build projects
- ✅ Frame Server can serve HTTP requests
- ✅ Frame Data can query databases
- ⬜ Frame UI can render components (SSR + CSR)
- ⬜ Frame Auth can authenticate users
- ⬜ Example full-stack app demonstrates all features
- ⬜ Documentation covers all major features

**Current v1.0 Progress:** ~80% complete

---

## 🚀 Getting Started

To continue development, start with the highest priority items:

1. **If you want to complete the server stack:**
   - Implement file-based routing in Frame Server
   - Add migration execution to Frame Data

2. **If you want to enable full-stack apps:**
   - Implement Frame UI SSR engine (CRITICAL PATH)
   - Implement Frame Auth session management (CRITICAL PATH)

3. **If you want to polish existing features:**
   - Complete OpenAPI/SDK generation in Frame CLI
   - Add WebSocket support to Frame Server

**Recommended:** Start with Frame UI SSR engine as it's the most critical missing piece for full-stack applications.

---

**Last Updated:** 2025-11-22
**Next Review:** After UI SSR implementation
