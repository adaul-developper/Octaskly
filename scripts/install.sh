#!/bin/bash

# Octaskly Installer for Linux, macOS, and Termux
# Downloads and installs octaskly binary from GitHub Releases
# Usage: curl -sSL https://github.com/adauldev/octaskly/releases/latest/download/install.sh | bash

set -e

# Configuration
PROJECT_REPO="adauldev/octaskly"
BINARY_NAME="octaskly"
GITHUB_API="https://api.github.com/repos/${PROJECT_REPO}/releases/latest"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""
    
    # Detect OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Check if Termux (Android)
        if [ -d "$PREFIX" ] && [ "$PREFIX" != "" ]; then
            os="android"
        else
            os="linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        os="macos"
    else
        print_error "Unsupported OS: $OSTYPE"
    fi
    
    # Detect architecture
    local machine=$(uname -m)
    case "$machine" in
        x86_64)
            arch="x64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            print_error "Unsupported architecture: $machine"
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Determine install location
get_install_path() {
    # Prefer user-local install to avoid sudo (e.g. ~/.local/bin)
    if [ "$PREFIX" != "" ]; then
        # Termux environment
        echo "$PREFIX/bin"
        return
    fi

    # If ~/.local/bin exists or can be created, prefer it
    if [ -n "$HOME" ]; then
        local user_bin="$HOME/.local/bin"
        if [ -d "$user_bin" ] || mkdir -p "$user_bin" 2>/dev/null; then
            echo "$user_bin"
            return
        fi
    fi

    # Fallback to system location
    echo "/usr/local/bin"
}

# Download binary from GitHub
download_binary() {
    local platform=$1
    local version=$2
    local temp_file=$(mktemp)
    
    print_info "Downloading binary for $platform..."
    
    # Try to download from releases
    local download_url="https://github.com/${PROJECT_REPO}/releases/download/${version}/${BINARY_NAME}-${platform}-${version}.tar.gz"
    
    if ! curl -fsSL "$download_url" -o "$temp_file"; then
        # Try without version in filename
        download_url="https://github.com/${PROJECT_REPO}/releases/download/${version}/${BINARY_NAME}-${platform}"
        if ! curl -fsSL "$download_url" -o "$temp_file"; then
            print_error "Failed to download binary from $download_url"
        fi
    fi
    
    echo "$temp_file"
}

# Extract and install binary
install_binary() {
    local binary_file=$1
    local install_path=$2
    local binary_name=$3
    
    print_info "Installing to $install_path..."
    
    # Check if tarball or single binary
    if file "$binary_file" | grep -q "gzip"; then
        # Extract tarball
        local temp_dir=$(mktemp -d)
        tar xzf "$binary_file" -C "$temp_dir"
        binary_file="$temp_dir/$binary_name"
    fi
    
    # Verify binary exists
    if [ ! -f "$binary_file" ]; then
        print_error "Binary file not found after extraction"
    fi
    
        # Ensure install directory exists
        if [ ! -d "$install_path" ]; then
            if mkdir -p "$install_path" 2>/dev/null; then
                print_info "Created install directory: $install_path"
            else
                # try with sudo
                sudo mkdir -p "$install_path"
            fi
        fi
    
        # Attempt non-sudo install (user-local directories should be writable)
        if cp "$binary_file" "$install_path/$binary_name" 2>/dev/null; then
            chmod +x "$install_path/$binary_name" 2>/dev/null || true
        else
            # Fallback to sudo
            print_info "Attempting system-wide install with sudo..."
            sudo cp "$binary_file" "$install_path/$binary_name"
            sudo chmod +x "$install_path/$binary_name"
        fi
    
    # Cleanup
    rm -f "$binary_file"
    rm -rf "$temp_dir" 2>/dev/null || true
}

# Get latest version
get_latest_version() {
    local version=$(curl -fsSL "$GITHUB_API" | grep '"tag_name"' | head -1 | sed 's/.*"v\([^"]*\)".*/v\1/')
    
    if [ -z "$version" ]; then
        print_error "Failed to fetch latest version from GitHub"
    fi
    
    echo "$version"
}

# Verify binary works
verify_installation() {
    local install_path=$1
    local binary_path="$install_path/$BINARY_NAME"
    
    print_info "Verifying installation..."
    
    if ! command -v "$BINARY_NAME" &> /dev/null; then
        # Binary might not be in PATH yet, check directly
        if [ ! -x "$binary_path" ]; then
            print_error "Binary installation failed - file not executable"
        fi
    fi
    
    # Try to run help
    if ! "$binary_path" --help > /dev/null 2>&1; then
        print_error "Binary verification failed"
    fi
    
    print_success "Installation verified"
}

# Add a directory to common user shell profiles if it's not in PATH
add_path_to_profile() {
    local dir="$1"
    local added=false

    # Shell profiles to try
    local profiles=("$HOME/.profile" "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile")

    for p in "${profiles[@]}"; do
        if [ -f "$p" ]; then
            if ! grep -q "export PATH=.*$dir" "$p" 2>/dev/null; then
                printf "\n# Added by Octaskly installer\nexport PATH=\"%s:\$PATH\"\n" "$dir" >> "$p"
                added=true
            fi
        fi
    done

    if [ "$added" = true ]; then
        print_info "Added $dir to your shell profile(s). Restart your shell or run 'source ~/.profile' to apply."
    fi
}

# Main installation
main() {
    echo "=========================================="
    echo "  Octaskly v1.0.0 Installer"
    echo "=========================================="
    echo ""
    
    # Detect platform
    print_info "Detecting platform..."
    local platform=$(detect_platform)
    print_success "Detected: $platform"
    
    # Get installation path
    local install_path=$(get_install_path)
    print_info "Installation path: $install_path"
    
    # Check if install path is in PATH, attempt to add for user shells if missing
    if ! echo ":$PATH:" | grep -q ":$install_path:"; then
        print_warning "Installation path $install_path not in PATH; attempting to add it to your shell profile"
        add_path_to_profile "$install_path"
    fi
    
    # Get latest version
    print_info "Fetching latest version from GitHub..."
    local version=$(get_latest_version)
    print_success "Latest version: $version"
    
    # Download binary
    local binary_file=$(download_binary "$platform" "$version")
    print_success "Binary downloaded"
    
    # Install binary
    install_binary "$binary_file" "$install_path" "$BINARY_NAME"
    print_success "Installation complete"
    
    # Verify installation
    verify_installation "$install_path"
    
    # Show next steps
    echo ""
    echo "=========================================="
    print_success "Installation successful!"
    echo ""
    echo "Quick start:"
    echo "  octaskly dispatcher --port 7878"
    echo "  octaskly worker --name mytermux"
    echo ""
    echo "For more info: octaskly --help"
    echo "=========================================="
}

# Run main
main
