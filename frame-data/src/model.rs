use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::{Connection, Row, QueryBuilder, InsertBuilder, UpdateBuilder, DeleteBuilder};

/// Trait that all ORM models must implement
pub trait Model: Sized + Serialize + for<'de> Deserialize<'de> {
	/// Get the table name for this model
	fn table_name() -> &'static str;

	/// Get the primary key field name
	fn primary_key() -> &'static str {
		"id"
	}

	/// Create a new record
	async fn create(conn: &mut Connection, data: Self) -> Result<Self> {
		// Serialize the model to JSON
		let json = serde_json::to_value(&data)?;
		let map = json.as_object()
			.ok_or_else(|| anyhow::anyhow!("Model must serialize to a JSON object"))?;

		// Separate columns and values, excluding the primary key if it's None/null
		let pk = Self::primary_key();
		let mut columns = vec![];
		let mut values = vec![];

		for (key, value) in map.iter() {
			// Skip primary key if it's null (auto-increment)
			if key == pk && value.is_null() {
				continue;
			}
			columns.push(key.clone());

			// Convert booleans to integers for SQLite compatibility
			let converted_value = if value.is_boolean() {
				serde_json::Value::Number(if value.as_bool().unwrap() { 1.into() } else { 0.into() })
			} else {
				value.clone()
			};
			values.push(converted_value);
		}

		// Build INSERT query
		let builder = InsertBuilder::new(Self::table_name())
			.columns(columns)
			.values(values);

		let (sql, params) = builder.to_sql();

		// Execute the insert
		conn.execute(&sql, params).await?;

		// For now, return the data as-is
		// In a real implementation, we'd fetch the inserted row to get the generated ID
		Ok(data)
	}

	/// Find a record by primary key
	async fn find(conn: &mut Connection, id: i64) -> Result<Option<Self>> {
		let query = QueryBuilder::new(Self::table_name())
			.where_eq(Self::primary_key(), serde_json::json!(id))
			.limit(1);

		let (sql, params) = query.to_sql();
		let rows = conn.query(&sql, params).await?;

		if let Some(row) = rows.first() {
			Ok(Some(Self::from_row(row)?))
		} else {
			Ok(None)
		}
	}

	/// Find all records matching a condition
	async fn where_clause(conn: &mut Connection, field: &str, value: serde_json::Value) -> Result<Vec<Self>> {
		let query = QueryBuilder::new(Self::table_name())
			.where_eq(field, value);

		let (sql, params) = query.to_sql();
		let rows = conn.query(&sql, params).await?;

		rows.iter()
			.map(|row| Self::from_row(row))
			.collect()
	}

	/// Get all records
	async fn all(conn: &mut Connection) -> Result<Vec<Self>> {
		let query = QueryBuilder::new(Self::table_name());
		let (sql, params) = query.to_sql();
		let rows = conn.query(&sql, params).await?;

		rows.iter()
			.map(|row| Self::from_row(row))
			.collect()
	}

	/// Update a record
	async fn update(conn: &mut Connection, id: i64, data: Self) -> Result<Self> {
		// Serialize the model to JSON
		let json = serde_json::to_value(&data)?;
		let map = json.as_object()
			.ok_or_else(|| anyhow::anyhow!("Model must serialize to a JSON object"))?;

		// Build UPDATE query, excluding the primary key
		let pk = Self::primary_key();
		let mut builder = UpdateBuilder::new(Self::table_name());

		for (key, value) in map.iter() {
			// Skip primary key in SET clause
			if key != pk {
				// Convert booleans to integers for SQLite compatibility
				let converted_value = if value.is_boolean() {
					serde_json::Value::Number(if value.as_bool().unwrap() { 1.into() } else { 0.into() })
				} else {
					value.clone()
				};
				builder = builder.set(key, converted_value);
			}
		}

		// Add WHERE clause for primary key
		builder = builder.where_eq(pk, serde_json::json!(id));

		let (sql, params) = builder.to_sql();

		// Execute the update
		let affected = conn.execute(&sql, params).await?;

		if affected == 0 {
			anyhow::bail!("No record found with {} = {}", pk, id);
		}

		Ok(data)
	}

	/// Delete a record
	async fn delete(conn: &mut Connection, id: i64) -> Result<bool> {
		let builder = DeleteBuilder::new(Self::table_name())
			.where_eq(Self::primary_key(), serde_json::json!(id));

		let (sql, params) = builder.to_sql();
		let affected = conn.execute(&sql, params).await?;
		Ok(affected > 0)
	}

	/// Convert a database row to this model
	fn from_row(row: &Row) -> Result<Self> {
		// SQLite stores booleans as INTEGER (0/1), but we need to be careful
		// not to convert ALL 0/1 values (like id fields) to booleans
		//
		// Strategy: Try deserializing first. If it fails due to a type mismatch,
		// examine the error to find which field failed, then convert ONLY that field's
		// integer to boolean.
		let json = serde_json::to_value(&row.columns)?;

		// Try to deserialize first
		match serde_json::from_value::<Self>(json.clone()) {
			Ok(model) => Ok(model),
			Err(e) => {
				let error_msg = e.to_string();

				// If the error is about expecting a boolean, convert integers to booleans
				// We use a heuristic: convert 0/1 integers to booleans for all fields,
				// EXCEPT fields that end with "id" or "_id" (which are likely primary/foreign keys)
				if error_msg.contains("expected a boolean") || error_msg.contains("expected `bool`") {
					let mut columns = row.columns.clone();
					for (key, value) in columns.iter_mut() {
						// Don't convert ID fields
						if key.to_lowercase().ends_with("id") || key == Self::primary_key() {
							continue;
						}

						// Convert 0/1 integers to booleans
						if let Some(num) = value.as_i64() {
							if num == 0 {
								*value = serde_json::Value::Bool(false);
							} else if num == 1 {
								*value = serde_json::Value::Bool(true);
							}
						}
					}
					let json = serde_json::to_value(&columns)?;
					Ok(serde_json::from_value(json)?)
				} else {
					Err(e.into())
				}
			},
		}
	}
}

