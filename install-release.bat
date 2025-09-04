@echo off
setlocal enableextensions

rem ---- Config
set "REPO=kagehq/port-kill"
set "API=https://api.github.com/repos/%REPO%/releases/latest"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo(Port Kill Release Installer for Windows
echo(==========================================
echo(
echo(Detected platform: Windows
echo(
echo(Fetching latest release information...

rem ---- Get latest tag via PowerShell and capture it into LATEST_TAG
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command ^
  "(Invoke-RestMethod '%API%').tag_name"`) do set "LATEST_TAG=%%i"

if not defined LATEST_TAG (
  echo(ERROR: Failed to get latest release tag.
  echo(Visit: https://github.com/%REPO%/releases
  exit /b 1
)

echo(Latest release: %LATEST_TAG%
echo(

rem ---- Ensure install dir
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo(Installing to: %INSTALL_DIR%
echo(Downloading port-kill-windows.exe...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-windows.exe' -OutFile '%INSTALL_DIR%\port-kill.exe'" || (
  echo(Download failed (port-kill.exe)
  exit /b 1
)

echo(Downloading port-kill-console-windows.exe...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console-windows.exe' -OutFile '%INSTALL_DIR%\port-kill-console.exe'" || (
  echo(Download failed (port-kill-console.exe)
  exit /b 1
)

echo(
echo(Installation complete!
echo(
echo(Usage:
echo(  System tray:    port-kill --ports 3000,8000
echo(  Console mode:   port-kill-console --console --ports 3000,8000
echo(
echo(Add to PATH (User): %INSTALL_DIR%
