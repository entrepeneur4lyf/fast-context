# Configuration
PROJECT_NAME := fast-context-go-sdk
VERSION := $(shell cat VERSION 2>/dev/null || echo "dev")
BUILD_DIR := build
DIST_DIR := dist
BIN_NAME := fast-context

# Go configuration
GO := go
GOOS := $(shell go env GOOS)
GOARCH := $(shell go env GOARCH)
CGO_ENABLED := 1

# Build flags
LDFLAGS := -s -w -X main.version=$(VERSION)
BUILD_TAGS := ""

# Cross-compilation targets
PLATFORMS := linux/amd64 linux/arm64 darwin/amd64 darwin/arm64 windows/amd64

# Docker configuration
DOCKER_IMAGE := $(PROJECT_NAME)
DOCKER_TAG := $(VERSION)

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

.PHONY: build clean test test-race test-unit test-integration test-all lint fmt fmt-check vet install-deps build-rust help build-all build-static build-debug build-race install verify docs deps deps-update release checksums ci-install ci-test ci-build run version dev watch

# Default target
all: build

# Build with release flags
build-release:
	CGO_ENABLED=1 $(GO) build -ldflags="$(LDFLAGS)" -trimpath -o $(BUILD_DIR)/$(BIN_NAME) ./cmd/fast-context

# Build with race detector
build-race:
	CGO_ENABLED=1 $(GO) build -race -ldflags="$(LDFLAGS)" -o $(BUILD_DIR)/$(BIN_NAME)-race ./cmd/fast-context

# Build debug version
build-debug:
	CGO_ENABLED=1 $(GO) build -gcflags="-N -l" -ldflags="$(LDFLAGS)" -o $(BUILD_DIR)/$(BIN_NAME)-debug ./cmd/fast-context

# Cross-compilation targets
build-all: ## Build for all supported platforms
	@echo "$(BLUE)[INFO]$(NC) Building $(PROJECT_NAME) for all platforms..."
	@mkdir -p $(DIST_DIR)
	@for platform in $(PLATFORMS); do \
		GOOS=$${platform%/*} ; \
		GOARCH=$${platform#*/} ; \
		output_name=$(DIST_DIR)/$(BIN_NAME)-$$GOOS-$$GOARCH ; \
		if [ "$$GOOS" = "windows" ]; then \
			output_name=$$output_name.exe ; \
		fi ; \
		echo "$(BLUE)[INFO]$(NC) Building for $$platform..." ; \
		GOOS=$$GOOS GOARCH=$$GOARCH CGO_ENABLED=1 $(GO) build -tags="$(BUILD_TAGS)" -ldflags="$(LDFLAGS)" -o $$output_name ./cmd/fast-context || exit 1; \
		echo "$(GREEN)[SUCCESS]$(NC) Built $$output_name" ; \
	done

# Build static binaries (Linux only)
build-static: ## Build static binaries for Linux
	@echo "$(BLUE)[INFO]$(NC) Building static $(PROJECT_NAME) for Linux..."
	@mkdir -p $(DIST_DIR)
	CGO_ENABLED=1 GOOS=linux GOARCH=amd64 $(GO) build -tags="$(BUILD_TAGS)" -ldflags="$(LDFLAGS) -extldflags=-static" -o $(DIST_DIR)/$(BIN_NAME)-linux-amd64-static ./cmd/fast-context
	CGO_ENABLED=1 GOOS=linux GOARCH=arm64 $(GO) build -tags="$(BUILD_TAGS)" -ldflags="$(LDFLAGS) -extldflags=-static" -o $(DIST_DIR)/$(BIN_NAME)-linux-arm64-static ./cmd/fast-context
	@echo "$(GREEN)[SUCCESS]$(NC) Static binaries built"

# Build for specific platform
build-platform: ## Build for specific platform (usage: make build-platform GOOS=linux GOARCH=amd64)
ifndef GOOS
	$(error GOOS is required)
endif
ifndef GOARCH
	$(error GOARCH is required)
