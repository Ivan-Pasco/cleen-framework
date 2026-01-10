# Compiler Plugin Runtime - Add Missing Math Functions

## Problem

The plugin runtime in `src/plugins/wasm_adapter.rs` is missing math functions that compiled WASM modules import by default. When a plugin uses string operations like `indexOf`, `substring`, or `length`, the compiler includes math stdlib imports, but the plugin runtime doesn't provide them.

**Error:**
```
Plugin 'frame.web' failed to expand 'endpoints:': Failed to instantiate plugin module: unknown import: `env::math_pow` has not been defined
```

## Missing Functions

The following math functions need to be added to `setup_linker()` in `src/plugins/wasm_adapter.rs`:

```
env.math_pow      (f64, f64) -> f64
env.math_sin      (f64) -> f64
env.math_cos      (f64) -> f64
env.math_tan      (f64) -> f64
env.math_asin     (f64) -> f64
env.math_acos     (f64) -> f64
env.math_atan     (f64) -> f64
env.math_atan2    (f64, f64) -> f64
env.math_sinh     (f64) -> f64
env.math_cosh     (f64) -> f64
env.math_tanh     (f64) -> f64
env.math_ln       (f64) -> f64
env.math_log10    (f64) -> f64
env.math_log2     (f64) -> f64
env.math_exp      (f64) -> f64
env.math_exp2     (f64) -> f64
```

## Implementation

Add to `setup_linker()` function in `src/plugins/wasm_adapter.rs`, after the existing env functions:

```rust
// =========================================
// MATH NAMESPACE - Math operations
// =========================================

// math_pow - Power function
linker.func_wrap(
    "env",
    "math_pow",
    |_: Caller<'_, PluginState>, base: f64, exp: f64| -> f64 {
        base.powf(exp)
    },
)?;

// math_sin - Sine
linker.func_wrap(
    "env",
    "math_sin",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.sin()
    },
)?;

// math_cos - Cosine (note: import name has dot)
linker.func_wrap(
    "env",
    "math.cos",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.cos()
    },
)?;

// math_tan - Tangent
linker.func_wrap(
    "env",
    "math_tan",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.tan()
    },
)?;

// math_asin - Arc sine (note: import name has dot)
linker.func_wrap(
    "env",
    "math.asin",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.asin()
    },
)?;

// math_acos - Arc cosine (note: import name has dot)
linker.func_wrap(
    "env",
    "math.acos",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.acos()
    },
)?;

// math_atan - Arc tangent
linker.func_wrap(
    "env",
    "math_atan",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.atan()
    },
)?;

// math_atan2 - Arc tangent of y/x (note: import name has dot)
linker.func_wrap(
    "env",
    "math.atan2",
    |_: Caller<'_, PluginState>, y: f64, x: f64| -> f64 {
        y.atan2(x)
    },
)?;

// math_sinh - Hyperbolic sine (note: import name has dot)
linker.func_wrap(
    "env",
    "math.sinh",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.sinh()
    },
)?;

// math_cosh - Hyperbolic cosine
linker.func_wrap(
    "env",
    "math_cosh",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.cosh()
    },
)?;

// math_tanh - Hyperbolic tangent
linker.func_wrap(
    "env",
    "math_tanh",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.tanh()
    },
)?;

// math_ln - Natural logarithm (note: import name has dot)
linker.func_wrap(
    "env",
    "math.ln",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.ln()
    },
)?;

// math_log10 - Base-10 logarithm
linker.func_wrap(
    "env",
    "math_log10",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.log10()
    },
)?;

// math_log2 - Base-2 logarithm (note: import name has dot)
linker.func_wrap(
    "env",
    "math.log2",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.log2()
    },
)?;

// math_exp - Exponential (e^x) (note: import name has dot)
linker.func_wrap(
    "env",
    "math.exp",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.exp()
    },
)?;

// math_exp2 - 2^x (note: import name has dot)
linker.func_wrap(
    "env",
    "math.exp2",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.exp2()
    },
)?;
```

## Additional Functions to Check

Also verify these common math/utility functions are present:

```rust
// math.floor
linker.func_wrap(
    "env",
    "math.floor",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.floor()
    },
)?;

// math.ceil
linker.func_wrap(
    "env",
    "math.ceil",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.ceil()
    },
)?;

// math.round
linker.func_wrap(
    "env",
    "math.round",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.round()
    },
)?;

// math.abs (for f64)
linker.func_wrap(
    "env",
    "math.abs",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.abs()
    },
)?;

// math.sqrt
linker.func_wrap(
    "env",
    "math.sqrt",
    |_: Caller<'_, PluginState>, x: f64| -> f64 {
        x.sqrt()
    },
)?;

// math.min
linker.func_wrap(
    "env",
    "math.min",
    |_: Caller<'_, PluginState>, a: f64, b: f64| -> f64 {
        a.min(b)
    },
)?;

// math.max
linker.func_wrap(
    "env",
    "math.max",
    |_: Caller<'_, PluginState>, a: f64, b: f64| -> f64 {
        a.max(b)
    },
)?;
```

## Testing

After implementing, test with:

```bash
cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-framework/plugins/frame.web
~/.cleen/bin/cln compile src/main.cln -o plugin.wasm
cp plugin.wasm ~/.cleen/plugins/frame.web/1.0.0/

cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-framework/examples/endpoints-test
~/.cleen/bin/cln compile main.cln -o test.wasm --plugins
```

If successful, the plugin should load and expand the `endpoints:` block without errors.

## Why This Matters

The Clean Language compiler includes these math imports in all WASM modules by default (likely for number-to-string conversions or other stdlib operations). The plugin runtime must provide matching host functions for plugins to work.

This is NOT code duplication - we're exposing Rust's standard library math functions as WASM host functions via wasmtime's linker.
