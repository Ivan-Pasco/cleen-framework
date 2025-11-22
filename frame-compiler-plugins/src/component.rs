/*!
 * Clean Frame Component Plugin - UI Component DSL
 *
 * This plugin transforms the `component:` DSL block into standard Clean
 * Language code that defines UI components for the Frame UI system.
 *
 * ## DSL Syntax
 *
 * ```clean
 * component:
 *     UserCard(user: User)
 *         div class="card"
 *             h2 {user.name}
 *             p {user.email}
 *             button onClick="editUser" "Edit"
 * ```
 *
 * ## Expansion
 *
 * The above expands to:
 *
 * ```clean
 * class UserCard
 *     user: User
 *
 *     functions:
 *         void __init(User user)
 *             this.user = user
 *
 *         Widget render()
 *             return Widget.create("div")
 *                 .attr("class", "card")
 *                 .child(Widget.create("h2").text(user.name))
 *                 .child(Widget.create("p").text(user.email))
 *                 .child(Widget.create("button")
 *                     .attr("onClick", "editUser")
 *                     .text("Edit"))
 * ```
 */

use clean_language_compiler::ast::{
    Class, Constructor, Expression, Field, Function, FunctionModifier, FunctionSyntax, Parameter,
    SourceLocation, Statement, Type, Value, Visibility,
};
use clean_language_compiler::plugins::{FrameworkBlock, FrameworkPlugin, PluginError, PluginResult};

/// Parsed component parameter
#[derive(Debug, Clone)]
pub struct ComponentParam {
    pub name: String,
    pub type_name: String,
}

/// HTML element node
#[derive(Debug, Clone)]
pub enum HtmlNode {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Vec<HtmlNode>,
    },
    Text(String),
    Interpolation(String), // {expression}
}

/// Parsed component definition
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub name: String,
    pub params: Vec<ComponentParam>,
    pub template: Vec<HtmlNode>,
    pub line: usize,
}

/// Component plugin for Clean Frame
pub struct ComponentPlugin;

impl ComponentPlugin {
    pub fn new() -> Self {
        Self
    }

    /// Parse the component block content
    fn parse_components(
        &self,
        content: &str,
        base_location: &Option<SourceLocation>,
    ) -> PluginResult<Vec<ComponentDef>> {
        let mut components = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Check if this line is a component declaration (contains parentheses)
            if line.contains('(') && line.contains(')') {
                // Parse component declaration: ComponentName(param: Type, ...)
                if let Some((name, params)) = self.parse_component_declaration(line)? {
                    // Parse template (HTML nodes)
                    i += 1;
                    let (template, lines_consumed) = self.parse_template(&lines[i..], base_location)?;

                    components.push(ComponentDef {
                        name,
                        params,
                        template,
                        line: i,
                    });

                    i += lines_consumed;
                } else {
                    return Err(PluginError::ParseError {
                        message: format!("Invalid component declaration: '{}'", line),
                        line: i + 1,
                        column: 1,
                        location: base_location.clone(),
                    });
                }
            } else {
                // Skip lines that don't look like component declarations
                i += 1;
            }
        }

