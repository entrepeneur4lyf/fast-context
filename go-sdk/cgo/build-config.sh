#!/bin/bash

# Fast-Context Go SDK CGO Build Configuration Script
# This script configures CGO for different platforms and architectures

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CGO_ENABLED=${CGO_ENABLED:-1}
GOOS=${GOOS:-$(go env GOOS)}
GOARCH=${GOARCH:-$(go env GOARCH)}
RUST_TARGET=${RUST_TARGET:-""}

# Rust library path
RUST_LIB_PATH="$PROJECT_ROOT/../target"
RUST_LIB_NAME="libfast_context"

# CGO configuration file
CGO_CONFIG_FILE="$PROJECT_ROOT/cgo/config.env"

# Platform-specific configurations
setup_cgo_linux_amd64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for Linux/amd64"
    
    export CGO_ENABLED=1
    export GOOS=linux
    export GOARCH=amd64
    export RUST_TARGET=x86_64-unknown-linux-gnu
    
    # Use system GCC by default
    export CC=${CC:-gcc}
    export CXX=${CXX:-g++}
    
    # Static linking flags
    export CGO_LDFLAGS="-Wl,-Bstatic -lstdc++ -lgcc_eh -lgcc -lpthread -lm -lrt -ldl"
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    # Rust-specific settings
    export RUSTFLAGS="-C target-feature=+crt-static"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for Linux/amd64"
}

setup_cgo_linux_arm64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for Linux/arm64"
    
    export CGO_ENABLED=1
    export GOOS=linux
    export GOARCH=arm64
    export RUST_TARGET=aarch64-unknown-linux-gnu
    
    # Use cross-compiler
    export CC=${CC:-aarch64-linux-gnu-gcc}
    export CXX=${CXX:-aarch64-linux-gnu-g++}
    
    # Static linking flags
    export CGO_LDFLAGS="-Wl,-Bstatic -lstdc++ -lgcc_eh -lgcc -lpthread -lm -lrt -ldl"
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    # Rust-specific settings
    export RUSTFLAGS="-C target-feature=+crt-static"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for Linux/arm64"
}

setup_cgo_darwin_amd64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for macOS/amd64"
    
    export CGO_ENABLED=1
    export GOOS=darwin
    export GOARCH=amd64
    export RUST_TARGET=x86_64-apple-darwin
    
    # Use Clang on macOS
    export CC=${CC:-clang}
    export CXX=${CXX:-clang++}
    
    # macOS-specific flags
    export CGO_LDFLAGS="-framework Foundation -framework CoreFoundation"
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    # Rust-specific settings
    export MACOSX_DEPLOYMENT_TARGET=10.15
    export RUSTFLAGS="-C link-arg=-macosx_version_min=10.15"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for macOS/amd64"
}

setup_cgo_darwin_arm64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for macOS/arm64"
    
    export CGO_ENABLED=1
    export GOOS=darwin
    export GOARCH=arm64
    export RUST_TARGET=aarch64-apple-darwin
    
    # Use Clang on macOS
    export CC=${CC:-clang}
    export CXX=${CXX:-clang++}
    
    # macOS-specific flags
    export CGO_LDFLAGS="-framework Foundation -framework CoreFoundation"
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    # Rust-specific settings
    export MACOSX_DEPLOYMENT_TARGET=11.0
    export RUSTFLAGS="-C link-arg=-macosx_version_min=11.0"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for macOS/arm64"
}

setup_cgo_windows_amd64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for Windows/amd64"
    
    export CGO_ENABLED=1
    export GOOS=windows
    export GOARCH=amd64
    export RUST_TARGET=x86_64-pc-windows-gnu
    
    # Use MinGW on Windows
    export CC=${CC:-x86_64-w64-mingw32-gcc}
    export CXX=${CXX:-x86_64-w64-mingw32-g++}
    
    # Windows-specific flags
    export CGO_LDFLAGS="-ladvapi32 -lkernel32 -luser32 -lws2_32 -lmsvcrt"
    export CGO_CFLAGS="-O2"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    # Rust-specific settings
    export RUSTFLAGS="-C target-feature=+crt-static"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for Windows/amd64"
}

