/// Example demonstrating the Host Bridge CRYPTO module
///
/// This example shows how to use all crypto functions:
/// - Random generation
/// - Password hashing (bcrypt and argon2)
/// - Password verification
/// - JWT signing and verification
/// - JWT decoding

use host_bridge::HostBridge;
use serde_json::json;

#[tokio::main]
async fn main() {
	let mut bridge = HostBridge::new();

	println!("=== Host Bridge CRYPTO Module Examples ===\n");

	// 1. Generate random bytes
	println!("1. Generating 32 random bytes...");
	let result = bridge
		.call("crypto", "random", json!({"bytes": 32}))
		.await
		.unwrap();

	if result["ok"] == true {
		println!("   Random bytes (base64): {}\n", result["data"]);
	}

	// 2. Hash a password with bcrypt
	println!("2. Hashing password with bcrypt (cost 10)...");
	let password = "my-super-secret-password";
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": password,
				"algorithm": "bcrypt",
				"cost": 10
			}),
		)
		.await
		.unwrap();

	let bcrypt_hash = result["data"].as_str().unwrap().to_string();
	println!("   Bcrypt hash: {}\n", bcrypt_hash);

	// 3. Verify the password
	println!("3. Verifying correct password...");
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": password,
				"hash": &bcrypt_hash
			}),
		)
		.await
		.unwrap();

	println!("   Password matches: {}\n", result["data"]);

	// 4. Verify wrong password
	println!("4. Verifying incorrect password...");
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": "wrong-password",
				"hash": &bcrypt_hash
			}),
		)
		.await
		.unwrap();

	println!("   Password matches: {}\n", result["data"]);

	// 5. Hash with argon2
	println!("5. Hashing password with argon2...");
	let result = bridge
		.call(
			"crypto",
			"hash",
			json!({
				"password": password,
				"algorithm": "argon2"
			}),
		)
		.await
		.unwrap();

	let argon2_hash = result["data"].as_str().unwrap().to_string();
	println!("   Argon2 hash: {}\n", argon2_hash);

	// 6. Verify argon2 password
	println!("6. Verifying argon2 password...");
	let result = bridge
		.call(
			"crypto",
			"verify",
			json!({
				"password": password,
				"hash": &argon2_hash
			}),
		)
		.await
		.unwrap();

	println!("   Password matches: {}\n", result["data"]);

	// 7. Sign a JWT token
	println!("7. Signing JWT token...");
	let exp = chrono::Utc::now().timestamp() + 3600; // 1 hour from now
	let result = bridge
		.call(
			"crypto",
			"sign",
			json!({
				"payload": {
					"userId": 123,
					"email": "user@example.com",
					"role": "admin",
					"exp": exp
				},
				"secret": "my-jwt-secret",
				"algorithm": "HS256"
			}),
		)
		.await
		.unwrap();

	let token = result["data"].as_str().unwrap().to_string();
	println!("   JWT token: {}...\n", &token[..50]);

	// 8. Verify JWT token
	println!("8. Verifying JWT token...");
	let result = bridge
		.call(
			"crypto",
			"verify_jwt",
			json!({
				"token": &token,
				"secret": "my-jwt-secret"
			}),
		)
		.await
		.unwrap();

	if result["ok"] == true {
		println!("   Token valid!");
		println!("   User ID: {}", result["data"]["userId"]);
		println!("   Email: {}", result["data"]["email"]);
		println!("   Role: {}\n", result["data"]["role"]);
	}

	// 9. Decode JWT without verification (debugging)
	println!("9. Decoding JWT without verification...");
	let result = bridge
		.call(
			"crypto",
			"decode_jwt",
			json!({
				"token": &token
			}),
		)
		.await
		.unwrap();

	if result["ok"] == true {
		println!("   Header: {}", serde_json::to_string_pretty(&result["data"]["header"]).unwrap());
		println!("   Payload: {}\n", serde_json::to_string_pretty(&result["data"]["payload"]).unwrap());
	}

	// 10. Test different JWT algorithms
	println!("10. Testing JWT algorithms...");
	for algorithm in &["HS256", "HS384", "HS512"] {
		let result = bridge
			.call(
				"crypto",
				"sign",
				json!({
					"payload": {
						"sub": 1,
						"exp": exp
					},
					"secret": "secret",
					"algorithm": algorithm
				}),
			)
			.await
			.unwrap();

		if result["ok"] == true {
			println!("   {} token signed successfully", algorithm);
		}
	}

	println!("\n=== All crypto operations completed successfully ===");
}