endif
	@echo "$(BLUE)[INFO]$(NC) Building $(PROJECT_NAME) for $(GOOS)/$(GOARCH)..."
	@mkdir -p $(DIST_DIR)
	output_name=$(DIST_DIR)/$(BIN_NAME)-$(GOOS)-$(GOARCH)
	ifeq ($(GOOS),windows)
		output_name=$(output_name).exe
	endif
	CGO_ENABLED=1 GOOS=$(GOOS) GOARCH=$(GOARCH) $(GO) build -tags="$(BUILD_TAGS)" -ldflags="$(LDFLAGS)" -o $(output_name) ./cmd/fast-context
	@echo "$(GREEN)[SUCCESS]$(NC) Built $(output_name)"

# Create distribution package
dist: build-all ## Create distribution packages
	@echo "$(BLUE)[INFO]$(NC) Creating distribution packages..."
	@mkdir -p $(DIST_DIR)
	# Create archives for each platform
	@for platform in $(PLATFORMS); do \
		GOOS=$${platform%/*} ; \
		GOARCH=$${platform#*/} ; \
		if [ "$$GOOS" = "windows" ]; then \
			cd $(DIST_DIR) && zip $(PROJECT_NAME)-$(VERSION)-$$GOOS-$$GOARCH.zip $(BIN_NAME)-$$GOOS-$$GOARCH.exe README.md LICENSE; \
		else \
			cd $(DIST_DIR) && tar -czf $(PROJECT_NAME)-$(VERSION)-$$GOOS-$$GOARCH.tar.gz $(BIN_NAME)-$$GOOS-$$GOARCH README.md LICENSE; \
		fi ; \
	done
	@echo "$(GREEN)[SUCCESS]$(NC) Distribution packages created"

# Generate checksums
checksums: dist ## Generate checksums for distribution files
	@echo "$(BLUE)[INFO]$(NC) Generating checksums..."
	@cd $(DIST_DIR) && sha256sum $(PROJECT_NAME)-$(VERSION)-*.tar.gz $(PROJECT_NAME)-$(VERSION)-*.zip > $(PROJECT_NAME)-$(VERSION)-checksums.txt
	@echo "$(GREEN)[SUCCESS]$(NC) Checksums generated"

# Docker build
docker-build: ## Build Docker image
	@echo "$(BLUE)[INFO]$(NC) Building Docker image..."
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	docker tag $(DOCKER_IMAGE):$(DOCKER_TAG) $(DOCKER_IMAGE):latest
	@echo "$(GREEN)[SUCCESS]$(NC) Docker image built"

# Docker run
docker-run: docker-build ## Run Docker container
	@echo "$(BLUE)[INFO]$(NC) Running Docker container..."
	docker run --rm -v "$(PWD):/workspace" $(DOCKER_IMAGE):$(DOCKER_TAG)

# Install to GOPATH
install: build ## Install binary to GOPATH
	@echo "$(BLUE)[INFO]$(NC) Installing $(PROJECT_NAME)..."
	$(GO) install -ldflags="$(LDFLAGS)" ./cmd/fast-context
	@echo "$(GREEN)[SUCCESS]$(NC) Installed to GOPATH"

# Install to local bin
install-local: build ## Install binary to local bin
	@echo "$(BLUE)[INFO]$(NC) Installing $(PROJECT_NAME) to local bin..."
	@mkdir -p $(HOME)/.local/bin
	cp $(BUILD_DIR)/$(BIN_NAME) $(HOME)/.local/bin/
	@echo "$(GREEN)[SUCCESS]$(NC) Installed to $(HOME)/.local/bin/$(BIN_NAME)"