setup_cgo_freebsd_amd64() {
    echo -e "${BLUE}[INFO]${NC} Setting up CGO for FreeBSD/amd64"
    
    export CGO_ENABLED=1
    export GOOS=freebsd
    export GOARCH=amd64
    export RUST_TARGET=x86_64-unknown-freebsd
    
    # Use system GCC
    export CC=${CC:-gcc}
    export CXX=${CXX:-g++}
    
    # FreeBSD-specific flags
    export CGO_LDFLAGS="-lpthread -lm -lrt"
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configured for FreeBSD/amd64"
}

# Detect platform automatically
detect_platform() {
    local os="$GOOS"
    local arch="$GOARCH"
    
    echo -e "${BLUE}[INFO]${NC} Detected platform: $os/$arch"
    
    case "$os/$arch" in
        "linux/amd64")
            setup_cgo_linux_amd64
            ;;
        "linux/arm64")
            setup_cgo_linux_arm64
            ;;
        "darwin/amd64")
            setup_cgo_darwin_amd64
            ;;
        "darwin/arm64")
            setup_cgo_darwin_arm64
            ;;
        "windows/amd64")
            setup_cgo_windows_amd64
            ;;
        "freebsd/amd64")
            setup_cgo_freebsd_amd64
            ;;
        *)
            echo -e "${YELLOW}[WARN]${NC} Unsupported platform: $os/$arch"
            echo -e "${YELLOW}[WARN]${NC} Using default CGO configuration"
            setup_cgo_default
            ;;
    esac
}

# Default CGO configuration
setup_cgo_default() {
    echo -e "${BLUE}[INFO]${NC} Setting up default CGO configuration"
    
    export CGO_ENABLED=1
    
    # Use system compiler
    export CC=${CC:-gcc}
    export CXX=${CXX:-g++}
    
    # Default flags
    export CGO_LDFLAGS=""
    export CGO_CFLAGS="-O2 -fPIC"
    export CGO_CPPFLAGS="-I$PROJECT_ROOT/../include"
    
    echo -e "${GREEN}[SUCCESS]${NC} Default CGO configuration applied"
}

# Validate CGO setup
validate_cgo_setup() {
    echo -e "${BLUE}[INFO]${NC} Validating CGO setup..."
    
    # Check if CGO is enabled
    if [ "$CGO_ENABLED" != "1" ]; then
        echo -e "${YELLOW}[WARN]${NC} CGO is disabled. CGO features will not be available."
        return 0
    fi
    
    # Check if compiler is available
    if ! command -v "$CC" >/dev/null 2>&1; then
        echo -e "${RED}[ERROR]${NC} Compiler not found: $CC"
        return 1
    fi
    
    # Check if C++ compiler is available
    if ! command -v "$CXX" >/dev/null 2>&1; then
        echo -e "${YELLOW}[WARN]${NC} C++ compiler not found: $CXX"
    fi
    
    # Check if Rust is available
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${YELLOW}[WARN]${NC} Rust not found. Rust features will not be available."
    fi
    
    # Check if Rust library exists
    local rust_lib=""
    for ext in so dylib dll a; do
        if [ -f "$RUST_LIB_PATH/release/$RUST_LIB_NAME.$ext" ]; then
            rust_lib="$RUST_LIB_PATH/release/$RUST_LIB_NAME.$ext"
            break
        fi
    done
    
    if [ -n "$rust_lib" ]; then
        echo -e "${GREEN}[INFO]${NC} Rust library found: $rust_lib"
    else
        echo -e "${YELLOW}[WARN]${NC} Rust library not found. Building..."
        (cd "$PROJECT_ROOT/.." && cargo build --release)
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO setup validated"
}

