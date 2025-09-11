@echo off
REM Port Kill - Windows Run Script
REM This script runs the port-kill application on Windows with logging enabled
REM Usage: run-windows.bat [options]
REM Examples:
REM   run-windows.bat                           REM Default: ports 2000-6000
REM   run-windows.bat --start-port 3000         REM Ports 3000-6000
REM   run-windows.bat --end-port 8080           REM Ports 2000-8080
REM   run-windows.bat --ports 3000,8000,8080    REM Specific ports only
REM   run-windows.bat --console                 REM Run in console mode
REM   run-windows.bat --verbose                 REM Enable verbose logging
REM   run-windows.bat --docker                  REM Enable Docker container monitoring
REM   run-windows.bat --docker --ports 3000,3001 REM Monitor specific ports with Docker
REM   run-windows.bat --show-pid                REM Show process IDs in output
REM   run-windows.bat --console --show-pid      REM Console mode with PIDs shown

echo 🪟 Starting Port Kill on Windows...
echo 📊 Status bar icon should appear shortly
echo.

REM Check if we're on Windows
if not "%OS%"=="Windows_NT" (
    echo ⚠️  Warning: This script is designed for Windows systems
    echo    Current OS: %OS%
    echo    For macOS, use: ./run.sh
    echo    For Linux, use: ./run-linux.sh
    echo.
)

REM Check if the Windows application is built
if not exist ".\target\release\port-kill.exe" (
    echo ❌ Windows application not built. Running Windows build first...
    call build-windows.bat
    if errorlevel 1 (
        echo ❌ Windows build failed!
        exit /b 1
    )
)

REM Run the Windows application with logging and pass through all arguments
set RUST_LOG=info
.\target\release\port-kill.exe %*