        Ok(components)
    }

    /// Parse component declaration line: ComponentName(param: Type, ...)
    fn parse_component_declaration(
        &self,
        line: &str,
    ) -> PluginResult<Option<(String, Vec<ComponentParam>)>> {
        // Find opening parenthesis
        if let Some(paren_pos) = line.find('(') {
            let name = line[..paren_pos].trim().to_string();

            // Find closing parenthesis
            if let Some(close_pos) = line.find(')') {
                let params_str = &line[paren_pos + 1..close_pos];

                // Parse parameters
                let params = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    params_str
                        .split(',')
                        .map(|param| {
                            let parts: Vec<&str> = param.trim().split(':').collect();
                            if parts.len() == 2 {
                                Ok(ComponentParam {
                                    name: parts[0].trim().to_string(),
                                    type_name: parts[1].trim().to_string(),
                                })
                            } else {
                                Err(PluginError::ParseError {
                                    message: format!("Invalid parameter syntax: '{}'", param),
                                    line: 0,
                                    column: 0,
                                    location: None,
                                })
                            }
                        })
                        .collect::<PluginResult<Vec<_>>>()?
                };

                Ok(Some((name, params)))
            } else {
                Err(PluginError::ParseError {
                    message: "Missing closing parenthesis in component declaration".to_string(),
                    line: 0,
                    column: 0,
                    location: None,
                })
            }
        } else {
            Ok(None)
        }
    }

    /// Parse HTML template
    fn parse_template(
        &self,
        lines: &[&str],
        _base_location: &Option<SourceLocation>,
    ) -> PluginResult<(Vec<HtmlNode>, usize)> {
        let mut nodes = Vec::new();
        let mut i = 0;

        // Parse template lines until we hit another component declaration
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Stop if we hit an empty line or another component declaration
            if trimmed.is_empty() {
                i += 1;
                break;
            }

            // Stop if this looks like another component declaration (has parentheses)
            if trimmed.contains('(') && trimmed.contains(')') {
                break;
            }

            // Parse HTML element or text
            if let Some(node) = self.parse_html_line(trimmed)? {
                nodes.push(node);
            }

            i += 1;
        }

        Ok((nodes, i))
    }

    /// Parse a single HTML line
    fn parse_html_line(&self, line: &str) -> PluginResult<Option<HtmlNode>> {
        // Check for interpolation: {expression}
        if line.starts_with('{') && line.ends_with('}') {
            return Ok(Some(HtmlNode::Interpolation(
                line[1..line.len() - 1].to_string(),
            )));
        }

        // Check for text content in quotes
        if (line.starts_with('"') && line.ends_with('"'))
            || (line.starts_with('\'') && line.ends_with('\''))
        {
            return Ok(Some(HtmlNode::Text(
                line[1..line.len() - 1].to_string(),
            )));
        }

        // Parse element: tag [attr="value"]* [text]
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let tag = parts[0].to_string();
        let mut attributes = Vec::new();
        let mut text_content = None;

        // Parse attributes and text
        let mut i = 1;
        while i < parts.len() {
            let part = parts[i];

            // Check for attribute (contains =)
            if let Some(eq_pos) = part.find('=') {
                let attr_name = part[..eq_pos].to_string();
                let mut attr_value = part[eq_pos + 1..].to_string();

                // Remove quotes from value
                if (attr_value.starts_with('"') && attr_value.ends_with('"'))
                    || (attr_value.starts_with('\'') && attr_value.ends_with('\''))
                {
                    attr_value = attr_value[1..attr_value.len() - 1].to_string();
                }

                attributes.push((attr_name, attr_value));
            } else if part.starts_with('{') && part.ends_with('}') {
                // Interpolation as text content
                text_content = Some(HtmlNode::Interpolation(
                    part[1..part.len() - 1].to_string(),
                ));
            } else {
                // Text content
                text_content = Some(HtmlNode::Text(part.to_string()));
            }

            i += 1;
        }

        let mut children = Vec::new();
        if let Some(text) = text_content {
            children.push(text);
        }

        Ok(Some(HtmlNode::Element {
            tag,
            attributes,
            children,
        }))
    }

    /// Generate class statement for a component
    fn generate_class(
        &self,
        component: &ComponentDef,
        location: &Option<SourceLocation>,
    ) -> Statement {
        // Generate fields for parameters
        let mut fields = Vec::new();
        for param in &component.params {
            fields.push(Field {
                name: param.name.clone(),
                type_: self.parse_type(&param.type_name),
                visibility: Visibility::Public,
                is_static: false,
                default_value: None,
            });
        }

        // Generate constructor
        let constructor = Some(self.generate_constructor(component, location));

        // Generate methods
        let mut methods = Vec::new();
        methods.push(self.generate_render_function(component, location));

        Statement::ClassDefinition {
            class: Class {
                name: component.name.clone(),
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

    /// Parse type from string
    fn parse_type(&self, type_str: &str) -> Type {
        match type_str {
            "integer" => Type::Integer,
            "string" => Type::String,
            "boolean" => Type::Boolean,
            "number" => Type::Number,
            _ => Type::Class {
                name: type_str.to_string(),
                type_args: Vec::new(),
            },
        }
    }

    /// Generate constructor
    fn generate_constructor(
        &self,
        component: &ComponentDef,
        location: &Option<SourceLocation>,
    ) -> Constructor {
        let mut parameters = Vec::new();
        let mut statements = Vec::new();

        for param in &component.params {
            parameters.push(Parameter {
                name: param.name.clone(),
                type_: self.parse_type(&param.type_name),
                default_value: None,
            });

            // this.paramName = paramName
            statements.push(Statement::Expression {
                expr: Expression::PropertyAssignment {
                    object: Box::new(Expression::Variable("this".to_string())),
                    property: param.name.clone(),
                    value: Box::new(Expression::Variable(param.name.clone())),
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

    /// Generate render() function
    fn generate_render_function(
        &self,
        component: &ComponentDef,
        location: &Option<SourceLocation>,
    ) -> Function {
        // For now, generate a simple string concatenation
        // In the future, this should generate Widget API calls

        let html_string = self.template_to_html(&component.template);

        Function {
            name: "render".to_string(),
            type_parameters: Vec::new(),
            type_constraints: Vec::new(),
            parameters: Vec::new(),
            return_type: Type::Class {
                name: "Widget".to_string(),
                type_args: Vec::new(),
            },
            body: vec![Statement::Return {
                value: Some(Expression::Literal(Value::String(html_string))),
                location: location.clone(),
            }],
            description: None,
            syntax: FunctionSyntax::Simple,
            visibility: Visibility::Public,
            modifier: FunctionModifier::None,
            location: location.clone(),
        }
    }

    /// Convert template to HTML string (simplified)
    fn template_to_html(&self, nodes: &[HtmlNode]) -> String {
        let mut html = String::new();

        for node in nodes {
            match node {
                HtmlNode::Element {
                    tag,
                    attributes,
                    children,
                } => {
                    html.push_str(&format!("<{}", tag));

                    // Add attributes
                    for (name, value) in attributes {
                        html.push_str(&format!(" {}=\"{}\"", name, value));
                    }

                    html.push('>');

                    // Add children
                    html.push_str(&self.template_to_html(children));

                    html.push_str(&format!("</{}>", tag));
                }
                HtmlNode::Text(text) => {
                    html.push_str(text);
                }
                HtmlNode::Interpolation(expr) => {
                    // For now, just placeholder
                    // In the future, this should be dynamic
                    html.push_str(&format!("{{{}}}", expr));
                }
            }
        }

        html
    }
}

impl FrameworkPlugin for ComponentPlugin {
    fn name(&self) -> &'static str {
        "frame.component"
    }

    fn handles(&self) -> &'static [&'static str] {
        &["component"]
    }

    fn expand(&self, block: &FrameworkBlock) -> PluginResult<Vec<Statement>> {
        // Parse components from block content
        let components = self.parse_components(&block.content, &block.location)?;

        if components.is_empty() {
            return Err(PluginError::ValidationFailed {
                plugin_name: self.name().to_string(),
                message: "No components found in component: block".to_string(),
                location: block.location.clone(),
            });
        }

        // Generate class statements for each component
        let statements: Vec<Statement> = components
            .iter()
            .map(|component| self.generate_class(component, &block.location))
            .collect();

        Ok(statements)
    }

    fn validate(&self, block: &FrameworkBlock) -> PluginResult<()> {
        // Basic validation: block must not be empty
        if block.content.trim().is_empty() {
            return Err(PluginError::ValidationFailed {
                plugin_name: self.name().to_string(),
                message: "component: block cannot be empty".to_string(),
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
    fn test_parse_component_declaration() {
        let plugin = ComponentPlugin::new();

        let (name, params) = plugin
            .parse_component_declaration("UserCard(user: User)")
            .unwrap()
            .unwrap();

        assert_eq!(name, "UserCard");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "user");
        assert_eq!(params[0].type_name, "User");
    }

    #[test]
    fn test_parse_multiple_params() {
        let plugin = ComponentPlugin::new();

        let (name, params) = plugin
            .parse_component_declaration("Button(text: string, onClick: string)")
            .unwrap()
            .unwrap();

        assert_eq!(name, "Button");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "text");
        assert_eq!(params[0].type_name, "string");
        assert_eq!(params[1].name, "onClick");
        assert_eq!(params[1].type_name, "string");
    }

    #[test]
    fn test_plugin_expand() {
        let plugin = ComponentPlugin::new();

        let block = FrameworkBlockBuilder::new("component")
            .content("UserCard(user: User)\n\tdiv class=\"card\"\n\t\th2 {user.name}")
            .build();

        let result = plugin.expand(&block);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        // Verify it's a class definition
        if let Statement::ClassDefinition { class, .. } = &statements[0] {
            assert_eq!(class.name, "UserCard");
            assert!(class.fields.len() >= 1); // At least user field
            assert!(class.methods.len() >= 1); // At least render method
        } else {
            panic!("Expected ClassDefinition");
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = ComponentPlugin::new();
        assert_eq!(plugin.name(), "frame.component");
        assert_eq!(plugin.handles(), &["component"]);
        assert!(!plugin.version().is_empty());
    }
}
