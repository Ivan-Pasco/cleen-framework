use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod component;
mod render;
mod tags;

pub use component::Component;
pub use render::Renderer;
pub use tags::*;

/// Widget trait for all UI components
/// Note: This trait is object-safe for dynamic dispatch
pub trait Widget {
	fn render(&self) -> String;
}

/// Main UI page structure
/// Note: Removed Vec<Box<dyn Widget>> because trait objects with Serialize/Deserialize
/// are problematic. Instead, components should be rendered to HTML strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
	pub title: String,
	// TODO: Redesign component tree structure
	// Reference: TASKS.md Phase 5 - Frame UI
	// Spec: documents/specification/05_frame_ui.md
	//
	// Options:
	// 1. Store pre-rendered HTML strings
	// 2. Use enum for known component types
	// 3. Use component IDs and lookup system
	// 4. Parse directly from Clean code at runtime
	//
	// For now, keep it simple with HTML content
	pub content: String,
}

impl Page {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			content: String::new(),
		}
	}

	pub fn with_content(title: impl Into<String>, content: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			content: content.into(),
		}
	}

	pub fn set_content(&mut self, content: impl Into<String>) {
		self.content = content.into();
	}
}

/// Section container
#[derive(Debug, Clone)]
pub struct Section {
	pub children: Vec<String>,
}

impl Widget for Section {
	fn render(&self) -> String {
		format!("<section>{}</section>", self.children.join(""))
	}
}

/// Card component
#[derive(Debug, Clone)]
pub struct Card {
	pub content: String,
}

impl Widget for Card {
	fn render(&self) -> String {
		format!("<div class=\"card\">{}</div>", self.content)
	}
}

/// Button component
#[derive(Debug, Clone)]
pub struct Button {
	pub label: String,
	pub kind: String,
	pub on_click: Option<String>,
}

impl Widget for Button {
	fn render(&self) -> String {
		let onclick = self.on_click.as_ref()
			.map(|h| format!(" onclick=\"{}\"", h))
			.unwrap_or_default();

		format!(
			"<button class=\"btn btn-{}\"{}>{}</button>",
			self.kind, onclick, self.label
		)
	}
}
