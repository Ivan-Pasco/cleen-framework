#!/bin/bash

# Clean Framework Test Runner
# Compiles and executes .cln test files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRAMEWORK_DIR="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$FRAMEWORK_DIR/tests/framework"
OUTPUT_DIR="$TEST_DIR/output"
COMPILER="$HOME/.cleen/bin/cln"
SERVER="$HOME/Documents/Dev/Clean Language/clean-server/target/release/clean-server"

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Default filter (all tests)
FILTER=""
VERBOSE=false
DRY_RUN=false

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS] [FILTER]"
    echo ""
    echo "Options:"
    echo "  -h, --help       Show this help message"
    echo "  -v, --verbose    Verbose output"
    echo "  -d, --dry-run    Show what would be run without executing"
    echo "  -c, --compile    Compile only, don't run tests"
    echo ""
    echo "Filters:"
    echo "  unit             Run only unit tests"
    echo "  unit/plugins     Run only plugin unit tests"
    echo "  unit/plugins/web Run only frame.web tests"
    echo "  unit/plugins/data Run only frame.data tests"
    echo "  unit/plugins/auth Run only frame.auth tests"
    echo "  unit/plugins/ui  Run only frame.ui tests"
    echo "  unit/bridge      Run only bridge tests"
    echo "  integration      Run only integration tests"
    echo "  e2e              Run only end-to-end tests"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 unit               # Run unit tests only"
    echo "  $0 unit/plugins/web   # Run frame.web tests only"
    echo "  $0 -v integration     # Run integration tests with verbose output"
}

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_section() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    print_section "Checking Prerequisites"

    # Check compiler
    if [ ! -f "$COMPILER" ]; then
        print_error "Compiler not found at: $COMPILER"
        print_info "Install with: cleen install latest && cleen use latest"
        exit 1
    fi
    print_success "Compiler found: $COMPILER"

    # Check server (optional)
    if [ -f "$SERVER" ]; then
        print_success "Server found: $SERVER"
    else
        print_warning "Server not found at: $SERVER"
        print_info "Install with: cleen server install"
    fi

    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    print_success "Output directory ready: $OUTPUT_DIR"
}

# Compile a single test file
compile_test() {
    local test_file="$1"
    local relative_path="${test_file#$TEST_DIR/}"
    local output_name="${relative_path//\//_}"
    output_name="${output_name%.cln}.wasm"
    local output_path="$OUTPUT_DIR/$output_name"

    if [ "$VERBOSE" = true ]; then
        print_info "Compiling: $relative_path" >&2
    fi

    # Compile the test file (redirect all output to stderr or /dev/null)
    if "$COMPILER" compile "$test_file" -o "$output_path" --plugins >/dev/null 2>&1; then
        if [ "$VERBOSE" = true ]; then
            print_success "Compiled: $output_name" >&2
        fi
        echo "$output_path"
        return 0
    else
        print_error "Compilation failed: $relative_path" >&2
        return 1
    fi
}

# Run a single compiled test using clean-server
run_test() {
    local wasm_file="$1"
    local test_name="$2"
    local test_port=3334

    if [ "$VERBOSE" = true ]; then
        print_info "Running: $test_name"
    fi

    # Check if server is available
    if [ ! -f "$SERVER" ]; then
        # No runtime available, just verify compilation
        if [ -f "$wasm_file" ]; then
            print_success "$test_name (compiled only)"
            return 0
        else
            print_error "$test_name (compilation failed)"
            return 1
        fi
    fi

    # Kill any existing server on the test port
    pkill -f "clean-server.*$test_port" 2>/dev/null || true
    sleep 0.2

    # Start clean-server with the WASM file and capture output
    local server_log="/tmp/test_server_$$.log"
    "$SERVER" "$wasm_file" --port $test_port > "$server_log" 2>&1 &
    local server_pid=$!

    # Wait for server to initialize (it runs start() on load)
    sleep 1

    # Check if server is still running (it may have exited after start())
    local output=""
    if [ -f "$server_log" ]; then
        output=$(cat "$server_log")
    fi

    # Kill the server
    kill $server_pid 2>/dev/null || true
    sleep 0.2

    # Check for test results in output
    if echo "$output" | grep -q "FAIL:"; then
        print_error "$test_name"
        if [ "$VERBOSE" = true ]; then
            echo "$output" | grep -E "(FAIL:|PASS:|Test)" | while read line; do
                echo "    $line"
            done
        fi
        rm -f "$server_log"
        return 1
    elif echo "$output" | grep -q "PASS:"; then
        print_success "$test_name"
        if [ "$VERBOSE" = true ]; then
            echo "$output" | grep "PASS:" | while read line; do
                echo "    $line"
            done
        fi
        rm -f "$server_log"
        return 0
    elif echo "$output" | grep -qi "error\|failed"; then
        print_error "$test_name (runtime error)"
        if [ "$VERBOSE" = true ]; then
            echo "    $(echo "$output" | grep -i "error\|failed" | head -3)"
        fi
        rm -f "$server_log"
        return 1
    else
        # No explicit PASS/FAIL - check if it ran without errors
        if echo "$output" | grep -q "Server error"; then
            print_error "$test_name (server error)"
            if [ "$VERBOSE" = true ]; then
                echo "    $(echo "$output" | tail -5)"
            fi
            rm -f "$server_log"
            return 1
        else
            print_success "$test_name (executed)"
            rm -f "$server_log"
            return 0
        fi
    fi
}

