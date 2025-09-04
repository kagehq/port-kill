@echo off
setlocal enabledelayedexpansion

echo Port Kill Release Installer for Windows
echo ==========================================
echo.

echo Detected platform: Windows

REM Get latest release info
echo Fetching latest release information...
powershell -Command "(Invoke-WebRequest -Uri 'https://api.github.com/repos/kagehq/port-kill/releases/latest' -UseBasicParsing).Content | ConvertFrom-Json | Select-Object -ExpandProperty tag_name" > temp_tag.txt
set /p LATEST_TAG=<temp_tag.txt
del temp_tag.txt

if "%LATEST_TAG%"=="" (
    echo ERROR: No releases found or failed to get latest release information
    echo.
    echo No releases are currently available. You have two options:
    echo.
    echo    1. Build from source (recommended):
    echo       install.sh
    echo.
    echo    2. Wait for a release to be published:
    echo       Visit: https://github.com/kagehq/port-kill/releases
    echo.
    pause
    exit /b 1
)

echo Latest release: %LATEST_TAG%

REM Create installation directory
set INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo Installing to: %INSTALL_DIR%

REM Download and install binary
echo Downloading port-kill-windows.exe...
powershell -Command "Invoke-WebRequest -Uri 'https://github.com/kagehq/port-kill/releases/download/%LATEST_TAG%/port-kill-windows.exe' -OutFile '%INSTALL_DIR%\port-kill.exe'"

REM Download and install console binary
echo Downloading port-kill-console-windows.exe...
powershell -Command "Invoke-WebRequest -Uri 'https://github.com/kagehq/port-kill/releases/download/%LATEST_TAG%/port-kill-console-windows.exe' -OutFile '%INSTALL_DIR%\port-kill-console.exe'"

echo.
echo Installation complete!
echo.
echo Usage:
echo    System tray mode: port-kill.exe --ports 3000,8000
echo    Console mode:     port-kill-console.exe --console --ports 3000,8000
echo.
echo Add to PATH:
echo    Add %INSTALL_DIR% to your system PATH environment variable
echo.
echo For more options: port-kill.exe --help
echo.
pause
