@echo off
setlocal enableextensions

echo Port Kill Manual Download Tool
echo ==============================
echo.

set "REPO=kagehq/port-kill"
set "BASE_URL=https://github.com/%REPO%/releases/latest/download"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo Creating installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo.
echo Downloading port-kill...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-windows.exe' -OutFile '%INSTALL_DIR%\port-kill.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill
    exit /b 1
)
echo ✅ Downloaded port-kill

echo.
echo Downloading port-kill-console...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-console-windows.exe' -OutFile '%INSTALL_DIR%\port-kill-console.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill-console
    exit /b 1
)
echo ✅ Downloaded port-kill-console

echo.
echo Adding to PATH...
powershell -NoProfile -Command "if (-not (Test-Path 'env:PATH' -PathType Container)) { [Environment]::SetEnvironmentVariable('PATH', '%INSTALL_DIR%', 'User') } else { $currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); if ($currentPath -notlike '*%INSTALL_DIR%*') { [Environment]::SetEnvironmentVariable('PATH', $currentPath + ';%INSTALL_DIR%', 'User') } }"

echo.
echo ==========================================
echo ✅ Installation Complete!
echo ==========================================
echo.
echo 📁 Files installed to: %INSTALL_DIR%
echo    - port-kill.exe
echo    - port-kill-console.exe
echo.
echo ⚠️  CRITICAL: You MUST completely restart your terminal!
echo.
echo 🔄 Next steps:
echo    1. Close this terminal window completely
echo    2. Open a new terminal window
echo    3. Test: port-kill --list
echo.
echo 🧪 Test NOW without restarting (use full path):
echo    "%INSTALL_DIR%\port-kill.exe" --list
echo    "%INSTALL_DIR%\port-kill-console.exe" --version
echo.
echo ❌ If you get 'not recognized' error AFTER restarting:
echo    Download diagnostics:
echo    powershell -Command "Invoke-WebRequest -UseBasicParsing -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/diagnose-installation.bat' -OutFile 'diagnose.bat'" ^&^& .\diagnose.bat
