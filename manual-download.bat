@echo off
setlocal enableextensions

echo Port Kill Manual Download Tool
echo ==============================
echo.

set "REPO=kagehq/port-kill"
set "API=https://api.github.com/repos/%REPO%/releases/latest"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo Getting latest release information...
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "(Invoke-RestMethod '%API%').tag_name"`) do set "LATEST_TAG=%%i"

if not defined LATEST_TAG (
    echo ❌ ERROR: Cannot get latest release information
    exit /b 1
)

echo ✅ Latest release: %LATEST_TAG%
echo.

echo Creating installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo.
echo Downloading port-kill-windows.exe...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-windows.exe' -OutFile '%INSTALL_DIR%\port-kill.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill-windows.exe
    exit /b 1
)
echo ✅ Downloaded port-kill-windows.exe

echo.
echo Downloading port-kill-console-windows.exe...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console-windows.exe' -OutFile '%INSTALL_DIR%\port-kill-console.exe'"
if %errorlevel% neq 0 (
    echo ❌ Failed to download port-kill-console-windows.exe
    exit /b 1
)
echo ✅ Downloaded port-kill-console-windows.exe

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
