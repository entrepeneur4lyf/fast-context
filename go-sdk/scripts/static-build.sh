#!/bin/bash

# Fast-Context Go SDK Static Build Script
# This script creates statically linked binaries for better portability

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
PROJECT_NAME="fast-context-go-sdk"
VERSION=$(cat "$PROJECT_ROOT/VERSION" 2>/dev/null || echo "dev")
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$PROJECT_ROOT/dist"

# Static linking flags
STATIC_LDFLAGS="-s -w -linkmode external -extldflags '-static'"
STATIC_CGO_LDFLAGS="-Wl,-Bstatic -lstdc++ -lgcc_eh -lgcc -lpthread -lm -lrt -ldl"

# Build tags for static linking
STATIC_BUILD_TAGS="netgo osusergo static_build"

# Function to print usage
show_usage() {
    echo "Fast-Context Go SDK Static Build Script"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  linux-amd64      - Build static binary for Linux x86_64"
    echo "  linux-arm64      - Build static binary for Linux ARM64"
    echo "  all              - Build static binaries for all supported platforms"
    echo "  verify           - Verify static linking of built binaries"
    echo "  clean            - Clean build artifacts"
    echo "  help             - Show this help message"
    echo ""
    echo "Options:"
    echo "  --debug          - Build with debug symbols"
    echo "  --race           - Build with race detector"
    echo "  --compress       - Compress binaries with UPX"
    echo "  --strip          - Strip symbols from binaries"
    echo "  --no-cgo         - Build without CGO"
    echo ""
    echo "Environment variables:"
    echo "  CGO_ENABLED      - Enable/disable CGO (default: 1)"
    echo "  GO_BUILDFLAGS    - Additional build flags"
    echo "  UPX_ENABLED      - Enable UPX compression (default: 0)"
    echo ""
    echo "Examples:"
    echo "  $0 linux-amd64              # Build static binary for Linux x86_64"
    echo "  $0 all --compress           # Build all platforms with UPX compression"
    echo "  $0 linux-amd64 --debug       # Build debug version"
}

# Function to check dependencies
check_dependencies() {
    echo -e "${BLUE}[INFO]${NC} Checking dependencies..."
    
    # Check Go
    if ! command -v go >/dev/null 2>&1; then
        echo -e "${RED}[ERROR]${NC} Go is not installed or not in PATH"
        exit 1
    fi
    
    # Check CGO dependencies if enabled
    if [ "$CGO_ENABLED" = "1" ]; then
        # Check for cross-compilation tools
        if ! command -v x86_64-linux-gnu-gcc >/dev/null 2>&1; then
            echo -e "${YELLOW}[WARN]${NC} x86_64-linux-gnu-gcc not found. Install with: sudo apt-get install gcc-x86_64-linux-gnu"
        fi
        
        if ! command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
            echo -e "${YELLOW}[WARN]${NC} aarch64-linux-gnu-gcc not found. Install with: sudo apt-get install gcc-aarch64-linux-gnu"
        fi
    fi
    
    # Check UPX if enabled
    if [ "${UPX_ENABLED:-0}" = "1" ] && ! command -v upx >/dev/null 2>&1; then
        echo -e "${YELLOW}[WARN]${NC} UPX not found. Install with: sudo apt-get install upx"
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} Dependencies checked"
}

# Function to build static binary for Linux AMD64
build_linux_amd64() {
    echo -e "${BLUE}[INFO]${NC} Building static binary for Linux AMD64..."
    
    local build_flags="$STATIC_LDFLAGS"
    local build_tags="$STATIC_BUILD_TAGS"
    local output_name="$BUILD_DIR/$PROJECT_NAME-linux-amd64-static"
    
    # Apply options
    if [ "$DEBUG_MODE" = "1" ]; then
        build_flags="-gcflags=\"-N -l\" -compressdwarf=false"
        build_tags="$build_tags debug"
    fi
    
    if [ "$RACE_MODE" = "1" ]; then
        build_flags="$build_flags -race"
        build_tags="$build_tags race"
    fi
    
    if [ "$NO_CGO" = "1" ]; then
        export CGO_ENABLED=0
        build_tags="$build_tags nocgo"
    else
        export CGO_ENABLED=1
        export CC=x86_64-linux-gnu-gcc
        export CXX=x86_64-linux-gnu-g++
        build_flags="$build_flags $STATIC_CGO_LDFLAGS"
    fi
    
    # Add custom build flags
    if [ -n "$GO_BUILDFLAGS" ]; then
        build_flags="$build_flags $GO_BUILDFLAGS"
    fi
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Build the binary
    cd "$PROJECT_ROOT"
    GOOS=linux GOARCH=amd64 CGO_ENABLED=$CGO_ENABLED \
        go build -tags="$build_tags" -ldflags="$build_flags" \
        -o "$output_name" ./cmd/fast-context
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} Built $output_name"
        
        # Strip symbols if requested
        if [ "$STRIP_MODE" = "1" ]; then
            strip "$output_name" 2>/dev/null || true
            echo -e "${BLUE}[INFO]${NC} Stripped symbols from $output_name"
        fi
        
        # Compress if requested
        if [ "${UPX_ENABLED:-0}" = "1" ] && command -v upx >/dev/null 2>&1; then
            upx --best --ultra-brute "$output_name" 2>/dev/null || true
            echo -e "${BLUE}[INFO]${NC} Compressed $output_name with UPX"
        fi
    else
        echo -e "${RED}[ERROR]${NC} Failed to build for Linux AMD64"
        exit 1
    fi
}

