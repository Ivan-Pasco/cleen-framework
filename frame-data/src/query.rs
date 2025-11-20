use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Query {
	pub table: String,
	pub select: Vec<String>,
	pub joins: Vec<Join>,
	pub where_clauses: Vec<WhereCondition>,
	pub order_by: Vec<OrderBy>,
	pub limit: Option<usize>,
	pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Join {
	pub join_type: JoinType,
	pub table: String,
	pub on_condition: String,
}

#[derive(Debug, Clone)]
pub enum JoinType {
	Inner,
	Left,
	Right,
	Full,
}

#[derive(Debug, Clone)]
pub enum WhereCondition {
	Simple(WhereClause),
	And(Vec<WhereCondition>),
	Or(Vec<WhereCondition>),
}

#[derive(Debug, Clone)]
pub struct WhereClause {
	pub field: String,
	pub operator: WhereOperator,
	pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhereOperator {
	Eq,           // =
	NotEq,        // !=
	Gt,           // >
	Gte,          // >=
	Lt,           // <
	Lte,          // <=
	Like,         // LIKE
	NotLike,      // NOT LIKE
	In,           // IN (...)
	NotIn,        // NOT IN (...)
	IsNull,       // IS NULL
	IsNotNull,    // IS NOT NULL
	Between,      // BETWEEN x AND y (value should be array of 2)
}

impl WhereOperator {
	fn to_sql(&self) -> &'static str {
		match self {
			WhereOperator::Eq => "=",
			WhereOperator::NotEq => "!=",
			WhereOperator::Gt => ">",
			WhereOperator::Gte => ">=",
			WhereOperator::Lt => "<",
			WhereOperator::Lte => "<=",
			WhereOperator::Like => "LIKE",
			WhereOperator::NotLike => "NOT LIKE",
			WhereOperator::In => "IN",
			WhereOperator::NotIn => "NOT IN",
			WhereOperator::IsNull => "IS NULL",
			WhereOperator::IsNotNull => "IS NOT NULL",
			WhereOperator::Between => "BETWEEN",
		}
	}
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
				joins: vec![],
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

	// JOIN methods
	pub fn inner_join(mut self, table: impl Into<String>, on_condition: impl Into<String>) -> Self {
		self.query.joins.push(Join {
			join_type: JoinType::Inner,
			table: table.into(),
			on_condition: on_condition.into(),
		});
		self
	}

	pub fn left_join(mut self, table: impl Into<String>, on_condition: impl Into<String>) -> Self {
		self.query.joins.push(Join {
			join_type: JoinType::Left,
			table: table.into(),
			on_condition: on_condition.into(),
		});
		self
	}

	pub fn right_join(mut self, table: impl Into<String>, on_condition: impl Into<String>) -> Self {
		self.query.joins.push(Join {
			join_type: JoinType::Right,
			table: table.into(),
			on_condition: on_condition.into(),
		});
		self
	}

	// WHERE methods
	pub fn where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Eq,
			value: Some(value),
		}));
		self
	}

	pub fn where_not_eq(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::NotEq,
			value: Some(value),
		}));
		self
	}

	pub fn where_gt(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Gt,
			value: Some(value),
		}));
		self
	}

	pub fn where_gte(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Gte,
			value: Some(value),
		}));
		self
	}

	pub fn where_lt(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Lt,
			value: Some(value),
		}));
		self
	}

	pub fn where_lte(mut self, field: impl Into<String>, value: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Lte,
			value: Some(value),
		}));
		self
	}

	pub fn where_like(mut self, field: impl Into<String>, pattern: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Like,
			value: Some(pattern),
		}));
		self
	}

	pub fn where_in(mut self, field: impl Into<String>, values: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::In,
			value: Some(values),
		}));
		self
	}

	pub fn where_null(mut self, field: impl Into<String>) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::IsNull,
			value: None,
		}));
		self
	}

	pub fn where_not_null(mut self, field: impl Into<String>) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::IsNotNull,
			value: None,
		}));
		self
	}

	pub fn where_between(mut self, field: impl Into<String>, min: Value, max: Value) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Between,
			value: Some(serde_json::json!([min, max])),
		}));
		self
	}

	pub fn or(mut self, conditions: Vec<WhereCondition>) -> Self {
		self.query.where_clauses.push(WhereCondition::Or(conditions));
		self
	}

	pub fn where_clause(mut self, clause: WhereClause) -> Self {
		self.query.where_clauses.push(WhereCondition::Simple(clause));
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

		// Add JOINs
		for join in &self.query.joins {
			let join_type_str = match join.join_type {
				JoinType::Inner => "INNER JOIN",
				JoinType::Left => "LEFT JOIN",
				JoinType::Right => "RIGHT JOIN",
				JoinType::Full => "FULL OUTER JOIN",
			};
			sql.push_str(&format!(" {} {} ON {}", join_type_str, join.table, join.on_condition));
		}

		// Add WHERE clause
		if !self.query.where_clauses.is_empty() {
			sql.push_str(" WHERE ");
			let where_sql = self.build_where_conditions(&self.query.where_clauses, &mut params);
			sql.push_str(&where_sql);
		}

		// Add ORDER BY
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

		// Add LIMIT
		if let Some(limit) = self.query.limit {
			sql.push_str(&format!(" LIMIT {}", limit));
		}

		// Add OFFSET
		if let Some(offset) = self.query.offset {
			sql.push_str(&format!(" OFFSET {}", offset));
		}

		(sql, params)
	}

	fn build_where_conditions(&self, conditions: &[WhereCondition], params: &mut Vec<Value>) -> String {
		let mut parts = vec![];

		for (i, condition) in conditions.iter().enumerate() {
			match condition {
				WhereCondition::Simple(clause) => {
					if i > 0 {
						parts.push(" AND ".to_string());
					}
					parts.push(self.build_where_clause(clause, params));
				}
				WhereCondition::And(sub_conditions) => {
					if i > 0 {
						parts.push(" AND ".to_string());
					}
					parts.push(format!("({})", self.build_where_conditions(sub_conditions, params)));
				}
				WhereCondition::Or(sub_conditions) => {
					if i > 0 {
						parts.push(" AND ".to_string());
					}
					let or_parts: Vec<String> = sub_conditions
						.iter()
						.map(|cond| match cond {
							WhereCondition::Simple(clause) => self.build_where_clause(clause, params),
							_ => self.build_where_conditions(&[cond.clone()], params),
						})
						.collect();
					parts.push(format!("({})", or_parts.join(" OR ")));
				}
			}
		}

		parts.join("")
	}

	fn build_where_clause(&self, clause: &WhereClause, params: &mut Vec<Value>) -> String {
		match clause.operator {
			WhereOperator::IsNull => {
				format!("{} IS NULL", clause.field)
			}
			WhereOperator::IsNotNull => {
				format!("{} IS NOT NULL", clause.field)
			}
			WhereOperator::In | WhereOperator::NotIn => {
				if let Some(value) = &clause.value {
					if let Some(arr) = value.as_array() {
						let placeholders: Vec<String> = arr
							.iter()
							.map(|v| {
								params.push(v.clone());
								format!("${}", params.len())
							})
							.collect();
						format!(
							"{} {} ({})",
							clause.field,
							clause.operator.to_sql(),
							placeholders.join(", ")
						)
					} else {
						// Fallback for non-array IN values
						params.push(value.clone());
						format!("{} {} (${})", clause.field, clause.operator.to_sql(), params.len())
					}
				} else {
					format!("{} {} ()", clause.field, clause.operator.to_sql())
				}
			}
			WhereOperator::Between => {
				if let Some(value) = &clause.value {
					if let Some(arr) = value.as_array() {
						if arr.len() >= 2 {
							params.push(arr[0].clone());
							let min_placeholder = format!("${}", params.len());
							params.push(arr[1].clone());
							let max_placeholder = format!("${}", params.len());
							return format!(
								"{} BETWEEN {} AND {}",
								clause.field, min_placeholder, max_placeholder
							);
						}
					}
				}
				// Fallback
				format!("{} BETWEEN $0 AND $0", clause.field)
			}
			_ => {
				if let Some(value) = &clause.value {
					params.push(value.clone());
					format!("{} {} ${}", clause.field, clause.operator.to_sql(), params.len())
				} else {
					format!("{} {}", clause.field, clause.operator.to_sql())
				}
			}
		}
	}
}

