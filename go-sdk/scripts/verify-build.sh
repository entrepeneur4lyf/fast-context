#!/bin/bash

# Fast-Context Go SDK Build Verification Script
# This script performs comprehensive build verification

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

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_TEST_NAMES=()

# Function to log test results
log_test() {
    local test_name="$1"
    local result="$2"
    local message="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    case "$result" in
        "PASS")
            PASSED_TESTS=$((PASSED_TESTS + 1))
            echo -e "${GREEN}[✓]${NC} $test_name: $message"
            ;;
        "FAIL")
            FAILED_TESTS=$((FAILED_TESTS + 1))
            FAILED_TEST_NAMES+=("$test_name")
            echo -e "${RED}[✗]${NC} $test_name: $message"
            ;;
        "SKIP")
            echo -e "${YELLOW}[~]${NC} $test_name: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[!]${NC} $test_name: $message"
            ;;
    esac
}

# Function to print final results
print_results() {
    echo ""
    echo -e "${BLUE}=== Build Verification Results ===${NC}"
    echo -e "Total tests: ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
    if [ $FAILED_TESTS -gt 0 ]; then
        echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"
        echo ""
        echo -e "${RED}Failed tests:${NC}"
        for test in "${FAILED_TEST_NAMES[@]}"; do
            echo -e "  - $test"
        done
    else
        echo -e "${GREEN}Failed: 0${NC}"
    fi
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✓ All build verification tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}✗ Build verification failed!${NC}"
        exit 1
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check build tools
check_build_tools() {
    echo -e "${BLUE}Checking build tools...${NC}"
    
    # Required tools
    tools=(
        "go:Go compiler"
        "git:Git version control"
        "gcc:C compiler"
        "make:Make build tool"
    )
    
    # Optional tools
    optional_tools=(
        "g++:C++ compiler"
        "cargo:Rust package manager"
        "strip:Binary stripper"
        "upx:UPX compressor"
        "docker:Docker container engine"
    )
    
    for tool in "${tools[@]}"; do
        cmd="${tool%%:*}"
        desc="${tool##*:}"
        
        if command_exists "$cmd"; then
            log_test "BuildTools_$cmd" "PASS" "$desc is available"
        else
            log_test "BuildTools_$cmd" "FAIL" "$desc is not available"
        fi
    done
    
    for tool in "${optional_tools[@]}"; do
        cmd="${tool%%:*}"
        desc="${tool##*:}"
        
        if command_exists "$cmd"; then
            log_test "BuildTools_$cmd" "PASS" "$desc is available"
        else
            log_test "BuildTools_$cmd" "WARN" "$desc is not available (optional)"
        fi
    done
}

# Function to check Go environment
check_go_environment() {
    echo -e "${BLUE}Checking Go environment...${NC}"
    
    # Check Go version
    if command_exists go; then
        go_version=$(go version)
        log_test "GoVersion" "PASS" "Go version: $go_version"
        
        # Check Go modules
        if [ -f "$PROJECT_ROOT/go.mod" ]; then
            if go mod verify >/dev/null 2>&1; then
                log_test "GoModules" "PASS" "Go modules verified"
            else
                log_test "GoModules" "FAIL" "Go modules verification failed"
            fi
        else
            log_test "GoModules" "FAIL" "go.mod not found"
        fi
        
        # Check GOPATH and GOROOT
        if [ -n "$GOPATH" ]; then
            log_test "GoPath" "PASS" "GOPATH: $GOPATH"
        else
            log_test "GoPath" "WARN" "GOPATH not set"
        fi
        
        if [ -n "$GOROOT" ]; then
            log_test "GoRoot" "PASS" "GOROOT: $GOROOT"
        else
            log_test "GoRoot" "WARN" "GOROOT not set"
        fi
    else
        log_test "GoEnvironment" "FAIL" "Go not found"
    fi
}

