# Clean Framework Test Suite

Comprehensive test suite for the Clean Framework, covering all plugins and Host Bridge functionality.

## Directory Structure

```
tests/framework/
├── README.md                           # This file
├── output/                             # Compiled WASM outputs
├── fixtures/                           # Test data and mocks
├── utils/                              # Test utilities
│   ├── assertions.cln                  # Assertion helpers
│   ├── bridge_mocks.cln                # Mock Host Bridge functions
│   └── http_helpers.cln                # HTTP testing utilities
│
├── unit/                               # Unit tests
│   ├── plugins/
│   │   ├── web/                        # frame.web tests
│   │   ├── data/                       # frame.data tests
│   │   ├── auth/                       # frame.auth tests
│   │   └── ui/                         # frame.ui tests
│   └── bridge/                         # Host Bridge tests
│
├── integration/                        # Integration tests
│   ├── web_auth/                       # web + auth integration
│   ├── web_data/                       # web + data integration
│   └── cross_plugin/                   # Multi-plugin tests
│
└── e2e/                                # End-to-end tests
```

## Running Tests

### Run All Tests

```bash
./scripts/run-framework-tests.sh
```

### Run Specific Categories

```bash
# Unit tests only
./scripts/run-framework-tests.sh unit

# Plugin tests only
./scripts/run-framework-tests.sh unit/plugins

# Specific plugin
./scripts/run-framework-tests.sh unit/plugins/web

# Integration tests
./scripts/run-framework-tests.sh integration

# E2E tests
./scripts/run-framework-tests.sh e2e
```

### Options

```bash
# Verbose output
./scripts/run-framework-tests.sh -v

# Dry run (show what would run)
./scripts/run-framework-tests.sh -d

# Compile only
./scripts/run-framework-tests.sh -c
```

## Writing Tests

### Test File Structure

Each test file follows this structure:

```clean
// tests/framework/unit/plugins/web/01_server_expansion.cln

// Import test utilities (when needed)
// import "../../../utils/assertions.cln"

// Test state
integer passed = 0
integer failed = 0

functions:
    // Test helper functions
    void runTest(string name, boolean result)
        if result
            printl("PASS: " + name)
            passed = passed + 1
        else
            printl("FAIL: " + name)
            failed = failed + 1

    // Assertion wrappers
    boolean assertEqual(string actual, string expected)
        return actual == expected

    boolean assertContains(string text, string pattern)
        return text.contains(pattern)

// Main test execution
start:
    printl("=== Test Suite: Server Expansion ===")
    printl("")

    // Test 1: Basic server block
    runTest("server block creates start function",
        assertEqual("expected", "expected"))

    // Test 2: Server with port
    runTest("server block accepts port config",
        assertContains("port: 8080", "port"))

    // Summary
    printl("")
    printl("Results: " + passed.toString() + " passed, " + failed.toString() + " failed")
```

### Assertion Functions

The `utils/assertions.cln` provides these assertions:

| Function | Description |
|----------|-------------|
| `assertEqual(actual, expected, message)` | Check string equality |
| `assertEqualInt(actual, expected, message)` | Check integer equality |
| `assertTrue(condition, message)` | Check boolean is true |
| `assertFalse(condition, message)` | Check boolean is false |
| `assertContains(haystack, needle, message)` | Check string contains |
| `assertNotContains(haystack, needle, message)` | Check string doesn't contain |
| `assertStartsWith(text, prefix, message)` | Check string starts with |
| `assertEndsWith(text, suffix, message)` | Check string ends with |
| `assertEmpty(text, message)` | Check string is empty |
| `assertNotEmpty(text, message)` | Check string is not empty |
| `assertGreaterThan(actual, expected, message)` | Check integer comparison |
| `assertLessThan(actual, expected, message)` | Check integer comparison |
| `assertOccurrences(text, pattern, count, message)` | Check pattern count |

### Mock Functions

The `utils/bridge_mocks.cln` provides mock implementations:

**Database Mocks:**
- `mockDbReset()` - Reset database state
- `mockDbAddRow(rowJson)` - Add a mock row
- `mockDbSetFail(errorMessage)` - Set failure mode
- `mockDbQuery(sql, params)` - Mock query
- `mockDbInsert(table, data)` - Mock insert
- `mockDbUpdate(table, id, data)` - Mock update
- `mockDbDelete(table, id)` - Mock delete

**Auth Mocks:**
- `mockAuthReset()` - Reset auth state
- `mockAuthSetUser(userId, role)` - Set authenticated user
- `mockAuthSetUnauthenticated()` - Clear authentication
- `mockAuthCreateToken(userId, role)` - Create mock token
- `mockAuthVerifyToken(token)` - Verify mock token
- `mockAuthHashPassword(password)` - Hash password
- `mockAuthVerifyPassword(password, hash)` - Verify password