# Function to build static binary for Linux ARM64
build_linux_arm64() {
    echo -e "${BLUE}[INFO]${NC} Building static binary for Linux ARM64..."
    
    local build_flags="$STATIC_LDFLAGS"
    local build_tags="$STATIC_BUILD_TAGS"
    local output_name="$BUILD_DIR/$PROJECT_NAME-linux-arm64-static"
    
    # Apply options
    if [ "$DEBUG_MODE" = "1" ]; then
        build_flags="-gcflags=\"-N -l\" -compressdwarf=false"
        build_tags="$build_tags debug"
    fi
    
    if [ "$RACE_MODE" = "1" ]; then
        build_flags="$build_flags -race"
        build_tags="$build_tags race"
    fi
    
    if [ "$NO_CGO" = "1" ]; then
        export CGO_ENABLED=0
        build_tags="$build_tags nocgo"
    else
        export CGO_ENABLED=1
        export CC=aarch64-linux-gnu-gcc
        export CXX=aarch64-linux-gnu-g++
        build_flags="$build_flags $STATIC_CGO_LDFLAGS"
    fi
    
    # Add custom build flags
    if [ -n "$GO_BUILDFLAGS" ]; then
        build_flags="$build_flags $GO_BUILDFLAGS"
    fi
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Build the binary
    cd "$PROJECT_ROOT"
    GOOS=linux GOARCH=arm64 CGO_ENABLED=$CGO_ENABLED \
        go build -tags="$build_tags" -ldflags="$build_flags" \
        -o "$output_name" ./cmd/fast-context
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} Built $output_name"
        
        # Strip symbols if requested
        if [ "$STRIP_MODE" = "1" ]; then
            aarch64-linux-gnu-strip "$output_name" 2>/dev/null || true
            echo -e "${BLUE}[INFO]${NC} Stripped symbols from $output_name"
        fi
        
        # Compress if requested
        if [ "${UPX_ENABLED:-0}" = "1" ] && command -v upx >/dev/null 2>&1; then
            upx --best --ultra-brute "$output_name" 2>/dev/null || true
            echo -e "${BLUE}[INFO]${NC} Compressed $output_name with UPX"
        fi
    else
        echo -e "${RED}[ERROR]${NC} Failed to build for Linux ARM64"
        exit 1
    fi
}

# Function to build static binaries for all platforms
build_all() {
    echo -e "${BLUE}[INFO]${NC} Building static binaries for all platforms..."
    
    # Create distribution directory
    mkdir -p "$DIST_DIR"
    
    # Build for each platform
    build_linux_amd64
    build_linux_arm64
    
    # Copy binaries to distribution directory
    cp "$BUILD_DIR/$PROJECT_NAME-linux-amd64-static" "$DIST_DIR/"
    cp "$BUILD_DIR/$PROJECT_NAME-linux-arm64-static" "$DIST_DIR/"
    
    echo -e "${GREEN}[SUCCESS]${NC} All static binaries built"
}

