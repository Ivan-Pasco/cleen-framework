/*!
 * Clean Frame Data Plugin - ORM Model DSL
 *
 * This plugin transforms the `data:` DSL block into standard Clean
 * Language code that defines models for the Frame Data ORM.
 *
 * ## DSL Syntax
 *
 * ```clean
 * data User
 *     integer id : pk, auto
 *     string  name
 *     string  email : unique
 *     integer age
 *     boolean active = true
 *     datetime createdAt : default=now
 * ```
 *
 * ## Expansion
 *
 * The above expands to:
 *
 * ```clean
 * class User
 *     integer? id
 *     string name
 *     string email
 *     integer age
 *     boolean active
 *     datetime createdAt
 *
 *     functions:
 *         void __init(integer? id, string name, string email, integer age, boolean active, datetime createdAt)
 *             this.id = id
 *             this.name = name
 *             this.email = email
 *             this.age = age
 *             this.active = active
 *             this.createdAt = createdAt
 *
 *         string __table_name()
 *             return "users"
 *
 *         string __primary_key()
 *             return "id"
 * ```
 */

use clean_language_compiler::ast::{
    Class, Constructor, Expression, Field, Function, FunctionModifier, FunctionSyntax, Parameter,
    SourceLocation, Statement, Type, Value, Visibility,
};
use clean_language_compiler::plugins::{FrameworkBlock, FrameworkPlugin, PluginError, PluginResult};

/// Field constraint enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FieldConstraint {
    PrimaryKey,
    Auto,
    Unique,
    Default(String),
}

/// Parsed field definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub type_name: String,
    pub name: String,
    pub constraints: Vec<FieldConstraint>,
    pub default_value: Option<String>,
    pub line: usize,
}

/// Parsed model definition
#[derive(Debug, Clone)]
pub struct ModelDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub line: usize,
}

/// Data plugin for Clean Frame
pub struct DataPlugin;

impl DataPlugin {
    pub fn new() -> Self {
        Self
    }

    /// Parse the data block content
    fn parse_models(
        &self,
        content: &str,
        base_location: &Option<SourceLocation>,
    ) -> PluginResult<Vec<ModelDef>> {
        let mut models = Vec::new();
        let mut current_model: Option<ModelDef> = None;

        // Type keywords that indicate a field definition
        const TYPE_KEYWORDS: &[&str] = &["integer", "string", "boolean", "number", "datetime"];

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            // Check if this line is a field definition (starts with a type keyword)
            let is_field = TYPE_KEYWORDS.iter().any(|&keyword| trimmed.starts_with(keyword));

            if is_field {
                // Field declaration
                if let Some(ref mut model) = current_model {
                    let field = self.parse_field_line(trimmed, line_num + 1, base_location)?;
                    model.fields.push(field);
                } else {
                    return Err(PluginError::ParseError {
                        message: format!("Field declaration without model: '{}'", trimmed),
                        line: line_num + 1,
                        column: 1,
                        location: base_location.clone(),
                    });
                }
            } else {
                // Model declaration
                if let Some(model) = current_model.take() {
                    models.push(model);
                }

                current_model = Some(ModelDef {
                    name: trimmed.to_string(),
                    fields: Vec::new(),
                    line: line_num + 1,
                });
            }
        }

        // Push the last model
        if let Some(model) = current_model {
            models.push(model);
        }

