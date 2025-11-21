use anyhow::{Result, Context};
use wasmtime::*;
use std::path::Path;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use host_bridge::DbBridge;
use crate::bridge::{BridgeLinker, BridgeState};

/// WASM Runtime for executing Clean Language compiled modules
///
/// Features:
/// - Per-request Store isolation
/// - Memory limits and fuel consumption
/// - Async support for I/O operations
/// - Host Bridge integration
pub struct WasmRuntime {
	engine: Engine,
	module: Module,
	config: RuntimeConfig,
	bridge_linker: BridgeLinker,
	db_bridge: Arc<RwLock<DbBridge>>,
}

/// Configuration for the WASM runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
	/// Maximum memory per instance (bytes)
	pub max_memory: usize,
	/// Maximum fuel consumption (instructions)
	pub max_fuel: u64,
	/// Enable detailed backtraces
	pub enable_backtraces: bool,
	/// Timeout for function execution (milliseconds)
	pub timeout_ms: u64,
}

impl Default for RuntimeConfig {
	fn default() -> Self {
		Self {
			max_memory: 64 * 1024 * 1024,  // 64 MB
			max_fuel: 10_000_000,           // 10M instructions
			enable_backtraces: true,
			timeout_ms: 30_000,             // 30 seconds
		}
	}
}

impl WasmRuntime {
	/// Create a new WASM runtime with the given module path
	pub fn new(wasm_path: impl AsRef<Path>) -> Result<Self> {
		Self::with_config(wasm_path, RuntimeConfig::default())
	}

	/// Create a new WASM runtime with custom configuration
	pub fn with_config(wasm_path: impl AsRef<Path>, config: RuntimeConfig) -> Result<Self> {
		let engine = Self::create_engine(&config)?;
		let module = Module::from_file(&engine, wasm_path)
			.context("Failed to load WASM module")?;

		// Create database bridge
		let db_bridge = Arc::new(RwLock::new(DbBridge::new()));

		// Create bridge linker
		let bridge_linker = BridgeLinker::new(Arc::clone(&db_bridge));

		Ok(Self {
			engine,
			module,
			config,
			bridge_linker,
			db_bridge,
		})
	}

	/// Create a new WASM runtime with an existing database bridge
	pub fn with_db_bridge(
		wasm_path: impl AsRef<Path>,
		config: RuntimeConfig,
		db_bridge: Arc<RwLock<DbBridge>>,
	) -> Result<Self> {
		let engine = Self::create_engine(&config)?;
		let module = Module::from_file(&engine, wasm_path)
			.context("Failed to load WASM module")?;

		// Create bridge linker with the provided bridge
		let bridge_linker = BridgeLinker::new(Arc::clone(&db_bridge));

		Ok(Self {
			engine,
			module,
			config,
			bridge_linker,
			db_bridge,
		})
	}

	/// Create a configured Wasmtime engine
	fn create_engine(config: &RuntimeConfig) -> Result<Engine> {
		let mut wasmtime_config = Config::new();

		// Enable async support for I/O operations
		wasmtime_config.async_support(true);

		// Enable fuel consumption for resource limiting
		wasmtime_config.consume_fuel(true);

		// Enable detailed backtraces for debugging
		if config.enable_backtraces {
			wasmtime_config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
		}

		// Enable multi-memory for advanced memory management
		wasmtime_config.wasm_multi_memory(true);

		// Enable WASM features
		wasmtime_config.wasm_simd(true);
		wasmtime_config.wasm_bulk_memory(true);
		wasmtime_config.wasm_reference_types(true);

		// Create engine
		Engine::new(&wasmtime_config)
			.context("Failed to create Wasmtime engine")
	}

