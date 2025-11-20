use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
	pub name: String,
	pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
	pub name: String,
	pub field_type: FieldType,
	pub constraints: Vec<Constraint>,
	pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
	Integer,
	String { max_length: Option<usize> },
	Boolean,
	DateTime,
	Number,
	Json,
	Related { model: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constraint {
	PrimaryKey,
	Auto,
	Unique,
	NotNull,
	Min(i64),
	Max(i64),
	Email,
}

impl Schema {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			fields: vec![],
		}
	}

	pub fn add_field(&mut self, field: Field) {
		self.fields.push(field);
	}

	pub fn to_sql(&self, dialect: &str) -> String {
		match dialect {
			"postgres" => self.to_postgres_sql(),
			"sqlite" => self.to_sqlite_sql(),
			_ => panic!("Unsupported dialect: {}", dialect),
		}
	}

	fn to_postgres_sql(&self) -> String {
		let mut sql = format!("CREATE TABLE {} (\n", self.name);

		for (i, field) in self.fields.iter().enumerate() {
			sql.push_str(&format!("  {} {}", field.name, field.to_postgres_type()));

			if field.constraints.contains(&Constraint::NotNull) {
				sql.push_str(" NOT NULL");
			}

			if field.constraints.contains(&Constraint::Unique) {
				sql.push_str(" UNIQUE");
			}

			if let Some(default) = &field.default {
				sql.push_str(&format!(" DEFAULT {}", default));
			}

			if i < self.fields.len() - 1 {
				sql.push_str(",\n");
			}
		}

		sql.push_str("\n)");
		sql
	}

	fn to_sqlite_sql(&self) -> String {
		// Similar to PostgreSQL but with SQLite types
		self.to_postgres_sql()
	}
}

impl Field {
	fn to_postgres_type(&self) -> String {
		match &self.field_type {
			FieldType::Integer => {
				if self.constraints.contains(&Constraint::Auto) {
					"SERIAL".to_string()
				} else {
					"INTEGER".to_string()
				}
			}
			FieldType::String { max_length } => {
				if let Some(len) = max_length {
					format!("VARCHAR({})", len)
				} else {
					"TEXT".to_string()
				}
			}
			FieldType::Boolean => "BOOLEAN".to_string(),
			FieldType::DateTime => "TIMESTAMP".to_string(),
			FieldType::Number => "DECIMAL".to_string(),
			FieldType::Json => "JSONB".to_string(),
			FieldType::Related { model } => format!("INTEGER REFERENCES {}(id)", model),
		}
	}
}
