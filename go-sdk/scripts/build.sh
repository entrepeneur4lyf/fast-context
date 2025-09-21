#!/bin/bash

# Fast-Context Go SDK Build Script
# This script handles building and testing the Go SDK

set -e

# Configuration
PROJECT_NAME="fast-context-go-sdk"
BUILD_DIR="build"
VERSION=${VERSION:-"dev"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
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

# Show help
show_help() {
    cat << EOF
Fast-Context Go SDK Build Script

Usage: $0 [command] [options]

Commands:
    build       Build the Go SDK
    test        Run tests
    clean       Clean build artifacts
    install     Install the CLI tool
    docker      Build Docker image
    release     Create a release (requires additional setup)
    help        Show this help message

Options:
    -v VERSION  Specify version (default: dev)
    -t TAG      Docker tag (default: latest)
    -r          Build with race detector
    -c          Build with coverage
    -h          Show this help message

Examples:
    $0 build                    # Build for current platform
    $0 build -v v1.0.0         # Build with version
    $0 test -r                  # Run tests with race detector
    $0 test -c                  # Run tests with coverage
    $0 docker -t v1.0.0         # Build Docker image with tag
    $0 release                  # Create a release

EOF
}

# Parse command line arguments
COMMAND="$1"
shift || true

VERSION_FLAG=""
RACE_FLAG=""
COVERAGE_FLAG=""
DOCKER_TAG="latest"

while getopts "v:t:rch" opt; do
    case $opt in
        v)
            VERSION_FLAG="$OPTARG"
            ;;
        t)
            DOCKER_TAG="$OPTARG"
            ;;
        r)
            RACE_FLAG="-race"
            ;;
        c)
            COVERAGE_FLAG="-coverprofile=coverage.out"
            ;;
        h)
            show_help
            exit 0
            ;;
        \?)
            log_error "Invalid option: -$OPTARG"
            show_help
            exit 1
            ;;
    esac
done

# Set version
if [ -n "$VERSION_FLAG" ]; then
    VERSION="$VERSION_FLAG"
fi

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    
    # Remove build directory
    rm -rf "$BUILD_DIR"
    
    # Remove binary files
    rm -f fast-context*
    
    # Remove coverage files
    rm -f coverage.out coverage.html
    
    # Remove test cache
    go clean -testcache
    
    log_success "Clean completed"
}

# Build the project
build() {
    log_info "Building $PROJECT_NAME version $VERSION..."
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Build flags
    LDFLAGS="-s -w -X main.version=$VERSION"
    
    # Build for current platform
    go build -ldflags="$LDFLAGS" -o "$BUILD_DIR/fast-context" ./cmd/fast-context
    
    if [ $? -eq 0 ]; then
        log_success "Build completed successfully"
        log_info "Binary: $BUILD_DIR/fast-context"
    else
        log_error "Build failed"
        exit 1
    fi
}

# Build for multiple platforms
build_all() {
    log_info "Building $PROJECT_NAME version $VERSION for all platforms..."
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Build flags
    LDFLAGS="-s -w -X main.version=$VERSION"
    
    platforms=(
        "linux/amd64"
        "linux/arm64"
        "darwin/amd64"
        "darwin/arm64"
        "windows/amd64"
    )
    
    for platform in "${platforms[@]}"; do
        GOOS=${platform%/*}
        GOARCH=${platform#*/}
        output_name="$BUILD_DIR/fast-context-$GOOS-$GOARCH"
        
        if [ "$GOOS" = "windows" ]; then
            output_name="$output_name.exe"
        fi
        
        log_info "Building for $platform..."
        GOOS=$GOOS GOARCH=$GOARCH go build -ldflags="$LDFLAGS" -o "$output_name" ./cmd/fast-context
        
        if [ $? -eq 0 ]; then
            log_success "Built $output_name"
        else
            log_error "Failed to build for $platform"
            exit 1
        fi
    done
    
    log_success "All platforms built successfully"
}

# Run tests
test() {
    log_info "Running tests..."
    
    # Run unit tests
    log_info "Running unit tests..."
    go test $RACE_FLAG $COVERAGE_FLAG -v ./... -short
    
    # Run integration tests
    log_info "Running integration tests..."
    go test $RACE_FLAG -v ./... -run Integration
    
    # Generate HTML coverage if requested
    if [ -n "$COVERAGE_FLAG" ] && [ -f "coverage.out" ]; then
        log_info "Generating coverage report..."
        go tool cover -html=coverage.out -o coverage.html
        log_success "Coverage report generated: coverage.html"
    fi
    
    log_success "All tests passed"
}

# Install the CLI tool
install() {
    log_info "Installing $PROJECT_NAME..."
    
    # Build first
    build
    
    # Install to GOPATH/bin
    go install -ldflags="-s -w -X main.version=$VERSION" ./cmd/fast-context
    
    if [ $? -eq 0 ]; then
        log_success "Installation completed successfully"
        log_info "Run 'fast-context --help' to get started"
    else
        log_error "Installation failed"
        exit 1
    fi
}

# Build Docker image
docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    log_info "Building Docker image..."
    
    # Build Docker image
    docker build -t "$PROJECT_NAME:$DOCKER_TAG" .
    
    if [ $? -eq 0 ]; then
        log_success "Docker image built successfully"
        log_info "Image: $PROJECT_NAME:$DOCKER_TAG"
        
        # Test the image
        log_info "Testing Docker image..."
        docker run --rm "$PROJECT_NAME:$DOCKER_TAG" version
    else
        log_error "Docker build failed"
        exit 1
    fi
}

# Run linting
lint() {
    log_info "Running linter..."
    
    if command -v golangci-lint &> /dev/null; then
        golangci-lint run
        log_success "Linting completed"
    else
        log_warning "golangci-lint not found, skipping linting"
    fi
}

# Format code
format() {
    log_info "Formatting code..."
    
    # Format Go code
    gofmt -s -w .
    
    # Format imports
    if command -v goimports &> /dev/null; then
        goimports -w .
    fi
    
    log_success "Code formatted successfully"
}

# Show version
version() {
    if [ -f "VERSION" ]; then
        cat VERSION
    else
        echo "$VERSION"
    fi
}

# Create release
release() {
    log_info "Creating release..."
    
    if [ ! -f "scripts/release.sh" ]; then
        log_error "Release script not found. Please run from the project root."
        exit 1
    fi
    
    # Run release script
    ./scripts/release.sh "$@"
}

# Main logic
case "$COMMAND" in
    "build")
        build
        ;;
    "build-all")
        build_all
        ;;
    "test")
        test
        ;;
    "clean")
        clean
        ;;
    "install")
        install
        ;;
    "docker")
        docker
        ;;
    "lint")
        lint
        ;;
    "format")
        format
        ;;
    "version")
        version
        ;;
    "release")
        release "$@"
        ;;
    "help"|"")
        show_help
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        show_help
        exit 1
        ;;
esac