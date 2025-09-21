#!/bin/bash

# Fast-Context Go SDK Cross-Compilation Script
# Usage: ./scripts/cross-compile.sh [target-os] [target-arch]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
TARGET_OS=${1:-linux}
TARGET_ARCH=${2:-amd64}
PROJECT_NAME="fast-context-go-sdk"
VERSION=$(cat "$PROJECT_ROOT/VERSION" 2>/dev/null || echo "dev")
DIST_DIR="$PROJECT_ROOT/dist"
BUILD_DIR="$PROJECT_ROOT/build"

# Supported platforms
SUPPORTED_PLATFORMS=(
    "linux/amd64"
    "linux/arm64"
    "darwin/amd64"
    "darwin/arm64"
    "windows/amd64"
)

# Cross-compilation toolchains setup
setup_cross_compilation() {
    local os="$1"
    local arch="$2"
    
    echo -e "${BLUE}[INFO]${NC} Setting up cross-compilation for $os/$arch"
    
    case "$os/$arch" in
        "linux/amd64")
            export GOOS=linux
            export GOARCH=amd64
            export CC=x86_64-linux-gnu-gcc
            export CXX=x86_64-linux-gnu-g++
            ;;
        "linux/arm64")
            export GOOS=linux
            export GOARCH=arm64
            export CC=aarch64-linux-gnu-gcc
            export CXX=aarch64-linux-gnu-g++
            ;;
        "darwin/amd64")
            export GOOS=darwin
            export GOARCH=amd64
            # Use native compilers for macOS
            unset CC CXX
            ;;
        "darwin/arm64")
            export GOOS=darwin
            export GOARCH=arm64
            # Use native compilers for macOS
            unset CC CXX
            ;;
        "windows/amd64")
            export GOOS=windows
            export GOARCH=amd64
            export CC=x86_64-w64-mingw32-gcc
            export CXX=x86_64-w64-mingw32-g++
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} Unsupported platform: $os/$arch"
            exit 1
            ;;
    esac
    
    # Verify tools are available
    if [ -n "$CC" ] && ! command -v "$CC" >/dev/null 2>&1; then
        echo -e "${YELLOW}[WARN]${NC} Cross-compiler $CC not found. Installing..."
        install_cross_compilers
    fi
}

# Install cross-compilation tools
install_cross_compilers() {
    case "$(uname -s)" in
        Linux*)
            echo -e "${BLUE}[INFO]${NC} Installing cross-compilation tools..."
            sudo apt-get update
            sudo apt-get install -y \
                gcc-x86-64-linux-gnu \
                g++-x86-64-linux-gnu \
                gcc-aarch64-linux-gnu \
                g++-aarch64-linux-gnu \
                gcc-mingw-w64-x86-64 \
                g++-mingw-w64-x86-64
            ;;
        Darwin*)
            echo -e "${YELLOW}[WARN]${NC} Cross-compilation tools for macOS may need manual setup"
            ;;
        *)
            echo -e "${YELLOW}[WARN]${NC} Automatic cross-compiler installation not supported on $(uname -s)"
            ;;
    esac
}

# Build for specific platform
build_platform() {
    local os="$1"
    local arch="$2"
    local output_name="$PROJECT_NAME-$os-$arch"
    
    if [ "$os" = "windows" ]; then
        output_name="$output_name.exe"
    fi
    
    echo -e "${BLUE}[INFO]${NC} Building for $os/$arch..."
    
    # Create build and dist directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DIST_DIR"
    
    # Build Rust library if needed
    if [ ! -f "$PROJECT_ROOT/internal/cgo/libfast_context.so" ] && \
       [ ! -f "$PROJECT_ROOT/internal/cgo/libfast_context.dylib" ] && \
       [ ! -f "$PROJECT_ROOT/internal/cgo/fast_context.dll" ]; then
        echo -e "${YELLOW}[WARN]${NC} Rust library not found. Building..."
        cd "$PROJECT_ROOT/.." && cargo build --release
        cp target/release/libfast_context.so "$PROJECT_ROOT/internal/cgo/" 2>/dev/null || \
        cp target/release/libfast_context.dylib "$PROJECT_ROOT/internal/cgo/" 2>/dev/null || \
        cp target/release/fast_context.dll "$PROJECT_ROOT/internal/cgo/" 2>/dev/null || \
        echo -e "${YELLOW}[WARN]${NC} Failed to copy Rust library"
        cd "$PROJECT_ROOT"
    fi
    
    # Set build flags
    LDFLAGS="-s -w -X main.version=$VERSION"
    BUILD_TAGS=""
    
    # Platform-specific adjustments
    if [ "$os" = "linux" ]; then
        # Enable static linking for Linux
        LDFLAGS="$LDFLAGS -extldflags=-static"
        BUILD_TAGS="$BUILD_TAGS netgo osusergo"
    fi
    
    # Build the binary
    cd "$PROJECT_ROOT"
    CGO_ENABLED=1 GOOS="$os" GOARCH="$arch" \
        go build -tags="$BUILD_TAGS" -ldflags="$LDFLAGS" \
        -o "$DIST_DIR/$output_name" ./cmd/fast-context
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} Built $DIST_DIR/$output_name"
    else
        echo -e "${RED}[ERROR]${NC} Failed to build for $os/$arch"
        exit 1
    fi
}

