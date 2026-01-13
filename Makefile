# Synarion Engine - Development Commands
# Usage: make <target>

.PHONY: all build check test lint fmt doc clean ci help

# Default target
all: check lint test

# Build in debug mode
build:
	cargo build --workspace

# Build in release mode
release:
	cargo build --workspace --release

# Quick compilation check
check:
	cargo check --workspace --all-targets

# Run all tests
test:
	cargo test --workspace

# Run tests with output
test-verbose:
	cargo test --workspace -- --nocapture

# Clippy lints (pedantic)
lint:
	cargo clippy --workspace --all-targets -- \
		-D warnings \
		-D clippy::pedantic \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate

# Format code
fmt:
	cargo fmt --all

# Check formatting (CI)
fmt-check:
	cargo fmt --all -- --check

# Generate documentation
doc:
	cargo doc --workspace --no-deps

# Open documentation in browser
doc-open:
	cargo doc --workspace --no-deps --open

# Serve mdBook documentation
doc-book:
	cd docs && mdbook serve

# Build mdBook documentation
doc-book-build:
	cd docs && mdbook build

# Clean build artifacts
clean:
	cargo clean

# Full CI pipeline (what GitHub Actions runs)
ci: fmt-check lint test doc
	@echo "CI pipeline passed!"

# Security audit
audit:
	cargo audit

# Check for outdated dependencies
outdated:
	cargo outdated

# Update dependencies
update:
	cargo update

# Run a specific example
example:
	@echo "Usage: make example NAME=hello_triangle"
	@test -n "$(NAME)" && cargo run --example $(NAME) || echo "Please specify NAME=<example>"

# Help
help:
	@echo "Synarion Engine - Available targets:"
	@echo ""
	@echo "  build        Build in debug mode"
	@echo "  release      Build in release mode"
	@echo "  check        Quick compilation check"
	@echo "  test         Run all tests"
	@echo "  lint         Run clippy with pedantic lints"
	@echo "  fmt          Format all code"
	@echo "  fmt-check    Check formatting (CI)"
	@echo "  doc          Generate API documentation"
	@echo "  doc-open     Generate and open documentation"
	@echo "  doc-book     Serve mdBook documentation"
	@echo "  clean        Clean build artifacts"
	@echo "  ci           Run full CI pipeline"
	@echo "  audit        Security audit dependencies"
	@echo "  outdated     Check for outdated dependencies"
	@echo "  help         Show this help"
