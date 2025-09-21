# Multi-stage build for Go SDK
FROM golang:1.21-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git cargo rust

# Set working directory
WORKDIR /app

# Copy go mod files
COPY go.mod go.sum ./

# Download dependencies
RUN go mod download

# Copy source code
COPY . .

# Build the application
RUN CGO_ENABLED=0 go build -a -installsuffix cgo -ldflags="-s -w" -o fast-context ./cmd/fast-context

# Final stage
FROM alpine:latest

# Install ca-certificates for HTTPS
RUN apk --no-cache add ca-certificates tzdata

# Create non-root user
RUN addgroup -g 1000 -S appgroup && \
    adduser -u 1000 -S appuser -G appgroup

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/fast-context .

# Copy configuration files
COPY --from=builder /app/config/ ./config/

# Set permissions
RUN chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Expose port for HTTP API (if implemented)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ./fast-context version || exit 1

# Default command
CMD ["./fast-context", "serve"]