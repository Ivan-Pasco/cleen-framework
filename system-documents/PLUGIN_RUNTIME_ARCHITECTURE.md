# Plugin Runtime Architecture

## Current Situation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           COMPILATION TIME                                   │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    CLEAN LANGUAGE COMPILER                            │   │
│  │                                                                       │   │
│  │  ┌─────────────────┐      ┌─────────────────────────────────────┐    │   │
│  │  │                 │      │         PLUGIN EXECUTOR             │    │   │
│  │  │   PARSER        │      │                                     │    │   │
│  │  │   SEMANTIC      │      │  ┌───────────────────────────────┐  │    │   │
│  │  │   CODEGEN       │      │  │   WASM Runtime (wasmtime?)    │  │    │   │
│  │  │                 │      │  │                               │  │    │   │
│  │  │  These modules  │      │  │   Host Functions:             │  │    │   │
│  │  │  KNOW how to    │      │  │   - print ✓                   │  │    │   │
│  │  │  generate code  │      │  │   - printl ✓                  │  │    │   │
│  │  │  for ALL stdlib │      │  │   - mem_alloc ✓               │  │    │   │
│  │  │  functions      │      │  │   - string.split ✗ (broken)   │  │    │   │
│  │  │                 │      │  │   - string.concat ✗           │  │    │   │
│  │  │                 │      │  │   - string.trim ✗             │  │    │   │
│  │  │                 │      │  │   - list.length ✗             │  │    │   │
│  │  │                 │      │  │   - ... (many missing)        │  │    │   │
│  │  └─────────────────┘      │  └───────────────────────────────┘  │    │   │
│  │                           └─────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘


┌─────────────────────────────────────────────────────────────────────────────┐
│                              RUN TIME                                        │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                       CLEAN SERVER                                    │   │
│  │                                                                       │   │
│  │  ┌───────────────────────────────────────────────────────────────┐   │   │
│  │  │                    WASM Runtime                                │   │   │
│  │  │                                                                │   │   │
│  │  │   Host Functions (ALL WORKING):                                │   │   │
│  │  │   - print ✓                                                    │   │   │
│  │  │   - printl ✓                                                   │   │   │
│  │  │   - mem_alloc ✓                                                │   │   │
│  │  │   - string.split ✓                                             │   │   │
│  │  │   - string.concat ✓                                            │   │   │
│  │  │   - string.trim ✓                                              │   │   │
│  │  │   - list.length ✓                                              │   │   │
│  │  │   - _http_route ✓                                              │   │   │
│  │  │   - _req_param ✓                                               │   │   │
│  │  │   - ... (all implemented)                                      │   │   │
│  │  └───────────────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## The Question

The compiler's CODEGEN module already knows how to generate WASM code for `string.split`,
`string.concat`, etc. These are "host functions" - the WASM calls out to the host (Rust)
to execute them.

**Why can't the compiler reuse this knowledge to execute plugins?**

## The Answer

The compiler generates **WASM bytecode** that calls host functions, but it doesn't
contain the **Rust implementation** of those host functions.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   COMPILER CODEGEN                        WHAT'S ACTUALLY NEEDED            │
│   (what compiler has)                     (to run the WASM)                 │
│                                                                             │
│   Generates this WASM:                    Rust code that does the work:     │
│                                                                             │
│   ┌─────────────────────────┐             ┌─────────────────────────────┐   │
│   │                         │             │                             │   │
│   │  (import "env"          │             │  fn string_split(           │   │
│   │    "string.split"       │  ──────►    │      ptr: i32,              │   │
│   │    (func ...))          │   CALLS     │      delim: i32             │   │
│   │                         │             │  ) -> i32 {                 │   │
│   │  ;; Call string.split   │             │      // Actual Rust code    │   │
│   │  call $string.split     │             │      // that splits string  │   │
│   │                         │             │      // and returns list    │   │
│   └─────────────────────────┘             │  }                          │   │
│                                           │                             │   │
│   Compiler knows the                      └─────────────────────────────┘   │
│   SIGNATURE and how to                                                      │
│   CALL it, but not how                    This Rust implementation          │
│   to IMPLEMENT it                         exists in clean-server,           │
│                                           NOT in the compiler               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Solution Options

