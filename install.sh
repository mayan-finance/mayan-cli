#!/bin/bash

# Mayan Utils CLI - Build and Install Script
# Supports macOS (Intel & Apple Silicon) and Linux (Debian & others)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed!"
        print_info "Please install Rust from https://rustup.rs/"
        print_info "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    print_success "Rust is installed: $(rustc --version)"
}

# Check if Cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed!"
        print_info "Please install Rust with Cargo from https://rustup.rs/"
        exit 1
    fi
    
    print_success "Cargo is installed: $(cargo --version)"
}

# Detect system architecture and OS
detect_system() {
    OS=$(uname -s)
    ARCH=$(uname -m)
    
    case "$OS" in
        Darwin)
            SYSTEM_OS="macOS"
            case "$ARCH" in
                x86_64)
                    SYSTEM_ARCH="Intel x64"
                    RUST_TARGET="x86_64-apple-darwin"
                    ;;
                arm64)
                    SYSTEM_ARCH="Apple Silicon (M1/M2/M3)"
                    RUST_TARGET="aarch64-apple-darwin"
                    ;;
                *)
                    print_warning "Unknown macOS architecture: $ARCH"
                    SYSTEM_ARCH="$ARCH"
                    RUST_TARGET=""
                    ;;
            esac
            INSTALL_DIR="/usr/local/bin"
            ;;
        Linux)
            SYSTEM_OS="Linux"
            case "$ARCH" in
                x86_64)
                    SYSTEM_ARCH="x64"
                    RUST_TARGET="x86_64-unknown-linux-gnu"
                    ;;
                aarch64|arm64)
                    SYSTEM_ARCH="ARM64"
                    RUST_TARGET="aarch64-unknown-linux-gnu"
                    ;;
                armv7l)
                    SYSTEM_ARCH="ARM v7"
                    RUST_TARGET="armv7-unknown-linux-gnueabihf"
                    ;;
                *)
                    print_warning "Unknown Linux architecture: $ARCH"
                    SYSTEM_ARCH="$ARCH"
                    RUST_TARGET=""
                    ;;
            esac
            INSTALL_DIR="/usr/local/bin"
            ;;
        *)
            print_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac
    
    print_info "Detected system: $SYSTEM_OS ($SYSTEM_ARCH)"
}

# Add Rust target if needed
add_rust_target() {
    if [ -n "$RUST_TARGET" ]; then
        print_info "Adding Rust target: $RUST_TARGET"
        rustup target add "$RUST_TARGET" || {
            print_warning "Failed to add target $RUST_TARGET, continuing with default target"
            RUST_TARGET=""
        }
    fi
}

# Build the project
build_project() {
    print_info "Building Mayan Utils CLI..."
    
    if [ -n "$RUST_TARGET" ]; then
        print_info "Building for target: $RUST_TARGET"
        cargo build --release --target "$RUST_TARGET"
        BINARY_PATH="target/$RUST_TARGET/release/mayan-utils"
    else
        print_info "Building with default target"
        cargo build --release
        BINARY_PATH="target/release/mayan-utils"
    fi
    
    if [ ! -f "$BINARY_PATH" ]; then
        print_error "Build failed! Binary not found at $BINARY_PATH"
        exit 1
    fi
    
    print_success "Build completed successfully!"
}

# Install the binary
install_binary() {
    if [ ! -f "$BINARY_PATH" ]; then
        print_error "Binary not found at $BINARY_PATH"
        exit 1
    fi
    
    print_info "Installing to $INSTALL_DIR..."
    
    # Check if we need sudo
    if [ ! -w "$INSTALL_DIR" ]; then
        print_warning "Need sudo permissions to install to $INSTALL_DIR"
        sudo cp "$BINARY_PATH" "$INSTALL_DIR/mayan-utils"
        sudo chmod +x "$INSTALL_DIR/mayan-utils"
    else
        cp "$BINARY_PATH" "$INSTALL_DIR/mayan-utils"
        chmod +x "$INSTALL_DIR/mayan-utils"
    fi
    
    print_success "Installed successfully to $INSTALL_DIR/mayan-utils"
}

# Verify installation
verify_installation() {
    if command -v mayan-utils &> /dev/null; then
        print_success "Installation verified! You can now use 'mayan-utils' command."
        print_info "Try: mayan-utils --help"
    else
        print_error "Installation verification failed!"
        print_info "Make sure $INSTALL_DIR is in your PATH"
        print_info "Add this to your shell profile:"
        print_info "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# Print system information
print_system_info() {
    echo
    print_info "=== System Information ==="
    print_info "OS: $SYSTEM_OS"
    print_info "Architecture: $SYSTEM_ARCH"
    print_info "Rust Target: ${RUST_TARGET:-default}"
    print_info "Install Directory: $INSTALL_DIR"
    echo
}

# Main execution
main() {
    echo
    print_info "🚀 Mayan Utils CLI - Build and Install Script"
    echo
    
    # Check prerequisites
    check_rust
    check_cargo
    
    # Detect system
    detect_system
    print_system_info
    
    # Add Rust target if needed
    add_rust_target
    
    # Build the project
    build_project
    
    # Install the binary
    install_binary
    
    # Verify installation
    verify_installation
    
    echo
    print_success "🎉 Installation completed successfully!"
    print_info "You can now use the following commands:"
    print_info "  mayan-utils gasa <ORDER_ID>                    # Get auction state address"
    print_info "  mayan-utils gas <ORDER_ID_OR_ADDRESS>          # Get auction state data"
    print_info "  mayan-utils --help                             # Show help"
    echo
}

# Run main function
main "$@"