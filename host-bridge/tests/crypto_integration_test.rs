use host_bridge::HostBridge;
use serde_json::json;

#[tokio::test]
async fn test_crypto_bridge_integration() {
	let mut bridge = HostBridge::new();

	// Test random generation
	let result = bridge
		.call("crypto", "random", json!({"bytes": 32}))
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert!(result["data"].is_string());

	// Test password hashing with bcrypt
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "test123",
				"algorithm": "bcrypt",
				"cost": 10
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	let bcrypt_hash = result["data"].as_str().unwrap();
	assert!(bcrypt_hash.starts_with("$2"));

	// Test password verification
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": "test123",
				"hash": bcrypt_hash
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert_eq!(result["data"], true);

	// Test wrong password
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": "wrong",
				"hash": bcrypt_hash
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert_eq!(result["data"], false);

	// Test argon2 hashing
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "test456",
				"algorithm": "argon2"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	let argon2_hash = result["data"].as_str().unwrap();
	assert!(argon2_hash.starts_with("$argon2"));

	// Test argon2 verification
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": "test456",
				"hash": argon2_hash
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert_eq!(result["data"], true);

	// Test JWT signing
	let result = bridge
		.call(
			"crypto",
			"sign",
			json!({
				"payload": {
					"userId": 123,
					"role": "admin",
					"exp": 9999999999_i64
				},
				"secret": "my-secret",
				"algorithm": "HS256"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	let token = result["data"].as_str().unwrap();
	assert!(token.contains('.'));

	// Test JWT verification
	let result = bridge
		.call(
			"crypto",
			"verify_jwt",
			json!({
				"token": token,
				"secret": "my-secret"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert_eq!(result["data"]["userId"], 123);
	assert_eq!(result["data"]["role"], "admin");

	// Test JWT decoding without verification
	let result = bridge
		.call(
			"crypto",
			"decode_jwt",
			json!({
				"token": token
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], true);
	assert_eq!(result["data"]["payload"]["userId"], 123);
	assert!(result["data"]["header"].is_object());
}

#[tokio::test]
async fn test_crypto_error_handling() {
	let mut bridge = HostBridge::new();

	// Test invalid algorithm
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "test",
				"algorithm": "invalid"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "ALGORITHM_ERROR");

	// Test empty password
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "",
				"algorithm": "bcrypt"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	// Test invalid JWT
	let result = bridge
		.call(
			"crypto",
			"verify_jwt",
			json!({
				"token": "invalid.jwt.token",
				"secret": "secret"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "AUTH_ERROR");

	// Test unknown function
	let result = bridge
		.call("crypto", "unknown", json!({}))
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "CRYPTO_ERROR");
}

#[tokio::test]
async fn test_crypto_security_features() {
	let mut bridge = HostBridge::new();

	// Test bcrypt cost validation
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "test",
				"algorithm": "bcrypt",
				"cost": 9  // Too low
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": "test",
				"algorithm": "bcrypt",
				"cost": 15  // Too high
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	// Test bcrypt password length limit
	let long_password = "a".repeat(73);
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": long_password,
				"algorithm": "bcrypt"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "VALIDATION_ERROR");

	// Test JWT expiration
	let result = bridge
		.call(
			"crypto",
			"sign",
			json!({
				"payload": {
					"sub": 1,
					"exp": 1000000000_i64  // Expired
				},
				"secret": "secret",
				"algorithm": "HS256"
			}),
		)
		.await
		.unwrap();
	let token = result["data"].as_str().unwrap();

	let result = bridge
		.call(
			"crypto",
			"verify_jwt",
			json!({
				"token": token,
				"secret": "secret"
			}),
		)
		.await
		.unwrap();
	assert_eq!(result["ok"], false);
	assert_eq!(result["err"]["code"], "AUTH_ERROR");
}