### Option A: Add Host Function Implementations to Compiler

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       CLEAN LANGUAGE COMPILER                                │
│                                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────────┐    │
│  │    PARSER      │  │    CODEGEN     │  │   HOST FUNCTIONS (NEW)     │    │
│  │    SEMANTIC    │  │                │  │                            │    │
│  │                │  │  Generates     │  │   fn string_split() {...}  │    │
│  │                │  │  WASM code     │  │   fn string_concat() {...} │    │
│  │                │  │  that calls    │  │   fn list_length() {...}   │    │
│  │                │  │  these ────────┼──►   fn mem_alloc() {...}     │    │
│  │                │  │  functions     │  │   ...                      │    │
│  └────────────────┘  └────────────────┘  └────────────────────────────┘    │
│                                                     │                       │
│                                                     │                       │
│  ┌──────────────────────────────────────────────────┼──────────────────┐   │
│  │              PLUGIN EXECUTOR                     │                   │   │
│  │                                                  ▼                   │   │
│  │   ┌─────────────────────────────────────────────────────────────┐   │   │
│  │   │   WASM Runtime                                               │   │   │
│  │   │   - Loads plugin.wasm                                        │   │   │
│  │   │   - Binds HOST FUNCTIONS from above                          │   │   │
│  │   │   - Calls expand(block_name, attrs, body)                    │   │   │
│  │   │   - Returns expanded Clean code                              │   │   │
│  │   └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘


┌─────────────────────────────────────────────────────────────────────────────┐
│                           CLEAN SERVER                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   WASM Runtime                                                       │   │
│   │   - Uses COMPILER'S host functions (as dependency)                   │   │
│   │   - Adds server-specific: _http_route, _req_param, etc.              │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Pros:** Single source of truth, clean architecture
**Cons:** Requires refactoring clean-server to use compiler as dependency


### Option B: Compiler Shells Out to Clean Server

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       COMPILATION TIME                                       │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    COMPILER                                           │   │
│  │                                                                       │   │
│  │   1. Encounters: import frame.web                                     │   │
│  │   2. Encounters: endpoints: block                                     │   │
│  │   3. Shells out to clean-server:                                      │   │
│  │                                                                       │   │
│  │      clean-server --run-plugin frame.web \                            │   │
│  │                    --function expand \                                │   │
│  │                    --args "endpoints" "" "GET /: ..."                 │   │
│  │                                                                       │   │
│  │   4. Captures stdout (expanded Clean code)                            │   │
│  │   5. Continues compilation with expanded code                         │   │
│  │                                                                       │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Pros:** No code duplication, clean-server already works
**Cons:** External dependency, slower compilation, more complex


### Option C: Shared Runtime Crate (Extracted)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    clean-runtime (shared crate)                      │   │
│   │                                                                      │   │
│   │   - string_split(), string_concat(), string_trim()                   │   │
│   │   - list_length(), list_get(), list_push()                           │   │
│   │   - mem_alloc(), mem_retain(), mem_release()                         │   │
│   │   - print(), printl()                                                │   │
│   │   - int_to_string(), float_to_string()                               │   │
│   │                                                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                          ▲                    ▲                             │
│                          │                    │                             │
│            ┌─────────────┘                    └─────────────┐               │
│            │                                                │               │
│   ┌────────┴─────────┐                          ┌──────────┴──────────┐    │
│   │     COMPILER     │                          │    CLEAN SERVER     │    │
│   │                  │                          │                     │    │
│   │  Plugin Executor │                          │  + _http_route      │    │
│   │  uses shared     │                          │  + _req_param       │    │
│   │  runtime         │                          │  + _req_body        │    │
│   │                  │                          │  + ... (HTTP only)  │    │
│   └──────────────────┘                          └─────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Pros:** Clean separation, reusable
**Cons:** Another crate to maintain


## Recommendation

**Option A** is the cleanest - the compiler should own the host function implementations
since it's the source of truth for the language. The plugin executor and clean-server
both need the same core functions.

The compiler already has a `runtime/` or `stdlib/` module that knows about these
functions. It just needs actual Rust implementations that can be bound to a WASM runtime.

Clean-server would then:
1. Depend on the compiler crate
2. Import the host function implementations
3. Add server-specific functions (_http_route, etc.)