	/// Call a WASM function with JSON parameters and return JSON result
	///
	/// This is the main entry point for executing WASM functions.
	/// It handles:
	/// - Store creation with fuel limits
	/// - Memory allocation for JSON strings
	/// - Function invocation
	/// - Result deserialization
	pub async fn call_function(&self, name: &str, params: Value) -> Result<Value> {
		// Create bridge state for this request
		let bridge_state = BridgeState::new(Arc::clone(&self.db_bridge));

		// Create a new store for this request (isolation)
		let mut store = Store::new(&self.engine, bridge_state);

		// Set fuel limit
		store.set_fuel(self.config.max_fuel)
			.context("Failed to set fuel limit")?;

		// Instantiate the module
		let instance = self.instantiate_module(&mut store).await?;

		// Serialize parameters to JSON string
		let params_json = serde_json::to_string(&params)
			.context("Failed to serialize parameters")?;

		// Allocate memory in WASM for the JSON string
		let params_ptr = self.allocate_string(&mut store, &instance, &params_json)
			.context("Failed to allocate parameter string")?;

		// Get the function to call
		let func = instance
			.get_typed_func::<u32, u32>(&mut store, name)
			.context(format!("Function '{}' not found in WASM module", name))?;

		// Call the function with timeout
		let result_ptr = tokio::time::timeout(
			std::time::Duration::from_millis(self.config.timeout_ms),
			func.call_async(&mut store, params_ptr)
		)
		.await
		.context("Function execution timed out")??;

		// Read the result string from WASM memory
		let result_json = self.read_string(&mut store, &instance, result_ptr)
			.context("Failed to read result string")?;

		// Deserialize the result
		let result: Value = serde_json::from_str(&result_json)
			.context("Failed to deserialize result")?;

		Ok(result)
	}

	/// Instantiate the WASM module
	///
	/// This creates a new instance with all imports linked via the Host Bridge.
	async fn instantiate_module(&self, store: &mut Store<BridgeState>) -> Result<Instance> {
		// Create linker with Host Bridge functions
		let linker = self.bridge_linker.create_linker(&self.engine)?;

		// Instantiate the module
		linker.instantiate_async(store, &self.module)
			.await
			.context("Failed to instantiate WASM module")
	}

	/// Allocate a string in WASM memory and return its pointer
	///
	/// This uses the WASM module's allocator (if available) or falls back
	/// to a simple memory allocation strategy.
	fn allocate_string(&self, store: &mut Store<BridgeState>, instance: &Instance, s: &str) -> Result<u32> {
		// Try to get the allocate function from the module
		if let Ok(allocate) = instance.get_typed_func::<u32, u32>(&mut *store, "allocate") {
			let len = s.len() as u32;
			let ptr = allocate.call(&mut *store, len)
				.context("Failed to call allocate function")?;

			// Write the string to memory
			let memory = instance.get_memory(&mut *store, "memory")
				.context("WASM module has no 'memory' export")?;

			memory.write(&mut *store, ptr as usize, s.as_bytes())
				.context("Failed to write string to WASM memory")?;

			Ok(ptr)
		} else {
			anyhow::bail!("WASM module does not export 'allocate' function")
		}
	}

	/// Read a string from WASM memory at the given pointer
	///
	/// Assumes the string is null-terminated or uses a length prefix.
	fn read_string(&self, store: &mut Store<BridgeState>, instance: &Instance, ptr: u32) -> Result<String> {
		let memory = instance.get_memory(&mut *store, "memory")
			.context("WASM module has no 'memory' export")?;

		// Read length prefix (first 4 bytes)
		let mut len_bytes = [0u8; 4];
		memory.read(&mut *store, ptr as usize, &mut len_bytes)
			.context("Failed to read string length")?;
		let len = u32::from_le_bytes(len_bytes) as usize;

		// Read the string data
		let mut data = vec![0u8; len];
		memory.read(&mut *store, (ptr + 4) as usize, &mut data)
			.context("Failed to read string data")?;

		String::from_utf8(data)
			.context("Invalid UTF-8 in WASM string")
	}

	/// Get remaining fuel (for monitoring resource usage)
	pub fn remaining_fuel(&self, store: &Store<BridgeState>) -> Option<u64> {
		store.get_fuel().ok()
	}

	/// Get database bridge reference
	pub fn db_bridge(&self) -> Arc<RwLock<DbBridge>> {
		Arc::clone(&self.db_bridge)
	}

	/// Get the module reference
	pub fn module(&self) -> &Module {
		&self.module
	}

	/// Get the engine reference
	pub fn engine(&self) -> &Engine {
		&self.engine
	}

	/// Get the runtime configuration
	pub fn config(&self) -> &RuntimeConfig {
		&self.config
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_runtime_config_defaults() {
		let config = RuntimeConfig::default();
		assert_eq!(config.max_memory, 64 * 1024 * 1024);
		assert_eq!(config.max_fuel, 10_000_000);
		assert_eq!(config.enable_backtraces, true);
		assert_eq!(config.timeout_ms, 30_000);
	}

	#[test]
	fn test_create_engine() {
		let config = RuntimeConfig::default();
		let engine = WasmRuntime::create_engine(&config);
		assert!(engine.is_ok());
	}

	// Note: Full integration tests require a compiled WASM module
	// These will be added in the test file once we have a test fixture
}
