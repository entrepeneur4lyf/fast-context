.PHONY: build clean test test-race test-unit test-integration test-all lint fmt fmt-check vet install-deps build-rust help

# Default target
all: build

# Build the Go SDK
build: build-rust
	go build -v ./...

# Build Rust library
build-rust:
	cargo build --release --manifest-path ../Cargo.toml
	cp ../target/release/libfast_context.so internal/cgo/ 2>/dev/null || cp ../target/release/libfast_context.dylib internal/cgo/ 2>/dev/null || cp ../target/release/fast_context.dll internal/cgo/ 2>/dev/null || echo "Rust library not found, please build it first"

# Clean build artifacts
clean:
	go clean -cache -testcache -modcache
	rm -f internal/cgo/libfast_context.*
	rm -rf build/ dist/

# Run all tests
test: test-all

# Run all tests with race detector
test-race: build
	go test -race -v ./...

# Run unit tests only
test-unit: build
	go test -v ./fastcontext/ ./config/ ./internal/cgo/ ./query/ ./graph/ ./streaming/ ./filewatch/

# Run integration tests only
test-integration: build
	go test -v ./tests/...

# Run all tests (unit + integration)
test-all: build
	go test -v ./...

# Run linting
lint:
	golangci-lint run

# Format code
fmt:
	go fmt ./...

# Check code formatting
fmt-check:
	@scripts/check-fmt.sh

# Run go vet
vet:
	go vet ./...

# Install development dependencies
install-deps:
	go install golangci-lint/cmd/golangci-lint@latest
	go install github.com/stretchr/testify@latest
	go mod tidy

# Build examples
examples: build
	go build -o bin/basic ./examples/basic/
	go build -o bin/streaming ./examples/streaming/
	go build -o bin/advanced ./examples/advanced/

# Build CLI tool
cli: build
	go build -o bin/fast-context ./cmd/fast-context/

# Run benchmarks
bench: build
	go test -bench=. -benchmem ./...

# Run benchmarks with specific patterns
bench-unit: build
	go test -bench=. -benchmem ./fastcontext/ ./query/ ./graph/

# Run integration benchmarks
bench-integration: build
	go test -bench=. -benchmem ./tests/...

# Generate documentation
docs:
	godoc -http=:6060

# Generate test coverage report
coverage: build
	go test -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html
	go tool cover -func=coverage.out

# Run tests with verbose output
test-verbose: build
	go test -v -count=1 ./...

# Run specific test package
test-pkg: build
	go test -v ./fastcontext/

# Clean and rebuild
rebuild: clean build

# Show help
help:
	@echo "Available targets:"
	@echo "  build           - Build the Go SDK and Rust library"
	@echo "  clean           - Clean build artifacts"
	@echo "  test            - Run all tests"
	@echo "  test-race       - Run tests with race detector"
	@echo "  test-unit       - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-all        - Run all tests (unit + integration)"
	@echo "  test-verbose    - Run tests with verbose output"
	@echo "  test-pkg        - Run specific test package"
	@echo "  coverage        - Generate test coverage report"
	@echo "  lint            - Run linter"
	@echo "  fmt             - Format code"
	@echo "  fmt-check       - Check code formatting"
	@echo "  vet             - Run go vet"
	@echo "  install-deps    - Install development dependencies"
	@echo "  examples        - Build examples"
	@echo "  cli             - Build CLI tool"
	@echo "  bench           - Run benchmarks"
	@echo "  bench-unit      - Run unit benchmarks"
	@echo "  bench-integration - Run integration benchmarks"
	@echo "  docs            - Generate documentation"
	@echo "  rebuild         - Clean and rebuild"
	@echo "  help            - Show this help message"