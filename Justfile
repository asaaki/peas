# Justfile for peas development
# Install just: https://github.com/casey/just

# Default recipe - show available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Format code and run linter (checks formatting and runs clippy)
lint:
    cargo fmt
    cargo clippy -- -D warnings

# Auto-fix issues (fixes clippy issues, then formats)
fix:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt

# Run all checks (lint includes fmt, so just lint + test)
check: lint test

# Clean build artifacts
clean:
    cargo clean

# Install peas locally
install:
    cargo install --path .

# Run the TUI
tui:
    cargo run --release -- tui

# Prime - show agent instructions
prime:
    cargo run -- prime

# List all peas
list:
    cargo run -- list

# Show pea by ID
show id:
    cargo run -- show {{ id }}

# Create a new pea
create title type="task" status="todo":
    cargo run -- create "{{ title }}" --type {{ type }} --status {{ status }}

# Search peas
search query:
    cargo run -- search "{{ query }}"

# Generate documentation
doc:
    cargo doc --no-deps --open

# Run cargo publish dry-run to verify package
publish-check:
    cargo publish --dry-run

# Bump version to patch (0.1.x)
version-patch:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Current version: $current"
    major=$(echo $current | cut -d. -f1)
    minor=$(echo $current | cut -d. -f2)
    patch=$(echo $current | cut -d. -f3)
    new_patch=$((patch + 1))
    new_version="$major.$minor.$new_patch"
    echo "New version: $new_version"
    sed -i.bak "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    echo "Version bumped to $new_version"

# Bump version to minor (0.x.0)
version-minor:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Current version: $current"
    major=$(echo $current | cut -d. -f1)
    minor=$(echo $current | cut -d. -f2)
    new_minor=$((minor + 1))
    new_version="$major.$new_minor.0"
    echo "New version: $new_version"
    sed -i.bak "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    echo "Version bumped to $new_version"

# Bump version to major (x.0.0)
version-major:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Current version: $current"
    major=$(echo $current | cut -d. -f1)
    new_major=$((major + 1))
    new_version="$new_major.0.0"
    echo "New version: $new_version"
    sed -i.bak "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    echo "Version bumped to $new_version"

# Tag and push a release (use after version bump)
release message="Release":
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Creating release for version $version"
    git add Cargo.toml Cargo.lock
    git commit -m "{{ message }} v$version"
    git tag "v$version"
    git push origin main
    git push origin "v$version"
    echo "Released v$version"

# Start GraphQL server
serve port="4000":
    cargo run -- serve --port {{ port }}

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run clippy
watch-lint:
    cargo watch -x "clippy -- -D warnings"

# Generate a code coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Run benchmark tests (if any)
bench:
    cargo bench

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit (requires cargo-audit)
audit:
    cargo audit

# Show current version
version:
    @grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2

# Full CI check (what CI would run)
ci: lint test
    cargo build --release
    cargo publish --dry-run

# Development workflow - lint (includes format) and test
dev: lint test
    @echo "✓ All checks passed"
