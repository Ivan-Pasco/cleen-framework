# Contributing to Frame Framework

Thank you for your interest in contributing to Frame! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing](#testing)
7. [Pull Request Process](#pull-request-process)
8. [Community Guidelines](#community-guidelines)

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ (for Frame compiler)
- **Node.js** 18+ (for runtime adapters)
- **Git** for version control
- **PostgreSQL** or **SQLite** (for testing database features)

### Installing Dependencies

```bash
# Clone the repository
git clone https://github.com/clean-lang/frame
cd frame

# Install Rust dependencies
cargo build

# Install Node.js dependencies
npm install

# Run tests to verify setup
cargo test
npm test
```

---

## Development Setup

### Building from Source

```bash
# Build the compiler
cargo build --release

# Build CLI
cd cli
cargo build --release

# Install locally for testing
cargo install --path .
```

### Running Tests

```bash
# Run all Rust tests
cargo test

# Run specific test suite
cargo test --test integration

# Run with verbose output
cargo test -- --nocapture

# Run Node.js tests
npm test

# Run end-to-end tests
npm run test:e2e
```

### Development Server

```bash
# Start development environment
frame serve --verbose

# Watch mode for compiler changes
cargo watch -x 'build'
```

---

## Project Structure

```
frame/
├── compiler/
│   ├── src/
│   │   ├── parser/       # Pest-based parser
│   │   ├── semantic/     # Type checking, analysis
│   │   ├── codegen/      # WASM code generation
│   │   └── ast/          # Abstract syntax tree
│   └── tests/            # Compiler tests
├── runtime/
│   ├── node/             # Node.js host adapter
│   ├── rust/             # Rust host adapter
│   └── browser/          # Browser bridge
├── cli/
│   └── src/              # CLI implementation
├── stdlib/
│   └── src/              # Standard library (math, string, list, etc.)
├── examples/
│   ├── hello-world/      # Basic examples
│   ├── blog/             # Blog platform example
│   └── e-commerce/       # E-commerce example
├── docs/
│   └── specification/    # Detailed specifications
└── tests/
    ├── unit/             # Unit tests
    ├── integration/      # Integration tests
    └── e2e/              # End-to-end tests
```

---

## Development Workflow

### Branching Strategy

- **`main`** - Stable production branch
- **`develop`** - Integration branch for features
- **`feat/*`** - Feature branches
- **`fix/*`** - Bug fix branches
- **`docs/*`** - Documentation branches
- **`refactor/*`** - Refactoring branches

### Branch Naming

```bash
# Features
git checkout -b feat/add-graphql-support

# Bug fixes
git checkout -b fix/session-timeout-issue

# Documentation
git checkout -b docs/update-api-reference

# Refactoring
git checkout -b refactor/simplify-parser
```

### Commit Messages

Follow **Conventional Commits** format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Adding or updating tests
- `chore` - Maintenance tasks

**Examples**:

```bash
feat(orm): add many-to-many relationship support

Implement junction table pattern for many-to-many relationships.
Includes query builder extensions and migration generator updates.

Closes #123
```

```bash
fix(compiler): resolve type inference for nested generics

The type checker was failing to properly infer types when generic
types were nested more than two levels deep.

Fixes #456
```

---

## Coding Standards

### Clean Language Standards

**Indentation**: Use tabs only

```clean
// ✅ Correct
functions:
    integer add(integer a, integer b)
        return a + b

// ❌ Incorrect (spaces)
functions:
  integer add(integer a, integer b)
      return a + b
```

**Naming**:
```clean
// Classes and Components - PascalCase
class UserProfile
component BlogPost

// Functions and Variables - camelCase
integer userCount
getUserById()

// Constants - SCREAMING_SNAKE_CASE
constant:
    integer MAX_RETRIES = 3
```

**Type Annotations**: Always explicit

```clean
// ✅ Good
integer count = 0
list<string> names = []

// ❌ Avoid (implicit)
count = 0
names = []
```

### Rust Standards

Follow standard Rust conventions:

```rust
// Use rustfmt
cargo fmt

// Check with clippy
cargo clippy -- -D warnings

// Follow naming conventions
struct UserProfile { }       // PascalCase for types
fn get_user_count() -> i32   // snake_case for functions
const MAX_RETRIES: i32 = 3;  // SCREAMING_SNAKE_CASE for constants
```

### Documentation

**Rust**:
```rust
/// Parses a Clean Language source file and returns an AST.
///
/// # Arguments
///
/// * `source` - The source code as a string
/// * `file_path` - Path to the source file for error reporting
///
/// # Returns
///
/// Returns `Ok(Ast)` on success, or `Err(ParseError)` on failure.
///
/// # Examples
///
/// ```
/// let source = "start()\n    print(\"Hello\")";
/// let ast = parse(source, "hello.cln")?;
/// ```
pub fn parse(source: &str, file_path: &str) -> Result<Ast, ParseError> {
    // Implementation
}
```

**Clean Language**:
```clean
/// Calculates the total price including tax and shipping.
///
/// Parameters:
///   - subtotal: The pre-tax amount
///   - taxRate: Tax rate as a decimal (e.g., 0.08 for 8%)
///   - shippingCost: Flat shipping fee
///
/// Returns: The total amount to charge
functions:
    number calculateTotal(number subtotal, number taxRate, number shippingCost)
        number tax = subtotal * taxRate
        return subtotal + tax + shippingCost
```

---

## Testing

### Writing Tests

**Unit Tests (Rust)**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_declaration() {
        let source = "functions:\n    integer add(integer a, integer b)\n        return a + b";
        let ast = parse(source, "test.cln").unwrap();

        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.functions[0].name, "add");
    }

    #[test]
    fn test_type_checker_validates_assignments() {
        let ast = create_test_ast();
        let result = type_check(&ast);

        assert!(result.is_ok());
    }
}
```

**Integration Tests (Clean)**:
```clean
// File: tests/integration/user_api_test.cln

tests:
    "creates user successfully": do:
        response = http.post("/api/users", {
            name: "Alice",
            email: "alice@test.com"
        })
        response.status = 201
        response.data.name = "Alice"

    "rejects duplicate email": do:
        http.post("/api/users", {
            name: "Alice",
            email: "alice@test.com"
        })
        response = http.post("/api/users", {
            name: "Bob",
            email: "alice@test.com"
        })
        response.status = 400
```

### Running Specific Tests

```bash
# Run compiler tests
cargo test --package frame-compiler

# Run ORM tests
cargo test --package frame-data

# Run specific test
cargo test test_parse_function_declaration

# Run tests with logging
RUST_LOG=debug cargo test
```

### Test Coverage

```bash
# Generate coverage report
cargo tarpaulin --out Html

# View coverage
open tarpaulin-report.html
```

---

## Pull Request Process

### Before Submitting

1. **Update your branch**:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout your-branch
   git rebase develop
   ```

2. **Run tests**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   npm test
   ```

3. **Update documentation** if needed

4. **Add tests** for new functionality

5. **Update CHANGELOG.md**

### Submitting a Pull Request

1. **Push your branch**:
   ```bash
   git push origin your-branch
   ```

2. **Create PR** via GitHub interface

3. **Fill out PR template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Added tests
   - [ ] All tests pass
   - [ ] Manual testing performed

   ## Checklist
   - [ ] Code follows project style
   - [ ] Documentation updated
   - [ ] No breaking changes (or documented)
   - [ ] CHANGELOG.md updated
   ```

4. **Respond to review feedback**

5. **Squash commits** if requested:
   ```bash
   git rebase -i HEAD~3
   git push --force-with-lease
   ```

### PR Requirements

✅ **Must Have**:
- Passing tests
- Code review approval
- Documentation updates (if applicable)
- Changelog entry

❌ **Avoid**:
- Mixing unrelated changes
- Large PRs (>500 lines - split into smaller PRs)
- Unclear commit messages
- Failing tests

---

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment:

- **Be respectful** - Treat everyone with respect and kindness
- **Be collaborative** - Work together towards common goals
- **Be patient** - Remember that we all have different experience levels
- **Be constructive** - Provide helpful feedback
- **Be inclusive** - Welcome newcomers and diverse perspectives

### Communication

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and discussions
- **Discord** - Real-time chat and community support
- **Email** - Security issues: security@cleanframework.dev

### Reporting Issues

**Bug Report Template**:
```markdown
## Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Frame version: 1.0.0
- OS: macOS 13.0
- Node.js: 18.0.0
- Rust: 1.70.0

## Additional Context
Any other relevant information
```

**Feature Request Template**:
```markdown
## Problem
Description of the problem this feature would solve

## Proposed Solution
How you envision the feature working

## Alternatives Considered
Other approaches you've thought about

## Additional Context
Any other relevant information
```

---

## Development Tips

### Debugging the Compiler

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- compile input.cln

# Print AST
cargo run --bin frame -- debug --show-ast input.cln

# Print type information
cargo run --bin frame -- debug --show-types input.cln
```

### Testing WASM Output

```bash
# Compile to WASM
frame build input.cln

# Inspect WASM
wasm-objdump -d output.wasm

# Validate WASM
wasm-validate output.wasm
```

### Performance Profiling

```bash
# Profile compilation
cargo flamegraph --bin frame-compiler -- compile large-file.cln

# Profile runtime
cargo bench
```

---

## Getting Help

- **Documentation**: Check [docs/](./README.md)
- **Examples**: Browse [examples/](../examples/)
- **Issues**: Search existing issues
- **Discord**: Ask in community channels
- **Discussions**: GitHub Discussions for questions

---

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md
- Release notes
- Project documentation

Thank you for contributing to Frame!
