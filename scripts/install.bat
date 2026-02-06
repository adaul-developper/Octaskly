@echo off
REM Octaskly Installer for Windows
REM Downloads and installs octaskly binary
REM Usage: Command Prompt or PowerShell

setlocal enabledelayedexpansion

REM Configuration
set PROJECT_REPO=adauldev/octaskly
set BINARY_NAME=octaskly
set GITHUB_API=https://api.github.com/repos/%PROJECT_REPO%/releases/latest
set INSTALL_DIR=%ProgramFiles%\Octaskly
set VERSION=v1.0.0

cls
echo.
echo ==========================================
echo   Octaskly v1.0.0 Installer for Windows
echo ==========================================
echo.

REM Check if running as administrator
REM Check if running as administrator, if not relaunch elevated
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Not running as administrator. Relaunching with elevation...
    powershell -Command "Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', '"%~f0 %*"' -Verb RunAs"
    exit /b
)

echo [INFO] Detected Windows x86_64
echo [INFO] Installation path: %INSTALL_DIR%
echo.

REM Create installation directory
if not exist "%INSTALL_DIR%" (
    echo [INFO] Creating installation directory...
    mkdir "%INSTALL_DIR%"
    echo [OK] Directory created
) else (
    echo [OK] Directory exists
)

REM Download binary
echo [INFO] Downloading octaskly-%VERSION%-windows-x64.exe...
set DOWNLOAD_URL=https://github.com/%PROJECT_REPO%/releases/download/%VERSION%/%BINARY_NAME%-windows-x64-%VERSION%.exe
set TEMP_FILE=%TEMP%\%BINARY_NAME%.exe

REM Use PowerShell for downloading (more reliable than bitsadmin)
powershell -Command "& {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile '%TEMP_FILE%' -ErrorAction Stop
}" 2>nul

if %errorlevel% neq 0 (
    echo [ERROR] Failed to download binary
    echo Download URL: %DOWNLOAD_URL%
    pause
    exit /b 1
)

echo [OK] Binary downloaded

REM Copy to installation directory
echo [INFO] Installing binary...
copy /Y "%TEMP_FILE%" "%INSTALL_DIR%\%BINARY_NAME%.exe" >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Failed to install binary
    pause
    exit /b 1
)

echo [OK] Binary installed to %INSTALL_DIR%

REM Add to PATH (user-level)
echo [INFO] Adding to PATH...
for /f "usebackq tokens=2,*" %%A in (`reg query HKCU\Environment /v PATH 2^>nul ^| find "PATH"`) do set "CURRENT_PATH=%%B"

    if not "!CURRENT_PATH!"=="" (
        if not "!CURRENT_PATH:%INSTALL_DIR%=!"=="!CURRENT_PATH!" (
            echo [OK] Already in PATH
        ) else (
            setx PATH "!CURRENT_PATH!;%INSTALL_DIR%"
            echo [OK] Added to PATH (restart Command Prompt for changes to take effect)
        )
    ) else (
        setx PATH "%INSTALL_DIR%"
        echo [OK] Added to PATH
    )

REM Verify installation
echo [INFO] Verifying installation...
"%INSTALL_DIR%\%BINARY_NAME%.exe" --help >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Binary verification failed
    pause
    exit /b 1
)

echo [OK] Installation verified

REM Cleanup
del /q "%TEMP_FILE%" 2>nul

echo.
echo ==========================================
echo [SUCCESS] Installation complete!
echo.
echo Quick start:
echo   octaskly dispatcher --port 7878
echo   octaskly worker --name myworker
echo.
echo For help: octaskly --help
echo ==========================================
echo.
echo Note: Close and reopen Command Prompt/PowerShell for PATH changes
pause