# Build verification
verify-build: build-all ## Verify all builds work
	@echo "$(BLUE)[INFO]$(NC) Verifying builds..."
	@for platform in $(PLATFORMS); do \
		GOOS=$${platform%/*} ; \
		GOARCH=$${platform#*/} ; \
		output_name=$(DIST_DIR)/$(BIN_NAME)-$$GOOS-$$GOARCH ; \
		if [ "$$GOOS" = "windows" ]; then \
			output_name=$$output_name.exe ; \
		fi ; \
		echo "$(BLUE)[INFO]$(NC) Testing $$platform..." ; \
		if [ -f $$output_name ]; then \
			if [ "$$GOOS" = "windows" ]; then \
				file $$output_name | grep -q "Windows" && echo "$(GREEN)[OK]$(NC) $$output_name" || echo "$(RED)[FAIL]$(NC) $$output_name"; \
			else \
				file $$output_name | grep -q "executable" && echo "$(GREEN)[OK]$(NC) $$output_name" || echo "$(RED)[FAIL]$(NC) $$output_name"; \
			fi ; \
		else \
			echo "$(RED)[FAIL]$(NC) $$output_name not found" ; \
		fi ; \
	done

# Build Rust library
build-rust:
	cargo build --release --manifest-path ../Cargo.toml
	cp ../target/release/libfast_context.so internal/cgo/ 2>/dev/null || cp ../target/release/libfast_context.dylib internal/cgo/ 2>/dev/null || cp ../target/release/fast_context.dll internal/cgo/ 2>/dev/null || echo "Rust library not found, please build it first"

# Clean build artifacts
clean:
	@echo "$(BLUE)[INFO]$(NC) Cleaning build artifacts..."
	$(GO) clean -cache -testcache -modcache
	rm -f internal/cgo/libfast_context.*
	rm -rf $(BUILD_DIR)/ $(DIST_DIR)/
	rm -f *.prof *.test *.out coverage.html coverage.out
	@echo "$(GREEN)[SUCCESS]$(NC) Cleaned"

# Deep clean (including Rust)
clean-deep: clean
	@echo "$(BLUE)[INFO]$(NC) Deep cleaning (including Rust)..."
	cd .. && cargo clean
	rm -rf ../target/
	@echo "$(GREEN)[SUCCESS]$(NC) Deep cleaned"

# Clean dist only
clean-dist:
	@echo "$(BLUE)[INFO]$(NC) Cleaning dist..."
	rm -rf $(DIST_DIR)/
	@echo "$(GREEN)[SUCCESS]$(NC) Dist cleaned"

# Build Rust library
build-rust:
	@echo "$(BLUE)[INFO]$(NC) Building Rust library..."
	cargo build --release --manifest-path ../Cargo.toml
	@cp ../target/release/libfast_context.so internal/cgo/ 2>/dev/null || \
	@cp ../target/release/libfast_context.dylib internal/cgo/ 2>/dev/null || \
	@cp ../target/release/fast_context.dll internal/cgo/ 2>/dev/null || \
	echo "$(YELLOW)[WARN]$(NC) Rust library not found, please build it first"
	@echo "$(GREEN)[SUCCESS]$(NC) Rust library built"

# Build Rust library with debug info
build-rust-debug:
	@echo "$(BLUE)[INFO]$(NC) Building Rust library (debug)..."
	cargo build --manifest-path ../Cargo.toml
	@cp ../target/debug/libfast_context.so internal/cgo/ 2>/dev/null || \
	@cp ../target/debug/libfast_context.dylib internal/cgo/ 2>/dev/null || \
	@cp ../target/debug/fast_context.dll internal/cgo/ 2>/dev/null || \
	echo "$(YELLOW)[WARN]$(NC) Rust library not found, please build it first"
	@echo "$(GREEN)[SUCCESS]$(NC) Rust library built (debug)"

# Update dependencies
deps-update:
	@echo "$(BLUE)[INFO]$(NC) Updating dependencies..."
	$(GO) get -u ./...
	$(GO) mod tidy
	$(GO) mod download
	@echo "$(GREEN)[SUCCESS]$(NC) Dependencies updated"

# Download dependencies
deps-download:
	@echo "$(BLUE)[INFO]$(NC) Downloading dependencies..."
	$(GO) mod download
	$(GO) mod verify
	@echo "$(GREEN)[SUCCESS]$(NC) Dependencies downloaded"

# Vendor dependencies
deps-vendor:
	@echo "$(BLUE)[INFO]$(NC) Vendoring dependencies..."
	$(GO) mod vendor
	@echo "$(GREEN)[SUCCESS]$(NC) Dependencies vendored"