/// INSERT query builder
pub struct InsertBuilder {
	table: String,
	columns: Vec<String>,
	values: Vec<Vec<Value>>,
}

impl InsertBuilder {
	pub fn new(table: impl Into<String>) -> Self {
		Self {
			table: table.into(),
			columns: vec![],
			values: vec![],
		}
	}

	pub fn columns(mut self, columns: Vec<String>) -> Self {
		self.columns = columns;
		self
	}

	pub fn values(mut self, values: Vec<Value>) -> Self {
		self.values.push(values);
		self
	}

	pub fn multi_values(mut self, values: Vec<Vec<Value>>) -> Self {
		self.values = values;
		self
	}

	pub fn to_sql(&self) -> (String, Vec<Value>) {
		if self.columns.is_empty() || self.values.is_empty() {
			return ("".to_string(), vec![]);
		}

		let mut sql = format!("INSERT INTO {} ({})", self.table, self.columns.join(", "));
		let mut params = vec![];

		sql.push_str(" VALUES ");

		let value_groups: Vec<String> = self
			.values
			.iter()
			.map(|row| {
				let placeholders: Vec<String> = row
					.iter()
					.map(|val| {
						params.push(val.clone());
						format!("${}", params.len())
					})
					.collect();
				format!("({})", placeholders.join(", "))
			})
			.collect();

		sql.push_str(&value_groups.join(", "));

		(sql, params)
	}
}

