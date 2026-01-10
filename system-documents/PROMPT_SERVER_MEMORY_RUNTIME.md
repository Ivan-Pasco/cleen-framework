# Clean Server Fix: Add Memory Runtime Functions

## Issue Summary

The Clean Language compiler (v0.20.3+) generates WASM modules that import memory runtime functions from the `memory_runtime` namespace. The clean-server (v1.5.0) does not provide these functions, causing instantiation failures.

## Error Message

```
WASM error: Failed to instantiate WASM module: unknown import: `memory_runtime::mem_scope_push` has not been defined
```

## Required Memory Runtime Functions

The compiler generates imports for these functions:

```
memory_runtime.mem_alloc      - Allocate memory, returns pointer
memory_runtime.mem_retain     - Increment reference count
memory_runtime.mem_release    - Decrement reference count (free when 0)
memory_runtime.mem_scope_push - Push a new memory scope
memory_runtime.mem_scope_pop  - Pop memory scope (release scoped allocations)
```

### Function Signatures (from WASM)

```
func[7]  sig=4 <mem_alloc>      <- memory_runtime.mem_alloc
func[8]  sig=5 <mem_retain>     <- memory_runtime.mem_retain
func[9]  sig=5 <mem_release>    <- memory_runtime.mem_release
func[10] sig=6 <mem_scope_push> <- memory_runtime.mem_scope_push
func[11] sig=6 <mem_scope_pop>  <- memory_runtime.mem_scope_pop
```

Where the signatures are:
- `sig=4`: `(i32, i32) -> i32` - mem_alloc(size, alignment) -> ptr
- `sig=5`: `(i32) -> nil` - mem_retain/release(ptr)
- `sig=6`: `() -> nil` - mem_scope_push/pop()

## Implementation Requirements

### 1. Add Memory Runtime Module

Create a memory runtime that the Host Bridge linker can provide to WASM modules:

```rust
// In host_bridge or linker module

fn register_memory_runtime(linker: &mut Linker<State>) -> Result<()> {
    // Memory allocator - allocates from WASM linear memory
    linker.func_wrap("memory_runtime", "mem_alloc", |mut caller: Caller<'_, State>, size: i32, align: i32| -> i32 {
        // Allocate from WASM memory
        // Return pointer to allocated block
        // Implementation can use a simple bump allocator or free list
    })?;

    // Reference counting
    linker.func_wrap("memory_runtime", "mem_retain", |mut caller: Caller<'_, State>, ptr: i32| {
        // Increment refcount for ptr
        // Can be no-op if using scope-based memory only
    })?;

    linker.func_wrap("memory_runtime", "mem_release", |mut caller: Caller<'_, State>, ptr: i32| {
        // Decrement refcount, free if zero
        // Can be no-op if using scope-based memory only
    })?;

    // Scope-based memory management
    linker.func_wrap("memory_runtime", "mem_scope_push", |mut caller: Caller<'_, State>| {
        // Push new allocation scope
        // Track current allocation pointer
    })?;

    linker.func_wrap("memory_runtime", "mem_scope_pop", |mut caller: Caller<'_, State>| {
        // Pop scope, reset allocation pointer to previous scope
        // Effectively frees all allocations in the popped scope
    })?;

    Ok(())
}
```

### 2. Memory Management Strategy Options

**Option A: Simple Bump Allocator with Scopes**
- Maintain a stack of "scope markers" (allocation pointer positions)
- `mem_alloc`: Bump pointer, return address
- `mem_scope_push`: Save current pointer to scope stack
- `mem_scope_pop`: Restore pointer from scope stack
- `mem_retain`/`mem_release`: No-op (rely on scopes)

**Option B: Reference Counting**
- Track allocations with reference counts
- `mem_retain`: Increment count
- `mem_release`: Decrement, free when zero
- Scopes track allocations made within them

**Recommended: Option A** - simpler and sufficient for request-scoped server operations.

### 3. Integration Point

In `src/linker.rs` or wherever the Host Bridge linker is configured:

```rust
pub fn create_linker(engine: &Engine) -> Result<Linker<State>> {
    let mut linker = Linker::new(engine);

    // Existing host functions...
    register_http_functions(&mut linker)?;
    register_db_functions(&mut linker)?;
    register_env_functions(&mut linker)?;

    // Add memory runtime
    register_memory_runtime(&mut linker)?;

    Ok(linker)
}
```

### 4. State Management

Add memory state to the host state struct:

```rust
pub struct MemoryState {
    // For bump allocator
    heap_base: i32,      // Start of allocatable memory
    heap_ptr: i32,       // Current allocation pointer
    scope_stack: Vec<i32>, // Stack of scope markers
}

impl MemoryState {
    pub fn new(heap_base: i32) -> Self {
        Self {
            heap_base,
            heap_ptr: heap_base,
            scope_stack: Vec::new(),
        }
    }

    pub fn alloc(&mut self, size: i32, align: i32) -> i32 {
        // Align the pointer
        let aligned = (self.heap_ptr + align - 1) & !(align - 1);
        let result = aligned;
        self.heap_ptr = aligned + size;
        result
    }

    pub fn push_scope(&mut self) {
        self.scope_stack.push(self.heap_ptr);
    }

    pub fn pop_scope(&mut self) {
        if let Some(marker) = self.scope_stack.pop() {
            self.heap_ptr = marker;
        }
    }
}
```

### 5. WASM Memory Initialization

Determine heap base from WASM memory exports or use a convention:

```rust
// After instantiation, find where data segment ends
// Set heap_base to that point + some padding
let memory = instance.get_memory(&mut store, "memory")?;
let heap_base = determine_heap_base(&instance, &mut store)?;
store.data_mut().memory = MemoryState::new(heap_base);
```

## Testing

After implementation, test with:

```bash
# Compile a simple app with plugins
~/.cleen/bin/cln compile app.cln -o app.wasm --plugins

# Run with server
clean-server app.wasm --port 3000

# Should start without "unknown import" errors
```

## Files to Modify

1. `src/linker.rs` (or equivalent) - Add memory runtime function registration
2. `src/state.rs` (or equivalent) - Add MemoryState to host state
3. `src/lib.rs` or `src/main.rs` - Initialize memory state after WASM load

## Priority

**HIGH** - This blocks all WASM modules compiled with the latest compiler from running on the server.