# Run with development configuration
dev: build
	@echo "$(BLUE)[INFO]$(NC) Starting development server..."
	./$(BUILD_DIR)/$(BIN_NAME) --log-level debug --config .fast-context-dev.yaml

# Watch for changes and rebuild
watch:
	@echo "$(BLUE)[INFO]$(NC) Watching for changes..."
	@if command -v air >/dev/null 2>&1; then \
		air -c .air.toml; \
	else \
		echo "$(YELLOW)[WARN]$(NC) air not found. Install with: go install github.com/cosmtrek/air@latest"; \
	fi

# Development shell
dev-shell:
	@echo "$(BLUE)[INFO]$(NC) Starting development shell..."
	@bash --rcfile <(echo "export PS1='[fast-context-dev] $$ '")

# CI build targets
ci-build: clean build-all
	@echo "$(GREEN)[SUCCESS]$(NC) CI build completed"

ci-test: clean test-race coverage
	@echo "$(GREEN)[SUCCESS]$(NC) CI tests completed"

ci-verify: clean fmt-check vet lint test-race coverage
	@echo "$(GREEN)[SUCCESS]$(NC) CI verification completed"

ci-release: clean build-all dist checksums
	@echo "$(GREEN)[SUCCESS]$(NC) CI release completed"

# Version management
version: ## Show version information
	@echo "$(BLUE)[INFO]$(NC) $(PROJECT_NAME) version information"
	@echo "Version: $(VERSION)"
	@echo "Build: $(shell date -u +'%Y-%m-%dT%H:%M:%SZ')"
	@echo "Go: $(shell $(GO) version)"
	@echo "Platform: $(shell uname -s)/$(shell uname -m)"

# Bump version
bump-patch: ## Bump patch version
	@echo "$(BLUE)[INFO]$(NC) Bumping patch version..."
	@echo $$(($(subst .,,$(VERSION)) + 1)) | sed 's/\(..\)/\1./g' > VERSION
	@echo "$(GREEN)[SUCCESS]$(NC) Version bumped to $$(cat VERSION)"

bump-minor: ## Bump minor version
	@echo "$(BLUE)[INFO]$(NC) Bumping minor version..."
	@echo "$(VERSION)" | awk -F. '{print $$1".$$2+1"."$$3}' | bc > VERSION
	@echo "$(GREEN)[SUCCESS]$(NC) Version bumped to $$(cat VERSION)"

bump-major: ## Bump major version
	@echo "$(BLUE)[INFO]$(NC) Bumping major version..."
	@echo "$(VERSION)" | awk -F. '{print $$1+1".0.0"}' | bc > VERSION
	@echo "$(GREEN)[SUCCESS]$(NC) Version bumped to $$(cat VERSION)"

# Quick development build
quick: build
	@echo "$(GREEN)[SUCCESS]$(NC) Quick build completed"

# Quick test
quick-test: test-unit
	@echo "$(GREEN)[SUCCESS]$(NC) Quick tests completed"

# Show environment info
env: ## Show environment information
	@echo "$(BLUE)[INFO]$(NC) Environment information"
	@echo "Go version: $(shell $(GO) version)"
	@echo "GOOS: $(GOOS)"
	@echo "GOARCH: $(GOARCH)"
	@echo "CGO_ENABLED: $(CGO_ENABLED)"
	@echo "GOPATH: $(GOPATH)"
	@echo "GOROOT: $(GOROOT)"
	@echo "Working directory: $(PWD)"
	@echo "Available Rust: $(shell which rustc)"
	@echo "Rust version: $(shell rustc --version 2>/dev/null || echo 'not found')"

# Show help
help: ## Show this help message
	@echo "$(BLUE)Fast-Context Go SDK - Available targets:$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "$(YELLOW)%-20s $(GREEN)%s$(NC)\n", "Target", "Description"} /^[a-zA-Z_-]+:.*?##/ { printf "$(YELLOW)%-20s $(GREEN)%s$(NC)\n", $$1, $$2 }' $(MAKEFILE_LIST)

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