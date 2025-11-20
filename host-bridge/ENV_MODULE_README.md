# Host Bridge ENV Module

## Overview

The ENV module provides secure access to environment variables through the Host Bridge. It implements all required functions from the Frame Bridge Contracts specification with comprehensive security controls.

## Features

### Core Functions

1. **`host:env.get`** - Get an environment variable value
2. **`host:env.set`** - Set an environment variable (if permitted)
3. **`host:env.has`** - Check if an environment variable exists
4. **`host:env.list`** / **`host:env.all`** - Get all accessible environment variables

### Security Features

- **Allowlist Support**: Restrict access to specific environment variables
- **Denylist Support**: Block access to sensitive environment variables
- **Default Security**: Sensitive variables (AWS keys, private keys, etc.) are blocked by default
- **Permission Control**: Setting variables requires explicit permission
- **Name Validation**: Environment variable names are validated for proper format

## Usage Examples

### Request Format

All requests follow the standard JSON envelope format:

```json
{
  "fn": "host:env.get",
  "args": { "name": "DATABASE_URL" }
}
```

### Response Format

**Success Response:**
```json
{
  "ok": true,
  "data": "postgres://user:pass@localhost/db"
}
```

**Error Response:**
```json
{
  "ok": false,
  "err": {
    "code": "NOT_FOUND",
    "message": "Environment variable 'DATABASE_URL' not found",
    "details": { "name": "DATABASE_URL" }
  }
}
```

## Function Details

### 1. Get Environment Variable

**Request:**
```json
{
  "fn": "host:env.get",
  "args": { "name": "DATABASE_URL" }
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": "postgres://localhost/mydb"
}
```

**Error Codes:**
- `NOT_FOUND` - Variable does not exist
- `PERMISSION_DENIED` - Access to this variable is not permitted
- `VALIDATION_ERROR` - Invalid variable name format

### 2. Set Environment Variable

**Request:**
```json
{
  "fn": "host:env.set",
  "args": {
    "name": "API_KEY",
    "value": "secret_key_123"
  }
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": {
    "name": "API_KEY",
    "value": "secret_key_123"
  }
}
```

**Error Codes:**
- `PERMISSION_DENIED` - Setting environment variables is not permitted, or access to this specific variable is blocked
- `VALIDATION_ERROR` - Invalid variable name format

**Note:** By default, setting environment variables is disabled for security. Use `EnvBridge::new_unrestricted()` or configure permissions explicitly.

### 3. Check if Variable Exists

**Request:**
```json
{
  "fn": "host:env.has",
  "args": { "name": "PORT" }
}
```

**Success Response (exists):**
```json
{
  "ok": true,
  "data": true
}
```

**Success Response (does not exist):**
```json
{
  "ok": true,
  "data": false
}
```

**Error Codes:**
- `PERMISSION_DENIED` - Access to this variable is not permitted
- `VALIDATION_ERROR` - Invalid variable name format

### 4. List All Variables

**Request:**
```json
{
  "fn": "host:env.list",
  "args": {}
}
```

**Alias:** `host:env.all` (same functionality)

**Success Response:**
```json
{
  "ok": true,
  "data": {
    "PATH": "/usr/bin:/bin",
    "HOME": "/Users/username",
    "DATABASE_URL": "postgres://localhost/db"
  }
}
```

**Note:** Only variables that pass security checks (allowlist/denylist) are included.

## Security Configuration

### Default Security Mode

```rust
// Creates bridge with default security settings:
// - Setting variables is disabled
// - Sensitive variables are blocked by default
let bridge = EnvBridge::new();
```

**Default Denylist:**
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `PRIVATE_KEY`
- `ENCRYPTION_KEY`
- `MASTER_KEY`
- `SSH_AUTH_SOCK`
- `GPG_AGENT_INFO`

### Unrestricted Mode (Development)

```rust
// All access permitted (use only in development)
let bridge = EnvBridge::new_unrestricted();
```

### Allowlist Mode

```rust
// Only specified variables are accessible
let bridge = EnvBridge::with_allowlist(vec![
    "DATABASE_URL".to_string(),
    "API_KEY".to_string(),
    "PORT".to_string(),
]);
```

### Runtime Configuration

```rust
let bridge = EnvBridge::new();

// Set allowlist
bridge.set_allowlist(vec!["VAR1".to_string(), "VAR2".to_string()]);

// Clear allowlist (allow all except denylisted)
bridge.clear_allowlist();

// Add to denylist
bridge.add_to_denylist("SENSITIVE_VAR".to_string());

// Remove from denylist
bridge.remove_from_denylist("SENSITIVE_VAR");

// Clear denylist
bridge.clear_denylist();
```

## Validation Rules

### Valid Variable Names

Environment variable names must:
- Not be empty
- Not start with a digit
- Only contain alphanumeric characters (a-z, A-Z, 0-9) and underscores (_)

**Valid Examples:**
- `DATABASE_URL`
- `API_KEY_123`
- `_INTERNAL_VAR`
- `PATH`