# Function to verify static linking
verify_static() {
    echo -e "${BLUE}[INFO]${NC} Verifying static linking..."
    
    local verify_failed=0
    
    # Check Linux AMD64
    if [ -f "$BUILD_DIR/$PROJECT_NAME-linux-amd64-static" ]; then
        echo -e "${BLUE}[INFO]${NC} Checking Linux AMD64 binary..."
        if file "$BUILD_DIR/$PROJECT_NAME-linux-amd64-static" | grep -q "statically linked"; then
            echo -e "${GREEN}[OK]${NC} Linux AMD64: statically linked"
        else
            echo -e "${YELLOW}[WARN]${NC} Linux AMD64: may not be fully static"
            # Check with ldd
            if ldd "$BUILD_DIR/$PROJECT_NAME-linux-amd64-static" 2>&1 | grep -q "not a dynamic executable"; then
                echo -e "${GREEN}[OK]${NC} Linux AMD64: no dynamic dependencies (confirmed with ldd)"
            else
                echo -e "${RED}[FAIL]${NC} Linux AMD64: has dynamic dependencies"
                verify_failed=1
            fi
        fi
    else
        echo -e "${YELLOW}[WARN]${NC} Linux AMD64 binary not found"
    fi
    
    # Check Linux ARM64
    if [ -f "$BUILD_DIR/$PROJECT_NAME-linux-arm64-static" ]; then
        echo -e "${BLUE}[INFO]${NC} Checking Linux ARM64 binary..."
        if file "$BUILD_DIR/$PROJECT_NAME-linux-arm64-static" | grep -q "statically linked"; then
            echo -e "${GREEN}[OK]${NC} Linux ARM64: statically linked"
        else
            echo -e "${YELLOW}[WARN]${NC} Linux ARM64: may not be fully static"
            # Check with ldd
            if ldd "$BUILD_DIR/$PROJECT_NAME-linux-arm64-static" 2>&1 | grep -q "not a dynamic executable"; then
                echo -e "${GREEN}[OK]${NC} Linux ARM64: no dynamic dependencies (confirmed with ldd)"
            else
                echo -e "${RED}[FAIL]${NC} Linux ARM64: has dynamic dependencies"
                verify_failed=1
            fi
        fi
    else
        echo -e "${YELLOW}[WARN]${NC} Linux ARM64 binary not found"
    fi
    
    if [ $verify_failed -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} Static linking verification completed"
    else
        echo -e "${RED}[ERROR]${NC} Static linking verification failed"
        exit 1
    fi
}

# Function to clean build artifacts
clean() {
    echo -e "${BLUE}[INFO]${NC} Cleaning build artifacts..."
    
    rm -rf "$BUILD_DIR" "$DIST_DIR"
    go clean -cache -testcache -modcache
    
    echo -e "${GREEN}[SUCCESS]${NC} Build artifacts cleaned"
}

# Function to show build information
show_info() {
    echo -e "${BLUE}[INFO]${NC} Static build information:"
    echo "  Project: $PROJECT_NAME"
    echo "  Version: $VERSION"
    echo "  Build directory: $BUILD_DIR"
    echo "  Distribution directory: $DIST_DIR"
    echo "  CGO enabled: $CGO_ENABLED"
    echo "  Static build tags: $STATIC_BUILD_TAGS"
    echo "  Static linker flags: $STATIC_LDFLAGS"
    echo "  Static CGO flags: $STATIC_CGO_LDFLAGS"
}

# Parse command line arguments
DEBUG_MODE=0
RACE_MODE=0
STRIP_MODE=1
NO_CGO=0

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            DEBUG_MODE=1
            shift
            ;;
        --race)
            RACE_MODE=1
            shift
            ;;
        --compress)
            export UPX_ENABLED=1
            shift
            ;;
        --strip)
            STRIP_MODE=1
            shift
            ;;
        --no-strip)
            STRIP_MODE=0
            shift
            ;;
        --no-cgo)
            NO_CGO=1
            shift
            ;;
        -h|--help|help)
            show_usage
            exit 0
            ;;
        *)
            break
            ;;
    esac
done

# Main command handling
case "${1:-help}" in
    "linux-amd64")
        check_dependencies
        build_linux_amd64
        verify_static
        ;;
    "linux-arm64")
        check_dependencies
        build_linux_arm64
        verify_static
        ;;
    "all")
        check_dependencies
        build_all
        verify_static
        ;;
    "verify")
        verify_static
        ;;
    "clean")
        clean
        ;;
    "info")
        show_info
        ;;
    -h|--help|help)
        show_usage
        ;;
    *)
        echo -e "${RED}[ERROR]${NC} Unknown command: $1"
        show_usage
        exit 1
        ;;
esac

echo -e "${GREEN}[SUCCESS]${NC} Static build completed!"