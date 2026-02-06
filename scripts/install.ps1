# Octaskly Installer for Windows (PowerShell)
# Run with: powershell -ExecutionPolicy Bypass -Command "& {$(Invoke-WebRequest https://github.com/adauldev/octaskly/releases/latest/download/install.ps1 -UseBasicParsing).Content | powershell}"

# Or locally:
# powershell -ExecutionPolicy Bypass -File install.ps1

param(
    [string]$Version = "v1.0.0",
    [string]$InstallDir = "C:\Program Files\Octaskly"
)

# Configuration
$ProjectRepo = "adauldev/octaskly"
$BinaryName = "octaskly"
$GitHubAPI = "https://api.github.com/repos/$ProjectRepo/releases/latest"

# Colors
function Write-Info {
    Write-Host "[INFO] $args" -ForegroundColor Cyan
}

function Write-Success {
    Write-Host "[OK] $args" -ForegroundColor Green
}

function Write-Error {
    Write-Host "[ERROR] $args" -ForegroundColor Red
    exit 1
}

function Write-Warning {
    Write-Host "[WARN] $args" -ForegroundColor Yellow
}

# Main function
function Install-Octaskly {
    Clear-Host
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  Octaskly v1.0.0 Installer for Windows" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""

    # Check for administrator privileges; if not, relaunch elevated
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $isAdmin) {
        Write-Info "Not running as Administrator. Relaunching with elevated privileges..."
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = 'powershell'
        $args = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
        if ($Version) { $args += " -Version $Version" }
        if ($InstallDir) { $args += " -InstallDir `"$InstallDir`"" }
        $psi.Arguments = $args
        $psi.Verb = 'runas'
        try {
            [System.Diagnostics.Process]::Start($psi) | Out-Null
            exit 0
        }
        catch {
            Write-Error "Failed to elevate privileges: $_"
        }
    }

    Write-Success "Running as Administrator"

    # Detect system info
    Write-Info "Detecting system..."
    
    $osInfo = Get-CimInstance Win32_OperatingSystem
    $processorInfo = Get-CimInstance Win32_Processor
    
    Write-Success "Detected: Windows $($osInfo.Caption) x86_64"
    Write-Info "Installation path: $InstallDir"

    # Ensure TLS 1.2 for downloads
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    # Get latest version if not specified
    Write-Info "Fetching latest version from GitHub..."
    
    try {
        $releaseInfo = Invoke-RestMethod -Uri $GitHubAPI -UseBasicParsing -ErrorAction Stop
        $Version = $releaseInfo.tag_name
        Write-Success "Latest version: $Version"
    }
    catch {
        Write-Warning "Could not fetch version from GitHub API, using default: $Version"
    }

    # Create installation directory
    if (-not (Test-Path $InstallDir)) {
        Write-Info "Creating installation directory..."
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Write-Success "Directory created"
    }
    else {
        Write-Success "Directory exists"
    }

    # Download binary
    $downloadUrl = "https://github.com/$ProjectRepo/releases/download/$Version/$BinaryName-windows-x64-$Version.exe"
    $tempFile = Join-Path $env:TEMP "$BinaryName.exe"

    Write-Info "Downloading binary..."
    Write-Info "URL: $downloadUrl"

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing -ErrorAction Stop
        Write-Success "Binary downloaded"
    }
    catch {
        Write-Error "Failed to download binary: $_"
    }

    # Install binary
    Write-Info "Installing binary..."
    
    $installPath = Join-Path $InstallDir "$BinaryName.exe"
    
    try {
        Copy-Item -Path $tempFile -Destination $installPath -Force -ErrorAction Stop
        Write-Success "Binary installed to $InstallDir"
    }
    catch {
        Write-Error "Failed to install binary: $_"
    }

    # Add to PATH (user scope)
    Write-Info "Adding to PATH..."
    
    $userPath = [Environment]::GetEnvironmentVariable("PATH", [EnvironmentVariableTarget]::User)
    
    if ($userPath -like "*$InstallDir*") {
        Write-Success "Already in PATH"
    }
    else {
        $newPath = "$userPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, [EnvironmentVariableTarget]::User)
        Write-Success "Added to PATH"
        Write-Warning "Please close and reopen PowerShell/Command Prompt for PATH changes to take effect"
    }

    # Verify installation
    Write-Info "Verifying installation..."
    
    if (-not (Test-Path $installPath)) {
        Write-Error "Binary file not found after installation"
    }

    # Try to run help
    try {
        & $installPath --help | Out-Null
        Write-Success "Installation verified"
    }
    catch {
        Write-Error "Binary verification failed: $_"
    }

    # Cleanup
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue

    # Show success message
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Success "Installation successful!"
    Write-Host ""
    Write-Host "Quick start:" -ForegroundColor Green
    Write-Host "  octaskly dispatcher --port 7878"
    Write-Host "  octaskly worker --name myworker"
    Write-Host ""
    Write-Host "For help: octaskly --help"
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""
}

# Run installation
Install-Octaskly
