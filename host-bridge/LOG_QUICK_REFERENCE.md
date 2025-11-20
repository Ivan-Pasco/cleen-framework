# LOG Bridge - Quick Reference

## Basic Usage

```rust
use host_host::LogBridge;
use serde_json::json;

let bridge = LogBridge::new();

// Simple message
bridge.call("info", json!({
    "message": "Application started"
}));

// With structured data
bridge.call("info", json!({
    "message": "User logged in",
    "data": {
        "userId": 123,
        "action": "login"
    }
}));
```

## Log Levels

```rust
// DEBUG - Detailed debugging information
bridge.call("debug", json!({"message": "Variable value: 42"}));

// INFO - General informational messages
bridge.call("info", json!({"message": "Request completed"}));

// WARN - Warning conditions
bridge.call("warn", json!({"message": "High response time"}));

// ERROR - Error conditions
bridge.call("error", json!({"message": "Database connection failed"}));
```

## Configuration

```rust
use host_host::{LogBridge, LogLevel, LogConfig};

// Production configuration
let config = LogConfig {
    min_level: LogLevel::Info,
    json_output: true,
    include_timestamp: true,
    max_message_size: 10_485_760, // 10MB
};
let bridge = LogBridge::with_config(config);

// Runtime changes
bridge.set_min_level(LogLevel::Warn);
bridge.set_json_output(false);
```

## Direct API

```rust
// Simple messages
bridge.debug("Debug message");
bridge.info("Info message");
bridge.warn("Warning message");
bridge.error("Error message");

// With data
bridge.info_with_data("User action", json!({"userId": 123}));
bridge.error_with_data("Error occurred", json!({"code": "DB_ERROR"}));
```

## Through HostBridge

```rust
use host_host::HostBridge;

#[tokio::main]
async fn main() {
    let mut bridge = HostBridge::new();

    let result = bridge.call("log", "info", json!({
        "message": "Event occurred",
        "data": {"event": "startup"}
    })).await;
}
```

## Error Handling

```rust
let result = bridge.call("info", json!({"message": ""}));

if let Ok(response) = result {
    if response["ok"] == false {
        println!("Error: {}", response["err"]["message"]);
        println!("Code: {}", response["err"]["code"]);
    }
}
```

## Common Patterns

### Request Tracing
```rust
let request_id = "req-123";
bridge.info_with_data("Request received", json!({"requestId": request_id}));
// ... processing ...
bridge.info_with_data("Request completed", json!({
    "requestId": request_id,
    "duration_ms": 145
}));
```

### Error Context
```rust
bridge.error_with_data("Payment failed", json!({
    "orderId": "ORD-123",
    "errorCode": "INSUFFICIENT_FUNDS",
    "amount": 99.99,
    "userId": 456
}));
```

### Performance Monitoring
```rust
bridge.warn_with_data("High latency detected", json!({
    "endpoint": "/api/users",
    "duration_ms": 850,
    "threshold_ms": 500
}));
```

## Log Output Format

### JSON (Default)
```json
{
  "timestamp": "2025-11-19T10:30:00.123Z",
  "level": "INFO",
  "message": "User logged in",
  "data": {"userId": 123}
}
```

### Plain Text
```
[2025-11-19T10:30:00.123Z] INFO - User logged in | data: {"userId":123}
```

## Error Codes

- `LOG_ERROR` - Unknown log function
- `VALIDATION_ERROR` - Invalid parameters, empty message, or message too large

## Best Practices

✅ **DO:**
- Use appropriate log levels
- Include context in data objects
- Use request IDs for tracing
- Configure for environment (dev vs prod)

❌ **DON'T:**
- Log sensitive data (passwords, API keys)
- Log large objects without limits
- Use debug logs in production
- Forget to handle errors

## Testing

```bash
# Run log tests
cargo test log::

# Run integration tests
cargo test --test log_integration_test

# Run example
cargo run --example log_example
```

## Performance Tips

1. **Set appropriate log level** - Filter debug logs in production
2. **Limit message size** - Set reasonable max_message_size
3. **Use structured data** - More efficient than string formatting
4. **Batch similar logs** - Reduce I/O operations

## Common Issues

### Issue: Logs not appearing
**Solution:** Check minimum log level
```rust
bridge.get_min_level(); // Check current level
bridge.set_min_level(LogLevel::Debug); // Set to show all logs
```

### Issue: Message too large error
**Solution:** Increase max size or reduce message
```rust
bridge.set_max_message_size(10_485_760); // 10MB
```

### Issue: Sensitive data in logs
**Solution:** Filter before logging
```rust
// Bad
bridge.info_with_data("Login", json!({"password": "secret"}));

// Good
bridge.info_with_data("Login", json!({"userId": 123}));
```

## Examples Location

- Full documentation: `LOG_MODULE_DOCUMENTATION.md`
- Usage examples: `examples/log_example.rs`
- Integration tests: `tests/log_integration_test.rs`

## Quick Command Reference

```bash
# Build
cargo build

# Test
cargo test log::

# Run example
cargo run --example log_example

# Check quality
cargo clippy
```
