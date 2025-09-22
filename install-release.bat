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
echo(Downloading port-kill...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill' -OutFile '%INSTALL_DIR%\port-kill.exe'" || (
  echo(Download failed (port-kill)
  exit /b 1
)

echo(Downloading port-kill-console...
powershell -NoProfile -Command "Invoke-WebRequest 'https://github.com/%REPO%/releases/download/%LATEST_TAG%/port-kill-console' -OutFile '%INSTALL_DIR%\port-kill-console.exe'" || (
  echo(Download failed (port-kill-console)
  exit /b 1
)

echo(
echo(Adding to PATH...
powershell -NoProfile -Command "if (-not (Test-Path 'env:PATH' -PathType Container)) { [Environment]::SetEnvironmentVariable('PATH', '%INSTALL_DIR%', 'User') } else { $currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); if ($currentPath -notlike '*%INSTALL_DIR%*') { [Environment]::SetEnvironmentVariable('PATH', $currentPath + ';%INSTALL_DIR%', 'User') } }"

echo(
echo(Installation complete!
echo(
echo(Usage:
echo(  System tray:    port-kill --ports 3000,8000
echo(  Console mode:   port-kill-console --console --ports 3000,8000
echo(
echo(Note: You may need to restart your terminal for PATH changes to take effect.
echo(Installation directory: %INSTALL_DIR%