/// Example model implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: Option<i64>,
	pub name: String,
	pub email: String,
	pub active: bool,
}

impl Model for User {
	fn table_name() -> &'static str {
		"users"
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::DbConfig;

	// Enable SQLite driver for testing
	fn install_driver() {
		use sqlx::any::install_default_drivers;
		static INIT: std::sync::Once = std::sync::Once::new();
		INIT.call_once(|| {
			install_default_drivers();
		});
	}

	async fn setup_test_db() -> Connection {
		install_driver();

		let mut conn = Connection::new("sqlite");

		// Use a unique named in-memory database for each test to avoid conflicts
		// Combine thread ID, timestamp, and random number for uniqueness
		use std::sync::atomic::{AtomicU64, Ordering};
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

		let db_name = format!("test_db_{}_{:?}",
			counter,
			std::thread::current().id());

		let config = DbConfig {
			database_url: format!("sqlite:file:{}?mode=memory&cache=shared", db_name),
			max_connections: 5,
			min_connections: 1,
			connection_timeout: 5000,
			query_timeout: 10000,
		};

		conn.connect(config).await.unwrap();

		// Create users table
		conn.execute(
			"CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT NOT NULL, active INTEGER NOT NULL)",
			vec![],
		)
		.await
		.unwrap();

		conn
	}

	#[test]
	fn test_user_model() {
		let user = User {
			id: Some(1),
			name: "Test User".to_string(),
			email: "test@example.com".to_string(),
			active: true,
		};

		assert_eq!(User::table_name(), "users");
		assert_eq!(User::primary_key(), "id");
	}

	#[tokio::test]
	async fn test_model_create() {
		let mut conn = setup_test_db().await;

		let user = User {
			id: None,
			name: "Alice".to_string(),
			email: "alice@example.com".to_string(),
			active: true,
		};

		let result = User::create(&mut conn, user).await;
		assert!(result.is_ok(), "Create failed: {:?}", result.err());
	}

