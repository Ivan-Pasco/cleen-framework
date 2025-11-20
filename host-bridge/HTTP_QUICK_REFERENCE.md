# HTTP Bridge Quick Reference

## Usage Examples

### Basic GET Request
```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/users"
  }
}
```

### POST with JSON
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

### Custom Timeout
```json
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/slow",
    "timeout": 60000
  }
}
```

### Disable Redirects
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

### SSR Response
```json
{
  "fn": "host:http.respond",
  "args": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "body": "{\"message\":\"ok\"}"
  }
}
```

## Error Codes

| Code | Meaning | Example |
|------|---------|---------|
| `NETWORK_FAIL` | Connection failed or SSRF blocked | DNS failure, localhost blocked |
| `TIMEOUT` | Request took too long | Exceeded timeout duration |
| `INVALID_URL` | Bad URL format | Missing scheme, unsupported protocol |
| `VALIDATION_ERROR` | Invalid parameters | Timeout > 5 min, invalid method |
| `HTTP_ERROR` | Protocol error | Malformed request |

## Limits

| Parameter | Minimum | Maximum | Default |
|-----------|---------|---------|---------|
| `timeout` | 0ms | 300,000ms (5 min) | 30,000ms (30 sec) |
| `max_redirects` | 0 | Unlimited* | 10 |

\* Configurable per request

## Blocked IPs (SSRF Prevention)

- `127.0.0.0/8` - localhost
- `10.0.0.0/8` - private
- `172.16.0.0/12` - private
- `192.168.0.0/16` - private
- `169.254.0.0/16` - link-local
- IPv6 loopback and link-local

## Supported Methods

- GET
- POST
- PUT
- PATCH
- DELETE
- HEAD
- OPTIONS

## Supported Schemes

- `http://`
- `https://`

## Compression

Automatically supports:
- gzip
- brotli

## Connection Pooling

- Max idle per host: 10
- Idle timeout: 90 seconds
- TCP keep-alive: 60 seconds
