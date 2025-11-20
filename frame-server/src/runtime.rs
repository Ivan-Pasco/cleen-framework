use anyhow::Result;
use wasmtime::*;
use std::path::Path;

pub struct WasmRuntime {
	engine: Engine,
	module: Module,
}

impl WasmRuntime {
	pub fn new(wasm_path: impl AsRef<Path>) -> Result<Self> {
		let engine = Engine::default();
		let module = Module::from_file(&engine, wasm_path)?;

		Ok(Self { engine, module })
	}

	pub fn call_function(&self, name: &str, params: Vec<serde_json::Value>) -> Result<serde_json::Value> {
		// This would:
		// 1. Create a Store
		// 2. Instantiate the module
		// 3. Call the function by name
		// 4. Convert results back to JSON

		Ok(serde_json::json!(null))
	}
}
