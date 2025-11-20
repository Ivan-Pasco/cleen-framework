#!/usr/bin/env bash
set -e

# Frame Framework Build Script
# Builds Frame CLI and all runtime components

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build"
VERBOSE=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
TARGET_TRIPLE=""
RELEASE_MODE="--release"
BUILD_WASM=1
BUILD_CLI=1

while [[ $# -gt 0 ]]; do
	case $1 in
		--target)
			TARGET_TRIPLE="$2"
			shift 2
			;;
		--debug)
			RELEASE_MODE=""
			shift
			;;
		--verbose)
			VERBOSE=1
			shift
			;;
		--cli-only)
			BUILD_WASM=0
			shift
			;;
		--wasm-only)
			BUILD_CLI=0
			shift
			;;
		--help)
			echo "Usage: $0 [OPTIONS]"
			echo ""
			echo "Options:"
			echo "  --target <triple>   Target triple (e.g., x86_64-unknown-linux-gnu)"
			echo "  --debug             Build in debug mode instead of release"
			echo "  --verbose           Verbose output"
			echo "  --cli-only          Build only the CLI binary"
			echo "  --wasm-only         Build only WASM components"
			echo "  --help              Show this help message"
			exit 0
			;;
		*)
			echo "Unknown option: $1"
			exit 1
			;;
	esac
done

# Functions
log_info() {
	echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
	echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
	echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
	echo -e "${RED}[ERROR]${NC} $1"
}

check_rust() {
	if ! command -v cargo &> /dev/null; then
		log_error "Cargo not found. Please install Rust from https://rustup.rs/"
		exit 1
	fi

	local rust_version=$(cargo --version)
	log_info "Using $rust_version"
}

check_wasm_target() {
	if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
		log_warning "WASM target not installed. Installing..."
		rustup target add wasm32-unknown-unknown
	fi
}

build_cli() {
	log_info "Building Frame CLI..."

	cd "$PROJECT_ROOT"

	local target_arg=""
	if [ -n "$TARGET_TRIPLE" ]; then
		target_arg="--target $TARGET_TRIPLE"
		log_info "Building for target: $TARGET_TRIPLE"
	fi

	local cargo_args="build $RELEASE_MODE --bin frame $target_arg"

	if [ $VERBOSE -eq 1 ]; then
		cargo_args="$cargo_args --verbose"
	fi

	if cargo $cargo_args; then
		log_success "Frame CLI built successfully"

		# Show binary location
		local build_type="release"
		if [ -z "$RELEASE_MODE" ]; then
			build_type="debug"
		fi

		local binary_path="target/${TARGET_TRIPLE:+$TARGET_TRIPLE/}$build_type/frame"
		if [ -f "$binary_path" ]; then
			log_info "Binary location: $binary_path"
			log_info "Binary size: $(du -h "$binary_path" | cut -f1)"
		fi
	else
		log_error "Failed to build Frame CLI"
		exit 1
	fi
}

build_wasm_components() {
	log_info "Building WASM runtime components..."

	cd "$PROJECT_ROOT"

	# Ensure WASM target is installed
	check_wasm_target

	local cargo_args="build $RELEASE_MODE --target wasm32-unknown-unknown -p host-bridge"

	if [ $VERBOSE -eq 1 ]; then
		cargo_args="$cargo_args --verbose"
	fi

	if cargo $cargo_args; then
		log_success "WASM components built successfully"

		# Show WASM locations
		local build_type="release"
		if [ -z "$RELEASE_MODE" ]; then
			build_type="debug"
		fi

		local wasm_dir="target/wasm32-unknown-unknown/$build_type"
		if [ -d "$wasm_dir" ]; then
			log_info "WASM modules location: $wasm_dir"
			find "$wasm_dir" -name "*.wasm" -exec echo "  - {}" \; -exec du -h {} \; | cut -f1
		fi
	else
		log_error "Failed to build WASM components"
		exit 1
	fi
}

optimize_wasm() {
	log_info "Optimizing WASM modules..."

	local build_type="release"
	if [ -z "$RELEASE_MODE" ]; then
		build_type="debug"
	fi

	local wasm_dir="target/wasm32-unknown-unknown/$build_type"

	if command -v wasm-opt &> /dev/null; then
		find "$wasm_dir" -name "*.wasm" | while read -r wasm_file; do
			log_info "Optimizing $(basename "$wasm_file")..."
			wasm-opt -Oz "$wasm_file" -o "${wasm_file}.opt"
			mv "${wasm_file}.opt" "$wasm_file"
		done
		log_success "WASM optimization complete"
	else
		log_warning "wasm-opt not found. Install binaryen for WASM optimization"
		log_warning "  brew install binaryen  (macOS)"
		log_warning "  apt-get install binaryen  (Ubuntu/Debian)"
	fi
}

create_build_manifest() {
	log_info "Creating build manifest..."

	local build_type="release"
	if [ -z "$RELEASE_MODE" ]; then
		build_type="debug"
	fi

	mkdir -p "$BUILD_DIR"

	cat > "$BUILD_DIR/manifest.json" << EOF
{
  "version": "1.0.0",
  "build_type": "$build_type",
  "target": "${TARGET_TRIPLE:-native}",
  "built_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "components": {
    "cli": {
      "path": "target/${TARGET_TRIPLE:+$TARGET_TRIPLE/}$build_type/frame",
      "type": "binary"
    },
    "runtime": {
      "host_bridge": {
        "path": "target/wasm32-unknown-unknown/$build_type/host_bridge.wasm",
        "type": "wasm"
      }
    }
  }
}
EOF

	log_success "Build manifest created: $BUILD_DIR/manifest.json"
}

# Main execution
main() {
	log_info "Frame Framework Build Script"
	log_info "============================"
	echo ""

	check_rust

	if [ $BUILD_CLI -eq 1 ]; then
		build_cli
		echo ""
	fi

	if [ $BUILD_WASM -eq 1 ]; then
		build_wasm_components
		echo ""

		# Optimize WASM in release mode
		if [ -n "$RELEASE_MODE" ]; then
			optimize_wasm
			echo ""
		fi
	fi

	create_build_manifest

	echo ""
	log_success "Build complete!"
	log_info "Run './target/release/frame --version' to verify installation"
}

main
