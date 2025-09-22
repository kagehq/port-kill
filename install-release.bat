@echo off
setlocal enableextensions

rem ---- Config
set "REPO=kagehq/port-kill"
set "BASE_URL=https://github.com/%REPO%/releases/latest/download"
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\port-kill"

echo(Port Kill Release Installer for Windows
echo(==========================================
echo(
echo(Detected platform: Windows

echo(
echo(Preparing installation directory...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo(Installing to: %INSTALL_DIR%

echo(Downloading port-kill...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill' -OutFile '%INSTALL_DIR%\port-kill.exe'" || (
  echo(Download failed (port-kill)
  exit /b 1
)

echo(Downloading port-kill-console...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-console' -OutFile '%INSTALL_DIR%\port-kill-console.exe'" || (
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