# Function to check project structure
check_project_structure() {
    echo -e "${BLUE}Checking project structure...${NC}"
    
    # Required directories
    required_dirs=(
        "cmd"
        "internal"
        "config"
        "query"
        "graph"
        "streaming"
        "filewatch"
        "export"
        "cli"
        "tests"
    )
    
    for dir in "${required_dirs[@]}"; do
        if [ -d "$PROJECT_ROOT/$dir" ]; then
            log_test "ProjectStructure_$dir" "PASS" "Directory $dir exists"
        else
            log_test "ProjectStructure_$dir" "FAIL" "Directory $dir missing"
        fi
    done
    
    # Required files
    required_files=(
        "go.mod"
        "go.sum"
        "Makefile"
        "README.md"
        "LICENSE"
        "main.go"
        "VERSION"
    )
    
    for file in "${required_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$file" ]; then
            log_test "ProjectStructure_$file" "PASS" "File $file exists"
        else
            log_test "ProjectStructure_$file" "FAIL" "File $file missing"
        fi
    done
}

# Function to check build process
check_build_process() {
    echo -e "${BLUE}Checking build process...${NC}"
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Test regular build
    build_output="$BUILD_DIR/fast-context-test"
    if go build -o "$build_output" ./cmd/fast-context >/dev/null 2>&1; then
        log_test "BuildProcess_Regular" "PASS" "Regular build successful"
        
        # Test binary execution
        if "$build_output" version >/dev/null 2>&1; then
            log_test "BuildProcess_Execution" "PASS" "Binary executes correctly"
        else
            log_test "BuildProcess_Execution" "FAIL" "Binary execution failed"
        fi
        
        # Clean up
        rm -f "$build_output"
    else
        log_test "BuildProcess_Regular" "FAIL" "Regular build failed"
    fi
    
    # Test race build
    build_output="$BUILD_DIR/fast-context-race"
    if go build -race -o "$build_output" ./cmd/fast-context >/dev/null 2>&1; then
        log_test "BuildProcess_Race" "PASS" "Race build successful"
        rm -f "$build_output"
    else
        log_test "BuildProcess_Race" "FAIL" "Race build failed"
    fi
    
    # Test CGO build
    if [ "$CGO_ENABLED" != "0" ]; then
        cgo_test_file="$PROJECT_ROOT/tests/build/test_cgo.go"
        if [ -f "$cgo_test_file" ]; then
            cgo_output="$BUILD_DIR/test-cgo"
            if go build -tags=cgo -o "$cgo_output" "$cgo_test_file" >/dev/null 2>&1; then
                log_test "BuildProcess_CGO" "PASS" "CGO build successful"
                rm -f "$cgo_output"
            else
                log_test "BuildProcess_CGO" "FAIL" "CGO build failed"
            fi
        else
            log_test "BuildProcess_CGO" "SKIP" "CGO test file not found"
        fi
    else
        log_test "BuildProcess_CGO" "SKIP" "CGO disabled"
    fi
}

# Function to check cross-compilation
check_cross_compilation() {
    echo -e "${BLUE}Checking cross-compilation...${NC}"
    
    # Skip if no cross-compilation tools
    if ! command_exists x86_64-linux-gnu-gcc; then
        log_test "CrossCompilation_Tools" "SKIP" "Cross-compilation tools not available"
        return
    fi
    
    platforms=(
        "linux:amd64:x86_64-linux-gnu-gcc"
        "linux:arm64:aarch64-linux-gnu-gcc"
    )
    
    for platform in "${platforms[@]}"; do
        IFS=':' read -r goos goarch cc <<< "$platform"
        
        build_output="$BUILD_DIR/fast-context-$goos-$goarch"
        if GOOS="$goos" GOARCH="$goarch" CC="$cc" CGO_ENABLED=1 \
           go build -o "$build_output" ./cmd/fast-context >/dev/null 2>&1; then
            log_test "CrossCompilation_${goos}_${goarch}" "PASS" "Cross-compilation for $goos/$goarch successful"
            rm -f "$build_output"
        else
            log_test "CrossCompilation_${goos}_${goarch}" "FAIL" "Cross-compilation for $goos/$goarch failed"
        fi
    done
}