**Environment Mocks:**
- `mockEnvReset()` - Reset environment
- `mockEnvSet(key, value)` - Set environment variable
- `mockEnvGet(key)` - Get environment variable
- `mockEnvHas(key)` - Check if variable exists

**Log Mocks:**
- `mockLogReset()` - Reset log state
- `mockLogInfo(message)` - Log info message
- `mockLogWarn(message)` - Log warning
- `mockLogError(message)` - Log error
- `mockLogGetMessages()` - Get all log messages
- `mockLogContains(substring)` - Check log contains
- `mockLogCount()` - Get log message count

### HTTP Testing

The `utils/http_helpers.cln` provides HTTP testing utilities:

```clean
// Set up a mock GET request
mockGet("/api/users")

// Set up a mock POST request with body
mockPost("/api/users", "{\"name\": \"John\"}")

// Set path parameters
mockParam("id", "123")

// Set query parameters
mockQuery("page", "1")

// Build JSON responses
string response = successResponse("{\"id\": 1}")
string error = errorResponse("NOT_FOUND", "User not found")

// Check responses
boolean isSuccess = isSuccessResponse(response)
boolean hasError = hasErrorCode(error, "NOT_FOUND")
```

## Test Categories

### Unit Tests (`unit/`)

Test individual components in isolation.

**Plugin Tests (`unit/plugins/`):**
- `web/` - Server blocks, endpoints, guards, validation
- `data/` - Models, queries, transactions
- `auth/` - JWT, sessions, protected blocks
- `ui/` - Components, hydration, HTML interpolation

**Bridge Tests (`unit/bridge/`):**
- HTTP bridge functions
- Database bridge functions
- Environment bridge functions
- Crypto bridge functions
- Time bridge functions
- Log bridge functions

### Integration Tests (`integration/`)

Test multiple components working together.

- `web_auth/` - Protected routes, JWT/session auth
- `web_data/` - CRUD endpoints, queries
- `cross_plugin/` - Full stack applications

### E2E Tests (`e2e/`)

Test complete application workflows.

- Todo app CRUD operations
- Authentication flow (register, login, refresh)
- Blog app with relationships
- API server full cycle

## Naming Conventions

### Test Files

- Use numbered prefixes: `01_`, `02_`, etc.
- Use descriptive names: `01_server_expansion.cln`
- Group related tests in directories

### Test Names

- Use descriptive test names
- Start with what is being tested
- Include expected behavior

Good examples:
- "GET route generates handler function"
- "POST endpoint validates required fields"
- "JWT token expires after configured time"

Bad examples:
- "test1"
- "it works"
- "check something"

## Test Output Format

Tests should output in this format:

```
=== Test Suite: Server Expansion ===

PASS: server block creates start function
PASS: server block accepts port config
FAIL: server block validates port range
  Expected: error message
  Actual:   no error

Results: 2 passed, 1 failed
```

The test runner looks for `PASS:` and `FAIL:` prefixes to determine results.

## Coverage Goals

| Category | Files | Target Coverage |
|----------|-------|-----------------|
| Plugin Unit Tests | 26 | 90%+ |
| Bridge Tests | 6 | 80%+ |
| Integration | 8 | 80%+ |
| E2E | 4 | Key scenarios |

## Adding New Tests

1. Choose the appropriate category (unit/integration/e2e)
2. Create a new `.cln` file with numbered prefix
3. Follow the test file structure above
4. Use assertion functions from utils
5. Run the test to verify it works
6. Update this README if adding new utilities

## Debugging Tests

### Verbose Mode

```bash
./scripts/run-framework-tests.sh -v unit/plugins/web
```

### Compile Only

Check if tests compile without running:

```bash
./scripts/run-framework-tests.sh -c
```

### Manual Execution

Compile and run a single test manually:

```bash
# Compile
~/.cleen/bin/cln compile tests/framework/unit/plugins/web/01_server_expansion.cln \
    -o tests/framework/output/test.wasm --plugins

# Run
wasmtime tests/framework/output/test.wasm
```

## CI Integration

The test suite can be integrated into CI/CD:

```yaml
# Example GitHub Actions step
- name: Run Framework Tests
  run: |
    ./scripts/run-framework-tests.sh
```

Exit codes:
- `0` - All tests passed
- `1` - One or more tests failed

## Contributing

When adding new framework features:

1. Write tests first (TDD approach)
2. Cover success and error cases
3. Include edge cases
4. Update documentation
5. Ensure all tests pass before PR
