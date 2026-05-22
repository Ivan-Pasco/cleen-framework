#!/bin/bash

# Comprehensive HTTP-based Test Runner for Clean Framework
# This script starts Clean Server with each test application and makes real HTTP requests
# to validate responses against expected values.

# set -e  # Disabled to see full error messages

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPS_DIR="$SCRIPT_DIR/apps"
OUTPUT_DIR="$SCRIPT_DIR/output"
CLEAN_SERVER="${HOME}/Documents/Dev/Clean Language/clean-server/target/release/clean-server"
CLN_COMPILER="${HOME}/.cleen/bin/cln"
PORT=3333
TIMEOUT=5

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
    ((TOTAL_TESTS++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    if [ -n "$2" ]; then
        echo -e "${RED}       Expected:${NC} $2"
        echo -e "${RED}       Got:${NC} $3"
    fi
    ((FAILED_TESTS++))
    ((TOTAL_TESTS++))
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED_TESTS++))
}

log_section() {
    echo ""
    echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}  $1${NC}"
    echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
}

# Check prerequisites
check_prerequisites() {
    log_section "Checking Prerequisites"

    if [ ! -f "$CLEAN_SERVER" ]; then
        echo -e "${RED}ERROR: Clean Server not found at $CLEAN_SERVER${NC}"
        echo "Install it with: cleen server install"
        exit 1
    fi
    log_pass "Clean Server found"

    if [ ! -f "$CLN_COMPILER" ]; then
        echo -e "${RED}ERROR: CLN compiler not found at $CLN_COMPILER${NC}"
        exit 1
    fi
    log_pass "CLN compiler found"

    if ! command -v curl &> /dev/null; then
        echo -e "${RED}ERROR: curl is required but not installed${NC}"
        exit 1
    fi
    log_pass "curl available"

    mkdir -p "$OUTPUT_DIR"
    log_pass "Output directory ready"
}

# Compile a test application
compile_app() {
    local app_name=$1
    local source_file="$APPS_DIR/${app_name}.cln"
    local output_file="$OUTPUT_DIR/${app_name}.wasm"

    if [ ! -f "$source_file" ]; then
        log_fail "Source file not found: $source_file"
        return 1
    fi

    log_info "Compiling $app_name..."
    if $CLN_COMPILER compile "$source_file" -o "$output_file" --plugins 2>/dev/null; then
        log_pass "Compiled $app_name"
        return 0
    else
        log_fail "Compilation failed for $app_name"
        return 1
    fi
}

# Start the server with a WASM file
start_server() {
    local wasm_file=$1

    # Kill any existing server on the port
    pkill -f "clean-server.*$PORT" 2>/dev/null || true
    sleep 0.5

    log_info "Starting server with $wasm_file on port $PORT..."

    # Start server in background
    "$CLEAN_SERVER" "$wasm_file" --port $PORT > /tmp/clean-server.log 2>&1 &
    SERVER_PID=$!

    # Wait for server to be ready
    local attempts=0
    local max_attempts=20
    while [ $attempts -lt $max_attempts ]; do
        if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
            log_pass "Server started (PID: $SERVER_PID)"
            return 0
        fi
        sleep 0.25
        ((attempts++))
    done

    log_fail "Server failed to start within timeout"
    cat /tmp/clean-server.log
    return 1
}

