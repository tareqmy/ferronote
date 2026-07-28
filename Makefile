.PHONY: all help build run test fmt clippy check release publish clean lint fmt-check ci

# Default target
all: build

help:
	@echo "Available commands:"
	@echo "  make build      - Build the project in debug mode"
	@echo "  make run        - Run the project"
	@echo "  make test       - Run tests"
	@echo "  make fmt        - Format the code using cargo fmt"
	@echo "  make fmt-check  - Check code formatting"
	@echo "  make clippy     - Run clippy for linting"
	@echo "  make lint       - Run strict clippy linting for CI"
	@echo "  make check      - Check the code for compilation errors without building"
	@echo "  make release    - Build the project in release mode"
	@echo "  make publish    - Publish to crates.io"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make ci         - Run CI pipeline (fmt-check, lint, test)"

build:
	cargo build

run:
	cargo run

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

check:
	cargo check

release:
	cargo build --release

publish:
	cargo publish

clean:
	cargo clean

lint:
	cargo clippy --workspace --all-targets -- -D warnings -A clippy::collapsible_if

fmt-check:
	cargo fmt -- --check

ci: fmt-check lint test