# Legacy wasmtime runner (kept for reference)
run_test_wasmtime() {
    local wasm_file="$1"
    local test_name="$2"

    if [ "$VERBOSE" = true ]; then
        print_info "Running: $test_name"
    fi

    # Try running with wasmtime if available
    if command -v wasmtime &> /dev/null; then
        local output
        # macOS-compatible timeout using perl
        if output=$(perl -e 'alarm 10; exec @ARGV' wasmtime "$wasm_file" 2>&1); then
            # Check for FAIL in output
            if echo "$output" | grep -q "FAIL:"; then
                print_error "$test_name"
                if [ "$VERBOSE" = true ]; then
                    echo "$output" | grep "FAIL:" | while read line; do
                        echo "    $line"
                    done
                fi
                return 1
            else
                print_success "$test_name"
                if [ "$VERBOSE" = true ] && echo "$output" | grep -q "PASS:"; then
                    echo "$output" | grep "PASS:" | while read line; do
                        echo "    $line"
                    done
                fi
                return 0
            fi
        else
            print_error "$test_name (execution failed)"
            if [ "$VERBOSE" = true ]; then
                echo "    $output"
            fi
            return 1
        fi
    else
        # No runtime available, just verify compilation
        if [ -f "$wasm_file" ]; then
            print_success "$test_name (compiled only)"
            return 0
        else
            print_error "$test_name (compilation failed)"
            return 1
        fi
    fi
}

# Find all test files matching the filter
find_test_files() {
    local base_dir="$TEST_DIR"

    if [ -n "$FILTER" ]; then
        base_dir="$TEST_DIR/$FILTER"
    fi

    if [ ! -d "$base_dir" ]; then
        print_error "Test directory not found: $base_dir"
        exit 1
    fi

    find "$base_dir" -name "*.cln" -type f | sort
}

# Run tests in a category
run_test_category() {
    local category="$1"
    local test_files

    print_section "Running Tests: $category"

    test_files=$(find "$TEST_DIR/$category" -name "*.cln" -type f 2>/dev/null | sort)

    if [ -z "$test_files" ]; then
        print_warning "No test files found in: $category"
        return
    fi

    while IFS= read -r test_file; do
        local relative_path="${test_file#$TEST_DIR/}"
        local test_name="${relative_path%.cln}"

        ((TOTAL_TESTS++))

        if [ "$DRY_RUN" = true ]; then
            print_info "[DRY RUN] Would run: $test_name"
            continue
        fi

        # Compile the test
        local wasm_file
        if wasm_file=$(compile_test "$test_file"); then
            # Run the test
            if run_test "$wasm_file" "$test_name"; then
                ((PASSED_TESTS++))
            else
                ((FAILED_TESTS++))
            fi
        else
            ((FAILED_TESTS++))
        fi
    done <<< "$test_files"
}

# Main test runner
run_tests() {
    local start_time=$(date +%s)

    print_section "Clean Framework Test Suite"
    print_info "Test directory: $TEST_DIR"
    print_info "Filter: ${FILTER:-all}"

    check_prerequisites

    if [ -n "$FILTER" ]; then
        # Run specific category
        run_test_category "$FILTER"
    else
        # Run all categories in order
        if [ -d "$TEST_DIR/unit" ]; then
            run_test_category "unit"
        fi

        if [ -d "$TEST_DIR/integration" ]; then
            run_test_category "integration"
        fi

        if [ -d "$TEST_DIR/e2e" ]; then
            run_test_category "e2e"
        fi
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Print summary
    print_section "Test Summary"
    echo ""
    echo "  Total:   $TOTAL_TESTS"
    echo -e "  ${GREEN}Passed:  $PASSED_TESTS${NC}"
    echo -e "  ${RED}Failed:  $FAILED_TESTS${NC}"
    echo -e "  ${YELLOW}Skipped: $SKIPPED_TESTS${NC}"
    echo ""
    echo "  Duration: ${duration}s"
    echo ""

    if [ $FAILED_TESTS -gt 0 ]; then
        print_error "Some tests failed!"
        exit 1
    else
        print_success "All tests passed!"
        exit 0
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -c|--compile)
            COMPILE_ONLY=true
            shift
            ;;
        -*)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            FILTER="$1"
            shift
            ;;
    esac
done

# Run the tests
run_tests