# Function to check static linking
check_static_linking() {
    echo -e "${BLUE}Checking static linking...${NC}"
    
    # Only test on Linux
    if [ "$(uname -s)" != "Linux" ]; then
        log_test "StaticLinking_Platform" "SKIP" "Static linking test only runs on Linux"
        return
    fi
    
    # Skip if no cross-compilation tools
    if ! command_exists x86_64-linux-gnu-gcc; then
        log_test "StaticLinking_Tools" "SKIP" "Cross-compilation tools not available"
        return
    fi
    
    build_output="$BUILD_DIR/fast-context-static"
    if GOOS=linux GOARCH=amd64 CC=x86_64-linux-gnu-gcc CGO_ENABLED=1 \
       go build -tags="netgo,osusergo,static_build" \
       -ldflags="-s -w -linkmode external -extldflags '-static'" \
       -o "$build_output" ./cmd/fast-context >/dev/null 2>&1; then
        
        # Check if it's really static
        if file "$build_output" 2>/dev/null | grep -q "statically linked"; then
            log_test "StaticLinking_Check" "PASS" "Binary is statically linked"
            
            # Check with ldd
            if ldd "$build_output" 2>&1 | grep -q "not a dynamic executable"; then
                log_test "StaticLinking_LDD" "PASS" "No dynamic dependencies"
            else
                log_test "StaticLinking_LDD" "FAIL" "Has dynamic dependencies"
            fi
        else
            log_test "StaticLinking_Check" "FAIL" "Binary is not statically linked"
        fi
        
        rm -f "$build_output"
    else
        log_test "StaticLinking_Build" "FAIL" "Static build failed"
    fi
}

# Function to check tests
check_tests() {
    echo -e "${BLUE}Checking tests...${NC}"
    
    # Check if tests can be discovered
    if go test ./... -list >/dev/null 2>&1; then
        log_test "Tests_Discovery" "PASS" "Tests can be discovered"
    else
        log_test "Tests_Discovery" "FAIL" "Test discovery failed"
        return
    fi
    
    # Run a quick test without execution
    if go test ./... -run=^$ >/dev/null 2>&1; then
        log_test "Tests_Compile" "PASS" "Tests compile successfully"
    else
        log_test "Tests_Compile" "FAIL" "Test compilation failed"
    fi
    
    # Check build verification tests specifically
    if [ -f "$PROJECT_ROOT/tests/build/build_verification_test.go" ]; then
        if go test ./tests/build/ -v >/dev/null 2>&1; then
            log_test "Tests_BuildVerification" "PASS" "Build verification tests pass"
        else
            log_test "Tests_BuildVerification" "FAIL" "Build verification tests failed"
        fi
    else
        log_test "Tests_BuildVerification" "SKIP" "Build verification tests not found"
    fi
}

# Function to check linting
check_linting() {
    echo -e "${BLUE}Checking linting...${NC}"
    
    # Check go vet
    if go vet ./... >/dev/null 2>&1; then
        log_test "Linting_GoVet" "PASS" "go vet passed"
    else
        log_test "Linting_GoVet" "FAIL" "go vet failed"
    fi
    
    # Check gofmt
    if [ "$(gofmt -l . | wc -l)" -eq 0 ]; then
        log_test "Linting_GoFmt" "PASS" "Code is properly formatted"
    else
        log_test "Linting_GoFmt" "FAIL" "Code formatting issues found"
        log_test "Linting_GoFmt_Details" "WARN" "Files needing formatting: $(gofmt -l . | tr '\n' ' ')"
    fi
    
    # Check golangci-lint if available
    if command_exists golangci-lint; then
        if golangci-lint run >/dev/null 2>&1; then
            log_test "Linting_GolangCILint" "PASS" "golangci-lint passed"
        else
            log_test "Linting_GolangCILint" "FAIL" "golangci-lint failed"
        fi
    else
        log_test "Linting_GolangCILint" "SKIP" "golangci-lint not available"
    fi
}

