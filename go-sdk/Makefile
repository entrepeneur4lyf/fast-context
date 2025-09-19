.PHONY: build clean test test-race lint fmt fmt-check vet install-deps build-rust help

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

# Run tests
test: build
	go test -v ./...

# Run tests with race detector
test-race: build
	go test -race -v ./...

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
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest

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

# Generate documentation
docs:
	godoc -http=:6060

# Show help
help:
	@echo "Available targets:"
	@echo "  build          - Build the Go SDK and Rust library"
	@echo "  clean          - Clean build artifacts"
	@echo "  test           - Run tests"
	@echo "  test-race      - Run tests with race detector"
	@echo "  lint           - Run linter"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  vet            - Run go vet"
	@echo "  install-deps   - Install development dependencies"
	@echo "  examples       - Build examples"
	@echo "  cli            - Build CLI tool"
	@echo "  bench          - Run benchmarks"
	@echo "  docs           - Generate documentation"
	@echo "  help           - Show this help message"