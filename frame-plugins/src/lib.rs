use anyhow::Result;
use std::collections::HashMap;

/// Plugin system for Frame
pub struct PluginManager {
	plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
	pub fn new() -> Self {
		Self {
			plugins: HashMap::new(),
		}
	}

	pub fn register(&mut self, name: impl Into<String>, plugin: Box<dyn Plugin>) {
		self.plugins.insert(name.into(), plugin);
	}

	pub fn get(&self, name: &str) -> Option<&Box<dyn Plugin>> {
		self.plugins.get(name)
	}

	pub fn init_all(&mut self) -> Result<()> {
		for (name, plugin) in &mut self.plugins {
			plugin.init()?;
		}
		Ok(())
	}
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
	fn name(&self) -> &str;
	fn version(&self) -> &str;
	fn init(&mut self) -> Result<()>;
	fn shutdown(&mut self) -> Result<()>;
}

/// Example plugin implementation
pub struct ExamplePlugin {
	initialized: bool,
}

impl Plugin for ExamplePlugin {
	fn name(&self) -> &str {
		"example"
	}

	fn version(&self) -> &str {
		"0.1.0"
	}

	fn init(&mut self) -> Result<()> {
		self.initialized = true;
		Ok(())
	}

	fn shutdown(&mut self) -> Result<()> {
		self.initialized = false;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_plugin_manager() {
		let mut manager = PluginManager::new();
		assert_eq!(manager.plugins.len(), 0);
	}
}
