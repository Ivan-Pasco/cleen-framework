/// Example demonstrating the LOG bridge functionality
///
/// This example shows:
/// - All log levels (debug, info, warn, error)
/// - Structured logging with data objects
/// - Log level filtering
/// - Configuration options
/// - Error handling

use host_bridge::{LogBridge, LogLevel, LogConfig};
use serde_json::json;

fn main() {
	println!("=== LOG Bridge Example ===\n");

	// Example 1: Basic logging with default configuration
	example_basic_logging();

	println!("\n---\n");

	// Example 2: Structured logging with data
	example_structured_logging();

	println!("\n---\n");

	// Example 3: Log level filtering
	example_log_level_filtering();

	println!("\n---\n");

	// Example 4: Custom configuration
	example_custom_configuration();

	println!("\n---\n");

	// Example 5: Error handling
	example_error_handling();

	println!("\n---\n");

	// Example 6: Direct API usage
	example_direct_api();

	println!("\n---\n");

	// Example 7: Production use case
	example_production_use_case();
}

fn example_basic_logging() {
	println!("Example 1: Basic Logging");
	println!("Output format: JSON (default)\n");

	let bridge = LogBridge::new();

	// Debug level
	let result = bridge.call("debug", json!({
		"message": "Debug message for development"
	}));
	println!("Debug result: {:?}", result.unwrap());

	// Info level
	let result = bridge.call("info", json!({
		"message": "Application started successfully"
	}));
	println!("Info result: {:?}", result.unwrap());

	// Warning level
	let result = bridge.call("warn", json!({
		"message": "Disk space running low"
	}));
	println!("Warn result: {:?}", result.unwrap());

	// Error level
	let result = bridge.call("error", json!({
		"message": "Failed to connect to database"
	}));
	println!("Error result: {:?}", result.unwrap());
}

fn example_structured_logging() {
	println!("Example 2: Structured Logging with Data");
	println!("Logging with additional context data\n");

	let bridge = LogBridge::new();

	// Log user action with context
	let result = bridge.call("info", json!({
		"message": "User logged in",
		"data": {
			"userId": 12345,
			"username": "john_doe",
			"ip": "192.168.1.100",
			"userAgent": "Mozilla/5.0",
			"timestamp": 1700000000
		}
	}));
	println!("Structured log result: {:?}", result.unwrap());

	// Log API request with details
	let result = bridge.call("info", json!({
		"message": "API request completed",
		"data": {
			"method": "POST",
			"path": "/api/v1/users",
			"statusCode": 201,
			"duration_ms": 45,
			"requestId": "req-abc-123"
		}
	}));
	println!("API log result: {:?}", result.unwrap());

	// Log error with context
	let result = bridge.call("error", json!({
		"message": "Payment processing failed",
		"data": {
			"orderId": "ORD-789",
			"amount": 99.99,
			"currency": "USD",
			"errorCode": "INSUFFICIENT_FUNDS",
			"userId": 12345
		}
	}));
	println!("Error with context: {:?}", result.unwrap());
}

fn example_log_level_filtering() {
	println!("Example 3: Log Level Filtering");
	println!("Only logs at or above the minimum level are output\n");

	let bridge = LogBridge::new();

	// Set minimum level to WARN
	bridge.set_min_level(LogLevel::Warn);
	println!("Set minimum level to: {:?}", bridge.get_min_level());

	// These won't be output (but still return success)
	let result = bridge.call("debug", json!({
		"message": "This debug message is filtered out"
	}));
	println!("Debug (filtered): {:?}", result.unwrap());

	let result = bridge.call("info", json!({
		"message": "This info message is filtered out"
	}));
	println!("Info (filtered): {:?}", result.unwrap());

	// These will be output
	let result = bridge.call("warn", json!({
		"message": "This warning is shown"
	}));
	println!("Warn (shown): {:?}", result.unwrap());

	let result = bridge.call("error", json!({
		"message": "This error is shown"
	}));
	println!("Error (shown): {:?}", result.unwrap());
}

fn example_custom_configuration() {
	println!("Example 4: Custom Configuration");
	println!("Using custom log configuration\n");

	let config = LogConfig {
		min_level: LogLevel::Info,
		json_output: false,  // Plain text output
		include_timestamp: true,
		max_message_size: 1024,
	};

	let bridge = LogBridge::with_config(config);

	let result = bridge.call("info", json!({
		"message": "Custom configured log message",
		"data": {
			"environment": "production",
			"version": "1.0.0"
		}
	}));
	println!("Custom config result: {:?}", result.unwrap());

	// Change configuration at runtime
	bridge.set_json_output(true);
	bridge.set_include_timestamp(true);
	bridge.set_max_message_size(2048);

	let result = bridge.call("info", json!({
		"message": "Log with updated configuration"
	}));
	println!("Updated config result: {:?}", result.unwrap());
}

