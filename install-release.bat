@echo off
setlocal enabledelayedexpansion

REM Port Kill Release Installer for Windows
REM Downloads and installs the latest release

set REPO=your-username/port-kill
set LATEST_RELEASE_URL=https://api.github.com/repos/%REPO%/releases/latest

echo 🚀 Port Kill Release Installer for Windows
echo ==========================================
echo.

echo ✅ Detected platform: Windows

REM Get latest release info
echo 📡 Fetching latest release information...
for /f "tokens=*" %%i in ('powershell -Command "(Invoke-WebRequest -Uri '%LATEST_RELEASE_URL%' -UseBasicParsing).Content | ConvertFrom-Json | Select-Object -ExpandProperty tag_name"') do set LATEST_TAG=%%i

if "%LATEST_TAG%"=="" (
    echo ❌ Failed to get latest release information
    echo    Please check: https://github.com/%REPO%/releases
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
powershell -Command "Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile '%INSTALL_DIR%\port-kill.exe'"

REM Download and install console binary
echo ⬇️  Downloading port-kill-console-windows.exe...
set CONSOLE_DOWNLOAD_URL=https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console-windows.exe
powershell -Command "Invoke-WebRequest -Uri '%CONSOLE_DOWNLOAD_URL%' -OutFile '%INSTALL_DIR%\port-kill-console.exe'"

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
