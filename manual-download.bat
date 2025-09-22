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
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill' -OutFile '%INSTALL_DIR%\port-kill.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill
    exit /b 1
)
echo ✅ Downloaded port-kill

echo.
echo Downloading port-kill-console...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-console' -OutFile '%INSTALL_DIR%\port-kill-console.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill-console
    exit /b 1
)
echo ✅ Downloaded port-kill-console

echo.
echo Adding to PATH...
powershell -NoProfile -Command "if (-not (Test-Path 'env:PATH' -PathType Container)) { [Environment]::SetEnvironmentVariable('PATH', '%INSTALL_DIR%', 'User') } else { $currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); if ($currentPath -notlike '*%INSTALL_DIR%*') { [Environment]::SetEnvironmentVariable('PATH', $currentPath + ';%INSTALL_DIR%', 'User') } }"

echo.
echo ✅ Installation complete!
echo.
echo 📁 Files installed to: %INSTALL_DIR%
echo    - port-kill.exe
echo    - port-kill-console.exe
echo.
echo 🔄 IMPORTANT: Restart your terminal for PATH changes to take effect
echo.
echo 🧪 Test the installation:
echo    port-kill-console --console --ports 3000,8000
echo.
echo 💡 If you still get 'not recognized' error:
echo    1. Restart your terminal completely
echo    2. Or use the full path: "%INSTALL_DIR%\port-kill-console.exe"
