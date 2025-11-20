use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use host_bridge::{DbBridge, DbConfig};

mod schema;
mod query;
mod transaction;
mod model;

pub use schema::{Schema, Field, FieldType, Constraint};
pub use query::{
	Query, QueryBuilder, WhereClause, WhereCondition, WhereOperator, OrderBy, OrderDirection,
	InsertBuilder, UpdateBuilder, DeleteBuilder, Join, JoinType,
};
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
	pub(crate) driver: String,
	pub(crate) bridge: Arc<RwLock<DbBridge>>,
	pub(crate) connected: bool,
}

impl Connection {
	/// Create a new connection with the given driver
	pub fn new(driver: impl Into<String>) -> Self {
		Self {
			driver: driver.into(),
			bridge: Arc::new(RwLock::new(DbBridge::new())),
			connected: false,
		}
	}

	/// Create a connection from an existing DbBridge instance
	pub fn from_bridge(driver: impl Into<String>, bridge: Arc<RwLock<DbBridge>>) -> Self {
		Self {
			driver: driver.into(),
			bridge,
			connected: true,
		}
	}

	/// Connect to the database using the provided configuration
	pub async fn connect(&mut self, config: DbConfig) -> Result<()> {
		let mut bridge = self.bridge.write().await;
		bridge.configure(config).await?;
		self.connected = true;
		Ok(())
	}

	/// Execute a SELECT query and return rows
	pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
		if !self.connected {
			anyhow::bail!("Not connected to database. Call connect() first.");
		}

		// Prepare request
		let request = serde_json::json!({
			"sql": sql,
			"params": params
		});

		// Call DbBridge
		let mut bridge = self.bridge.write().await;
		let response = bridge.call("query", request).await?;

		// Parse response
		if response["ok"] == true {
			let rows = response["data"]["rows"]
				.as_array()
				.ok_or_else(|| anyhow::anyhow!("Invalid response format: missing rows array"))?;

			// Convert JSON rows to Row structs
			let result: Result<Vec<Row>> = rows
				.iter()
				.map(|row| {
					let obj = row
						.as_object()
						.ok_or_else(|| anyhow::anyhow!("Invalid row format"))?;

					let columns: HashMap<String, serde_json::Value> = obj
						.iter()
						.map(|(k, v)| (k.clone(), v.clone()))
						.collect();

					Ok(Row { columns })
				})
				.collect();

			result
		} else {
			// Extract error information
			let err = &response["err"];
			let code = err["code"]
				.as_str()
				.unwrap_or("DB_ERROR");
			let message = err["message"]
				.as_str()
				.unwrap_or("Database query failed");

			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Execute an INSERT/UPDATE/DELETE query and return affected rows
	pub async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64> {
		if !self.connected {
			anyhow::bail!("Not connected to database. Call connect() first.");
		}

		// Prepare request
		let request = serde_json::json!({
			"sql": sql,
			"params": params
		});

		// Call DbBridge
		let mut bridge = self.bridge.write().await;
		let response = bridge.call("execute", request).await?;

		// Parse response
		if response["ok"] == true {
			let affected_rows = response["data"]["affected_rows"]
				.as_u64()
				.ok_or_else(|| anyhow::anyhow!("Invalid response format: missing affected_rows"))?;

			Ok(affected_rows)
		} else {
			// Extract error information
			let err = &response["err"];
			let code = err["code"]
				.as_str()
				.unwrap_or("DB_ERROR");
			let message = err["message"]
				.as_str()
				.unwrap_or("Database execute failed");

			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Get reference to the underlying DbBridge (for advanced usage)
	pub fn bridge(&self) -> Arc<RwLock<DbBridge>> {
		Arc::clone(&self.bridge)
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

	// Enable SQLite driver for testing
	fn install_driver() {
		use sqlx::any::install_default_drivers;
		static INIT: std::sync::Once = std::sync::Once::new();
		INIT.call_once(|| {
			install_default_drivers();
		});
	}

	#[test]
	fn test_connection_creation() {
		let conn = Connection::new("postgres");
		assert!(!conn.connected);
		assert_eq!(conn.driver, "postgres");
	}

	#[tokio::test]
	async fn test_connection_connect() {
		install_driver();

		let mut conn = Connection::new("sqlite");

		// Use in-memory SQLite database for testing
		let config = DbConfig {
			database_url: "sqlite:file::memory:?cache=shared".to_string(),
			max_connections: 5,
			min_connections: 1,
			connection_timeout: 5000,
			query_timeout: 10000,
		};

		let result = conn.connect(config).await;
		assert!(result.is_ok(), "Failed to connect: {:?}", result.err());
		assert!(conn.connected);
	}

	#[tokio::test]
	async fn test_query_without_connection() {
		let conn = Connection::new("postgres");

		let result = conn.query("SELECT 1", vec![]).await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("Not connected"));
	}

	#[tokio::test]
	async fn test_execute_without_connection() {
		let conn = Connection::new("postgres");

		let result = conn.execute("INSERT INTO test VALUES (1)", vec![]).await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("Not connected"));
	}

	#[tokio::test]
	async fn test_query_with_connection() {
		install_driver();

		let mut conn = Connection::new("sqlite");

		// Use in-memory SQLite database for testing
		let config = DbConfig {
			database_url: "sqlite:file::memory:?cache=shared".to_string(),
			max_connections: 5,
			min_connections: 1,
			connection_timeout: 5000,
			query_timeout: 10000,
		};

		conn.connect(config).await.unwrap();

		// Create a test table
		conn.execute(
			"CREATE TABLE test_users (id INTEGER PRIMARY KEY, name TEXT)",
			vec![],
		)
		.await
		.unwrap();

		// Insert test data
		conn.execute(
			"INSERT INTO test_users (id, name) VALUES (?, ?)",
			vec![serde_json::json!(1), serde_json::json!("Alice")],
		)
		.await
		.unwrap();

		// Query the data
		let rows = conn.query("SELECT * FROM test_users", vec![]).await.unwrap();

		assert_eq!(rows.len(), 1);

		let id: i64 = rows[0].get("id").unwrap();
		assert_eq!(id, 1);

		let name: String = rows[0].get("name").unwrap();
		assert_eq!(name, "Alice");
	}

	#[tokio::test]
	async fn test_execute_returns_affected_rows() {
		install_driver();

		let mut conn = Connection::new("sqlite");

		// Use in-memory SQLite database for testing
		let config = DbConfig {
			database_url: "sqlite:file::memory:?cache=shared".to_string(),
			max_connections: 5,
			min_connections: 1,
			connection_timeout: 5000,
			query_timeout: 10000,
		};

		conn.connect(config).await.unwrap();

		// Create table
		conn.execute(
			"CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT)",
			vec![],
		)
		.await
		.unwrap();

		// Insert multiple rows
		let affected = conn
			.execute(
				"INSERT INTO test_data (id, value) VALUES (?, ?)",
				vec![serde_json::json!(1), serde_json::json!("test")],
			)
			.await
			.unwrap();

		assert_eq!(affected, 1);
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

	#[test]
	fn test_row_get_missing_column() {
		let columns = HashMap::new();
		let row = Row { columns };

		let result: Result<i32> = row.get("nonexistent");
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("Column not found"));
	}
}