# Function to check documentation
check_documentation() {
    echo -e "${BLUE}Checking documentation...${NC}"
    
    # Check README
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        if [ -s "$PROJECT_ROOT/README.md" ]; then
            log_test "Documentation_README" "PASS" "README.md exists and is not empty"
        else
            log_test "Documentation_README" "FAIL" "README.md is empty"
        fi
    else
        log_test "Documentation_README" "FAIL" "README.md not found"
    fi
    
    # Check LICENSE
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        if [ -s "$PROJECT_ROOT/LICENSE" ]; then
            log_test "Documentation_LICENSE" "PASS" "LICENSE exists and is not empty"
        else
            log_test "Documentation_LICENSE" "FAIL" "LICENSE is empty"
        fi
    else
        log_test "Documentation_LICENSE" "FAIL" "LICENSE not found"
    fi
    
    # Check Go doc
    if go doc ./cmd/fast-context >/dev/null 2>&1; then
        log_test "Documentation_GoDoc" "PASS" "Go documentation available"
    else
        log_test "Documentation_GoDoc" "FAIL" "Go documentation not available"
    fi
}

# Function to check version consistency
check_version_consistency() {
    echo -e "${BLUE}Checking version consistency...${NC}"
    
    # Check VERSION file
    if [ -f "$PROJECT_ROOT/VERSION" ]; then
        version=$(cat "$PROJECT_ROOT/VERSION")
        if [ -n "$version" ]; then
            log_test "VersionConsistency_FILE" "PASS" "VERSION file exists with content: $version"
        else
            log_test "VersionConsistency_FILE" "FAIL" "VERSION file is empty"
        fi
    else
        log_test "VersionConsistency_FILE" "FAIL" "VERSION file not found"
    fi
    
    # Build and check version
    build_output="$BUILD_DIR/fast-context-version"
    if go build -ldflags="-X main.version=$version" -o "$build_output" ./cmd/fast-context >/dev/null 2>&1; then
        if "$build_output" version 2>/dev/null | grep -q "$version"; then
            log_test "VersionConsistency_Build" "PASS" "Version in binary matches VERSION file"
        else
            log_test "VersionConsistency_Build" "FAIL" "Version in binary doesn't match VERSION file"
        fi
        rm -f "$build_output"
    else
        log_test "VersionConsistency_Build" "FAIL" "Failed to build version test"
    fi
}

# Function to check clean state
check_clean_state() {
    echo -e "${BLUE}Checking clean state...${NC}"
    
    # Check if git working directory is clean
    if command_exists git; then
        if [ -d "$PROJECT_ROOT/.git" ]; then
            if git diff --quiet 2>/dev/null; then
                log_test "CleanState_Git" "PASS" "Git working directory is clean"
            else
                log_test "CleanState_Git" "WARN" "Git working directory has uncommitted changes"
            fi
        else
            log_test "CleanState_Git" "SKIP" "Not a git repository"
        fi
    else
        log_test "CleanState_Git" "SKIP" "Git not available"
    fi
    
    # Check for temporary files
    temp_patterns=(
        "*.tmp"
        "*.temp"
        "*~"
        "*.swp"
        ".DS_Store"
        "Thumbs.db"
    )
    
    temp_found=false
    for pattern in "${temp_patterns[@]}"; do
        if find "$PROJECT_ROOT" -name "$pattern" -type f 2>/dev/null | head -1 | grep -q .; then
            temp_found=true
            break
        fi
    done
    
    if [ "$temp_found" = false ]; then
        log_test "CleanState_TempFiles" "PASS" "No temporary files found"
    else
        log_test "CleanState_TempFiles" "WARN" "Temporary files found"
    fi
}

# Main execution
main() {
    echo -e "${BLUE}Fast-Context Go SDK Build Verification${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Run all checks
    check_build_tools
    check_go_environment
    check_project_structure
    check_build_process
    check_cross_compilation
    check_static_linking
    check_tests
    check_linting
    check_documentation
    check_version_consistency
    check_clean_state
    
    # Print results
    print_results
}

# Handle signals
trap 'echo -e "\n${RED}Build verification interrupted!${NC}"; exit 1' INT TERM

# Handle help
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "Fast-Context Go SDK Build Verification Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -v, --verbose  Enable verbose output"
    echo ""
    echo "This script performs comprehensive build verification including:"
    echo "  - Build tools availability"
    echo "  - Go environment setup"
    echo "  - Project structure"
    echo "  - Build process"
    echo "  - Cross-compilation"
    echo "  - Static linking"
    echo "  - Tests"
    echo "  - Linting"
    echo "  - Documentation"
    echo "  - Version consistency"
    echo "  - Clean state"
    exit 0
fi

# Run main function
main "$@"