fn example_error_handling() {
	println!("Example 5: Error Handling");
	println!("Demonstrating validation errors\n");

	let bridge = LogBridge::new();

	// Empty message error
	let result = bridge.call("info", json!({
		"message": ""
	}));
	match result {
		Ok(response) => {
			if response["ok"] == false {
				println!("Empty message error:");
				println!("  Code: {}", response["err"]["code"]);
				println!("  Message: {}", response["err"]["message"]);
			}
		}
		Err(e) => println!("Error: {}", e),
	}

	// Invalid params error
	let result = bridge.call("info", json!(123));
	match result {
		Ok(response) => {
			if response["ok"] == false {
				println!("\nInvalid params error:");
				println!("  Code: {}", response["err"]["code"]);
				println!("  Message: {}", response["err"]["message"]);
			}
		}
		Err(e) => println!("Error: {}", e),
	}

	// Message too long error
	bridge.set_max_message_size(50);
	let long_message = "a".repeat(100);
	let result = bridge.call("info", json!({
		"message": long_message
	}));
	match result {
		Ok(response) => {
			if response["ok"] == false {
				println!("\nMessage too long error:");
				println!("  Code: {}", response["err"]["code"]);
				println!("  Message: {}", response["err"]["message"]);
			}
		}
		Err(e) => println!("Error: {}", e),
	}

	// Unknown function error
	let result = bridge.call("unknown", json!({
		"message": "test"
	}));
	match result {
		Ok(response) => {
			if response["ok"] == false {
				println!("\nUnknown function error:");
				println!("  Code: {}", response["err"]["code"]);
				println!("  Message: {}", response["err"]["message"]);
			}
		}
		Err(e) => println!("Error: {}", e),
	}
}

fn example_direct_api() {
	println!("Example 6: Direct API Usage");
	println!("Using direct methods instead of bridge calls\n");

	let bridge = LogBridge::new();

	// Simple messages
	bridge.debug("This is a debug message");
	bridge.info("This is an info message");
	bridge.warn("This is a warning message");
	bridge.error("This is an error message");

	// Messages with data
	bridge.info_with_data(
		"User action",
		json!({
			"action": "login",
			"userId": 123
		})
	);

	bridge.error_with_data(
		"Database error",
		json!({
			"error": "Connection timeout",
			"database": "postgres",
			"host": "db.example.com"
		})
	);

	println!("Direct API calls completed (check stderr for output)");
}

fn example_production_use_case() {
	println!("Example 7: Production Use Case");
	println!("Real-world application logging scenario\n");

	// Configure for production
	let config = LogConfig {
		min_level: LogLevel::Info,  // Don't log debug in production
		json_output: true,          // Structured logs for parsing
		include_timestamp: true,
		max_message_size: 10_485_760,  // 10MB max
	};

	let bridge = LogBridge::with_config(config);

	// Application startup
	bridge.info_with_data(
		"Application starting",
		json!({
			"version": "1.2.3",
			"environment": "production",
			"port": 8080
		})
	);

	// Incoming request
	let request_id = "req-xyz-456";
	let result = bridge.call("info", json!({
		"message": "Incoming request",
		"data": {
			"requestId": request_id,
			"method": "POST",
			"path": "/api/v1/orders",
			"contentType": "application/json",
			"contentLength": 1234
		}
	}));
	println!("Request log: {:?}", result.unwrap());

	// Database query
	let result = bridge.call("debug", json!({
		"message": "Database query executed",
		"data": {
			"requestId": request_id,
			"query": "SELECT * FROM orders WHERE user_id = $1",
			"duration_ms": 12
		}
	}));
	// Debug is filtered out in production
	println!("DB query log (filtered): {:?}", result.unwrap());

	// Warning condition
	let result = bridge.call("warn", json!({
		"message": "High response time detected",
		"data": {
			"requestId": request_id,
			"duration_ms": 950,
			"threshold_ms": 500
		}
	}));
	println!("Warning log: {:?}", result.unwrap());

	// Successful response
	let result = bridge.call("info", json!({
		"message": "Request completed",
		"data": {
			"requestId": request_id,
			"statusCode": 200,
			"duration_ms": 145
		}
	}));
	println!("Success log: {:?}", result.unwrap());

	// Error scenario
	let result = bridge.call("error", json!({
		"message": "Payment processing failed",
		"data": {
			"requestId": request_id,
			"errorCode": "PAYMENT_DECLINED",
			"errorMessage": "Card declined by issuer",
			"orderId": "ORD-12345",
			"userId": 789
		}
	}));
	println!("Error log: {:?}", result.unwrap());

	println!("\nProduction logging example complete");
}
