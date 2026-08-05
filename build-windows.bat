@echo off
echo 🪟 Building Port Kill for Windows...

REM Check if we're on Windows
if not "%OS%"=="Windows_NT" (
    echo ⚠️  Warning: This script is designed for Windows systems
    echo    Current OS: %OS%
    echo    You can still build, but testing may not work correctly
    echo.
)

REM Make sure cargo is on PATH (default rustup location)
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

REM GNU toolchain only: raw-dylib needs binutils' as.exe next to rustup's dlltool
if exist "%USERPROFILE%\.cargo\mingw64\bin\as.exe" set "PATH=%USERPROFILE%\.cargo\mingw64\bin;%PATH%"

REM Check if Rust is installed
rustc --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Rust is not installed or not in PATH
    echo    Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Build the Windows version (Cargo.toml handles Windows natively via cfg targets)
echo 🔨 Building with cargo...
cargo build --release --features embed_icon

if errorlevel 1 (
    echo ❌ Build failed!
    echo.
    echo 💡 Common solutions:
    echo    1. Install Rust and Cargo
    echo    2. Try console mode: .\target\release\port-kill-console.exe --console --ports 3000,8000
    exit /b 1
) else (
    echo ✅ Windows version built successfully!
    echo 📦 Binary location: .\target\release\port-kill.exe
    echo 📦 Console binary: .\target\release\port-kill-console.exe
    echo.
    echo 🧪 To test the tray UI:
    echo    .\target\release\port-kill.exe --ports 3000,8000
    echo    Look for the Port Kill icon in your system tray ^(near the clock^)
    echo.
    echo 🧪 To test console mode:
    echo    .\target\release\port-kill.exe --console --ports 3000,8000 --verbose
)
