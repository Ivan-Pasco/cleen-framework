# HTTP Bridge Implementation

## Overview

The HTTP Bridge module provides outbound HTTP request capabilities for the Frame Framework. It implements the `host:http` namespace as specified in the Frame Bridge Contracts.

## Implementation Details

### Location
- **File**: `/Users/earcandy/Documents/Dev/Clean Language/clean-framework/host-bridge/src/http.rs`
- **Module**: `host_host::http`

### Key Features

#### 1. HTTP Methods Support
- ✅ GET
- ✅ POST
- ✅ PUT
- ✅ PATCH
- ✅ DELETE
- ✅ HEAD
- ✅ OPTIONS

#### 2. Request Features
- ✅ Custom headers (full support)
- ✅ Request body handling (JSON, text, binary)
- ✅ Configurable timeout (default: 30s, max: 5 minutes)
- ✅ Redirect following (configurable, max 10 redirects by default)
- ✅ Automatic compression support (gzip, brotli)
- ✅ Connection pooling and reuse
- ✅ TCP keep-alive

#### 3. Security Features
- ✅ TLS/SSL with certificate validation (rustls)
- ✅ SSRF prevention (blocks private IP addresses)
- ✅ URL scheme validation (only http/https allowed)
- ✅ Timeout enforcement
- ✅ Redirect limit enforcement

#### 4. Response Handling
- ✅ Status code extraction
- ✅ Header extraction
- ✅ Body extraction (text/binary)
- ✅ Final URL tracking (after redirects)
- ✅ Automatic decompression

#### 5. Error Handling
- ✅ Network failures: `NETWORK_FAIL`
- ✅ Timeouts: `TIMEOUT`
- ✅ Invalid URLs: `INVALID_URL`
- ✅ Validation errors: `VALIDATION_ERROR`
- ✅ HTTP errors: `HTTP_ERROR`

## API Reference

### host:http.request

Execute an HTTP request.

**Request Envelope:**
```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS",
    "url": "https://api.example.com/data",
    "headers": {
      "Accept": "application/json",
      "Authorization": "Bearer token"
    },
    "body": "{\"key\":\"value\"}",
    "timeout": 30000,
    "follow_redirects": true,
    "max_redirects": 10
  }
}
```

**Success Response:**
```json
{
  "ok": true,
  "data": {
    "status": 200,
    "headers": {
      "content-type": "application/json"
    },
    "body": "{\"result\":\"ok\"}",
    "url": "https://api.example.com/data"
  }
}
```

**Error Response:**
```json
{
  "ok": false,
  "err": {
    "code": "NETWORK_FAIL",
    "message": "Connection failed: ...",
    "details": {
      "url": "https://api.example.com/data"
    }
  }
}
```

### host:http.respond

Handle HTTP response for SSR mode.

**Request Envelope:**
```json
{
  "fn": "host:http.respond",
  "args": {
    "status": 302,
    "headers": {
      "Location": "/login"
    },
    "body": ""
  }
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "status": 302,
    "headers": {
      "Location": "/login"
    },
    "body": ""
  }
}
```

## Configuration

### Client Configuration

The HTTP client is configured with these defaults:

- **Timeout**: 30 seconds (configurable per request)
- **Connect Timeout**: 10 seconds
- **Pool Max Idle per Host**: 10 connections
- **Pool Idle Timeout**: 90 seconds
- **TCP Keep-Alive**: 60 seconds
- **TLS**: rustls (no OpenSSL dependency)
- **Compression**: gzip and brotli enabled
- **Redirects**: Manual handling with configurable limits

### Security Settings

#### SSRF Prevention

The following private IP ranges are blocked:
- `127.0.0.0/8` (localhost)
- `10.0.0.0/8` (private)
- `172.16.0.0/12` (private)
- `192.168.0.0/16` (private)
- `169.254.0.0/16` (link-local)
- IPv6 loopback and link-local addresses

#### Timeout Limits

- **Minimum**: 0ms (no minimum enforced)
- **Maximum**: 300,000ms (5 minutes)
- **Default**: 30,000ms (30 seconds)

#### Redirect Limits

- **Default Max**: 10 redirects
- **Configurable**: Can be set per request

## Error Codes

| Code | Description |
|------|-------------|
| `NETWORK_FAIL` | Connection failure, DNS resolution failure, or private IP blocked |
| `TIMEOUT` | Request exceeded timeout duration |
| `INVALID_URL` | Malformed URL or unsupported scheme |
| `VALIDATION_ERROR` | Invalid request parameters |
| `HTTP_ERROR` | HTTP protocol error or request building failure |

## Examples

### Simple GET Request

```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/users"
  }
}
```

### POST with JSON Body

```json
{
  "fn": "host:http.request",
  "args": {
    "method": "POST",
    "url": "https://api.example.com/users",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": "{\"name\":\"Alice\",\"email\":\"alice@example.com\"}"
  }
}
```

### Request with Custom Timeout

```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/slow-endpoint",
    "timeout": 60000
  }
}
```

### Request Without Following Redirects

```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://example.com/redirect",
    "follow_redirects": false
  }
}
```

## Testing

### Unit Tests

All unit tests are located in `src/http.rs` under the `#[cfg(test)]` module.

**Run unit tests:**
```bash
cargo test http::tests --lib
```

**Note**: Some tests require network access to httpbin.org. If the service is unavailable, those tests will fail with 503 errors, but this doesn't indicate a code issue.

### Integration Tests

Integration tests that don't require external services are located in `tests/http_integration_test.rs`.

**Run integration tests:**
```bash
cargo test --test http_integration_test
```

### Test Coverage

The implementation includes tests for:
- ✅ All HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- ✅ Custom headers
- ✅ Request bodies
- ✅ Timeout handling
- ✅ Redirect following
- ✅ Compression handling
- ✅ URL validation
- ✅ Method validation
- ✅ SSRF prevention
- ✅ Error handling
- ✅ Response parsing
- ✅ SSR response handling

## Dependencies

```toml
reqwest = { version = "0.11", features = ["json", "gzip", "brotli", "rustls-tls", "stream"] }
url = "2.5"
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
```

## Performance Characteristics

### Connection Pooling

The HTTP client maintains a connection pool with:
- Up to 10 idle connections per host
- 90-second idle timeout
- Automatic connection reuse

### Memory Usage

- Minimal memory footprint
- Streaming response handling
- Efficient buffer management

### Throughput

- Connection pooling enables high throughput
- Keep-alive reduces connection overhead
- Compression reduces bandwidth usage

## Integration with Frame Server

The HTTP bridge integrates seamlessly with the Frame Server through the HostBridge:

```rust
let mut bridge = HostBridge::new();

// Execute HTTP request via bridge
let params = json!({
    "method": "GET",
    "url": "https://api.example.com/data"
});

let result = bridge.call("http", "request", params).await?;
```

## Compliance

This implementation fully complies with:
- ✅ Frame Bridge Contracts specification
- ✅ Frame Server specification
- ✅ Clean Language security requirements
- ✅ Rust best practices and idioms

## Future Enhancements

Potential future improvements:
- HTTP/2 support
- Request/response interceptors
- Retry logic with exponential backoff
- Circuit breaker pattern
- Request caching
- Binary body handling optimization
- WebSocket support (if needed)

## Changelog

### Version 1.0.0 (2025-11-19)
- Initial implementation
- Full HTTP method support
- SSRF prevention
- Compression support
- Redirect handling
- Comprehensive error handling
- Integration tests
- Production-ready code (no placeholders or TODOs)
