# Release Notes - Octaskly v1.0.0

## Highlights
- Production-ready v1.0.0 release: cross-platform binaries, automated installers, and CI release workflow.
- Automatic installers for Linux/macOS/Termux (`install.sh`) and Windows (`install.ps1`, `install.bat`).
- Release artifacts include tar.gz/zip archives with preserved executable permissions and `SHA256SUMS.txt` checksums.

## Changes
- Feature: Multi-platform distribution (Linux, macOS Intel/ARM, Windows, Android/Termux).
- CI: GitHub Actions workflow builds per-target artifacts, packages archives, and generates checksums.
- Docs: Updated `DISTRIBUTION.md`, `README.md`, `QUICK_REFERENCE.md`, `FEATURES.md`, and `PROJECT_STRUCTURE.md` to document installation and verification.
- Installers: `scripts/install.sh` (auto user-install + PATH update, sudo fallback), `scripts/install.ps1` (auto-elevate, install, PATH), `scripts/install.bat` (auto-elevate, install, PATH).

## Assets
- Binaries and archives (example names):
  - `octaskly-linux-x64.tar.gz`
  - `octaskly-macos-x64.tar.gz`
  - `octaskly-macos-arm64.tar.gz`
  - `octaskly-windows-x64.zip`
  - `octaskly-android-arm64.tar.gz`
  - `install.sh`, `install.ps1`, `install.bat`
  - `SHA256SUMS.txt`

## Verification
1. Download the artifact and checksum file from GitHub Releases.

Linux / macOS / Termux:
```bash
TAG=v1.0.0
BASE=https://github.com/adauldev/octaskly/releases/download/$TAG
curl -LO $BASE/octaskly-linux-x64.tar.gz
curl -LO $BASE/SHA256SUMS.txt
sha256sum --check SHA256SUMS.txt
```

Windows (PowerShell):
```powershell
$TAG = 'v1.0.0'
$base = "https://github.com/adauldev/octaskly/releases/download/$TAG"
Invoke-WebRequest "$base/octaskly-windows-x64.zip" -OutFile octaskly-windows-x64.zip
Invoke-WebRequest "$base/SHA256SUMS.txt" -OutFile SHA256SUMS.txt
$expected = (Select-String -Path SHA256SUMS.txt -Pattern "octaskly-windows-x64.zip").Line.Split()[0]
$actual = (Get-FileHash .\octaskly-windows-x64.zip -Algorithm SHA256).Hash
if ($expected -eq $actual) { Write-Host "Checksum OK" } else { Write-Error "Checksum mismatch" }
```

## Upgrade Notes
- No breaking API changes from prior release candidates.
- If upgrading in-place, restart dispatcher and worker processes after replacing the binary.

## Release Checklist
- [ ] Verify CI produced all platform artifacts
- [ ] Confirm checksums are included in `SHA256SUMS.txt`
- [ ] Publish GitHub Release with attached artifacts and release notes
- [ ] Announce release and update documentation site

---

Generated from `RELEASE_NOTES_TEMPLATE.md`.
