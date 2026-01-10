# Guard Blocks Implementation for Frame Framework

## Overview

This document describes the implementation of authentication guard blocks for the Frame framework. Guard blocks allow declarative authentication and authorization checks in `endpoints:` blocks.

## Components Implemented

### 1. Frame Runtime Auth Functions

Location: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/frame-runtime/src/bridge.rs`

Added the following host functions to the WASM bridge:

#### `_auth_get_session()`
Returns the current session/auth context as JSON.
- Returns session info if authenticated
- Returns "null" if not authenticated

#### `_auth_require_auth()`
Checks if the user is authenticated.
- Returns `1` if authenticated (has auth context or Authorization header)
- Returns `0` if not authenticated
- Checks both `state.auth_context` and `Authorization` header

#### `_auth_require_role(role: string)`
Checks if the user has a specific role.
- Returns `1` if user has the required role or is admin
- Returns `0` if user doesn't have the role
- Admin role has access to all endpoints
- Supports `X-User-Role` header for testing

#### `_auth_can(permission: string)`
Checks if the user has a specific permission.
- Returns `1` if user has the permission or is admin
- Returns `0` if user doesn't have the permission
- Supports `X-User-Permissions` header (comma-separated) for testing

#### `_auth_has_any_role()`
Checks if the user has any role (generic authenticated check).
- Returns `1` if user has any non-empty role
- Returns `0` if no role found

## Usage in Endpoints

### Basic Authentication Guard

```clean
endpoints:
	GET /api/protected:
		guard:
			requireAuth()
		handle:
			return "{\"message\": \"Protected resource\"}"
```

Generated code:
```clean
functions:
	string __route_handler_N()
		integer auth_ok = _auth_require_auth()
		if auth_ok == 0
			return "Unauthorized: 401"
		return "{\"message\": \"Protected resource\"}"
```

### Role-Based Guard

```clean
endpoints:
	GET /api/admin/dashboard:
		guard:
			requireAuth()
			requireRole("admin")
		handle:
			return "{\"message\": \"Admin dashboard\"}"
```

Generated code:
```clean
functions:
	string __route_handler_N()
		integer auth_ok = _auth_require_auth()
		if auth_ok == 0
			return "Unauthorized: 401"
		integer role_ok = _auth_require_role("admin")
		if role_ok == 0
			return "Forbidden: 403"
		return "{\"message\": \"Admin dashboard\"}"
```

### Permission-Based Guard

```clean
endpoints:
	GET /api/posts/publish:
		guard:
			requireAuth()
			can("post.publish")
		handle:
			return "{\"message\": \"Post published\"}"
```

Generated code:
```clean
functions:
	string __route_handler_N()
		integer auth_ok = _auth_require_auth()
		if auth_ok == 0
			return "Unauthorized: 401"
		integer perm_ok = _auth_can("post.publish")
		if perm_ok == 0
			return "Forbidden: 403"
		return "{\"message\": \"Post published\"}"
```

## Testing

### Test Endpoints Created

File: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/auth-guards/main.cln`

The example includes:
1. Public endpoint (no guards)
2. Protected endpoint (requireAuth only)
3. Admin-only endpoint (requireAuth + requireRole("admin"))
4. Editor endpoint (requireAuth + requireRole("editor"))
5. Permission-based endpoint (requireAuth + can("post.publish"))
6. Route with parameters and guards

### Testing with cURL

```bash
# Public endpoint - should work
curl http://localhost:3000/api/public

# Protected endpoint - should fail without auth
curl http://localhost:3000/api/protected

# Protected endpoint - should work with Authorization header
curl -H "Authorization: Bearer fake-token" http://localhost:3000/api/protected

# Admin endpoint - should fail without admin role
curl -H "Authorization: Bearer fake-token" http://localhost:3000/api/admin/dashboard

# Admin endpoint - should work with admin role
curl -H "Authorization: Bearer fake-token" -H "X-User-Role: admin" http://localhost:3000/api/admin/dashboard

# Editor endpoint - should work with editor role
curl -H "Authorization: Bearer fake-token" -H "X-User-Role: editor" http://localhost:3000/api/editor/posts

# Permission endpoint - should work with permission header
curl -H "Authorization: Bearer fake-token" -H "X-User-Permissions: post.publish" http://localhost:3000/api/posts/publish
```

## Integration with Router

The router (`frame-runtime/src/router.rs`) already supports protected routes via the `RouteHandler` struct:

```rust
pub struct RouteHandler {
    pub method: HttpMethod,
    pub path: String,
    pub handler_index: u32,
    pub protected: bool,          // Marks route as protected
    pub required_role: Option<String>,  // Required role (if any)
}
```

Routes are registered using:
- `_http_route(method, path, handler_idx)` - for public routes
- `_http_route_protected(method, path, handler_idx, role)` - for protected routes

## Server-Side Auth Checking

The server (`frame-runtime/src/server.rs`) checks authentication at routing time (lines 206-219):

```rust
// Check authentication for protected routes
if route_handler.protected {
    // Check for Authorization header
    if !headers.contains_key(header::AUTHORIZATION) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // Check role if required
    if let Some(required_role) = &route_handler.required_role {
        debug!("Route requires role: {}", required_role);
        // Role checking happens in the handler via _auth_require_role
    }
}
```

## Future Enhancements

1. **Token Validation**: Implement actual JWT/session token validation instead of just checking for presence of headers

2. **Permission System**: Build a full role-permission mapping system (e.g., admin -> all permissions, editor -> post.create, post.edit, etc.)

3. **Auth Context Population**: Populate `WasmState.auth_context` from validated tokens instead of just checking headers

4. **Session Store**: Implement actual session storage (Redis, database, etc.)

5. **CSRF Protection**: Add CSRF token validation for state-changing operations

6. **Rate Limiting**: Add rate limiting for authentication endpoints

7. **Audit Logging**: Log authentication attempts and failures

## Security Considerations

1. **Current Implementation is for Testing**: The current implementation accepts any Authorization header and relies on test headers (X-User-Role, X-User-Permissions) which is NOT production-ready

2. **Production Requirements**:
   - Validate JWT tokens using proper crypto
   - Verify signatures and expiration
   - Use secure session storage
   - Implement HTTPS-only cookies
   - Add brute-force protection
   - Implement proper CORS policies

3. **Guard Execution**: Guards run inside WASM handler functions, providing security through:
   - Type-safe checks (returns integers, not booleans)
   - Early return on failure (no bypass possible)
   - Consistent error responses

## Files Modified

1. `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/frame-runtime/src/bridge.rs`
   - Added 5 new auth guard functions
   - Lines 1046-1239

2. `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/plugins/frame.web/src/main.cln`
   - Enhanced endpoints expansion to include guard-protected handlers
   - Added handlers 4-7 with various guard combinations

3. `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/examples/auth-guards/main.cln`
   - Created new example demonstrating guard syntax
   - Shows all guard types: requireAuth, requireRole, can

## Build Status

- **frame-runtime**: ✅ Built successfully with new auth functions
- **frame.web plugin**: ⚠️ Needs manual compilation (string literal escaping issues in code generation)
- **Test example**: ✅ Created and documented

## Next Steps

1. Manually test the auth functions with the existing endpoints-test example
2. Implement proper JWT token validation
3. Add session store integration
4. Create comprehensive integration tests
5. Add auth middleware for automatic context population
6. Document auth configuration options
