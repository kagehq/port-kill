@echo off
setlocal enableextensions

echo Port Kill Installation Verification
echo ===================================
echo.

set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo Checking installation directory: %INSTALL_DIR%
echo.

if exist "%INSTALL_DIR%\port-kill.exe" (
    echo ✅ port-kill.exe found
) else (
    echo ❌ port-kill.exe NOT found
)

if exist "%INSTALL_DIR%\port-kill-console.exe" (
    echo ✅ port-kill-console.exe found
) else (
    echo ❌ port-kill-console.exe NOT found
)

echo.
echo Checking PATH environment variable...
echo %PATH% | findstr /i "%INSTALL_DIR%" >nul
if %errorlevel% equ 0 (
    echo ✅ Installation directory is in PATH
) else (
    echo ❌ Installation directory is NOT in PATH
    echo.
    echo To fix this, run the install script again or manually add to PATH:
    echo %INSTALL_DIR%
)

echo.
echo Current PATH entries containing 'port-kill':
echo %PATH% | findstr /i "port-kill"

echo.
echo Testing executables...
if exist "%INSTALL_DIR%\port-kill-console.exe" (
    echo Testing port-kill-console.exe...
    "%INSTALL_DIR%\port-kill-console.exe" --help >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✅ port-kill-console.exe is working
    ) else (
        echo ❌ port-kill-console.exe failed to run
    )
)

echo.
echo If you see ❌ errors above, please:
echo 1. Run the install script again: install-release.bat
echo 2. Restart your terminal
echo 3. Try running: port-kill-console --console --ports 3000,8000
