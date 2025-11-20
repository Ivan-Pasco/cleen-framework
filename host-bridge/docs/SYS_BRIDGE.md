# SYS Bridge Module Documentation

## Overview

The SYS Bridge module provides system-level information and operations for the Frame Framework. It enables WASM applications to query platform details, architecture information, framework version, and manage process lifecycle.

## Functions

### `host:sys.platform()`

Returns the operating system platform.

**Request:**
```json
{
  "fn": "host:sys.platform",
  "args": {}
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": "macos"
}
```

**Supported Platforms:**
- `"linux"` - Linux operating systems
- `"macos"` - macOS / Mac OS X
- `"windows"` - Microsoft Windows
- `"android"` - Android mobile OS
- `"ios"` - Apple iOS
- `"web"` - WebAssembly in browser

**Use Cases:**
- Platform-specific feature detection
- Conditional code execution based on OS
- Platform compatibility checks
- Cross-platform application logic

---

### `host:sys.arch()`

Returns the CPU architecture.

**Request:**
```json
{
  "fn": "host:sys.arch",
  "args": {}
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": "aarch64"
}
```

**Supported Architectures:**
- `"x86_64"` - 64-bit x86 (Intel/AMD)
- `"aarch64"` - 64-bit ARM (Apple Silicon, ARM servers)
- `"arm"` - 32-bit ARM
- `"wasm32"` - WebAssembly 32-bit

**Use Cases:**
- Architecture-specific optimizations
- Binary compatibility checks
- Feature availability detection
- Performance tuning

---

### `host:sys.version()`

Returns the Frame Framework version.

**Request:**
```json
{
  "fn": "host:sys.version",
  "args": {}
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": "1.0.0"
}
```

**Version Format:**
- Follows semantic versioning (semver): `MAJOR.MINOR.PATCH`
- Example: `"1.2.3"` means version 1.2.3

**Use Cases:**
- Version compatibility checks
- Feature availability detection (based on version)
- Debugging and logging
- Migration planning

---

### `host:sys.exit(code: integer)`

Exits the current process with the specified exit code.

**Request:**
```json
{
  "fn": "host:sys.exit",
  "args": {
    "code": 0
  }
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": null
}
```

**Note:** The process terminates immediately after this call. The success response may not be received by the caller.

**Exit Code Requirements:**
- Must be an integer in the range 0-255
- `0` typically indicates success
- Non-zero values indicate errors or specific exit conditions

**Error Response (Invalid Code):**
```json
{
  "ok": false,
  "err": {
    "code": "VALIDATION_ERROR",
    "message": "Exit code must be in range 0-255",
    "details": {
      "code": 256,
      "min": 0,
      "max": 255
    }
  }
}
```

**Use Cases:**
- Graceful application shutdown
- Error handling and termination
- CLI tool exit codes
- Process lifecycle management

**Important:** This function terminates the entire process, including all running tasks and threads. Use with caution.

---

### `host:sys.env_info()`

Returns comprehensive environment information including platform, architecture, version, uptime, and process IDs.

**Request:**
```json
{
  "fn": "host:sys.env_info",
  "args": {}
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": {
    "platform": "macos",
    "arch": "aarch64",
    "version": "1.0.0",
    "uptime_seconds": 12345,
    "process_id": 98765,
    "parent_process_id": 1234
  }
}
```

**Response Fields:**
- `platform` (string): Operating system (see platform values above)
- `arch` (string): CPU architecture (see arch values above)
- `version` (string): Frame Framework version (semver format)
- `uptime_seconds` (integer): Process uptime in seconds since startup
- `process_id` (integer): Current process ID (PID)
- `parent_process_id` (integer): Parent process ID (PPID)
  - On Unix systems: actual parent process ID
  - On Windows: 0 (simplified implementation)
  - On other platforms: 0

**Use Cases:**
- System diagnostics and monitoring
- Debug information collection
- Process management and tracking
- Performance profiling
- System health checks

---

## Error Handling

All SYS bridge functions return standard JSON envelopes with error information when operations fail.

**Standard Error Response:**
```json
{
  "ok": false,
  "err": {
    "code": "ERROR_CODE",
    "message": "Human-readable error description",
    "details": {}
  }
}
```

**Error Codes:**

- `SYS_ERROR` - Unknown function or general system error
- `VALIDATION_ERROR` - Invalid parameters (e.g., exit code out of range)

**Error Examples:**

1. **Unknown Function:**
```json
{
  "ok": false,
  "err": {
    "code": "SYS_ERROR",
    "message": "Unknown sys function: invalid_function",
    "details": {}
  }
}
```

2. **Invalid Exit Code:**
```json
{
  "ok": false,
  "err": {
    "code": "VALIDATION_ERROR",
    "message": "Exit code must be in range 0-255",
    "details": {
      "code": 300,
      "min": 0,
      "max": 255
    }
  }
}
```

---

## Usage Examples

### Clean Language

```clean
// Get platform information
let platform_result = bridge.call("sys", "platform", {})
if platform_result.ok:
	print("Running on: " + platform_result.data)

// Get system architecture
let arch_result = bridge.call("sys", "arch", {})
if arch_result.ok:
	print("Architecture: " + arch_result.data)

// Get framework version
let version_result = bridge.call("sys", "version", {})
if version_result.ok:
	print("Frame version: " + version_result.data)

// Get comprehensive environment info
let env_info = bridge.call("sys", "env_info", {})
if env_info.ok:
	let data = env_info.data
	print("Platform: " + data.platform)
	print("Architecture: " + data.arch)
	print("Version: " + data.version)
	print("Uptime: " + data.uptime_seconds + " seconds")
	print("Process ID: " + data.process_id)
	print("Parent PID: " + data.parent_process_id)

// Exit with success code
bridge.call("sys", "exit", {"code": 0})
```