# Create distribution package
create_package() {
    local os="$1"
    local arch="$2"
    
    echo -e "${BLUE}[INFO]${NC} Creating package for $os/$arch..."
    
    cd "$DIST_DIR"
    
    local package_name="$PROJECT_NAME-$VERSION-$os-$arch"
    local binary_name="$PROJECT_NAME-$os-$arch"
    
    if [ "$os" = "windows" ]; then
        binary_name="$binary_name.exe"
        # Create ZIP archive for Windows
        zip "$package_name.zip" \
            "$binary_name" \
            README.md \
            LICENSE \
            CHANGELOG.md 2>/dev/null || true
    else
        # Create tar.gz for Unix-like systems
        tar -czf "$package_name.tar.gz" \
            "$binary_name" \
            README.md \
            LICENSE \
            CHANGELOG.md 2>/dev/null || true
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} Package created: $package_name"
}

# Generate checksums
generate_checksums() {
    echo -e "${BLUE}[INFO]${NC} Generating checksums..."
    
    cd "$DIST_DIR"
    sha256sum *.tar.gz *.zip > "$PROJECT_NAME-$VERSION-checksums.txt" 2>/dev/null || true
    
    echo -e "${GREEN}[SUCCESS]${NC} Checksums generated"
}

# Verify builds
verify_builds() {
    echo -e "${BLUE}[INFO]${NC} Verifying builds..."
    
    cd "$DIST_DIR"
    
    for file in "$PROJECT_NAME"-*-*; do
        if [ -f "$file" ]; then
            if [[ "$file" == *.exe ]]; then
                # Windows executable
                file "$file" | grep -q "Windows" && \
                    echo -e "${GREEN}[OK]${NC} $file" || \
                    echo -e "${RED}[FAIL]${NC} $file"
            else
                # Unix executable
                file "$file" | grep -q "executable" && \
                    echo -e "${GREEN}[OK]${NC} $file" || \
                    echo -e "${RED}[FAIL]${NC} $file"
            fi
        fi
    done
}

# Show usage
show_usage() {
    echo "Fast-Context Go SDK Cross-Compilation Script"
    echo ""
    echo "Usage: $0 [target-os] [target-arch]"
    echo ""
    echo "Supported platforms:"
    for platform in "${SUPPORTED_PLATFORMS[@]}"; do
        echo "  $platform"
    done
    echo ""
    echo "Examples:"
    echo "  $0 linux amd64    # Build for Linux x86_64"
    echo "  $0 darwin arm64   # Build for macOS ARM64"
    echo "  $0 windows amd64  # Build for Windows x86_64"
    echo ""
    echo "Environment variables:"
    echo "  VERSION          - Override version from VERSION file"
    echo "  SKIP_RUST_BUILD  - Skip Rust library build (1 = skip)"
    echo "  SKIP_PACKAGE     - Skip package creation (1 = skip)"
    echo "  SKIP_CHECKSUMS   - Skip checksum generation (1 = skip)"
}

# Main execution
main() {
    # Parse command line arguments
    case "$1" in
        -h|--help|help)
            show_usage
            exit 0
            ;;
        "all")
            echo -e "${BLUE}[INFO]${NC} Building for all supported platforms..."
            for platform in "${SUPPORTED_PLATFORMS[@]}"; do
                os="${platform%/*}"
                arch="${platform#*/}"
                setup_cross_compilation "$os" "$arch"
                build_platform "$os" "$arch"
                if [ "${SKIP_PACKAGE:-0}" != "1" ]; then
                    create_package "$os" "$arch"
                fi
            done
            if [ "${SKIP_CHECKSUMS:-0}" != "1" ]; then
                generate_checksums
            fi
            verify_builds
            ;;
        *)
            if [ -n "$1" ] && [ -n "$2" ]; then
                TARGET_OS="$1"
                TARGET_ARCH="$2"
            fi
            
            echo -e "${BLUE}[INFO]${NC} Cross-compiling for $TARGET_OS/$TARGET_ARCH"
            
            # Verify platform is supported
            local platform="$TARGET_OS/$TARGET_ARCH"
            local supported=false
            for supported_platform in "${SUPPORTED_PLATFORMS[@]}"; do
                if [ "$supported_platform" = "$platform" ]; then
                    supported=true
                    break
                fi
            done
            
            if [ "$supported" = false ]; then
                echo -e "${RED}[ERROR]${NC} Unsupported platform: $platform"
                echo "Supported platforms: ${SUPPORTED_PLATFORMS[*]}"
                exit 1
            fi
            
            setup_cross_compilation "$TARGET_OS" "$TARGET_ARCH"
            build_platform "$TARGET_OS" "$TARGET_ARCH"
            
            if [ "${SKIP_PACKAGE:-0}" != "1" ]; then
                create_package "$TARGET_OS" "$TARGET_ARCH"
            fi
            
            if [ "${SKIP_CHECKSUMS:-0}" != "1" ]; then
                generate_checksums
            fi
            
            verify_builds
            ;;
    esac
    
    echo -e "${GREEN}[SUCCESS]${NC} Cross-compilation completed!"
}

# Run main function
main "$@"