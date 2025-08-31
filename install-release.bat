@echo off
setlocal enabledelayedexpansion

REM Port Kill Release Installer for Windows
REM Downloads and installs the latest release

set REPO=kagehq/port-kill
set LATEST_RELEASE_URL=https://api.github.com/repos/%REPO%/releases/latest

echo 🚀 Port Kill Release Installer for Windows
echo ==========================================
echo.

echo ✅ Detected platform: Windows

REM Get latest release info
echo 📡 Fetching latest release information...
for /f "tokens=*" %%i in ('powershell -Command "(Invoke-WebRequest -Uri '%LATEST_RELEASE_URL%' -UseBasicParsing).Content | ConvertFrom-Json | Select-Object -ExpandProperty tag_name"') do set LATEST_TAG=%%i

if "%LATEST_TAG%"=="" (
    echo ❌ No releases found or failed to get latest release information
    echo.
    echo 📋 No releases are currently available. You have two options:
    echo.
    echo    1. 🏗️  Build from source (recommended):
    echo       install.sh
    echo.
    echo    2. 📦 Wait for a release to be published:
    echo       Visit: https://github.com/%REPO%/releases
    echo.
    echo    To create a release, the repository owner needs to:
    echo    - Go to GitHub repository
    echo    - Click 'Releases' → 'Create a new release'
    echo    - Set tag (e.g., v0.1.0) and publish
    echo.
    pause
    exit /b 1
)

echo 📦 Latest release: %LATEST_TAG%

REM Create installation directory
set INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo 📁 Installing to: %INSTALL_DIR%

REM Download and install binary
echo ⬇️  Downloading port-kill-windows.exe...
set DOWNLOAD_URL=https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-windows.exe
powershell -Command "try { Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile '%INSTALL_DIR%\port-kill.exe' } catch { Write-Host '❌ Failed to download port-kill-windows.exe'; Write-Host '   URL: %DOWNLOAD_URL%'; Write-Host '   Please check if the release assets are available'; exit 1 }"

REM Download and install console binary
echo ⬇️  Downloading port-kill-console-windows.exe...
set CONSOLE_DOWNLOAD_URL=https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console-windows.exe
powershell -Command "try { Invoke-WebRequest -Uri '%CONSOLE_DOWNLOAD_URL%' -OutFile '%INSTALL_DIR%\port-kill-console.exe' } catch { Write-Host '❌ Failed to download port-kill-console-windows.exe'; Write-Host '   URL: %CONSOLE_DOWNLOAD_URL%'; Write-Host '   Please check if the release assets are available'; exit 1 }"

echo.
echo ✅ Installation complete!
echo.
echo 📋 Usage:
echo    System tray mode: port-kill.exe --ports 3000,8000
echo    Console mode:     port-kill-console.exe --console --ports 3000,8000
echo.
echo 🔧 Add to PATH:
echo    Add %INSTALL_DIR% to your system PATH environment variable
echo.
echo 📖 For more options: port-kill.exe --help
echo.
pause
