use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod schema;
mod query;
mod transaction;
mod model;

pub use schema::{Schema, Field, FieldType, Constraint};
pub use query::{Query, QueryBuilder, WhereClause, OrderBy};
pub use transaction::Transaction;
pub use model::Model;

/// Main ORM interface for Frame Data
pub struct Data {
	connection: Connection,
}

impl Data {
	pub fn new(connection: Connection) -> Self {
		Self { connection }
	}

	/// Start a new database transaction
	pub async fn tx<F, T>(&mut self, f: F) -> Result<T>
	where
		F: FnOnce(&mut Transaction) -> Result<T>,
	{
		let mut tx = Transaction::new(&mut self.connection);
		let result = f(&mut tx)?;
		tx.commit().await?;
		Ok(result)
	}

	/// Execute a raw query
	pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
		self.connection.query(sql, params).await
	}

	/// Get the underlying connection
	pub fn connection(&mut self) -> &mut Connection {
		&mut self.connection
	}
}

/// Database connection abstraction
pub struct Connection {
	// This would interface with host-bridge::DbBridge
	pub(crate) driver: String,
	pub(crate) connected: bool,
}

impl Connection {
	pub fn new(driver: impl Into<String>) -> Self {
		Self {
			driver: driver.into(),
			connected: false,
		}
	}

	pub async fn connect(&mut self) -> Result<()> {
		// Would call host-bridge to establish connection
		self.connected = true;
		Ok(())
	}

	pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
		// TODO: Implement database query via Host Bridge
		// Reference: TASKS.md Phase 4 - Frame Data
		// Spec: documents/specification/04_frame_data.md
		//
		// Implementation steps:
		// 1. Call host_bridge::db::query with sql and params
		// 2. Parse BridgeResponse
		// 3. Convert DbResult rows to Vec<Row>
		// 4. Handle errors (DB_ERROR, TIMEOUT, etc.)

		unimplemented!("Connection::query - See TASKS.md Phase 4")
	}

	pub async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64> {
		// TODO: Implement database execute via Host Bridge
		// Reference: TASKS.md Phase 4 - Frame Data
		// Spec: documents/specification/04_frame_data.md
		//
		// Used for INSERT, UPDATE, DELETE
		// Returns number of affected rows

		unimplemented!("Connection::execute - See TASKS.md Phase 4")
	}
}

/// Database row abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
	pub columns: HashMap<String, serde_json::Value>,
}

impl Row {
	pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
		let value = self.columns.get(key)
			.ok_or_else(|| anyhow::anyhow!("Column not found: {}", key))?;
		Ok(serde_json::from_value(value.clone())?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_connection_creation() {
		let mut conn = Connection::new("postgres");
		assert!(!conn.connected);
		assert_eq!(conn.driver, "postgres");
	}

	#[test]
	fn test_row_get() {
		let mut columns = HashMap::new();
		columns.insert("id".to_string(), serde_json::json!(42));
		columns.insert("name".to_string(), serde_json::json!("Test"));

		let row = Row { columns };

		let id: i32 = row.get("id").unwrap();
		assert_eq!(id, 42);

		let name: String = row.get("name").unwrap();
		assert_eq!(name, "Test");
	}
}
