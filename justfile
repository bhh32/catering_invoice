# Justfile for Invoice Flatpak packaging

# Default recipe - shows available commands
default:
    @just --list

# Variables

app_id := "com.bhh32.CateringInvoice"
manifest := "com.bhh32.CateringInvoice.yaml"
build_dir := "build-dir"
repo_dir := "repo"
bundle_file := "catering_invoice.flatpak"

# Install required Flatpak runtimes and SDKs
install-runtime:
    @echo "Installing Flatpak runtimes..."
    flatpak install -y flathub org.freedesktop.Platform//24.08
    flatpak install -y flathub org.freedesktop.Sdk//24.08
    flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08

# Check if runtimes are installed
check-runtime:
    @echo "Checking for required runtimes..."
    @flatpak list --runtime | grep "org.freedesktop.Platform.*24.08" || (echo "Runtime not found! Run: just install-runtime" && exit 1)
    @echo "✓ Runtimes are installed"

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    rm -rf {{ build_dir }}
    rm -rf {{ repo_dir }}
    rm -rf .flatpak-builder
    rm -f {{ bundle_file }}
    @echo "✓ Clean complete"

# Build the Flatpak (no installation)
build: check-runtime
    @echo "Building Flatpak..."
    flatpak-builder --force-clean {{ build_dir }} {{ manifest }}
    @echo "✓ Build complete"

# Build and install the Flatpak (user-level)
install: check-runtime
    @echo "Building and installing Flatpak..."
    flatpak-builder --user --install --force-clean {{ build_dir }} {{ manifest }}
    @echo "✓ Build and install complete"

# Build and install the Flatpak (system-level)
install-system: check-runtime
    @echo "Building and installing Flatpak (system-level)..."
    flatpak-builder --install --force-clean {{ build_dir }} {{ manifest }}
    @echo "✓ Build and install complete"

# Run the installed Flatpak
run:
    @echo "Running {{ app_id }}..."
    flatpak run {{ app_id }}

# Run with verbose output (for debugging)
run-verbose:
    @echo "Running {{ app_id }} with verbose output..."
    flatpak run --verbose {{ app_id }}

# Build into a repository
build-repo: check-runtime
    @echo "Building Flatpak into repository..."
    flatpak-builder --repo={{ repo_dir }} --force-clean {{ build_dir }} {{ manifest }}
    @echo "✓ Repository build complete"

# Create a distributable bundle
bundle: build-repo
    @echo "Creating Flatpak bundle..."
    flatpak build-bundle {{ repo_dir }} {{ bundle_file }} {{ app_id }}
    @echo "✓ Bundle created: {{ bundle_file }}"
    @ls -lh {{ bundle_file }}

# Full release: build, install, and create bundle
release: install bundle
    @echo "=========================================="
    @echo "✓ Release build complete!"
    @echo "=========================================="
    @echo "Installed: flatpak run {{ app_id }}"
    @echo "Bundle: {{ bundle_file }}"
    @echo "=========================================="

# Uninstall the Flatpak (user-level)
uninstall:
    @echo "Uninstalling {{ app_id }}..."
    flatpak uninstall -y {{ app_id }} || echo "Application not installed"

# Uninstall the Flatpak (system-level)
uninstall-system:
    @echo "Uninstalling {{ app_id }} (system-level)..."
    sudo flatpak uninstall -y {{ app_id }} || echo "Application not installed"

# Show information about the installed Flatpak
info:
    @echo "Application information:"
    flatpak info {{ app_id }} || echo "Application not installed"

# List all installed Flatpaks
list:
    @echo "Installed Flatpak applications:"
    @flatpak list --app

# Update Cargo.lock (run after dependency changes)
update-deps:
    @echo "Updating Cargo dependencies..."
    cargo update
    @echo "✓ Cargo.lock updated"

# Full workflow: clean, build, install
fresh: clean install
    @echo "✓ Fresh build and install complete"

# Full clean and rebuild with bundle
fresh-release: clean release
    @echo "✓ Fresh release complete"

# Rebuild (clean + build without install)
rebuild: clean build

# Show Flatpak logs
logs:
    @echo "Showing Flatpak logs..."
    flatpak run --log-session-bus {{ app_id }}

# Open a shell in the Flatpak build environment
shell:
    @echo "Opening shell in build environment..."
    flatpak-builder --run {{ build_dir }} {{ manifest }} bash

# Test the bundle installation (in a temporary location)
test-bundle: bundle
    @echo "Testing bundle installation..."
    @echo "To test, run: flatpak install --user {{ bundle_file }}"
    @echo "Then: flatpak run {{ app_id }}"

# Show build status and file sizes
status:
    @echo "=========================================="
    @echo "Build Status"
    @echo "=========================================="
    @echo "Build directory:"
    @test -d {{ build_dir }} && du -sh {{ build_dir }} || echo "  Not built"
    @echo ""
    @echo "Repository:"
    @test -d {{ repo_dir }} && du -sh {{ repo_dir }} || echo "  Not built"
    @echo ""
    @echo "Bundle:"
    @test -f {{ bundle_file }} && ls -lh {{ bundle_file }} || echo "  Not created"
    @echo ""
    @echo "Installed:"
    @flatpak list --app | grep {{ app_id }} && echo "  ✓ Installed" || echo "  ✗ Not installed"
    @echo "=========================================="

# Help with common workflows
help:
    @echo "=========================================="
    @echo "Invoice Flatpak - Common Workflows"
    @echo "=========================================="
    @echo ""
    @echo "First time setup:"
    @echo "  just install-runtime    # Install required runtimes"
    @echo ""
    @echo "Development:"
    @echo "  just install            # Build and install"
    @echo "  just run                # Run the application"
    @echo "  just fresh              # Clean rebuild and install"
    @echo ""
    @echo "Distribution:"
    @echo "  just bundle             # Create distributable .flatpak"
    @echo "  just release            # Build, install, and create bundle"
    @echo "  just fresh-release      # Clean + full release"
    @echo ""
    @echo "Maintenance:"
    @echo "  just clean              # Remove build artifacts"
    @echo "  just uninstall          # Remove installed app"
    @echo "  just status             # Show current status"
    @echo ""
    @echo "Debugging:"
    @echo "  just run-verbose        # Run with verbose output"
    @echo "  just logs               # Show application logs"
    @echo "  just shell              # Open build environment shell"
    @echo ""
    @echo "For all commands: just --list"
    @echo "=========================================="
