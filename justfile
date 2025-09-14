# Justfile for OCTO CHIP-8 Emulator
# Run `just` to see all available commands

# Default command - show help
default:
    @just --list

# Build the project
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without making changes
fmt-check:
    cargo fmt -- --check

# Run all checks (formatting, linting, tests)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Show version information
version:
    cargo run -- version

# Show detailed version information
version-detailed:
    cargo run -- version --detailed

# Create a new release (updates version, commits, tags)
# Usage: just release patch|minor|major
release TYPE:
    #!/bin/bash
    set -euo pipefail

    # Check if working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        echo "Error: Working directory is not clean. Commit or stash changes first."
        exit 1
    fi

    # Get current version
    CURRENT=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "Current version: $CURRENT"

    # Parse version components
    IFS='.' read -r major minor patch <<< "$CURRENT"

    # Calculate new version
    case "{{TYPE}}" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            echo "Error: Invalid release type '{{TYPE}}'. Use: patch, minor, or major"
            exit 1
            ;;
    esac

    NEW_VERSION="$major.$minor.$patch"
    echo "New version: $NEW_VERSION"

    # Update Cargo.toml
    sed -i.bak "s/^version = .*/version = \"$NEW_VERSION\"/" Cargo.toml
    rm Cargo.toml.bak

    # Update README.md with new version
    just update-readme-version $NEW_VERSION

    # Commit and tag
    git add Cargo.toml README.md
    git commit -m "chore: bump version to $NEW_VERSION"
    git tag "v$NEW_VERSION"

    echo "✅ Release $NEW_VERSION created!"
    echo "📝 To publish: git push origin main --tags"

# Run the emulator with a ROM file (when implemented)
# Usage: just run path/to/rom.ch8
run ROM:
    cargo run -- run {{ROM}}

# Update README.md with new version
# Usage: just update-readme-version 0.1.4
update-readme-version VERSION:
    #!/bin/bash
    set -euo pipefail

    NEW_TAG="v{{VERSION}}"
    echo "Updating README.md to use version $NEW_TAG"

    # Use a simpler approach with targeted sed commands
    # Update only the lines in "From Latest Release" and "Updating" sections

    # Update the main installation command
    sed -i.bak "/### From Latest Release/,/### Updating/ s/--tag v[0-9]*\.[0-9]*\.[0-9]*/--tag $NEW_TAG/g" README.md

    # Update the updating section commands
    sed -i.bak "/### Updating/,/### From Specific Version/ s/--tag v[0-9]*\.[0-9]*\.[0-9]*/--tag $NEW_TAG/g" README.md

    # Clean up backup file
    rm README.md.bak

    echo "✅ README.md updated with version $NEW_TAG"
    echo "📝 Note: The 'From Specific Version' example was left unchanged as intended"

# Show git status and recent commits
status:
    @echo "=== Git Status ==="
    @git status --short
    @echo ""
    @echo "=== Recent Commits ==="
    @git log --oneline -5
    @echo ""
    @echo "=== Tags ==="
    @git tag -l | tail -5

# Install development dependencies
dev-setup:
    @echo "Installing development tools..."
    rustup component add rustfmt clippy
    @echo "✅ Development setup complete!"
    @echo "💡 Optional tools: cargo install cargo-watch just"
