@echo off
REM Port Kill Dashboard Launcher for Windows
REM This script builds the Rust application and starts the dashboard

echo 🚀 Starting Port Kill Dashboard...

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo ❌ Error: Please run this script from the port-kill root directory
    exit /b 1
)

REM Build the Rust application
echo 🔨 Building Port Kill Rust application...
cargo build --release

REM Check if build was successful
if not exist "target\release\port-kill-console.exe" (
    echo ❌ Error: Failed to build port-kill-console binary
    exit /b 1
)

echo ✅ Rust application built successfully

REM Check if dashboard directory exists
if not exist "port-kill-dashboard" (
    echo ❌ Error: Dashboard directory not found. Please ensure port-kill-dashboard exists.
    exit /b 1
)

REM Install dashboard dependencies if needed
if not exist "port-kill-dashboard\node_modules" (
    echo 📦 Installing dashboard dependencies...
    cd port-kill-dashboard
    npm install
    cd ..
)

REM Start the dashboard
echo 🌐 Starting dashboard on http://localhost:3001...
cd port-kill-dashboard
npm run dev