	#[tokio::test]
	async fn test_model_find() {
		let mut conn = setup_test_db().await;

		// Insert a user directly
		conn.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Bob"),
				serde_json::json!("bob@example.com"),
				serde_json::json!(1),
			],
		)
		.await
		.unwrap();

		// Find the user
		let result = User::find(&mut conn, 1).await;
		assert!(result.is_ok());

		let user = result.unwrap();
		assert!(user.is_some());

		let user = user.unwrap();
		assert_eq!(user.name, "Bob");
		assert_eq!(user.email, "bob@example.com");
		assert_eq!(user.active, true);
	}

	#[tokio::test]
	async fn test_model_find_not_found() {
		let mut conn = setup_test_db().await;

		let result = User::find(&mut conn, 999).await;
		assert!(result.is_ok());
		assert!(result.unwrap().is_none());
	}

	#[tokio::test]
	async fn test_model_where_clause() {
		let mut conn = setup_test_db().await;

		// Insert multiple users
		conn.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Active User"),
				serde_json::json!("active@example.com"),
				serde_json::json!(1),
			],
		)
		.await
		.unwrap();

		conn.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Inactive User"),
				serde_json::json!("inactive@example.com"),
				serde_json::json!(0),
			],
		)
		.await
		.unwrap();

		// Find active users
		let result = User::where_clause(&mut conn, "active", serde_json::json!(1)).await;
		assert!(result.is_ok());

		let users = result.unwrap();
		assert_eq!(users.len(), 1);
		assert_eq!(users[0].name, "Active User");
	}

	#[tokio::test]
	async fn test_model_all() {
		let mut conn = setup_test_db().await;

		// Insert multiple users
		for i in 1..=3 {
			conn.execute(
				"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
				vec![
					serde_json::json!(format!("User {}", i)),
					serde_json::json!(format!("user{}@example.com", i)),
					serde_json::json!(1),
				],
			)
			.await
			.unwrap();
		}

		// Get all users
		let result = User::all(&mut conn).await;
		assert!(result.is_ok());

		let users = result.unwrap();
		assert_eq!(users.len(), 3);
	}

	#[tokio::test]
	async fn test_model_update() {
		let mut conn = setup_test_db().await;

		// Insert a user
		conn.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Original Name"),
				serde_json::json!("original@example.com"),
				serde_json::json!(1),
			],
		)
		.await
		.unwrap();

		// Update the user
		let updated_user = User {
			id: Some(1),
			name: "Updated Name".to_string(),
			email: "updated@example.com".to_string(),
			active: false,
		};

		let result = User::update(&mut conn, 1, updated_user).await;
		assert!(result.is_ok(), "Update failed: {:?}", result.err());

		// Verify the update
		let user = User::find(&mut conn, 1).await.unwrap().unwrap();
		assert_eq!(user.name, "Updated Name");
		assert_eq!(user.email, "updated@example.com");
		assert_eq!(user.active, false);
	}

	#[tokio::test]
	async fn test_model_update_not_found() {
		let mut conn = setup_test_db().await;

		let user = User {
			id: Some(999),
			name: "Non-existent".to_string(),
			email: "none@example.com".to_string(),
			active: true,
		};

		let result = User::update(&mut conn, 999, user).await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("No record found"));
	}

	#[tokio::test]
	async fn test_model_delete() {
		let mut conn = setup_test_db().await;

		// Insert a user
		conn.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("To Delete"),
				serde_json::json!("delete@example.com"),
				serde_json::json!(1),
			],
		)
		.await
		.unwrap();

		// Delete the user
		let result = User::delete(&mut conn, 1).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), true);

		// Verify deletion
		let user = User::find(&mut conn, 1).await.unwrap();
		assert!(user.is_none());
	}

	#[tokio::test]
	async fn test_model_delete_not_found() {
		let mut conn = setup_test_db().await;

		let result = User::delete(&mut conn, 999).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), false);
	}

	#[tokio::test]
	async fn test_model_from_row() {
		let mut columns = std::collections::HashMap::new();
		columns.insert("id".to_string(), serde_json::json!(1));
		columns.insert("name".to_string(), serde_json::json!("Test"));
		columns.insert("email".to_string(), serde_json::json!("test@example.com"));
		columns.insert("active".to_string(), serde_json::json!(true));

		let row = Row { columns };
		let result = User::from_row(&row);

		assert!(result.is_ok());
		let user = result.unwrap();
		assert_eq!(user.id, Some(1));
		assert_eq!(user.name, "Test");
		assert_eq!(user.email, "test@example.com");
		assert_eq!(user.active, true);
	}
}
