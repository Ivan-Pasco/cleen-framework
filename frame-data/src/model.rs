use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::{Connection, Row, QueryBuilder};
use crate::query::OrderDirection;

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
		// Would serialize to SQL INSERT and execute
		// For now, just return the input
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
		// Would serialize to SQL UPDATE and execute
		Ok(data)
	}

	/// Delete a record
	async fn delete(conn: &mut Connection, id: i64) -> Result<bool> {
		let sql = format!("DELETE FROM {} WHERE {} = $1", Self::table_name(), Self::primary_key());
		let affected = conn.execute(&sql, vec![serde_json::json!(id)]).await?;
		Ok(affected > 0)
	}

	/// Convert a database row to this model
	fn from_row(row: &Row) -> Result<Self> {
		let json = serde_json::to_value(&row.columns)?;
		Ok(serde_json::from_value(json)?)
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
}
