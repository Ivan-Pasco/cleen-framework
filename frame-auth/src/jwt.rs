use anyhow::Result;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
	pub secret: String,
	pub ttl_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: String,
	pub exp: usize,
}

pub fn encode_jwt(user_id: &str, config: &JwtConfig) -> Result<String> {
	let expiration = chrono::Utc::now()
		.checked_add_signed(chrono::Duration::minutes(config.ttl_minutes as i64))
		.unwrap()
		.timestamp() as usize;

	let claims = Claims {
		sub: user_id.to_string(),
		exp: expiration,
	};

	Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(config.secret.as_bytes()))?)
}

pub fn decode_jwt(token: &str, config: &JwtConfig) -> Result<Claims> {
	let token_data = decode::<Claims>(
		token,
		&DecodingKey::from_secret(config.secret.as_bytes()),
		&Validation::default()
	)?;

	Ok(token_data.claims)
}