/// UPDATE query builder
pub struct UpdateBuilder {
	table: String,
	set_values: Vec<(String, Value)>,
	where_clauses: Vec<WhereCondition>,
}

impl UpdateBuilder {
	pub fn new(table: impl Into<String>) -> Self {
		Self {
			table: table.into(),
			set_values: vec![],
			where_clauses: vec![],
		}
	}

	pub fn set(mut self, field: impl Into<String>, value: Value) -> Self {
		self.set_values.push((field.into(), value));
		self
	}

	pub fn where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
		self.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Eq,
			value: Some(value),
		}));
		self
	}

	pub fn where_clause(mut self, clause: WhereClause) -> Self {
		self.where_clauses.push(WhereCondition::Simple(clause));
		self
	}

	pub fn to_sql(&self) -> (String, Vec<Value>) {
		if self.set_values.is_empty() {
			return ("".to_string(), vec![]);
		}

		let mut params = vec![];
		let mut sql = format!("UPDATE {}", self.table);

		// SET clause
		sql.push_str(" SET ");
		let set_parts: Vec<String> = self
			.set_values
			.iter()
			.map(|(field, value)| {
				params.push(value.clone());
				format!("{} = ${}", field, params.len())
			})
			.collect();
		sql.push_str(&set_parts.join(", "));

		// WHERE clause
		if !self.where_clauses.is_empty() {
			sql.push_str(" WHERE ");
			let where_sql = build_where_conditions_helper(&self.where_clauses, &mut params);
			sql.push_str(&where_sql);
		}

		(sql, params)
	}
}

/// DELETE query builder
pub struct DeleteBuilder {
	table: String,
	where_clauses: Vec<WhereCondition>,
}

impl DeleteBuilder {
	pub fn new(table: impl Into<String>) -> Self {
		Self {
			table: table.into(),
			where_clauses: vec![],
		}
	}

