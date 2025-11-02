@echo off
setlocal enableextensions

rem ---- Config
set "REPO=treadiehq/port-kill"
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
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-windows.exe' -OutFile '%INSTALL_DIR%\port-kill.exe'" || (
  echo(Download failed (port-kill)
  exit /b 1
)

echo(Downloading port-kill-console...
powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing '%BASE_URL%/port-kill-console-windows.exe' -OutFile '%INSTALL_DIR%\port-kill-console.exe'" || (
  echo(Download failed (port-kill-console)
  exit /b 1
)

echo(
echo(Verifying installation...
if not exist "%INSTALL_DIR%\port-kill.exe" (
  echo(ERROR: port-kill.exe not found after download
  exit /b 1
)
if not exist "%INSTALL_DIR%\port-kill-console.exe" (
  echo(ERROR: port-kill-console.exe not found after download
  exit /b 1
)
echo(✅ Both binaries downloaded successfully

echo(
echo(Adding to PATH...
powershell -NoProfile -Command "if (-not (Test-Path 'env:PATH' -PathType Container)) { [Environment]::SetEnvironmentVariable('PATH', '%INSTALL_DIR%', 'User') } else { $currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); if ($currentPath -notlike '*%INSTALL_DIR%*') { [Environment]::SetEnvironmentVariable('PATH', $currentPath + ';%INSTALL_DIR%', 'User') } }"

echo(
echo(==========================================
echo(✅ Installation Complete!
echo(==========================================
echo(
echo(📁 Files installed to: %INSTALL_DIR%
echo(   - port-kill.exe
echo(   - port-kill-console.exe
echo(
echo(⚠️  IMPORTANT: You MUST restart your terminal for PATH changes to take effect!
echo(
echo(🔄 To apply changes:
echo(   1. Close this terminal window completely
echo(   2. Open a new terminal window
echo(   3. Run: port-kill --list
echo(
echo(🧪 Test immediately without restarting (use full path):
echo(   "%INSTALL_DIR%\port-kill.exe" --list
echo(   "%INSTALL_DIR%\port-kill-console.exe" --version
echo(
echo(❌ If you get "not recognized" error after restarting:
echo(   Run diagnostics: 
echo(   powershell -Command "Invoke-WebRequest -UseBasicParsing -Uri 'https://raw.githubusercontent.com/treadiehq/port-kill/main/diagnose-installation.bat' -OutFile 'diagnose.bat'" ^&^& .\diagnose.bat
echo(
echo(📖 Quick start (after restarting terminal):
echo(   port-kill --list
echo(   port-kill 3000
echo(   port-kill --guard 3000
echo(   port-kill cache --list