**Invalid Examples:**
- `` (empty string)
- `123_VAR` (starts with digit)
- `MY-VAR` (contains hyphen)
- `MY VAR` (contains space)
- `MY.VAR` (contains dot)

## Error Codes

| Code | Meaning | When It Occurs |
|------|---------|----------------|
| `ENV_ERROR` | General environment error | Unknown function name |
| `NOT_FOUND` | Variable not found | Variable does not exist in environment |
| `PERMISSION_DENIED` | Access denied | Variable is blocked by allowlist/denylist, or set operation not permitted |
| `VALIDATION_ERROR` | Invalid input | Variable name has invalid format, or missing required fields |

## Direct API Usage (Internal)

For Rust code, the bridge provides direct methods:

```rust
let bridge = EnvBridge::new_unrestricted();

// Get variable
if let Some(value) = bridge.get("DATABASE_URL") {
    println!("Database URL: {}", value);
}

// Set variable
bridge.set("NEW_VAR", "value")?;

// Check existence
if bridge.has("PORT") {
    println!("PORT is set");
}

// Get all variables
let all_vars = bridge.all();
for (key, value) in all_vars {
    println!("{}: {}", key, value);
}
```

## Integration with Frame Framework

In Clean Language, environment variables are accessed through the Host Bridge:

```clean
// Example endpoint using environment variables
endpoints:
	get /api/config:
		returns:
			json({
				"database": env.get("DATABASE_URL"),
				"port": env.get("PORT")
			})
```

The Frame compiler translates these calls to the appropriate Host Bridge JSON envelopes.

## Testing

The module includes comprehensive tests:

### Unit Tests
```bash
cargo test --lib env
```

### Integration Tests
```bash
cargo test --test env_integration_test
```

### All Tests
```bash
cargo test
```

## Thread Safety

The EnvBridge uses `RwLock` for thread-safe access to allowlist and denylist configurations. Multiple threads can safely:
- Read from the environment
- Update security configurations
- Call bridge functions concurrently

## Performance Considerations

- **Get/Set Operations**: O(1) - Direct system calls
- **List Operations**: O(n) - Iterates all environment variables with permission checks
- **Permission Checks**: O(1) average - HashSet lookups for allowlist/denylist
- **Name Validation**: O(n) - Where n is the length of the variable name

## Best Practices

1. **Use Allowlist in Production**: Explicitly define which variables can be accessed
2. **Disable Set in Production**: Unless absolutely necessary, keep `allow_set = false`
3. **Validate Early**: Let the bridge handle validation; don't duplicate it in application code
4. **Log Access Attempts**: Consider logging when sensitive variables are accessed
5. **Review Default Denylist**: Add application-specific sensitive variables to the denylist
6. **Use Direct API Internally**: For Rust code, prefer direct methods over JSON envelope calls

## Examples

### Complete Request/Response Flow

**Scenario:** Get database connection string

1. **Clean Language Code:**
```clean
string dbUrl = env.get("DATABASE_URL")
```

2. **Compiled to Host Bridge Call:**
```json
{
  "fn": "host:env.get",
  "args": { "name": "DATABASE_URL" }
}
```

3. **Host Bridge Processes:**
```rust
// 1. Validates variable name
// 2. Checks permissions (allowlist/denylist)
// 3. Reads from environment
// 4. Returns envelope response
```

4. **Success Response:**
```json
{
  "ok": true,
  "data": "postgres://user:pass@localhost:5432/mydb"
}
```

5. **Value Used in Clean Code:**
```clean
// dbUrl now contains "postgres://user:pass@localhost:5432/mydb"
Database.connect(dbUrl)
```

### Error Handling Flow

**Scenario:** Variable doesn't exist

1. **Request:**
```json
{
  "fn": "host:env.get",
  "args": { "name": "NONEXISTENT_VAR" }
}
```

2. **Error Response:**
```json
{
  "ok": false,
  "err": {
    "code": "NOT_FOUND",
    "message": "Environment variable 'NONEXISTENT_VAR' not found",
    "details": { "name": "NONEXISTENT_VAR" }
  }
}
```

3. **Clean Language Error Handling:**
```clean
try:
	string value = env.get("NONEXISTENT_VAR")
onError err:
	log.error("Failed to get env var: " + err.message)
```

## Implementation Status

✅ **Completed Features:**
- All four required functions (get, set, has, list/all)
- Standard JSON envelope format
- Complete error handling with proper error codes
- Security controls (allowlist, denylist)
- Name validation
- Thread-safe implementation
- Comprehensive unit tests
- Integration tests
- Direct API methods
- Documentation

✅ **Production Ready:** This module is fully implemented with no placeholders or TODO items. All functionality is operational and tested.

## Version

- **Module Version:** 1.0.0
- **Frame Version:** 0.1.0
- **Host Bridge Version:** 0.1.0

## License

This module is part of the Frame Framework for Clean Language.