	pub fn where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
		self.where_clauses.push(WhereCondition::Simple(WhereClause {
			field: field.into(),
			operator: WhereOperator::Eq,
			value: Some(value),
		}));
		self
	}

	pub fn where_clause(mut self, clause: WhereClause) -> Self {
		self.where_clauses.push(WhereCondition::Simple(clause));
		self
	}

	pub fn to_sql(&self) -> (String, Vec<Value>) {
		let mut params = vec![];
		let mut sql = format!("DELETE FROM {}", self.table);

		// WHERE clause
		if !self.where_clauses.is_empty() {
			sql.push_str(" WHERE ");
			let where_sql = build_where_conditions_helper(&self.where_clauses, &mut params);
			sql.push_str(&where_sql);
		}

		(sql, params)
	}
}

// Helper function for WHERE clause building (used by UPDATE and DELETE)
fn build_where_conditions_helper(conditions: &[WhereCondition], params: &mut Vec<Value>) -> String {
	let mut parts = vec![];

	for (i, condition) in conditions.iter().enumerate() {
		match condition {
			WhereCondition::Simple(clause) => {
				if i > 0 {
					parts.push(" AND ".to_string());
				}
				parts.push(build_where_clause_helper(clause, params));
			}
			WhereCondition::And(sub_conditions) => {
				if i > 0 {
					parts.push(" AND ".to_string());
				}
				parts.push(format!("({})", build_where_conditions_helper(sub_conditions, params)));
			}
			WhereCondition::Or(sub_conditions) => {
				if i > 0 {
					parts.push(" AND ".to_string());
				}
				let or_parts: Vec<String> = sub_conditions
					.iter()
					.map(|cond| match cond {
						WhereCondition::Simple(clause) => build_where_clause_helper(clause, params),
						_ => build_where_conditions_helper(&[cond.clone()], params),
					})
					.collect();
				parts.push(format!("({})", or_parts.join(" OR ")));
			}
		}
	}

	parts.join("")
}

