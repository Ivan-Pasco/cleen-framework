// Frame CLI library
// Exposes command modules for testing

pub mod commands;
pub mod compiler; // Legacy process-based compiler (kept for compatibility)
pub mod frame_compiler; // New library-based compiler with Frame plugins
pub mod manifest;
pub mod openapi; // OpenAPI 3.1 spec generation
pub mod sdk_typescript; // TypeScript SDK generation