        Ok(models)
    }

    /// Parse a single field line
    fn parse_field_line(
        &self,
        line: &str,
        line_num: usize,
        base_location: &Option<SourceLocation>,
    ) -> PluginResult<FieldDef> {
        // Expected format: type name : constraints
        // Or: type name = default
        // Or: type name

        // Split by : to separate constraints
        let (field_part, constraints_part) = if let Some(colon_pos) = line.find(':') {
            (line[..colon_pos].trim(), Some(line[colon_pos + 1..].trim()))
        } else {
            (line.trim(), None)
        };

        // Split field_part by = to separate default value
        let (type_and_name, default_part) = if let Some(eq_pos) = field_part.find('=') {
            (
                field_part[..eq_pos].trim(),
                Some(field_part[eq_pos + 1..].trim()),
            )
        } else {
            (field_part, None)
        };

        // Parse type and name
        let parts: Vec<&str> = type_and_name.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(PluginError::ParseError {
                message: format!("Invalid field syntax: '{}'", line),
                line: line_num,
                column: 1,
                location: base_location.clone(),
            });
        }

        let type_name = parts[0].to_string();
        let name = parts[1].to_string();

        // Parse constraints
        let mut constraints = Vec::new();
        if let Some(constraint_str) = constraints_part {
            for constraint in constraint_str.split(',') {
                let trimmed = constraint.trim();
                if trimmed == "pk" || trimmed == "primary_key" {
                    constraints.push(FieldConstraint::PrimaryKey);
                } else if trimmed == "auto" {
                    constraints.push(FieldConstraint::Auto);
                } else if trimmed == "unique" {
                    constraints.push(FieldConstraint::Unique);
                } else if let Some(default_val) = trimmed.strip_prefix("default=") {
                    constraints.push(FieldConstraint::Default(default_val.to_string()));
                }
            }
        }

        let default_value = default_part.map(|s| s.to_string());

        Ok(FieldDef {
            type_name,
            name,
            constraints,
            default_value,
            line: line_num,
        })
    }

    /// Generate class statement for a model
    fn generate_class(
        &self,
        model: &ModelDef,
        location: &Option<SourceLocation>,
    ) -> Statement {
        // Generate fields
        let mut fields = Vec::new();
        for field in &model.fields {
            let is_nullable = field.constraints.contains(&FieldConstraint::PrimaryKey)
                && field.constraints.contains(&FieldConstraint::Auto);

            fields.push(Field {
                name: field.name.clone(),
                type_: self.parse_field_type(&field.type_name, is_nullable),
                visibility: Visibility::Public,
                is_static: false,
                default_value: field.default_value.as_ref().map(|v| Expression::Literal(Value::String(v.clone()))),
            });
        }

        // Generate constructor
        let constructor = Some(self.generate_constructor(model, location));

        // Generate methods
        let mut methods = Vec::new();
        methods.push(self.generate_table_name_function(model, location));
        methods.push(self.generate_primary_key_function(model, location));

        Statement::ClassDefinition {
            class: Class {
                name: model.name.clone(),
                type_parameters: Vec::new(),
                description: None,
                base_class: None,
                base_class_type_args: Vec::new(),
                fields,
                methods,
                constructor,
                location: location.clone(),
            },
            location: location.clone(),
        }
    }

    /// Parse field type from string
    fn parse_field_type(&self, type_str: &str, _nullable: bool) -> Type {
        // Note: Clean Language doesn't have explicit Optional types in AST
        // Nullability is handled at runtime/semantic level
        match type_str {
            "integer" => Type::Integer,
            "string" => Type::String,
            "boolean" => Type::Boolean,
            "number" => Type::Number,
            "datetime" => Type::Class {
                name: "DateTime".to_string(),
                type_args: Vec::new(),
            },
            _ => Type::Class {
                name: type_str.to_string(),
                type_args: Vec::new(),
            },
        }
    }

    /// Generate constructor
    fn generate_constructor(&self, model: &ModelDef, location: &Option<SourceLocation>) -> Constructor {
        let mut parameters = Vec::new();
        let mut statements = Vec::new();

        for field in &model.fields {
            let is_nullable = field.constraints.contains(&FieldConstraint::PrimaryKey)
                && field.constraints.contains(&FieldConstraint::Auto);

            parameters.push(Parameter {
                name: field.name.clone(),
                type_: self.parse_field_type(&field.type_name, is_nullable),
                default_value: field.default_value.as_ref().map(|v| Expression::Literal(Value::String(v.clone()))),
            });

            // this.fieldName = fieldName
            statements.push(Statement::Expression {
                expr: Expression::PropertyAssignment {
                    object: Box::new(Expression::Variable("this".to_string())),
                    property: field.name.clone(),
                    value: Box::new(Expression::Variable(field.name.clone())),
                    location: location.clone().unwrap_or_default(),
                },
                location: location.clone(),
            });
        }

        Constructor {
            parameters,
            body: statements,
            location: location.clone(),
        }
    }

    /// Generate __table_name method
    fn generate_table_name_function(&self, model: &ModelDef, location: &Option<SourceLocation>) -> Function {
        // Convert PascalCase to snake_case and pluralize
        let table_name = self.pluralize(&self.to_snake_case(&model.name));

        Function {
            name: "__table_name".to_string(),
            type_parameters: Vec::new(),
            type_constraints: Vec::new(),
            parameters: Vec::new(),
            return_type: Type::String,
            body: vec![Statement::Return {
                value: Some(Expression::Literal(Value::String(table_name))),
                location: location.clone(),
            }],
            description: None,
            syntax: FunctionSyntax::Simple,
            visibility: Visibility::Public,
            modifier: FunctionModifier::None,
            location: location.clone(),
        }
    }

    /// Generate __primary_key method
    fn generate_primary_key_function(&self, model: &ModelDef, location: &Option<SourceLocation>) -> Function {
        let primary_key = model
            .fields
            .iter()
            .find(|f| f.constraints.contains(&FieldConstraint::PrimaryKey))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "id".to_string());

        Function {
            name: "__primary_key".to_string(),
            type_parameters: Vec::new(),
            type_constraints: Vec::new(),
            parameters: Vec::new(),
            return_type: Type::String,
            body: vec![Statement::Return {
                value: Some(Expression::Literal(Value::String(primary_key))),
                location: location.clone(),
            }],
            description: None,
            syntax: FunctionSyntax::Simple,
            visibility: Visibility::Public,
            modifier: FunctionModifier::None,
            location: location.clone(),
        }
    }

    /// Convert PascalCase to snake_case
    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }
        result
    }

    /// Simple pluralization (add 's')
    fn pluralize(&self, s: &str) -> String {
        // Simple pluralization rules
        if s.ends_with('s') || s.ends_with("ch") || s.ends_with("sh") {
            format!("{}es", s)
        } else if s.ends_with('y') {
            format!("{}ies", &s[..s.len() - 1])
        } else {
            format!("{}s", s)
        }
    }
}

