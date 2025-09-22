@echo off
setlocal enableextensions

echo Port Kill Installation Diagnostic Tool
echo =====================================
echo.

set "REPO=kagehq/port-kill"
set "API=https://api.github.com/repos/%REPO%/releases/latest"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo 1. Checking GitHub API connectivity...
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "(Invoke-RestMethod '%API%').tag_name"`) do set "LATEST_TAG=%%i"

if not defined LATEST_TAG (
    echo ❌ ERROR: Cannot connect to GitHub API
    echo    This might be a network or firewall issue
    echo    Please check your internet connection
    exit /b 1
)

echo ✅ Latest release: %LATEST_TAG%
echo.

echo 2. Testing download URLs...
echo Testing port-kill download...
powershell -NoProfile -Command "try { $response = Invoke-WebRequest -Uri 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill' -Method Head; Write-Host '✅ port-kill: OK (Size: ' $response.Headers.'Content-Length' ' bytes)' } catch { Write-Host '❌ port-kill: FAILED -' $_.Exception.Message }"

echo Testing port-kill-console download...
powershell -NoProfile -Command "try { $response = Invoke-WebRequest -Uri 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console' -Method Head; Write-Host '✅ port-kill-console: OK (Size: ' $response.Headers.'Content-Length' ' bytes)' } catch { Write-Host '❌ port-kill-console: FAILED -' $_.Exception.Message }"

echo.
echo 3. Checking installation directory...
if exist "%INSTALL_DIR%" (
    echo ✅ Installation directory exists: %INSTALL_DIR%
    echo.
    echo Files in installation directory:
    dir "%INSTALL_DIR%" /b
) else (
    echo ❌ Installation directory does not exist: %INSTALL_DIR%
    echo    This means the install script has not been run yet
)

echo.
echo 4. Checking PATH configuration...
echo %PATH% | findstr /i "%INSTALL_DIR%" >nul
if %errorlevel% equ 0 (
    echo ✅ Installation directory is in PATH
) else (
    echo ❌ Installation directory is NOT in PATH
    echo    This is why 'port-kill-console.exe' is not recognized
)

echo.
echo 5. Recommendations:
echo.
if not exist "%INSTALL_DIR%" (
    echo 📥 Run the installation script:
    echo    powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'" ^&^& .\install-release.bat
) else (
    echo 🔄 Re-run the installation script to fix PATH:
    echo    powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'" ^&^& .\install-release.bat
)

echo.
echo 🔄 After installation, restart your terminal and try:
echo    port-kill-console --console --ports 3000,8000

echo.
echo 📞 If issues persist, please share this diagnostic output with support.