# Save CGO configuration
save_cgo_config() {
    echo -e "${BLUE}[INFO]${NC} Saving CGO configuration..."
    
    mkdir -p "$PROJECT_ROOT/cgo"
    
    cat > "$CGO_CONFIG_FILE" << EOF
# Fast-Context Go SDK CGO Configuration
# Generated by $(basename "$0")

# CGO settings
export CGO_ENABLED=$CGO_ENABLED
export GOOS=$GOOS
export GOARCH=$GOARCH
export RUST_TARGET=$RUST_TARGET

# Compiler settings
export CC=$CC
export CXX=$CXX

# CGO flags
export CGO_LDFLAGS="$CGO_LDFLAGS"
export CGO_CFLAGS="$CGO_CFLAGS"
export CGO_CPPFLAGS="$CGO_CPPFLAGS"

# Rust settings
export RUSTFLAGS="$RUSTFLAGS"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-}"

# Library paths
export RUST_LIB_PATH="$RUST_LIB_PATH"
export RUST_LIB_NAME="$RUST_LIB_NAME"

# Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO configuration saved to $CGO_CONFIG_FILE"
}

# Load CGO configuration
load_cgo_config() {
    if [ -f "$CGO_CONFIG_FILE" ]; then
        echo -e "${BLUE}[INFO]${NC} Loading CGO configuration from $CGO_CONFIG_FILE"
        source "$CGO_CONFIG_FILE"
    else
        echo -e "${YELLOW}[WARN]${NC} No CGO configuration found. Using defaults."
    fi
}

# Build CGO wrapper
build_cgo_wrapper() {
    echo -e "${BLUE}[INFO]${NC} Building CGO wrapper..."
    
    # Ensure Rust library is built
    if ! ls "$RUST_LIB_PATH/release/$RUST_LIB_NAME".* >/dev/null 2>&1; then
        echo -e "${BLUE}[INFO]${NC} Building Rust library..."
        (cd "$PROJECT_ROOT/.." && cargo build --release)
    fi
    
    # Copy Rust library to CGO directory
    mkdir -p "$PROJECT_ROOT/internal/cgo"
    
    # Try different library extensions
    for ext in so dylib dll a; do
        if [ -f "$RUST_LIB_PATH/release/$RUST_LIB_NAME.$ext" ]; then
            cp "$RUST_LIB_PATH/release/$RUST_LIB_NAME.$ext" "$PROJECT_ROOT/internal/cgo/"
            echo -e "${GREEN}[INFO]${NC} Copied $RUST_LIB_NAME.$ext to CGO directory"
            break
        fi
    done
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO wrapper built"
}

# Test CGO setup
test_cgo_setup() {
    echo -e "${BLUE}[INFO]${NC} Testing CGO setup..."
    
    # Create a simple test
    cd "$PROJECT_ROOT"
    
    # Test Go compilation with CGO
    if CGO_ENABLED=1 go build -v ./internal/cgo/... >/dev/null 2>&1; then
        echo -e "${GREEN}[SUCCESS]${NC} CGO compilation test passed"
    else
        echo -e "${RED}[ERROR]${NC} CGO compilation test failed"
        return 1
    fi
    
    # Test Rust library linkage
    if go run -tags=cgo ./tests/test_cgo_binding.go >/dev/null 2>&1; then
        echo -e "${GREEN}[SUCCESS]${NC} Rust library linkage test passed"
    else
        echo -e "${YELLOW}[WARN]${NC} Rust library linkage test failed (may be expected)"
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO setup tested"
}

