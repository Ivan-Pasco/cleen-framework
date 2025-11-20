use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Query {
	pub table: String,
	pub select: Vec<String>,
	pub where_clauses: Vec<WhereClause>,
	pub order_by: Vec<OrderBy>,
	pub limit: Option<usize>,
	pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
	pub field: String,
	pub operator: String,
	pub value: Value,
}

#[derive(Debug, Clone)]
pub struct OrderBy {
	pub field: String,
	pub direction: OrderDirection,
}

#[derive(Debug, Clone)]
pub enum OrderDirection {
	Asc,
	Desc,
}

pub struct QueryBuilder {
	query: Query,
}

impl QueryBuilder {
	pub fn new(table: impl Into<String>) -> Self {
		Self {
			query: Query {
				table: table.into(),
				select: vec!["*".to_string()],
				where_clauses: vec![],
				order_by: vec![],
				limit: None,
				offset: None,
			},
		}
	}

	pub fn select(mut self, fields: Vec<String>) -> Self {
		self.query.select = fields;
		self
	}

	pub fn where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereClause {
			field: field.into(),
			operator: "=".to_string(),
			value,
		});
		self
	}

	pub fn where_clause(mut self, clause: WhereClause) -> Self {
		self.query.where_clauses.push(clause);
		self
	}

	pub fn order_by(mut self, field: impl Into<String>, direction: OrderDirection) -> Self {
		self.query.order_by.push(OrderBy {
			field: field.into(),
			direction,
		});
		self
	}

	pub fn limit(mut self, limit: usize) -> Self {
		self.query.limit = Some(limit);
		self
	}

	pub fn offset(mut self, offset: usize) -> Self {
		self.query.offset = Some(offset);
		self
	}

	pub fn build(self) -> Query {
		self.query
	}

	pub fn to_sql(&self) -> (String, Vec<Value>) {
		let mut sql = format!("SELECT {} FROM {}", self.query.select.join(", "), self.query.table);
		let mut params = vec![];

		if !self.query.where_clauses.is_empty() {
			sql.push_str(" WHERE ");
			for (i, clause) in self.query.where_clauses.iter().enumerate() {
				if i > 0 {
					sql.push_str(" AND ");
				}
				sql.push_str(&format!("{} {} ${}", clause.field, clause.operator, params.len() + 1));
				params.push(clause.value.clone());
			}
		}

		if !self.query.order_by.is_empty() {
			sql.push_str(" ORDER BY ");
			for (i, order) in self.query.order_by.iter().enumerate() {
				if i > 0 {
					sql.push_str(", ");
				}
				let dir = match order.direction {
					OrderDirection::Asc => "ASC",
					OrderDirection::Desc => "DESC",
				};
				sql.push_str(&format!("{} {}", order.field, dir));
			}
		}

		if let Some(limit) = self.query.limit {
			sql.push_str(&format!(" LIMIT {}", limit));
		}

		if let Some(offset) = self.query.offset {
			sql.push_str(&format!(" OFFSET {}", offset));
		}

		(sql, params)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple_query() {
		let builder = QueryBuilder::new("users")
			.where_eq("active", serde_json::json!(true))
			.limit(10);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("SELECT * FROM users"));
		assert!(sql.contains("WHERE active = $1"));
		assert!(sql.contains("LIMIT 10"));
		assert_eq!(params.len(), 1);
	}

	#[test]
	fn test_complex_query() {
		let builder = QueryBuilder::new("posts")
			.select(vec!["id".to_string(), "title".to_string()])
			.where_eq("published", serde_json::json!(true))
			.order_by("created_at", OrderDirection::Desc)
			.limit(20)
			.offset(10);

		let (sql, _) = builder.to_sql();
		assert!(sql.contains("SELECT id, title FROM posts"));
		assert!(sql.contains("ORDER BY created_at DESC"));
		assert!(sql.contains("LIMIT 20"));
		assert!(sql.contains("OFFSET 10"));
	}
}
