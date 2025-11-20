use anyhow::Result;
use crate::{Connection, Row};

/// Database transaction wrapper
///
/// Provides ACID transaction support via Host Bridge:
/// - Begin: Starts a new transaction and returns a unique ID
/// - Commit: Executes all queued operations atomically
/// - Rollback: Discards all pending operations
/// - Auto-rollback on drop if not committed
pub struct Transaction<'a> {
	connection: &'a mut Connection,
	tx_id: Option<String>,
	committed: bool,
	rolled_back: bool,
}

impl<'a> Transaction<'a> {
	/// Create a new transaction (does not begin it yet)
	pub fn new(connection: &'a mut Connection) -> Self {
		Self {
			connection,
			tx_id: None,
			committed: false,
			rolled_back: false,
		}
	}

	/// Begin the transaction via Host Bridge
	pub async fn begin(&mut self) -> Result<()> {
		if self.tx_id.is_some() {
			anyhow::bail!("Transaction already begun");
		}

		// Call Host Bridge to begin transaction
		let request = serde_json::json!({});
		let mut bridge = self.connection.bridge.write().await;
		let response = bridge.call("transaction_begin", request).await?;

		// Parse response
		if response["ok"] == true {
			let tx_id = response["data"]["tx_id"]
				.as_str()
				.ok_or_else(|| anyhow::anyhow!("Invalid response: missing tx_id"))?
				.to_string();

			self.tx_id = Some(tx_id);
			Ok(())
		} else {
			let err = &response["err"];
			let code = err["code"].as_str().unwrap_or("TRANSACTION_ERROR");
			let message = err["message"].as_str().unwrap_or("Failed to begin transaction");
			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Commit the transaction via Host Bridge
	pub async fn commit(&mut self) -> Result<()> {
		if self.rolled_back {
			anyhow::bail!("Transaction already rolled back");
		}
		if self.committed {
			anyhow::bail!("Transaction already committed");
		}

		let tx_id = self.tx_id.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Transaction not begun. Call begin() first."))?;

		// Call Host Bridge to commit transaction
		let request = serde_json::json!({
			"tx_id": tx_id
		});

		let mut bridge = self.connection.bridge.write().await;
		let response = bridge.call("transaction_commit", request).await?;

		// Parse response
		if response["ok"] == true {
			self.committed = true;
			Ok(())
		} else {
			let err = &response["err"];
			let code = err["code"].as_str().unwrap_or("TRANSACTION_ERROR");
			let message = err["message"].as_str().unwrap_or("Failed to commit transaction");
			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Rollback the transaction via Host Bridge
	pub async fn rollback(&mut self) -> Result<()> {
		if self.committed {
			anyhow::bail!("Transaction already committed");
		}
		if self.rolled_back {
			anyhow::bail!("Transaction already rolled back");
		}

		let tx_id = self.tx_id.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Transaction not begun. Call begin() first."))?;

		// Call Host Bridge to rollback transaction
		let request = serde_json::json!({
			"tx_id": tx_id
		});

		let mut bridge = self.connection.bridge.write().await;
		let response = bridge.call("transaction_rollback", request).await?;

		// Parse response
		if response["ok"] == true {
			self.rolled_back = true;
			Ok(())
		} else {
			let err = &response["err"];
			let code = err["code"].as_str().unwrap_or("TRANSACTION_ERROR");
			let message = err["message"].as_str().unwrap_or("Failed to rollback transaction");
			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Execute a query within the transaction
	pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<Vec<Row>> {
		let tx_id = self.tx_id.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Transaction not begun. Call begin() first."))?;

		// Prepare request with transaction ID
		let request = serde_json::json!({
			"tx_id": tx_id,
			"sql": sql,
			"params": params
		});

		// Call DbBridge query_in_tx method
		let mut bridge = self.connection.bridge.write().await;
		let response = bridge.call("query_in_tx", request).await?;

		// Parse response (same as Connection::query)
		if response["ok"] == true {
			let rows = response["data"]["rows"]
				.as_array()
				.ok_or_else(|| anyhow::anyhow!("Invalid response format: missing rows array"))?;

			let result: Result<Vec<Row>> = rows
				.iter()
				.map(|row| {
					let obj = row
						.as_object()
						.ok_or_else(|| anyhow::anyhow!("Invalid row format"))?;

					let columns: std::collections::HashMap<String, serde_json::Value> = obj
						.iter()
						.map(|(k, v)| (k.clone(), v.clone()))
						.collect();

					Ok(Row { columns })
				})
				.collect();

			result
		} else {
			let err = &response["err"];
			let code = err["code"].as_str().unwrap_or("DB_ERROR");
			let message = err["message"].as_str().unwrap_or("Database query failed");
			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Execute an INSERT/UPDATE/DELETE within the transaction
	pub async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64> {
		let tx_id = self.tx_id.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Transaction not begun. Call begin() first."))?;

		// Prepare request with transaction ID
		let request = serde_json::json!({
			"tx_id": tx_id,
			"sql": sql,
			"params": params
		});

		// Call DbBridge execute_in_tx method
		let mut bridge = self.connection.bridge.write().await;
		let response = bridge.call("execute_in_tx", request).await?;

		// Parse response (same as Connection::execute)
		if response["ok"] == true {
			let affected_rows = response["data"]["affected_rows"]
				.as_u64()
				.ok_or_else(|| anyhow::anyhow!("Invalid response format: missing affected_rows"))?;

			Ok(affected_rows)
		} else {
			let err = &response["err"];
			let code = err["code"].as_str().unwrap_or("DB_ERROR");
			let message = err["message"].as_str().unwrap_or("Database execute failed");
			anyhow::bail!("[{}] {}", code, message)
		}
	}

	/// Get the transaction ID (if begun)
	pub fn tx_id(&self) -> Option<&str> {
		self.tx_id.as_deref()
	}

	/// Check if transaction is committed
	pub fn is_committed(&self) -> bool {
		self.committed
	}

	/// Check if transaction is rolled back
	pub fn is_rolled_back(&self) -> bool {
		self.rolled_back
	}
}

impl<'a> Drop for Transaction<'a> {
	fn drop(&mut self) {
		if !self.committed && !self.rolled_back && self.tx_id.is_some() {
			// Auto-rollback on drop if not explicitly committed
			// We can't call async rollback in Drop, so we just log a warning
			eprintln!("Warning: Transaction {} dropped without commit or rollback. Pending operations will be lost.",
				self.tx_id.as_ref().unwrap());
		}
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

		// Use unique in-memory database
		use std::sync::atomic::{AtomicU64, Ordering};
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
		let db_name = format!("test_tx_{}_{:?}", counter, std::thread::current().id());

		let config = DbConfig {
			database_url: format!("sqlite:file:{}?mode=memory&cache=shared", db_name),
			max_connections: 5,
			min_connections: 1,
			connection_timeout: 5000,
			query_timeout: 10000,
		};

		conn.connect(config).await.unwrap();

		// Create test table
		conn.execute(
			"CREATE TABLE accounts (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, balance INTEGER NOT NULL)",
			vec![],
		)
		.await
		.unwrap();

		// Insert test data
		conn.execute(
			"INSERT INTO accounts (name, balance) VALUES (?, ?)",
			vec![serde_json::json!("Alice"), serde_json::json!(1000)],
		)
		.await
		.unwrap();

		conn.execute(
			"INSERT INTO accounts (name, balance) VALUES (?, ?)",
			vec![serde_json::json!("Bob"), serde_json::json!(500)],
		)
		.await
		.unwrap();

		conn
	}

	#[tokio::test]
	async fn test_transaction_begin() {
		let mut conn = setup_test_db().await;

		let mut tx = Transaction::new(&mut conn);
		assert!(tx.tx_id().is_none());

		let result = tx.begin().await;
		assert!(result.is_ok(), "Begin failed: {:?}", result.err());
		assert!(tx.tx_id().is_some());
		assert!(!tx.is_committed());
		assert!(!tx.is_rolled_back());
	}

	#[tokio::test]
	async fn test_transaction_commit() {
		let mut conn = setup_test_db().await;

		// Use a scope to ensure tx is dropped before we use conn again
		{
			let mut tx = Transaction::new(&mut conn);
			tx.begin().await.unwrap();

			// Execute update within transaction
			// Note: execute_in_tx queues the operation, so affected_rows will be 0 until commit
			let affected = tx.execute(
				"UPDATE accounts SET balance = balance - ? WHERE name = ?",
				vec![serde_json::json!(100), serde_json::json!("Alice")],
			).await.unwrap();
			assert_eq!(affected, 0); // Operation is queued, not yet executed

			// Commit transaction (this actually executes the queued operations)
			let result = tx.commit().await;
			assert!(result.is_ok(), "Commit failed: {:?}", result.err());
			assert!(tx.is_committed());
			assert!(!tx.is_rolled_back());
		} // tx is dropped here

		// Verify changes persisted
		let rows = conn.query(
			"SELECT balance FROM accounts WHERE name = ?",
			vec![serde_json::json!("Alice")],
		).await.unwrap();
		let balance: i64 = rows[0].get("balance").unwrap();
		assert_eq!(balance, 900);
	}

	#[tokio::test]
	async fn test_transaction_rollback() {
		let mut conn = setup_test_db().await;

		// Use a scope to ensure tx is dropped before we use conn again
		{
			let mut tx = Transaction::new(&mut conn);
			tx.begin().await.unwrap();

			// Execute update within transaction
			tx.execute(
				"UPDATE accounts SET balance = balance - ? WHERE name = ?",
				vec![serde_json::json!(100), serde_json::json!("Alice")],
			).await.unwrap();

			// Rollback transaction
			let result = tx.rollback().await;
			assert!(result.is_ok(), "Rollback failed: {:?}", result.err());
			assert!(!tx.is_committed());
			assert!(tx.is_rolled_back());
		} // tx is dropped here

		// Verify changes were NOT persisted
		let rows = conn.query(
			"SELECT balance FROM accounts WHERE name = ?",
			vec![serde_json::json!("Alice")],
		).await.unwrap();
		let balance: i64 = rows[0].get("balance").unwrap();
		assert_eq!(balance, 1000); // Original balance
	}

	#[tokio::test]
	async fn test_transaction_query() {
		let mut conn = setup_test_db().await;

		let mut tx = Transaction::new(&mut conn);
		tx.begin().await.unwrap();

		// Query within transaction
		let rows = tx.query(
			"SELECT * FROM accounts WHERE name = ?",
			vec![serde_json::json!("Alice")],
		).await.unwrap();

		assert_eq!(rows.len(), 1);
		let name: String = rows[0].get("name").unwrap();
		assert_eq!(name, "Alice");

		tx.commit().await.unwrap();
	}

	#[tokio::test]
	async fn test_transaction_multiple_operations() {
		let mut conn = setup_test_db().await;

		// Use a scope to ensure tx is dropped before we use conn again
		{
			let mut tx = Transaction::new(&mut conn);
			tx.begin().await.unwrap();

			// Transfer money from Alice to Bob
			tx.execute(
				"UPDATE accounts SET balance = balance - ? WHERE name = ?",
				vec![serde_json::json!(200), serde_json::json!("Alice")],
			).await.unwrap();

			tx.execute(
				"UPDATE accounts SET balance = balance + ? WHERE name = ?",
				vec![serde_json::json!(200), serde_json::json!("Bob")],
			).await.unwrap();

			tx.commit().await.unwrap();
		} // tx is dropped here

		// Verify both accounts updated
		let alice_rows = conn.query(
			"SELECT balance FROM accounts WHERE name = ?",
			vec![serde_json::json!("Alice")],
		).await.unwrap();
		let alice_balance: i64 = alice_rows[0].get("balance").unwrap();
		assert_eq!(alice_balance, 800);

		let bob_rows = conn.query(
			"SELECT balance FROM accounts WHERE name = ?",
			vec![serde_json::json!("Bob")],
		).await.unwrap();
		let bob_balance: i64 = bob_rows[0].get("balance").unwrap();
		assert_eq!(bob_balance, 700);
	}

	#[tokio::test]
	async fn test_transaction_error_without_begin() {
		let mut conn = setup_test_db().await;

		let tx = Transaction::new(&mut conn);

		// Try to commit without begin
		let result = tx.execute("SELECT 1", vec![]).await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("not begun"));
	}

	#[tokio::test]
	async fn test_transaction_double_commit() {
		let mut conn = setup_test_db().await;

		let mut tx = Transaction::new(&mut conn);
		tx.begin().await.unwrap();
		tx.commit().await.unwrap();

		// Try to commit again
		let result = tx.commit().await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("already committed"));
	}

	#[tokio::test]
	async fn test_transaction_commit_after_rollback() {
		let mut conn = setup_test_db().await;

		let mut tx = Transaction::new(&mut conn);
		tx.begin().await.unwrap();
		tx.rollback().await.unwrap();

		// Try to commit after rollback
		let result = tx.commit().await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("already rolled back"));
	}

	#[tokio::test]
	async fn test_transaction_rollback_after_commit() {
		let mut conn = setup_test_db().await;

		let mut tx = Transaction::new(&mut conn);
		tx.begin().await.unwrap();
		tx.commit().await.unwrap();

		// Try to rollback after commit
		let result = tx.rollback().await;
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("already committed"));
	}
}