### Rust (Direct API)

```rust
use host_host::SysBridge;

#[tokio::main]
async fn main() {
    let bridge = SysBridge::new();

    // Get platform
    let platform = bridge.get_platform();
    println!("Platform: {}", platform);

    // Get architecture
    let arch = bridge.get_arch();
    println!("Architecture: {}", arch);

    // Get version
    let version = bridge.get_version();
    println!("Version: {}", version);

    // Get process ID
    let pid = bridge.get_process_id();
    println!("Process ID: {}", pid);

    // Call through bridge interface
    let result = bridge.call("env_info", serde_json::json!({})).await.unwrap();
    println!("Environment info: {}", result);
}
```

---

## Platform-Specific Behavior

### Unix Systems (Linux, macOS)
- Full support for all functions
- Parent process ID is retrieved using `getppid()` system call
- Process uptime is tracked from module initialization

### Windows
- Full support for platform, arch, version, exit functions
- Parent process ID returns `0` (simplified implementation)
- Process uptime is tracked from module initialization

### WebAssembly (Browser)
- Platform returns `"web"`
- Architecture returns `"wasm32"`
- Parent process ID returns `0`
- Exit function may have limited behavior depending on WASM runtime
- Process uptime is tracked from module initialization

---

## Thread Safety

All SYS bridge functions are thread-safe and can be called concurrently from multiple threads:

- `platform()`, `arch()`, `version()` - Read-only operations, fully thread-safe
- `exit()` - Thread-safe but terminates entire process
- `env_info()` - Thread-safe with static initialization for uptime tracking

---

## Performance Considerations

- **platform()**, **arch()**, **version()** - Extremely fast (compile-time constants)
- **env_info()** - Fast (O(1) operations)
- **exit()** - Immediate process termination
- **Process uptime** - Calculated on-demand using cached start time

All functions have minimal overhead and are suitable for frequent calls.

---

## Security Considerations

1. **Exit Function:**
   - Can terminate the entire process
   - Should be restricted to trusted code paths
   - May bypass cleanup handlers

2. **Process Information:**
   - Process IDs and parent process IDs are exposed
   - Could be used for process tracking or identification
   - Consider security implications in multi-tenant environments

3. **Platform Detection:**
   - Platform and architecture information is publicly available
   - No sensitive data is exposed
   - Safe for use in all contexts

---

## Testing

The SYS bridge module includes comprehensive test coverage:

### Unit Tests (17 tests)
- Platform detection
- Architecture detection
- Version information
- Exit code validation
- Environment info structure
- JSON envelope compliance
- Consistency checks
- Direct API methods

### Integration Tests (12 tests)
- Full Host Bridge integration
- Multi-function workflows
- Error handling scenarios
- Concurrent operations
- Uptime tracking

Run tests:
```bash
cargo test sys
cargo test --test sys_integration_test
```

---

## Comparison with Other Bridges

| Feature | SYS | ENV | TIME | LOG |
|---------|-----|-----|------|-----|
| Read system info | ✅ | ❌ | ❌ | ❌ |
| Read configuration | ❌ | ✅ | ❌ | ❌ |
| Time operations | ❌ | ❌ | ✅ | ❌ |
| Process control | ✅ | ❌ | ❌ | ❌ |
| Stateless | ✅ | ✅ | ✅ | ❌ (logs stored) |
| Synchronous | ✅ | ✅ | ❌ (sleep) | ✅ |

---

## API Stability

The SYS bridge API follows semantic versioning:

- **Stable APIs (1.0+):**
  - `platform()`
  - `arch()`
  - `version()`
  - `exit(code)`
  - `env_info()`

Breaking changes will only occur in major version updates with advance notice and migration paths.

---

## Future Enhancements

Potential future additions (not currently implemented):

1. **System Resources:**
   - `sys.memory()` - Memory usage and limits
   - `sys.cpu_count()` - Number of CPU cores
   - `sys.load_average()` - System load metrics

2. **Environment Variables:**
   - `sys.env_vars()` - List environment variables
   - (Consider security implications)

3. **Process Information:**
   - `sys.process_name()` - Current process name
   - `sys.working_directory()` - Current working directory

4. **Signal Handling:**
   - `sys.signal(signal)` - Send signals to process
   - (Unix-specific)

---

## Related Documentation

- [Frame Bridge Contracts](../documents/specification/frame_bridge_contracts.md)
- [ENV Bridge Documentation](./ENV_BRIDGE.md)
- [TIME Bridge Documentation](./TIME_BRIDGE.md)
- [LOG Bridge Documentation](./LOG_BRIDGE.md)
- [Frame Architecture](../documents/ARCHITECTURE.md)

---

## Support

For issues, questions, or feature requests related to the SYS host:

1. Check the Frame Framework documentation
2. Review the test cases for usage examples
3. Open an issue on GitHub
4. Consult the Frame community discussions

---

**Last Updated:** 2025-11-19
**Framework Version:** 1.0.0
**Module Status:** Stable ✅
