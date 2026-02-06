# Octaskly Cross-Platform Distribution Guide

## Overview

Octaskly is built and distributed as a cross-platform binary for multiple operating systems and architectures. This guide explains building, distributing, and installing Octaskly across different platforms.

## Supported Platforms

| Platform | Architecture | Target Triple | Status |
|----------|--------------|----------------|--------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | ✅ Supported |
| macOS | Intel (x86_64) | `x86_64-apple-darwin` | ✅ Supported |
| macOS | Apple Silicon (ARM64) | `aarch64-apple-darwin` | ✅ Supported |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | ✅ Supported |
| Android/Termux | ARM64 | `aarch64-linux-android` | ✅ Supported |

## Building for Distribution

### Prerequisites

```bash
# Install Rust and Cargo (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-linux-android
```

### Using the Build Script

```bash
# Build for all available platforms
./scripts/build-release.sh all

# Build for specific platform
./scripts/build-release.sh linux
./scripts/build-release.sh macos
./scripts/build-release.sh windows
./scripts/build-release.sh android
```

### Manual Build Commands

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS ARM64 (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Windows MSVC
cargo build --release --target x86_64-pc-windows-msvc

# Android/Termux ARM64
cargo build --release --target aarch64-linux-android
```

**Output:**
- Linux/macOS/Termux: `target/<target>/release/octaskly`
- Windows: `target/x86_64-pc-windows-msvc/release/octaskly.exe`

## Installation Methods

### Option 1: Automated Installer (Recommended)

#### Linux, macOS, Termux
```bash
curl -sSL https://github.com/adauldev/octaskly/releases/latest/download/install.sh | bash
```

#### Windows (PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -Command "& { Invoke-WebRequest https://github.com/adauldev/octaskly/releases/latest/download/install.ps1 -UseBasicParsing | Invoke-Expression }"
```

#### Windows (Command Prompt)
1. Download `install.bat` from releases
2. Right-click and select "Run as administrator"
3. Follow the prompts

### Option 2: Manual Download and Installation

#### Linux / macOS
```bash
# Download binary for your platform
# Replace linux-x64 with: macos-x64, macos-arm64, etc.
curl -LO https://github.com/adauldev/octaskly/releases/download/v1.0.0/octaskly-linux-x64

# Make executable
chmod +x octaskly-linux-x64

# Install to global PATH
sudo mv octaskly-linux-x64 /usr/local/bin/octaskly
```

#### Termux (Android)
```bash
# Install wget if needed
pkg install wget tar

# Download and extract
wget https://github.com/adauldev/octaskly/releases/download/v1.0.0/octaskly-android-arm64.tar.gz
tar xzf octaskly-android-arm64.tar.gz

# Install to Termux bin directory (automatically global)
mv octaskly $PREFIX/bin/
chmod +x $PREFIX/bin/octaskly
```

#### Windows
1. Download `octaskly-windows-x64-v1.0.0.exe` from releases
2. Create folder: `C:\Program Files\Octaskly`
3. Move binary to this folder
4. Add `C:\Program Files\Octaskly` to system PATH:
   - Press `Windows Key + X` → System
   - Advanced system settings → Environment Variables
   - Under "User variables", edit PATH
   - Add: `C:\Program Files\Octaskly`
5. Restart Command Prompt or PowerShell for changes to take effect

### Option 3: Install from Source

```bash
# Clone repository
git clone https://github.com/adauldev/octaskly.git
cd octaskly

# Build and install
cargo install --path .

# Binary installed at: ~/.cargo/bin/octaskly
# (Usually already in PATH if Rust was installed properly)
```

## Verifying Installation

After installation, verify with:

```bash
# Check version
octaskly --version

# Check help
octaskly --help

# Test dispatcher mode
octaskly dispatcher --help

# Test worker mode
octaskly worker --help
```

## Global PATH Configuration

### Linux & macOS
Binary should be in `/usr/local/bin/`.

Verify PATH:
```bash
echo $PATH
which octaskly
```

### Termux (Android)
Binary should be in `$PREFIX/bin/`.

Verify:
```bash
echo $PREFIX
which octaskly
```

### Windows
Binary should be in `C:\Program Files\Octaskly\` and this path must be in system PATH.

Verify in PowerShell:
```powershell
$env:PATH
Get-Command octaskly
```

## Distributing Releases

### Via GitHub Releases

1. Tag a release in Git:
```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

2. GitHub Actions automatically:
   - Builds binaries for all platforms
   - Creates checksums
   - Publishes GitHub Release with all binaries

Note: Release archives (`.tar.gz` on Unix, `.zip` for Windows) are built by CI to preserve executable permission bits for Unix targets. Users installing from the release archives do not need to run `chmod +x` when using the provided tarballs.

## Verifying downloads and checksums

Always verify downloads before running installers or extracting archives. CI publishes a `SHA256SUMS.txt` file in each release containing checksums for all artifacts.

Linux / macOS / Termux (example)

```bash
# Replace v1.0.0 with the desired tag
TAG=v1.0.0
BASE=https://github.com/adauldev/octaskly/releases/download/$TAG

# Download artifact and checksums
curl -LO $BASE/octaskly-linux-x64.tar.gz
curl -LO $BASE/SHA256SUMS.txt

# Verify (Linux)
sha256sum --check SHA256SUMS.txt

# Or on macOS
shasum -a 256 -c SHA256SUMS.txt
```

Windows (PowerShell) example

```powershell
# Replace tag as needed
$TAG = 'v1.0.0'
$base = "https://github.com/adauldev/octaskly/releases/download/$TAG"

# Download files
Invoke-WebRequest "$base/octaskly-windows-x64.zip" -OutFile octaskly-windows-x64.zip
Invoke-WebRequest "$base/SHA256SUMS.txt" -OutFile SHA256SUMS.txt

# Extract expected checksum for the file
$expected = (Select-String -Path SHA256SUMS.txt -Pattern "octaskly-windows-x64.zip").Line.Split()[0]
$actual = (Get-FileHash .\octaskly-windows-x64.zip -Algorithm SHA256).Hash

if ($expected -eq $actual) { Write-Host "Checksum OK" } else { Write-Error "Checksum mismatch" }
```

Manual verification notes

- Ensure the file names in `SHA256SUMS.txt` match the downloaded artifact names. The `sha256sum --check` and `shasum -a 256 -c` commands expect the file names used by CI.
- For CI-produced `.tar.gz` on Unix targets the checksum is created from the archive, so verifying the archive is sufficient before extraction.
- For additional security, validate the Git tag and review release commit/signatures before trusting binaries in high-security environments.

3. Users can download from: `https://github.com/adauldev/octaskly/releases`

### Manual Release Process

```bash
# Build all targets
./scripts/build-release.sh all

# Files are in target/releases/
# Upload to GitHub Releases manually
ls -lh target/releases/
```

### Creating Distribution Archives

For easy sharing, create platform-specific archives:

```bash
cd target/releases

# Linux
tar czf octaskly-linux-x64.tar.gz octaskly-linux-x64

# macOS Intel
tar czf octaskly-macos-x64.tar.gz octaskly-macos-x64

# macOS ARM64
tar czf octaskly-macos-arm64.tar.gz octaskly-macos-arm64

# Android/Termux
tar czf octaskly-android-arm64.tar.gz octaskly-android-arm64

# Windows (if available)
zip octaskly-windows-x64.zip octaskly-windows-x64.exe
```

## Cross-Compilation Details

### Linux to Android/Termux

Requirements:
```bash
# Install NDK
rustup target add aarch64-linux-android

# Download NDK (or install via package manager)
# Set NDK path
export NDK_HOME=/path/to/android-ndk
export CC_aarch64_linux_android=$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-clang
export AR_aarch64_linux_android=$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar

# Build
cargo build --release --target aarch64-linux-android
```

### macOS to Other Platforms

Due to Apple's restrictions, cross-compiling FROM macOS to Linux/Windows is not supported. However, you can:
- Build native macOS binaries (Intel & ARM)
- Use GitHub Actions for building Linux/Windows/Android binaries

## Performance Optimization

### Strip Binaries (Reduce Size)

```bash
# Linux/macOS/Android
strip target/<target>/release/octaskly

# Typical size reduction: 40-50%
# Example: 12MB → 6-8MB
```

### Build with Optimizations

```bash
# Maximum optimization (slower compile, faster runtime)
RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
  cargo build --release --target <target>

# Link-time optimization (very slow, best performance)
RUSTFLAGS="-C lto" cargo build --release --target <target>
```

## Troubleshooting

### "Command not found: octaskly"
- Verify binary is in PATH: `echo $PATH`
- Verify binary exists: `ls -la /usr/local/bin/octaskly`
- Restart terminal to reload PATH

### "Permission denied" on Linux/macOS
- Make binary executable: `chmod +x /usr/local/bin/octaskly`

### Windows PATH not updating
- Close, and **fully restart** Command Prompt/PowerShell
- Restart computer if still not working

### Build fails for specific target
- Ensure target is installed: `rustup target add <target>`
- Check platform requirements for Android NDK
- Use `cargo build -vv` for verbose output

## Version Management

Current version: **1.0.0**

Version in multiple locations:
- `Cargo.toml`: `version = "1.0.0"`
- `src/api/mod.rs`: API health check response
- GitHub Releases: Tag `v1.0.0`
- Installer scripts

Update version before release:
```bash
# Update Cargo.toml
sed -i 's/version = ".*"/version = "1.1.0"/' Cargo.toml

# Create git tag
git tag v1.1.0

# Push to trigger GitHub Actions
git push origin v1.1.0
```

## Analytics & Monitoring

Track downloads via:
- GitHub Releases download counts
- Custom analytics in installer scripts (optional)
- User feedback and issue reports

## Future Enhancements

- [ ] homebrew formula for macOS
- [ ] apt/deb packages for Ubuntu/Debian
- [ ] Custom Termux repository
- [ ] Automatic update checker built-in
- [ ] Signed binaries (code signing)
- [ ] Nightly builds from main branch
