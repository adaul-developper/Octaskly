#!/bin/bash

# Octaskly Cross-Platform Build Script
# Build binaries for: Linux x86_64, macOS (Intel/ARM), Windows MSVC, Android/Termux
# Usage: ./scripts/build-release.sh [target] [all]
#   all     - Build all targets
#   linux   - Linux x86_64
#   macos   - macOS (Intel + ARM)
#   windows - Windows x86_64
#   android - Android ARM64
#   termux  - Termux (alias for android)

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_NAME="octaskly"
VERSION="1.0.0"
BUILD_DIR="./target/releases"

# Function to print section headers
print_header() {
    echo -e "\n${BLUE}[BUILD]${NC} $1"
}

# Function to print success
print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Create releases directory
mkdir -p "$BUILD_DIR"

# Function to build for target
build_target() {
    local target=$1
    local target_name=$2
    
    print_header "Building for $target_name ($target)"
    
    # Add target if not installed
    rustup target add "$target" 2>/dev/null || true
    
    # Build release binary
    cargo build --release --target "$target"
    
    # Determine output path
    local source=""
    local output_name="${PROJECT_NAME}-${target_name}-v${VERSION}"
    
    if [[ "$target" == *"windows"* ]]; then
        source="./target/${target}/release/${PROJECT_NAME}.exe"
        output="${BUILD_DIR}/${output_name}.exe"
    else
        source="./target/${target}/release/${PROJECT_NAME}"
        output="${BUILD_DIR}/${output_name}"
    fi
    
    # Copy and strip binary
    if [ -f "$source" ]; then
        cp "$source" "$output"
        
        # Strip binary if not Windows
        if [[ "$target" != *"windows"* ]]; then
            strip "$output" 2>/dev/null || true
        fi
        
        # Show file size
        local size=$(du -h "$output" | cut -f1)
        print_success "Built $output_name ($size)"
    else
        print_warning "Build output not found at $source"
    fi
}

# Function to create checksums
create_checksums() {
    print_header "Creating checksums"
    
    cd "$BUILD_DIR"
    if command -v sha256sum &> /dev/null; then
        sha256sum * > SHA256SUMS.txt 2>/dev/null || true
    elif command -v shasum &> /dev/null; then
        shasum -a 256 * > SHA256SUMS.txt 2>/dev/null || true
    fi
    cd - > /dev/null
    
    print_success "Checksums created at $BUILD_DIR/SHA256SUMS.txt"
}

# Function to create tarball for Unix systems
create_tarballs() {
    print_header "Creating distribution archives"
    
    cd "$BUILD_DIR"
    
    # Find binaries (not checksums file)
    for binary in $(find . -maxdepth 1 -type f ! -name "SHA256SUMS.txt" ! -name "*.exe"); do
        local basename=$(basename "$binary")
        local tarname="${basename}.tar.gz"
        
        tar czf "$tarname" "$basename"
        print_success "Created $tarname"
    done
    
    cd - > /dev/null
}

# Build targets based on argument
case "${1:-all}" in
    all)
        # Linux x86_64
        build_target "x86_64-unknown-linux-gnu" "linux-x64"
        
        # macOS x86_64 (Intel)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            build_target "x86_64-apple-darwin" "macos-x64"
            build_target "aarch64-apple-darwin" "macos-arm64"
        else
            print_warning "Skipping macOS builds (not on macOS)"
        fi
        
        # Windows MSVC
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
            build_target "x86_64-pc-windows-msvc" "windows-x64"
        else
            print_warning "Skipping Windows build (not on Windows)"
        fi
        
        # Android ARM64 (Termux compatible)
        build_target "aarch64-linux-android" "android-arm64"
        
        # Create artifacts
        create_checksums
        create_tarballs
        ;;
        
    linux)
        build_target "x86_64-unknown-linux-gnu" "linux-x64"
        create_checksums
        ;;
        
    macos)
        if [[ "$OSTYPE" != "darwin"* ]]; then
            print_warning "macOS builds require running on macOS"
            exit 1
        fi
        build_target "x86_64-apple-darwin" "macos-x64"
        build_target "aarch64-apple-darwin" "macos-arm64"
        create_checksums
        ;;
        
    windows)
        if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
            print_warning "Windows MSVC builds require running on Windows"
            exit 1
        fi
        build_target "x86_64-pc-windows-msvc" "windows-x64"
        create_checksums
        ;;
        
    android|termux)
        build_target "aarch64-linux-android" "android-arm64"
        create_checksums
        ;;
        
    *)
        echo "Usage: $0 [target]"
        echo ""
        echo "Targets:"
        echo "  all       - Build all available targets"
        echo "  linux     - Linux x86_64"
        echo "  macos     - macOS (Intel + ARM)"
        echo "  windows   - Windows x86_64 MSVC"
        echo "  android   - Android ARM64 (Termux)"
        echo "  termux    - Alias for android"
        exit 1
        ;;
esac

print_header "Build Summary"
echo "Release builds available at: $BUILD_DIR"
ls -lh "$BUILD_DIR" | tail -n +2
print_success "All builds completed"