# Show CGO configuration
show_cgo_config() {
    echo -e "${BLUE}[INFO]${NC} Current CGO configuration:"
    echo "  CGO_ENABLED: $CGO_ENABLED"
    echo "  GOOS: $GOOS"
    echo "  GOARCH: $GOARCH"
    echo "  RUST_TARGET: $RUST_TARGET"
    echo "  CC: $CC"
    echo "  CXX: $CXX"
    echo "  CGO_LDFLAGS: $CGO_LDFLAGS"
    echo "  CGO_CFLAGS: $CGO_CFLAGS"
    echo "  CGO_CPPFLAGS: $CGO_CPPFLAGS"
    echo "  RUSTFLAGS: $RUSTFLAGS"
    
    if [ -n "$MACOSX_DEPLOYMENT_TARGET" ]; then
        echo "  MACOSX_DEPLOYMENT_TARGET: $MACOSX_DEPLOYMENT_TARGET"
    fi
    
    echo "  RUST_LIB_PATH: $RUST_LIB_PATH"
    echo "  RUST_LIB_NAME: $RUST_LIB_NAME"
}

# Show usage
show_usage() {
    echo "Fast-Context Go SDK CGO Build Configuration Script"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  setup [os/arch]   - Set up CGO for specific platform"
    echo "  detect            - Auto-detect and setup CGO"
    echo "  validate          - Validate CGO setup"
    echo "  save              - Save CGO configuration"
    echo "  load              - Load CGO configuration"
    echo "  build             - Build CGO wrapper"
    echo "  test              - Test CGO setup"
    echo "  show              - Show current configuration"
    echo "  clean             - Clean CGO build artifacts"
    echo ""
    echo "Platforms:"
    echo "  linux/amd64, linux/arm64, darwin/amd64, darwin/arm64, windows/amd64, freebsd/amd64"
    echo ""
    echo "Examples:"
    echo "  $0 setup linux/amd64    # Setup for Linux x86_64"
    echo "  $0 detect              # Auto-detect platform"
    echo "  $0 test                # Test CGO setup"
    echo ""
    echo "Environment variables:"
    echo "  CGO_ENABLED       - Enable/disable CGO (default: 1)"
    echo "  GOOS              - Target operating system"
    echo "  GOARCH            - Target architecture"
    echo "  RUST_TARGET       - Rust target triple"
    echo "  CC                - C compiler"
    echo "  CXX               - C++ compiler"
}

# Clean CGO build artifacts
clean_cgo() {
    echo -e "${BLUE}[INFO]${NC} Cleaning CGO build artifacts..."
    
    # Remove CGO configuration
    rm -f "$CGO_CONFIG_FILE"
    
    # Remove copied libraries
    rm -f "$PROJECT_ROOT/internal/cgo/libfast_context."*
    
    # Clean Go cache
    go clean -cache -testcache -modcache
    
    echo -e "${GREEN}[SUCCESS]${NC} CGO artifacts cleaned"
}

# Main execution
main() {
    case "${1:-detect}" in
        setup)
            if [ -n "$2" ]; then
                case "$2" in
                    "linux/amd64") setup_cgo_linux_amd64 ;;
                    "linux/arm64") setup_cgo_linux_arm64 ;;
                    "darwin/amd64") setup_cgo_darwin_amd64 ;;
                    "darwin/arm64") setup_cgo_darwin_arm64 ;;
                    "windows/amd64") setup_cgo_windows_amd64 ;;
                    "freebsd/amd64") setup_cgo_freebsd_amd64 ;;
                    *)
                        echo -e "${RED}[ERROR]${NC} Unsupported platform: $2"
                        echo "Supported platforms: linux/amd64, linux/arm64, darwin/amd64, darwin/arm64, windows/amd64, freebsd/amd64"
                        exit 1
                        ;;
                esac
            else
                echo -e "${RED}[ERROR]${NC} Platform required for setup command"
                echo "Usage: $0 setup [platform]"
                exit 1
            fi
            ;;
        detect)
            detect_platform
            ;;
        validate)
            validate_cgo_setup
            ;;
        save)
            save_cgo_config
            ;;
        load)
            load_cgo_config
            ;;
        build)
            build_cgo_wrapper
            ;;
        test)
            test_cgo_setup
            ;;
        show)
            show_cgo_config
            ;;
        clean)
            clean_cgo
            ;;
        -h|--help|help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} Unknown command: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"