impl FrameworkPlugin for DataPlugin {
    fn name(&self) -> &'static str {
        "frame.data"
    }

    fn handles(&self) -> &'static [&'static str] {
        &["data"]
    }

    fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
        // Parse models from block content
        let models = self.parse_models(&block.content, &block.location)?;

        if models.is_empty() {
            return Err(PluginError::ValidationFailed {
                plugin_name: self.name().to_string(),
                message: "No models found in data: block".to_string(),
                location: block.location.clone(),
            });
        }

        // Generate class statements for each model
        let statements: Vec<Statement> = models
            .iter()
            .map(|model| self.generate_class(model, &block.location))
            .collect();

        Ok(statements)
    }

    fn validate(&self, block: &FrameworkBlock) -> PluginResult<()> {
        // Basic validation: block must not be empty
        if block.content.trim().is_empty() {
            return Err(PluginError::ValidationFailed {
                plugin_name: self.name().to_string(),
                message: "data: block cannot be empty".to_string(),
                location: block.location.clone(),
            });
        }

        Ok(())
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clean_language_compiler::plugins::FrameworkBlockBuilder;

    #[test]
    fn test_parse_simple_model() {
        let plugin = DataPlugin::new();

        // Note: Framework block parser strips indentation
        let content = "User\ninteger id : pk, auto\nstring name\nstring email";

        let models = plugin.parse_models(content, &None).unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "User");
        assert_eq!(models[0].fields.len(), 3);
        assert_eq!(models[0].fields[0].name, "id");
        assert_eq!(models[0].fields[0].type_name, "integer");
        assert!(models[0].fields[0]
            .constraints
            .contains(&FieldConstraint::PrimaryKey));
        assert!(models[0].fields[0]
            .constraints
            .contains(&FieldConstraint::Auto));
    }

    #[test]
    fn test_plugin_expand() {
        let plugin = DataPlugin::new();

        let block = FrameworkBlockBuilder::new("data")
            .content(
                r#"User
    integer id : pk, auto
    string name
    string email : unique"#,
            )
            .build();

        let result = plugin.expand(&block);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        // Verify it's a class definition
        if let Statement::ClassDefinition { class, .. } = &statements[0] {
            assert_eq!(class.name, "User");
            assert!(class.fields.len() >= 3); // At least 3 fields
            assert!(class.methods.len() >= 2); // At least 2 methods (__table_name, __primary_key)
        } else {
            panic!("Expected ClassDefinition");
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = DataPlugin::new();
        assert_eq!(plugin.name(), "frame.data");
        assert_eq!(plugin.handles(), &["data"]);
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_to_snake_case() {
        let plugin = DataPlugin::new();
        assert_eq!(plugin.to_snake_case("User"), "user");
        assert_eq!(plugin.to_snake_case("UserProfile"), "user_profile");
        assert_eq!(plugin.to_snake_case("BlogPost"), "blog_post");
    }

    #[test]
    fn test_pluralize() {
        let plugin = DataPlugin::new();
        assert_eq!(plugin.pluralize("user"), "users");
        assert_eq!(plugin.pluralize("category"), "categories");
        assert_eq!(plugin.pluralize("post"), "posts");
        assert_eq!(plugin.pluralize("class"), "classes");
    }

    #[test]
    fn test_parse_multiple_models() {
        let plugin = DataPlugin::new();

        // Note: Framework block parser strips indentation
        let content = "User\ninteger id : pk, auto\nstring name\nPost\ninteger id : pk, auto\nstring title";

        let models = plugin.parse_models(content, &None).unwrap();

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "User");
        assert_eq!(models[0].fields.len(), 2);
        assert_eq!(models[1].name, "Post");
        assert_eq!(models[1].fields.len(), 2);
    }
}