# Stop the server
stop_server() {
    if [ -n "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        SERVER_PID=""
    fi
    pkill -f "clean-server.*$PORT" 2>/dev/null || true
}

# Make HTTP request and return response body
http_get() {
    local path=$1
    curl -s -m $TIMEOUT "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_post() {
    local path=$1
    local body=$2
    curl -s -m $TIMEOUT -X POST -d "$body" "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_put() {
    local path=$1
    local body=$2
    curl -s -m $TIMEOUT -X PUT -d "$body" "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_delete() {
    local path=$1
    curl -s -m $TIMEOUT -X DELETE "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_patch() {
    local path=$1
    local body=$2
    curl -s -m $TIMEOUT -X PATCH -d "$body" "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_get_with_header() {
    local path=$1
    local header=$2
    curl -s -m $TIMEOUT -H "$header" "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

http_post_with_header() {
    local path=$1
    local body=$2
    local header=$3
    curl -s -m $TIMEOUT -X POST -H "$header" -d "$body" "http://localhost:$PORT$path" 2>/dev/null || echo "REQUEST_FAILED"
}

# Assert response contains a substring
assert_contains() {
    local response=$1
    local expected=$2
    local test_name=$3

    if [[ "$response" == *"$expected"* ]]; then
        log_pass "$test_name"
        return 0
    else
        log_fail "$test_name" "contains '$expected'" "$response"
        return 1
    fi
}

# Assert response equals a value
assert_equals() {
    local response=$1
    local expected=$2
    local test_name=$3

    if [ "$response" == "$expected" ]; then
        log_pass "$test_name"
        return 0
    else
        log_fail "$test_name" "$expected" "$response"
        return 1
    fi
}

# Assert response does not contain a substring
assert_not_contains() {
    local response=$1
    local not_expected=$2
    local test_name=$3

    if [[ "$response" != *"$not_expected"* ]]; then
        log_pass "$test_name"
        return 0
    else
        log_fail "$test_name" "should not contain '$not_expected'" "$response"
        return 1
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# TODO APP TESTS
# ═══════════════════════════════════════════════════════════════════════════════
test_todo_app() {
    log_section "Testing Todo App"

    compile_app "todo_app" || return 1
    start_server "$OUTPUT_DIR/todo_app.wasm" || return 1

    local response

    # Test 1: Home page
    response=$(http_get "/")
    assert_contains "$response" "Welcome to Todo App" "GET / returns welcome message"

    # Test 2: Health check
    response=$(http_get "/health")
    assert_contains "$response" "status:healthy" "GET /health returns healthy status"

    # Test 3: List todos (empty)
    response=$(http_get "/api/todos")
    assert_equals "$response" "[]" "GET /api/todos returns empty array"

    # Test 4: Create todo
    response=$(http_post "/api/todos" "title=New Todo")
    assert_contains "$response" "id:1" "POST /api/todos returns created todo with id"
    assert_contains "$response" "title:New Todo" "POST /api/todos returns correct title"
    assert_contains "$response" "completed:false" "POST /api/todos sets completed to false"

    # Test 5: Get todo by ID
    response=$(http_get "/api/todos/1")
    assert_contains "$response" "id:1" "GET /api/todos/1 returns correct id"
    assert_contains "$response" "title:Todo Item" "GET /api/todos/1 returns title"

    # Test 6: Get todo with different ID
    response=$(http_get "/api/todos/42")
    assert_contains "$response" "id:42" "GET /api/todos/42 returns correct id parameter"

    # Test 7: Update todo
    response=$(http_put "/api/todos/1" "title=Updated")
    assert_contains "$response" "id:1" "PUT /api/todos/1 returns updated todo"
    assert_contains "$response" "Updated Todo" "PUT /api/todos/1 updates title"
    assert_contains "$response" "completed:true" "PUT /api/todos/1 sets completed"

    # Test 8: Delete todo
    response=$(http_delete "/api/todos/1")
    assert_contains "$response" "deleted:true" "DELETE /api/todos/1 confirms deletion"
    assert_contains "$response" "id:1" "DELETE /api/todos/1 returns deleted id"

    # Test 9: Toggle todo
    response=$(http_patch "/api/todos/5/toggle" "")
    assert_contains "$response" "id:5" "PATCH /api/todos/5/toggle returns correct id"
    assert_contains "$response" "completed:true" "PATCH /api/todos/5/toggle toggles completion"

    # Test 10: Filter todos
    response=$(http_get "/api/todos/filter?status=completed")
    assert_equals "$response" "[]" "GET /api/todos/filter returns filtered results"

    stop_server
}

# ═══════════════════════════════════════════════════════════════════════════════
# AUTH APP TESTS
# ═══════════════════════════════════════════════════════════════════════════════
test_auth_app() {
    log_section "Testing Auth App"

    compile_app "auth_app" || return 1
    start_server "$OUTPUT_DIR/auth_app.wasm" || return 1

    local response

    # Test 1: Home page
    response=$(http_get "/")
    assert_contains "$response" "Auth Service" "GET / returns auth service message"

    # Test 2: Health check
    response=$(http_get "/health")
    assert_contains "$response" "status:healthy" "GET /health returns healthy"
    assert_contains "$response" "service:auth" "GET /health identifies auth service"

    # Test 3: Register user
    response=$(http_post "/auth/register" "email=test@test.com&password=secret123")
    assert_contains "$response" "success:true" "POST /auth/register succeeds"
    assert_contains "$response" "userId:1" "POST /auth/register returns user ID"

    # Test 4: Login with valid credentials
    response=$(http_post "/auth/login" "email=test@test.com&password=secret123")
    assert_contains "$response" "success:true" "POST /auth/login with valid creds succeeds"
    assert_contains "$response" "token:jwt_token_123" "POST /auth/login returns JWT token"
    assert_contains "$response" "refreshToken:refresh_456" "POST /auth/login returns refresh token"

    # Test 5: Login with invalid credentials
    response=$(http_post "/auth/login" "email=wrong@test.com&password=wrong")
    assert_contains "$response" "error:AUTH_ERROR" "POST /auth/login with invalid creds fails"
    assert_contains "$response" "Invalid credentials" "POST /auth/login returns error message"

    # Test 6: Access protected route without auth
    response=$(http_get "/api/profile")
    assert_contains "$response" "error:UNAUTHORIZED" "GET /api/profile without auth fails"
    assert_contains "$response" "No token provided" "GET /api/profile returns auth error"

    # Test 7: Access protected route with auth
    response=$(http_get_with_header "/api/profile" "Authorization: Bearer jwt_token_123")
    assert_contains "$response" "id:1" "GET /api/profile with auth returns user id"
    assert_contains "$response" "email:test@test.com" "GET /api/profile returns email"
    assert_contains "$response" "name:Test User" "GET /api/profile returns name"

    # Test 8: Refresh token
    response=$(http_post "/auth/refresh" "refresh_token=refresh_456")
    assert_contains "$response" "success:true" "POST /auth/refresh succeeds"
    assert_contains "$response" "new_jwt_token" "POST /auth/refresh returns new token"

    # Test 9: Refresh with invalid token
    response=$(http_post "/auth/refresh" "invalid_data")
    assert_contains "$response" "error:AUTH_ERROR" "POST /auth/refresh with invalid token fails"

    # Test 10: Logout
    response=$(http_post "/auth/logout" "")
    assert_contains "$response" "success:true" "POST /auth/logout succeeds"
    assert_contains "$response" "Logged out" "POST /auth/logout confirms logout"

    # Test 11: Change password with valid data
    response=$(http_put "/auth/password" "oldPassword=secret123&newPassword=newsecret456")
    assert_contains "$response" "success:true" "PUT /auth/password succeeds"
    assert_contains "$response" "Password changed" "PUT /auth/password confirms change"

    # Test 12: Change password with invalid data
    response=$(http_put "/auth/password" "invalid_data")
    assert_contains "$response" "error:VALIDATION_ERROR" "PUT /auth/password with invalid data fails"

    # Test 13: Validation - invalid email
    response=$(http_post "/auth/validate" "invalid_email_format")
    assert_contains "$response" "error:VALIDATION_ERROR" "POST /auth/validate catches invalid email"

    # Test 14: Validation - short password
    response=$(http_post "/auth/validate" "short_password")
    assert_contains "$response" "error:VALIDATION_ERROR" "POST /auth/validate catches short password"

    stop_server
}

# ═══════════════════════════════════════════════════════════════════════════════
# BLOG APP TESTS
# ═══════════════════════════════════════════════════════════════════════════════
test_blog_app() {
    log_section "Testing Blog App"

    compile_app "blog_app" || return 1
    start_server "$OUTPUT_DIR/blog_app.wasm" || return 1

    local response

    # Test 1: Home page
    response=$(http_get "/")
    assert_contains "$response" "Blog Application" "GET / returns blog app message"

    # Test 2: Health check
    response=$(http_get "/health")
    assert_contains "$response" "status:healthy" "GET /health returns healthy"
    assert_contains "$response" "service:blog" "GET /health identifies blog service"

    # Test 3: List posts
    response=$(http_get "/api/posts")
    assert_contains "$response" "posts:" "GET /api/posts returns posts"
    assert_contains "$response" "First Post" "GET /api/posts includes sample post"
    assert_contains "$response" "author:John" "GET /api/posts includes author"

    # Test 4: Create post
    response=$(http_post "/api/posts" "title=New Post&content=Content")
    assert_contains "$response" "id:1" "POST /api/posts returns post id"
    assert_contains "$response" "title:New Post" "POST /api/posts returns title"
    assert_contains "$response" "authorId:1" "POST /api/posts returns author id"

    # Test 5: Get post by ID
    response=$(http_get "/api/posts/1")
    assert_contains "$response" "id:1" "GET /api/posts/1 returns post id"
    assert_contains "$response" "Blog Post" "GET /api/posts/1 returns title"
    assert_contains "$response" "Full content" "GET /api/posts/1 returns content"

    # Test 6: Get post with different ID
    response=$(http_get "/api/posts/99")
    assert_contains "$response" "id:99" "GET /api/posts/99 returns correct id parameter"

    # Test 7: Update post
    response=$(http_put "/api/posts/1" "title=Updated")
    assert_contains "$response" "id:1" "PUT /api/posts/1 returns post id"
    assert_contains "$response" "Updated Post" "PUT /api/posts/1 updates title"
    assert_contains "$response" "updated:true" "PUT /api/posts/1 confirms update"

    # Test 8: Delete post
    response=$(http_delete "/api/posts/1")
    assert_contains "$response" "deleted:true" "DELETE /api/posts/1 confirms deletion"
    assert_contains "$response" "id:1" "DELETE /api/posts/1 returns deleted id"

    # Test 9: List comments for post
    response=$(http_get "/api/posts/5/comments")
    assert_contains "$response" "comments:" "GET /api/posts/5/comments returns comments"
    assert_contains "$response" "postId:5" "GET /api/posts/5/comments returns correct post id"
    assert_contains "$response" "Great post" "GET /api/posts/5/comments includes comment content"
    assert_contains "$response" "author:Jane" "GET /api/posts/5/comments includes comment author"

    # Test 10: Add comment to post
    response=$(http_post "/api/posts/3/comments" "content=New comment")
    assert_contains "$response" "id:1" "POST /api/posts/3/comments returns comment id"
    assert_contains "$response" "postId:3" "POST /api/posts/3/comments returns correct post id"
    assert_contains "$response" "New comment" "POST /api/posts/3/comments includes content"

    # Test 11: Get posts by author
    response=$(http_get "/api/users/7/posts")
    assert_contains "$response" "posts:" "GET /api/users/7/posts returns posts"
    assert_contains "$response" "authorId:7" "GET /api/users/7/posts filters by author"

    # Test 12: Search posts
    response=$(http_get "/api/posts/search?q=test")
    assert_contains "$response" "results:" "GET /api/posts/search returns results"
    assert_contains "$response" "Matching Post" "GET /api/posts/search includes matching posts"

    stop_server
}

# ═══════════════════════════════════════════════════════════════════════════════
# API SERVER TESTS
# ═══════════════════════════════════════════════════════════════════════════════
test_api_server() {
    log_section "Testing API Server"

    compile_app "api_server" || return 1
    start_server "$OUTPUT_DIR/api_server.wasm" || return 1

    local response

    # Test 1: Home page
    response=$(http_get "/")
    assert_contains "$response" "API Server v1.0" "GET / returns API server version"

    # Test 2: Health check
    response=$(http_get "/health")
    assert_contains "$response" "status:healthy" "GET /health returns healthy status"
    assert_contains "$response" "uptime:1000" "GET /health returns uptime"
    assert_contains "$response" "version:1.0.0" "GET /health returns version"

    # Test 3: Server info
    response=$(http_get "/api/info")
    assert_contains "$response" "name:Test API Server" "GET /api/info returns server name"
    assert_contains "$response" "environment:test" "GET /api/info returns environment"

    # Test 4: Echo endpoint
    response=$(http_post "/api/echo" "Hello World")
    assert_contains "$response" "echo:Hello World" "POST /api/echo echoes body"

    # Test 5: Echo with different body
    response=$(http_post "/api/echo" "Testing 123")
    assert_contains "$response" "echo:Testing 123" "POST /api/echo echoes different body"

    # Test 6: Headers test
    response=$(http_get_with_header "/api/headers" "Content-Type: application/json")
    assert_contains "$response" "content-type:" "GET /api/headers returns content-type header"

    # Test 7: Query params - single param
    response=$(http_get "/api/params?param1=value1")
    assert_contains "$response" "param1:value1" "GET /api/params reads param1"

    # Test 8: Query params - multiple params
    response=$(http_get "/api/params?param1=foo&param2=bar")
    assert_contains "$response" "param1:foo" "GET /api/params reads param1"
    assert_contains "$response" "param2:bar" "GET /api/params reads param2"

    # Test 9: Path params - single level
    response=$(http_get "/api/items/123/details/456")
    assert_contains "$response" "id:123" "GET /api/items/:id/details/:subId reads id"
    assert_contains "$response" "subId:456" "GET /api/items/:id/details/:subId reads subId"

    # Test 10: Path params - different values
    response=$(http_get "/api/items/abc/details/xyz")
    assert_contains "$response" "id:abc" "GET /api/items/:id/details/:subId reads string id"
    assert_contains "$response" "subId:xyz" "GET /api/items/:id/details/:subId reads string subId"

    # Test 11: Method test
    response=$(http_get "/api/method")
    assert_contains "$response" "method:GET" "GET /api/method returns GET method"
    assert_contains "$response" "path:/api/method" "GET /api/method returns correct path"

    # Test 12: Error endpoint
    response=$(http_get "/api/error")
    assert_contains "$response" "error:INTERNAL_ERROR" "GET /api/error returns error code"
    assert_contains "$response" "Simulated server error" "GET /api/error returns error message"
    assert_contains "$response" "code:500" "GET /api/error returns error status"

    # Test 13: Rate limited endpoint
    response=$(http_get "/api/rate-limited")
    assert_contains "$response" "message:Request allowed" "GET /api/rate-limited returns message"
    assert_contains "$response" "remaining:99" "GET /api/rate-limited returns remaining quota"

    # Test 14: Protected endpoint without auth
    response=$(http_get "/api/protected")
    assert_contains "$response" "error:UNAUTHORIZED" "GET /api/protected without auth fails"
    assert_contains "$response" "Token required" "GET /api/protected returns auth error"

    # Test 15: Protected endpoint with auth
    response=$(http_get_with_header "/api/protected" "Authorization: Bearer token123")
    assert_contains "$response" "data:Protected data" "GET /api/protected with auth returns data"
    assert_contains "$response" "authorized:true" "GET /api/protected confirms authorization"

    # Test 16: Validate endpoint - valid data
    response=$(http_post "/api/validate" "name=John&email=john@example.com")
    assert_contains "$response" "valid:true" "POST /api/validate with valid data succeeds"
    assert_contains "$response" "Validation passed" "POST /api/validate returns success message"

    # Test 17: Validate endpoint - missing fields
    response=$(http_post "/api/validate" "incomplete_data")
    assert_contains "$response" "valid:false" "POST /api/validate with missing fields fails"
    assert_contains "$response" "error:VALIDATION_ERROR" "POST /api/validate returns error code"
    assert_contains "$response" "Missing required fields" "POST /api/validate returns error message"

    stop_server
}

# ═══════════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════════
main() {
    echo ""
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║     Clean Framework - Comprehensive HTTP Test Suite          ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

    # Trap to ensure server is stopped on exit
    trap stop_server EXIT

    check_prerequisites

    # Run all test suites
    test_todo_app
    test_auth_app
    test_blog_app
    test_api_server

    # Print summary
    log_section "Test Results Summary"
    echo ""
    echo -e "  Total Tests:   ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "  ${GREEN}Passed:        $PASSED_TESTS${NC}"
    echo -e "  ${RED}Failed:        $FAILED_TESTS${NC}"
    echo -e "  ${YELLOW}Skipped:       $SKIPPED_TESTS${NC}"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                    ALL TESTS PASSED!                          ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
        exit 0
    else
        echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║                 SOME TESTS FAILED                             ║${NC}"
        echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
        exit 1
    fi
}

# Run main
main "$@"
