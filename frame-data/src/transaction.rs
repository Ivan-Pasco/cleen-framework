use anyhow::Result;
use crate::Connection;

pub struct Transaction<'a> {
	connection: &'a mut Connection,
	committed: bool,
	rolled_back: bool,
}

impl<'a> Transaction<'a> {
	pub fn new(connection: &'a mut Connection) -> Self {
		Self {
			connection,
			committed: false,
			rolled_back: false,
		}
	}

	pub async fn commit(&mut self) -> Result<()> {
		if self.rolled_back {
			anyhow::bail!("Transaction already rolled back");
		}
		if self.committed {
			anyhow::bail!("Transaction already committed");
		}

		// Would call host-bridge to commit transaction
		self.committed = true;
		Ok(())
	}

	pub async fn rollback(&mut self) -> Result<()> {
		if self.committed {
			anyhow::bail!("Transaction already committed");
		}
		if self.rolled_back {
			anyhow::bail!("Transaction already rolled back");
		}

		// Would call host-bridge to rollback transaction
		self.rolled_back = true;
		Ok(())
	}

	pub async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64> {
		self.connection.execute(sql, params).await
	}
}

impl<'a> Drop for Transaction<'a> {
	fn drop(&mut self) {
		if !self.committed && !self.rolled_back {
			// Auto-rollback on drop if not explicitly committed
			// In a real implementation, this would call the rollback
		}
	}
}
