#!/bin/bash

# Fast-Context Go SDK Release Automation Script
# This script automates the release process for the Go SDK

set -e

# Configuration
PROJECT_NAME="fast-context-go-sdk"
GITHUB_REPO="fast-context/go-sdk"
VERSION_FILE="VERSION"
CHANGELOG_FILE="CHANGELOG.md"
DOCKER_IMAGE="fastcontext/go-sdk"

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [ ! -f "go.mod" ]; then
        log_error "Not in Go SDK directory. Please run this script from the go-sdk directory."
        exit 1
    fi
    
    # Check if git is clean
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Please commit all changes."
        exit 1
    fi
    
    # Check if GitHub CLI is available
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI not found. Some features may not work."
    fi
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        log_warning "Docker not found. Docker image build will be skipped."
    fi
    
    log_success "Prerequisites check passed"
}

# Get version from VERSION file or prompt user
get_version() {
    if [ -f "$VERSION_FILE" ]; then
        VERSION=$(cat "$VERSION_FILE")
        log_info "Found version $VERSION in $VERSION_FILE"
    else
        read -p "Enter version (e.g., v1.0.0): " VERSION
        if [[ ! $VERSION =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            log_error "Invalid version format. Use v1.0.0 format."
            exit 1
        fi
        echo "$VERSION" > "$VERSION_FILE"
        log_info "Version $VERSION saved to $VERSION_FILE"
    fi
}

# Update changelog
update_changelog() {
    if [ ! -f "$CHANGELOG_FILE" ]; then
        log_info "Creating $CHANGELOG_FILE..."
        cat > "$CHANGELOG_FILE" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [$VERSION] - $(date +%Y-%m-%d)

### Added
- Initial release of Fast-Context Go SDK
- Comprehensive CGO integration with Rust core
- Symbol analysis and dependency extraction
- Graph algorithms and semantic search
- Streaming analysis with progress tracking
- File watching capabilities
- CLI application with multiple commands
- Export functionality for various formats
- Configuration file support (YAML, JSON, TOML)
- Structured logging and metrics collection
- Performance profiling and optimization
- Security enhancements and input validation

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Added comprehensive security validation for CGO interface
- Implemented path traversal prevention
- Added null byte injection protection
- Enhanced memory safety and bounds checking
- Added resource limit controls

EOF
    else
        log_info "Updating $CHANGELOG_FILE for version $VERSION..."
        # Add new version section at the top
        sed -i "s/^## \[Unreleased\]/## [$VERSION] - $(date +%Y-%m-%d)\n\n### Added\n- Release version $VERSION\n\n## [Unreleased]/" "$CHANGELOG_FILE"
    fi
    
    log_success "Changelog updated"
}

# Build the project
build_project() {
    log_info "Building project..."
    
    # Clean previous builds
    rm -f fast-context*
    
    # Build for different platforms
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
        output_name="fast-context-$GOOS-$GOARCH"
        
        if [ "$GOOS" = "windows" ]; then
            output_name="$output_name.exe"
        fi
        
        log_info "Building for $platform..."
        GOOS=$GOOS GOARCH=$GOARCH go build -ldflags="-s -w" -o "$output_name" ./cmd/fast-context
        
        if [ $? -eq 0 ]; then
            log_success "Built $output_name"
        else
            log_error "Failed to build for $platform"
            exit 1
        fi
    done
    
    log_success "All builds completed successfully"
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    # Run unit tests
    go test -v ./... -short
    
    # Run tests with race detection
    go test -v ./... -race -short
    
    # Run integration tests
    go test -v ./... -run Integration
    
    # Generate coverage report
    go test -v -coverprofile=coverage.out ./...
    go tool cover -html=coverage.out -o coverage.html
    
    log_success "All tests passed"
}

# Build Docker image
build_docker() {
    if ! command -v docker &> /dev/null; then
        log_warning "Docker not available, skipping Docker build"
        return
    fi
    
    log_info "Building Docker image..."
    
    # Build Docker image
    docker build -t "$DOCKER_IMAGE:$VERSION" .
    docker build -t "$DOCKER_IMAGE:latest" .
    
    # Test Docker image
    log_info "Testing Docker image..."
    docker run --rm "$DOCKER_IMAGE:$VERSION" version
    
    log_success "Docker image built and tested successfully"
}

# Create GitHub release
create_github_release() {
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI not available, skipping GitHub release"
        return
    fi
    
    log_info "Creating GitHub release..."
    
    # Create release notes
    cat > release-notes.md << EOF
# $PROJECT_NAME $VERSION

## Installation

\`\`\`bash
# Install using Go
go install github.com/fast-context/go-sdk/cmd/fast-context@$VERSION

# Or download binary for your platform
\`\`\`

## Changes

$(sed -n "/## \\[$VERSION\\]/,/## /p" "$CHANGELOG_FILE" | head -n -1)
EOF
    
    # Create release
    gh release create "$VERSION" \
        --title "$PROJECT_NAME $VERSION" \
        --notes-file release-notes.md \
        fast-context-* \
        || log_warning "GitHub release creation failed"
    
    # Clean up
    rm -f release-notes.md
    
    log_success "GitHub release created successfully"
}

# Update documentation
update_documentation() {
    log_info "Updating documentation..."
    
    # Update version in README if it exists
    if [ -f "README.md" ]; then
        # Update version references
        sed -i "s/fast-context@v[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+/fast-context@$VERSION/g" README.md
        log_success "Documentation updated"
    fi
}

# Tag the release
tag_release() {
    log_info "Tagging release..."
    
    # Create git tag
    git tag -a "$VERSION" -m "Release $VERSION"
    
    # Push tag
    git push origin "$VERSION"
    
    log_success "Release tagged successfully"
}

# Main release process
main() {
    log_info "Starting $PROJECT_NAME release process..."
    
    check_prerequisites
    get_version
    update_changelog
    run_tests
    build_project
    build_docker
    update_documentation
    tag_release
    create_github_release
    
    log_success "$PROJECT_NAME $VERSION released successfully!"
    log_info "Release artifacts:"
    ls -la fast-context-*
    
    log_info "Next steps:"
    echo "  1. Verify the release on GitHub"
    echo "  2. Update documentation if needed"
    echo "  3. Announce the release"
    echo "  4. Monitor for issues"
}

# Parse command line arguments
while getopts "h" opt; do
    case $opt in
        h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  -h  Show this help message"
            echo ""
            echo "This script automates the release process for the Fast-Context Go SDK."
            echo "It will build the project, run tests, create Docker images, and create a GitHub release."
            exit 0
            ;;
        \?)
            log_error "Invalid option: -$OPTARG"
            exit 1
            ;;
    esac
done

# Run main function
main "$@"