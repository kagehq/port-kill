@echo off
setlocal enableextensions

echo Port Kill Installation Diagnostic Tool
echo =====================================
echo.

set "REPO=kagehq/port-kill"
set "BASE_URL=https://github.com/%REPO%/releases/latest/download"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo 1. Checking GitHub connectivity...
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "try { Invoke-WebRequest -UseBasicParsing 'https://github.com' -Method Head | Out-Null; 'OK' } catch { 'FAIL' }"`) do set "NET_STATUS=%%i"
if /I "%NET_STATUS%" NEQ "OK" (
    echo ❌ ERROR: Cannot reach GitHub. Please check your internet or proxy settings.
    exit /b 1
)

echo ✅ Network OK

echo.
echo 2. Testing download URLs (latest release)...
echo Testing port-kill download...
powershell -NoProfile -Command "try { $r = Invoke-WebRequest -UseBasicParsing -Uri '%BASE_URL%/port-kill' -Method Head; Write-Host '✅ port-kill: OK (Size:' $r.Headers.'Content-Length' 'bytes)' } catch { Write-Host '❌ port-kill: FAILED -' $_.Exception.Message }"

echo Testing port-kill-console download...
powershell -NoProfile -Command "try { $r = Invoke-WebRequest -UseBasicParsing -Uri '%BASE_URL%/port-kill-console' -Method Head; Write-Host '✅ port-kill-console: OK (Size:' $r.Headers.'Content-Length' 'bytes)' } catch { Write-Host '❌ port-kill-console: FAILED -' $_.Exception.Message }"

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

echo 📥 Download and run installer (bypass cache):
echo    powershell -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing -Headers @{Pragma='no-cache'; 'Cache-Control'='no-cache'} -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'" ^&^& .\install-release.bat

echo.
echo 🔄 After installation, restart your terminal and try:
echo    port-kill-console --console --ports 3000,8000

echo.
echo 📞 If issues persist, please share this diagnostic output with support.
