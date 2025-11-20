use crate::Widget;

pub struct Renderer;

impl Renderer {
	pub fn render_to_string(widget: &dyn Widget) -> String {
		widget.render()
	}

	pub fn render_to_html(widget: &dyn Widget, title: &str) -> String {
		format!(
			r#"<!DOCTYPE html>
<html>
<head>
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width, initial-scale=1">
	<title>{}</title>
	<link rel="stylesheet" href="/css/frame.css">
</head>
<body>
{}
<script src="/js/frame.js"></script>
</body>
</html>"#,
			title,
			widget.render()
		)
	}
}