fn build_where_clause_helper(clause: &WhereClause, params: &mut Vec<Value>) -> String {
	match clause.operator {
		WhereOperator::IsNull => {
			format!("{} IS NULL", clause.field)
		}
		WhereOperator::IsNotNull => {
			format!("{} IS NOT NULL", clause.field)
		}
		WhereOperator::In | WhereOperator::NotIn => {
			if let Some(value) = &clause.value {
				if let Some(arr) = value.as_array() {
					let placeholders: Vec<String> = arr
						.iter()
						.map(|v| {
							params.push(v.clone());
							format!("${}", params.len())
						})
						.collect();
					format!(
						"{} {} ({})",
						clause.field,
						clause.operator.to_sql(),
						placeholders.join(", ")
					)
				} else {
					params.push(value.clone());
					format!("{} {} (${})", clause.field, clause.operator.to_sql(), params.len())
				}
			} else {
				format!("{} {} ()", clause.field, clause.operator.to_sql())
			}
		}
		WhereOperator::Between => {
			if let Some(value) = &clause.value {
				if let Some(arr) = value.as_array() {
					if arr.len() >= 2 {
						params.push(arr[0].clone());
						let min_placeholder = format!("${}", params.len());
						params.push(arr[1].clone());
						let max_placeholder = format!("${}", params.len());
						return format!(
							"{} BETWEEN {} AND {}",
							clause.field, min_placeholder, max_placeholder
						);
					}
				}
			}
			format!("{} BETWEEN $0 AND $0", clause.field)
		}
		_ => {
			if let Some(value) = &clause.value {
				params.push(value.clone());
				format!("{} {} ${}", clause.field, clause.operator.to_sql(), params.len())
			} else {
				format!("{} {}", clause.field, clause.operator.to_sql())
			}
		}
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
		assert!(sql.contains("SELECT * FROM users"), "SQL: {}", sql);
		assert!(sql.contains("WHERE active = $1"), "SQL: {}", sql);
		assert!(sql.contains("LIMIT 10"), "SQL: {}", sql);
		assert_eq!(params.len(), 1);
		assert_eq!(params[0], serde_json::json!(true));
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
		assert!(sql.contains("SELECT id, title FROM posts"), "SQL: {}", sql);
		assert!(sql.contains("WHERE published = $1"), "SQL: {}", sql);
		assert!(sql.contains("ORDER BY created_at DESC"), "SQL: {}", sql);
		assert!(sql.contains("LIMIT 20"), "SQL: {}", sql);
		assert!(sql.contains("OFFSET 10"), "SQL: {}", sql);
	}

	#[test]
	fn test_join_query() {
		let builder = QueryBuilder::new("users")
			.select(vec!["users.id".to_string(), "posts.title".to_string()])
			.inner_join("posts", "posts.user_id = users.id")
			.where_eq("users.active", serde_json::json!(true))
			.limit(10);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("SELECT users.id, posts.title FROM users"), "SQL: {}", sql);
		assert!(sql.contains("INNER JOIN posts ON posts.user_id = users.id"), "SQL: {}", sql);
		assert!(sql.contains("WHERE users.active = $1"), "SQL: {}", sql);
		assert_eq!(params.len(), 1);
	}

	#[test]
	fn test_left_join() {
		let builder = QueryBuilder::new("users")
			.left_join("posts", "posts.user_id = users.id");

		let (sql, _) = builder.to_sql();
		assert!(sql.contains("LEFT JOIN posts ON posts.user_id = users.id"), "SQL: {}", sql);
	}

	#[test]
	fn test_where_operators() {
		// Greater than
		let (sql, params) = QueryBuilder::new("products")
			.where_gt("price", serde_json::json!(100))
			.to_sql();
		assert!(sql.contains("WHERE price > $1"), "SQL: {}", sql);
		assert_eq!(params[0], serde_json::json!(100));

		// Less than or equal
		let (sql, params) = QueryBuilder::new("products")
			.where_lte("stock", serde_json::json!(10))
			.to_sql();
		assert!(sql.contains("WHERE stock <= $1"), "SQL: {}", sql);

		// Like
		let (sql, params) = QueryBuilder::new("users")
			.where_like("name", serde_json::json!("%john%"))
			.to_sql();
		assert!(sql.contains("WHERE name LIKE $1"), "SQL: {}", sql);
		assert_eq!(params[0], serde_json::json!("%john%"));

		// Not equal
		let (sql, params) = QueryBuilder::new("users")
			.where_not_eq("status", serde_json::json!("deleted"))
			.to_sql();
		assert!(sql.contains("WHERE status != $1"), "SQL: {}", sql);
	}

	#[test]
	fn test_where_in() {
		let builder = QueryBuilder::new("users")
			.where_in("id", serde_json::json!([1, 2, 3, 4, 5]));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE id IN ($1, $2, $3, $4, $5)"), "SQL: {}", sql);
		assert_eq!(params.len(), 5);
		assert_eq!(params[0], serde_json::json!(1));
		assert_eq!(params[4], serde_json::json!(5));
	}

	#[test]
	fn test_where_null() {
		let builder = QueryBuilder::new("users")
			.where_null("deleted_at");

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE deleted_at IS NULL"), "SQL: {}", sql);
		assert_eq!(params.len(), 0);
	}

	#[test]
	fn test_where_not_null() {
		let builder = QueryBuilder::new("users")
			.where_not_null("email");

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE email IS NOT NULL"), "SQL: {}", sql);
		assert_eq!(params.len(), 0);
	}

	#[test]
	fn test_where_between() {
		let builder = QueryBuilder::new("products")
			.where_between("price", serde_json::json!(10), serde_json::json!(100));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE price BETWEEN $1 AND $2"), "SQL: {}", sql);
		assert_eq!(params.len(), 2);
		assert_eq!(params[0], serde_json::json!(10));
		assert_eq!(params[1], serde_json::json!(100));
	}

	#[test]
	fn test_multiple_where_clauses() {
		let builder = QueryBuilder::new("users")
			.where_eq("active", serde_json::json!(true))
			.where_gt("age", serde_json::json!(18))
			.where_like("email", serde_json::json!("%@gmail.com"));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE active = $1 AND age > $2 AND email LIKE $3"), "SQL: {}", sql);
		assert_eq!(params.len(), 3);
	}

	#[test]
	fn test_or_conditions() {
		let cond1 = WhereCondition::Simple(WhereClause {
			field: "status".to_string(),
			operator: WhereOperator::Eq,
			value: Some(serde_json::json!("active")),
		});
		let cond2 = WhereCondition::Simple(WhereClause {
			field: "status".to_string(),
			operator: WhereOperator::Eq,
			value: Some(serde_json::json!("pending")),
		});

		let builder = QueryBuilder::new("orders")
			.or(vec![cond1, cond2]);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE (status = $1 OR status = $2)"), "SQL: {}", sql);
		assert_eq!(params.len(), 2);
	}

	#[test]
	fn test_complex_mixed_conditions() {
		let or_cond1 = WhereCondition::Simple(WhereClause {
			field: "role".to_string(),
			operator: WhereOperator::Eq,
			value: Some(serde_json::json!("admin")),
		});
		let or_cond2 = WhereCondition::Simple(WhereClause {
			field: "role".to_string(),
			operator: WhereOperator::Eq,
			value: Some(serde_json::json!("moderator")),
		});

		let builder = QueryBuilder::new("users")
			.where_eq("active", serde_json::json!(true))
			.or(vec![or_cond1, or_cond2])
			.where_gt("posts_count", serde_json::json!(10));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("WHERE active = $1 AND (role = $2 OR role = $3) AND posts_count > $4"), "SQL: {}", sql);
		assert_eq!(params.len(), 4);
	}

	#[test]
	fn test_full_query_with_all_features() {
		let builder = QueryBuilder::new("users")
			.select(vec!["users.id".to_string(), "users.name".to_string(), "COUNT(posts.id) AS post_count".to_string()])
			.left_join("posts", "posts.user_id = users.id")
			.where_eq("users.active", serde_json::json!(true))
			.where_gte("users.created_at", serde_json::json!("2024-01-01"))
			.where_not_null("users.email")
			.order_by("users.name", OrderDirection::Asc)
			.limit(50)
			.offset(0);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("SELECT users.id, users.name, COUNT(posts.id) AS post_count FROM users"), "SQL: {}", sql);
		assert!(sql.contains("LEFT JOIN posts ON posts.user_id = users.id"), "SQL: {}", sql);
		assert!(sql.contains("WHERE users.active = $1 AND users.created_at >= $2 AND users.email IS NOT NULL"), "SQL: {}", sql);
		assert!(sql.contains("ORDER BY users.name ASC"), "SQL: {}", sql);
		assert!(sql.contains("LIMIT 50"), "SQL: {}", sql);
		assert!(sql.contains("OFFSET 0"), "SQL: {}", sql);
		assert_eq!(params.len(), 2); // active and created_at (email IS NOT NULL has no param)
	}

	// INSERT tests
	#[test]
	fn test_insert_single_row() {
		let builder = InsertBuilder::new("users")
			.columns(vec!["name".to_string(), "email".to_string(), "age".to_string()])
			.values(vec![
				serde_json::json!("Alice"),
				serde_json::json!("alice@example.com"),
				serde_json::json!(30),
			]);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("INSERT INTO users (name, email, age)"), "SQL: {}", sql);
		assert!(sql.contains("VALUES ($1, $2, $3)"), "SQL: {}", sql);
		assert_eq!(params.len(), 3);
		assert_eq!(params[0], serde_json::json!("Alice"));
		assert_eq!(params[1], serde_json::json!("alice@example.com"));
		assert_eq!(params[2], serde_json::json!(30));
	}

	#[test]
	fn test_insert_multiple_rows() {
		let builder = InsertBuilder::new("users")
			.columns(vec!["name".to_string(), "email".to_string()])
			.multi_values(vec![
				vec![serde_json::json!("Alice"), serde_json::json!("alice@example.com")],
				vec![serde_json::json!("Bob"), serde_json::json!("bob@example.com")],
				vec![serde_json::json!("Charlie"), serde_json::json!("charlie@example.com")],
			]);

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("INSERT INTO users (name, email)"), "SQL: {}", sql);
		assert!(sql.contains("VALUES ($1, $2), ($3, $4), ($5, $6)"), "SQL: {}", sql);
		assert_eq!(params.len(), 6);
		assert_eq!(params[0], serde_json::json!("Alice"));
		assert_eq!(params[2], serde_json::json!("Bob"));
		assert_eq!(params[4], serde_json::json!("Charlie"));
	}

	#[test]
	fn test_insert_empty() {
		let builder = InsertBuilder::new("users");
		let (sql, params) = builder.to_sql();
		assert_eq!(sql, "");
		assert_eq!(params.len(), 0);
	}

	// UPDATE tests
	#[test]
	fn test_update_single_field() {
		let builder = UpdateBuilder::new("users")
			.set("name", serde_json::json!("Alice Updated"))
			.where_eq("id", serde_json::json!(123));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("UPDATE users"), "SQL: {}", sql);
		assert!(sql.contains("SET name = $1"), "SQL: {}", sql);
		assert!(sql.contains("WHERE id = $2"), "SQL: {}", sql);
		assert_eq!(params.len(), 2);
		assert_eq!(params[0], serde_json::json!("Alice Updated"));
		assert_eq!(params[1], serde_json::json!(123));
	}

	#[test]
	fn test_update_multiple_fields() {
		let builder = UpdateBuilder::new("users")
			.set("name", serde_json::json!("Alice"))
			.set("email", serde_json::json!("newemail@example.com"))
			.set("age", serde_json::json!(31))
			.where_eq("id", serde_json::json!(123));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("UPDATE users"), "SQL: {}", sql);
		assert!(sql.contains("SET name = $1, email = $2, age = $3"), "SQL: {}", sql);
		assert!(sql.contains("WHERE id = $4"), "SQL: {}", sql);
		assert_eq!(params.len(), 4);
	}

	#[test]
	fn test_update_without_where() {
		let builder = UpdateBuilder::new("users")
			.set("active", serde_json::json!(false));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("UPDATE users SET active = $1"), "SQL: {}", sql);
		assert!(!sql.contains("WHERE"), "SQL: {}", sql);
		assert_eq!(params.len(), 1);
	}

	#[test]
	fn test_update_empty() {
		let builder = UpdateBuilder::new("users");
		let (sql, params) = builder.to_sql();
		assert_eq!(sql, "");
		assert_eq!(params.len(), 0);
	}

	// DELETE tests
	#[test]
	fn test_delete_with_where() {
		let builder = DeleteBuilder::new("users")
			.where_eq("id", serde_json::json!(123));

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("DELETE FROM users"), "SQL: {}", sql);
		assert!(sql.contains("WHERE id = $1"), "SQL: {}", sql);
		assert_eq!(params.len(), 1);
		assert_eq!(params[0], serde_json::json!(123));
	}

	#[test]
	fn test_delete_with_multiple_conditions() {
		let builder = DeleteBuilder::new("users")
			.where_clause(WhereClause {
				field: "active".to_string(),
				operator: WhereOperator::Eq,
				value: Some(serde_json::json!(false)),
			})
			.where_clause(WhereClause {
				field: "created_at".to_string(),
				operator: WhereOperator::Lt,
				value: Some(serde_json::json!("2020-01-01")),
			});

		let (sql, params) = builder.to_sql();
		assert!(sql.contains("DELETE FROM users"), "SQL: {}", sql);
		assert!(sql.contains("WHERE active = $1 AND created_at < $2"), "SQL: {}", sql);
		assert_eq!(params.len(), 2);
	}

	#[test]
	fn test_delete_without_where() {
		let builder = DeleteBuilder::new("logs");
		let (sql, params) = builder.to_sql();
		assert_eq!(sql, "DELETE FROM logs");
		assert_eq!(params.len(), 0);
	}
}
