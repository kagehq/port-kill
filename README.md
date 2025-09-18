# 🚧 Port Kill

A cross-platform observability, control and status bar that monitors and manages development processes running on configurable ports. The app provides real-time process detection and allows you to kill individual processes or all processes at once.

**Works everywhere:** Local dev, Docker, Kubernetes, WSL, macOS, Linux, Windows, CI/CD pipelines, and cloud environments.

**Supported Platforms:**
- ✅ **macOS**: Native system tray with full functionality
- ✅ **Linux**: Native system tray with full functionality (requires GTK packages)
- ✅ **Windows**: Native system tray with full functionality
- ✅ **Console Mode**: Works on all platforms without GUI dependencies
- ✅ **Docker**: Full container monitoring and management
- ✅ **Kubernetes**: Works in pods and containers for cluster monitoring
- ✅ **WSL**: Full support in Windows Subsystem for Linux
- ✅ **CI/CD**: Perfect for automated environments and build pipelines

![Port Kill Status Bar Icon](image-short.png)

## Community & Support

Join our Discord community for discussions, support, and updates:

[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/KqdBcqRk5E)

## Features

### Core Features (All Platforms)
- **Real-time Monitoring**: Scans configurable ports every 2 seconds using `lsof` commands
- **Process Detection**: Identifies processes by name, PID, and Docker containers
- **Safe Process Termination**: Uses SIGTERM → SIGKILL termination strategy
- **Configurable Port Ranges**: Monitor specific ports or port ranges
- **Ignore Lists**: Exclude specific ports or processes from monitoring
- **Docker Support**: Detect and display Docker container information
- **PID Display**: Optional PID display for better process identification
- **Graceful Error Handling**: Handles permission errors and process failures
- **Log Level Control**: Configurable logging verbosity (info, warn, error, none)
- **Smart Filtering**: Advanced filtering with pattern matching and group-based filtering
- **Performance Monitoring**: Real-time CPU and memory usage tracking
- **Process Grouping**: Automatic categorization of processes (Node.js, Python, Docker, etc.)
- **Project Context**: Extract project names from working directories
- **Process History**: Track killed processes with persistent storage
- **JSON Output**: Machine-readable output for API integration
- **Development Port Reset**: One-command cleanup of common development ports (3000, 5000, 8000, 5432, 3306, 6379, 27017, 8080, 9000)
- **Smart Root Cause Analysis**: Intelligent analysis of process conflicts, workflow patterns, and actionable recommendations
- **Port Guard Mode**: Proactive port conflict prevention with background daemon and auto-resolution
- **Security Audit Mode**: Comprehensive security analysis with suspicious port detection and risk assessment

### Platform-Specific Features
- **macOS**: Native system tray with dynamic context menu and visual status icon
- **Linux**: Native system tray with dynamic context menu and visual status icon
- **Windows**: Native system tray with dynamic context menu and visual status icon
- **Console Mode**: Cross-platform console interface with real-time updates (recommended for full-screen mode)

### Advanced Features
- **Individual Process Killing**: Kill specific processes by clicking menu items
- **Bulk Process Killing**: Kill all detected processes with one click
- **Group-based Killing**: Kill processes by type (Node.js, Python, Docker, etc.)
- **Project-based Killing**: Kill processes by project name
- **Process Restart**: Restart processes after killing them
- **Process Tree View**: Display hierarchical process relationships
- **Ignore Configuration**: Exclude system processes (Chromecast, AirDrop, etc.)
- **Docker Integration**: Display container names and IDs for Docker processes
- **Verbose Mode**: Detailed process information including command line, working directory, and PID for better process identification
- **Smart Defaults**: Intelligent filtering to focus on development processes
- **Pattern Matching**: Advanced regex-based process filtering
- **Quick Actions**: Command-line utilities for bulk operations
- **Status Indicators**: Visual feedback for process health and system status

## Dashboard

A web dashboard is available in the `dashboard/` directory, providing a rich graphical interface for monitoring and managing processes. The dashboard runs independently as a Nuxt.js application and communicates with the Port Kill binary via HTTP API.

![Port Kill Dashboard](dashboard/assets/img/portkill-dashboard.png)

**Features:**
- Real-time process monitoring with auto-refresh
- Visual process table with advanced filtering and search
- System resource monitoring (CPU, Memory, Disk, Load Average)
- Port conflict detection and highlighting
- Process history tracking with persistent storage
- Group and project-based process management
- Performance metrics display (CPU usage, memory usage)
- Container information and Docker integration
- Smart filtering with pattern matching
- Process context and working directory display
- Bulk operations (kill all, kill by group, kill by project)
- Process restart functionality
- Process tree visualization
- **Remote Mode**: SSH-based remote server management
- **Remote Connection Status**: Visual indicators for remote connections

**Quick Start:**
```bash
cd dashboard
npm install
npm run dev
```

The dashboard will be available at `http://localhost:3002` and automatically connects to the Port Kill binary running on the same system.

**Remote Dashboard Configuration:**

**Method 1: Environment Variables**
```bash
# Set environment variables for remote mode
export REMOTE_MODE=true
export REMOTE_HOST=user@staging.company.com

# Start the dashboard with remote mode
cd dashboard && npm run dev
```

**Method 2: Settings Panel (Recommended)**
1. Start the dashboard: `cd dashboard && npm run dev`
2. Open any page (Overview, Processes, or Advanced)
3. Click the **Settings** button in the top-right corner
4. Toggle **Remote Mode (SSH)** checkbox
5. Enter your remote host (e.g., `user@staging.company.com`)
6. Click **Save Settings**

The dashboard will automatically use remote mode when configured, allowing you to manage remote servers through the web interface.

### Dashboard API Endpoints

The dashboard provides a REST API for programmatic access to process monitoring and management:

**Process Management:**
- `GET /api/processes` - Get all processes with filtering options
- `DELETE /api/processes/{pid}` - Kill a specific process by PID
- `POST /api/processes/kill-all` - Kill all processes
- `POST /api/processes/kill-group` - Kill processes by group
- `POST /api/processes/kill-project` - Kill processes by project
- `POST /api/processes/restart` - Restart processes
- `GET /api/processes/tree` - Get process tree view

**History Management:**
- `GET /api/history` - Get process kill history
- `POST /api/history/clear` - Clear process history
- `GET /api/history/offenders` - Get frequent offenders analysis
- `GET /api/history/stats` - Get history statistics
- `GET /api/history/suggestions` - Get ignore suggestions
- `GET /api/history/root-cause` - Get smart root cause analysis

**Port Guard Management:**
- `GET /api/guard/status` - Get Port Guard daemon status

**Security Audit:**
- `GET /api/security/audit` - Perform comprehensive security audit with suspicious port detection

**System Information:**
- `GET /api/system/resources` - Get system resource usage
- `GET /api/filters` - Get filter information

**Example API Usage:**
```bash
# Get all processes
curl "http://localhost:3002/api/processes?ports=3000,8000&performance=true"

# Kill all processes
curl -X POST "http://localhost:3002/api/processes/kill-all?ports=3000,8000"

# Get process history
curl "http://localhost:3002/api/history"

# Get system resources
curl "http://localhost:3002/api/system/resources"
```

### Unified Quick Start

Prerequisites:
- Rust toolchain (via `rustup`)
- Node.js 18+ and npm

Clone and install (CLI only):
```bash
git clone https://github.com/kagehq/port-kill.git
cd port-kill
./install.sh
```

Run the CLI:
```bash
./run.sh                   # macOS
./run-linux.sh             # Linux
run-windows.bat            # Windows
```

Install dashboard deps only (Nuxt):
```bash
./install.sh --dashboard
cd dashboard && npm run dev  # http://localhost:3002
```

Full setup (CLI build + dashboard deps):
```bash
./install.sh --all
```

Build dashboard for deployment:
```bash
cd dashboard
npm run build     # Nitro server output: .output/
# or static build
npm run generate  # Static output: dist/
```

Serve the dashboard from the Rust binary (optional):
- Build a static dashboard (`npm run generate`) and serve `dist/` with a static-files handler in your Rust app, or run the Nitro output alongside the binary.

## Status Bar Icon

The status bar icon provides instant visual feedback:

- **Green**: 0 processes (safe, no development servers)
- **Orange**: 1-9 processes (some development servers)
- **Red**: 10+ processes (many development servers)

Hover over the icon to see the exact process count in the tooltip.

## Menu Options

- **Kill All Processes**: Terminates all detected development processes
- **Individual Process Entries**: 
  - Docker containers: "Kill: Port 3001: node [Docker: my-react-app]"
  - Regular processes: "Kill: Port 3001: node" (or "Kill: Port 3001: node (PID 1234)" with `--show-pid`)
- **Quit**: Exits the application gracefully without affecting monitored processes

**Safety Features**: 
- The "Kill All Processes" option terminates only development processes (respects ignore lists)
- The "Quit" option exits the application without killing any processes
- Individual process killing respects ignore configurations

## Requirements

### macOS
- macOS 10.15 or later
- Rust 1.70 or later
- `lsof` command (included with macOS)
- Docker (optional, for container monitoring)

### Linux
- Linux with GTK support
- Rust 1.70 or later
- `lsof` command
- Docker (optional, for container monitoring)
- **Required packages for system tray**: `libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libxdo-dev`
- **Note**: If GTK packages are missing, the app automatically falls back to console mode

### Windows
- Windows 10 or later
- Rust 1.70 or later
- `netstat` command (included with Windows)
- `tasklist` command (included with Windows)
- Docker (optional, for container monitoring)

### Console Mode (All Platforms)
- Rust 1.70 or later
- `lsof` command (Unix-like systems)
- `netstat` command (Windows)
- Docker (optional, for container monitoring)
- **No GUI dependencies required**

## Deployment Environments

Port Kill works seamlessly across all modern development and deployment environments:

### 🐳 **Docker & Container Environments**
```bash
# Run in Docker container
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock port-kill-console --docker

# Monitor containerized applications
./port-kill-console --docker --ports 3000,8000,8080
```

### ☸️ **Kubernetes & Cluster Environments**
```bash
# Deploy as a sidecar container in pods
kubectl run port-kill --image=port-kill-console -- --audit --json

# Monitor services across the cluster
./port-kill-console --console --ports 3000-8080 --json > audit-results.json
```

### 🐧 **WSL (Windows Subsystem for Linux)**
```bash
# Full Linux compatibility in WSL
./port-kill-console --console --ports 3000,8000,8080

# Works with Windows processes via WSL
./port-kill-console --audit --security-mode
```

### 🏗️ **CI/CD & Build Pipelines**
```bash
# Automated security auditing in CI
./port-kill-console --audit --json > security-report.json

# Clean up development ports in build scripts
./port-kill-console --reset --ports 3000,8000,8080
```

### 🌐 **Cloud & Remote Environments**
```bash
# SSH into remote servers
ssh user@server "./port-kill-console --audit --json"

# Fleet-wide security auditing
for server in $(cat server-list.txt); do
  ssh $server "./port-kill-console --audit --json" > audit-$server.json
done
```

### 🚀 **Remote Mode - Instant Staging/Prod Management**
```bash
# Monitor remote staging server
./port-kill-console --remote user@staging.company.com --ports 3000,8000,8080

# Security audit on production server
./port-kill-console --remote admin@prod.company.com --audit --json

# Kill processes on remote server
./port-kill-console --remote deploy@server.com --reset --ports 3000,8000

# Monitor with Docker support on remote
./port-kill-console --remote user@server.com --docker --ports 3000-8080 --json
```

## Installation

**⚠️ Important**: The first release is currently being prepared. If you get a "Not Found" error when using the install scripts, it means the release binaries haven't been published yet. Use the manual installation method below until the first release is available.

### Automated Releases

The latest binaries for all platforms are automatically built and released on GitHub when a release is published.

#### Quick Install (When Releases Are Available)
```bash
# macOS/Linux: Download and run the installer
curl -fsSL https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.sh | bash

# Windows: Download and run the installer
# Option 1: Download and run manually
Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'
# .\install-release.bat

# Option 2: One-liner (PowerShell)
powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'; .\install-release.bat"
```

**Note**: If no releases are available yet, the install scripts will guide you to build from source instead.

#### Manual Download (When Releases Are Available)
1. **Download from Releases**: Go to [GitHub Releases](https://github.com/kagehq/port-kill/releases) and download the appropriate binary for your platform
2. **Direct Downloads**:
   - **macOS**: `port-kill-macos` (system tray) or `port-kill-console-macos` (console mode)
   - **Linux**: `port-kill-linux` (system tray) or `port-kill-console-linux` (console mode)  
   - **Windows**: `port-kill-windows.exe` (system tray) or `port-kill-console-windows.exe` (console mode)
3. **Archive Downloads**: Compressed archives with both binaries for each platform

#### Creating a Release

**Option 1: Automated Release (Recommended)**
```bash
# Create a new release with automatic tag creation
./release.sh 0.x.x
```

This will:
1. Create a git tag `v0.x.x`
2. Push the tag to GitHub
3. Automatically trigger release creation
4. Build and upload binaries for all platforms

**Option 2: Manual Release**
1. Go to the repository on GitHub
2. Click "Releases" in the right sidebar
3. Click "Create a new release"
4. Set the tag version (e.g., `v0.x.x`)
5. Add a title and release notes
6. Click "Publish release"

Both methods will automatically trigger the build workflow and upload all platform binaries.

### Manual Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd port-kill
```

2. Install and build (recommended):
```bash
./install.sh
```

Or manually:
```bash
./build-macos.sh
./run.sh
```

### Linux Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd port-kill
```

2. Install required packages:
```bash
# Ubuntu/Debian
sudo apt-get install libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libappindicator3-dev

# Fedora/RHEL
sudo dnf install atk-devel gdk-pixbuf2-devel gtk3-devel libappindicator-gtk3-devel

# Arch Linux
sudo pacman -S atk gdk-pixbuf2 gtk3 libappindicator-gtk3
```

3. Install and build (recommended):
```bash
./install.sh
```

Or manually:
```bash
./build-linux.sh
./run-linux.sh
```

### Windows Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd port-kill
```

2. Install Rust (if not already installed):
```bash
# Download and run rustup-init.exe from https://rustup.rs/
```

3. Install and build (recommended):
```bash
./install.sh
```

Or manually:
```bash
build-windows.bat
run-windows.bat
```

#### Windows Tray Icon (embedded)

To embed a proper tray icon in the Windows executable:

1) The source SVG lives at `assets/port-kill.svg`.

2) Convert it to a multi-size `.ico` (requires ImageMagick):

```bash
convert -background none assets/port-kill.svg -define icon:auto-resize=16,24,32,48,64,128,256 assets/port-kill.ico
```

3) Build on Windows (or let CI build). The build script (`build.rs`) will detect `assets/port-kill.ico` and embed it.

Notes:
- If the icon is missing, the app tries common resource fallbacks. If tray creation still fails, it automatically falls back to console mode.
- The code first attempts resource `APPICON`, then other common IDs.

### Console Mode (All Platforms)

Console mode works on all platforms without GUI dependencies:

```bash
# Build (works on any platform)
cargo build --release

# Run console mode
./target/release/port-kill-console --console --ports 3000,8000 --verbose
```


## Usage

### Basic Usage

**Platform-Specific Run Scripts:**
- **macOS**: Use `./run.sh` 
- **Linux**: Use `./run-linux.sh`
- **Windows**: Use `run-windows.bat`

1. **Start the Application**: Run the appropriate script for your platform with default settings (ports 2000-6000)
2. **Monitor Status**: Check the status bar for the process count indicator
3. **Access Menu**: Click on the status bar icon to open the context menu
4. **Kill Processes**: 
   - Click "Kill All Processes" to terminate all development processes
   - Click individual process entries to kill specific processes
5. **Quit**: Click "Quit" to exit the application

### Configurable Port Monitoring

The application now supports configurable port ranges and specific port monitoring:

#### Port Range Examples
```bash
# Monitor ports 3000-8080
./run.sh --start-port 3000 --end-port 8080          # macOS
./run-linux.sh --start-port 3000 --end-port 8080    # Linux
run-windows.bat --start-port 3000 --end-port 8080   # Windows

# Monitor ports 8000-9000
./run.sh -s 8000 -e 9000                            # macOS
./run-linux.sh -s 8000 -e 9000                      # Linux
run-windows.bat -s 8000 -e 9000                     # Windows
```

#### Specific Ports Examples
```bash
# Monitor only specific ports (common dev ports)
./run.sh --ports 3000,8000,8080,5000                # macOS
./run-linux.sh --ports 3000,8000,8080,5000          # Linux
run-windows.bat --ports 3000,8000,8080,5000         # Windows

# Monitor React, Node.js, and Python dev servers
./run.sh -p 3000,3001,8000,8080                     # macOS
./run-linux.sh -p 3000,3001,8000,8080               # Linux
run-windows.bat -p 3000,3001,8000,8080              # Windows
```

#### Console Mode
```bash
# Run in console mode for debugging
./run.sh --console --ports 3000,8000,8080

# Console mode with verbose logging
./run.sh -c -p 3000,8000,8080 -v

# Console mode with PIDs shown
./run.sh --console --show-pid --ports 3000,8000,8080

# Console mode for full-screen mode users (recommended)
./run.sh --console --log-level warn --ports 3000,8000,8080

#### Log Level Control
```bash
# Show all logs (default)
./run.sh --log-level info --ports 3000,8000

# Show only warnings and errors
./run.sh --log-level warn --ports 3000,8000

# Show only errors
./run.sh --log-level error --ports 3000,8000

# Show no logs (quiet mode)
./run.sh --log-level none --ports 3000,8000

# Verbose mode (overrides log-level)
./run.sh --verbose --ports 3000,8000
```

### Development Port Reset

The `--reset` flag provides cleanup of common development ports, perfect for quickly freeing up your development environment:

```bash
# Reset all common development ports
./target/release/port-kill-console --reset
```

### Enhanced History & Analytics

Port Kill now includes powerful history analysis features to help you understand and optimize your development workflow:

#### **Frequent Offenders Detection**
```bash
./target/release/port-kill-console --show-offenders
```
Shows processes that have been killed multiple times, helping you identify recurring conflicts:
```
🚨 Frequent Offenders (killed 2+ times):
────────────────────────────────────────────────────────────────────────────────
1. Node.js Server on port 3001 (killed 2 times over 0m)
   Group: Node.js
   Project: port-kill
   First killed: 2025-09-15 21:44
   Last killed: 2025-09-15 21:44

💡 Consider adding these to your ignore lists to avoid repeated kills!
```

#### **Time Pattern Analysis**
```bash
./target/release/port-kill-console --show-patterns
```
Analyzes when you most commonly kill processes, showing peak hours and days:
```
📊 Time Patterns Analysis:
──────────────────────────────────────────────────
Total kills: 5
Peak hour: 14:00
Peak day: Mon

📈 Hour Distribution:
14:00 │████████████████████ 5 kills
```

#### **Auto-Suggestions for Ignore Lists**
```bash
./target/release/port-kill-console --show-suggestions
```
Provides intelligent suggestions for ignore lists based on your kill history:
```
💡 Auto-Suggestions for Ignore Lists:
────────────────────────────────────────────────────────────
🔌 Suggested Ports to Ignore:
  --ignore-ports 3001

⚙️  Suggested Process Names to Ignore:
  --ignore-processes Node.js Server

📋 Complete Command Example:
./port-kill-console --console --ports 3000,8000 --ignore-ports 3001 --ignore-processes "Node.js Server"
```

#### **Detailed Statistics**
```bash
./target/release/port-kill-console --show-stats
```
Shows comprehensive statistics about your process kill history:
```
📊 History Statistics:
──────────────────────────────────────────────────
Total kills: 5
Unique processes: 3
Unique ports: 2
Unique projects: 1
Average kills per day: 5.0
Most killed process: Python Process (2 times)
Most killed port: 3001 (3 times)
Most killed project: port-kill (2 times)
```

#### **Smart Root Cause Analysis**
```bash
./target/release/port-kill-console --show-root-cause
```
Provides intelligent analysis of your development workflow patterns:
```
🧠 Smart Root Cause Analysis:
────────────────────────────────────────────────────────────
Analysis of 5 process kills revealed 1 conflicts, 1 workflow patterns, and 2 recommendations for improvement.

⚠️  Detected Conflicts:
1. Port 3001 - PortCollision (Severity: Low)
   Processes: Node.js Process, Node.js Server
   Recommendation: Consider using different ports for different services. Port 3001 is being used by multiple processes.

📊 Workflow Patterns:
1. Most kills happen around 22:00 (Confidence: 70%)
   Type: TimeBased
   Frequency: 3 kills at this hour
   Recommendation: Consider scheduling development work or adding processes to ignore list during peak hours.

💡 Smart Recommendations:
1. Add Frequent Offenders to Ignore List (Priority: High)
   Category: ProcessManagement
   Description: 2 processes are being killed repeatedly
   Action: Use --ignore-processes flag to prevent repeated kills
   Impact: Reduces manual intervention and improves workflow efficiency

2. Resolve Port Conflicts (Priority: Medium)
   Category: PortOptimization
   Description: 1 ports have conflicting processes
   Action: Use different ports for different services or add conflicting ports to ignore list
   Impact: Prevents port binding errors and improves service reliability
```

#### **Port Guard Mode**
```bash
./target/release/port-kill-console --guard-mode --auto-resolve
```
Proactive port conflict prevention with background daemon:
```
🛡️  Port Guard daemon started, watching ports: [3000, 3001, 3002, 8000, 8080, 9000]
⚠️  Port conflict detected on port 3000: node vs python
🔧 Auto-resolving port conflict on 3000 by killing process node (PID: 1234)
✅ Port conflict resolved on port 3000
```

**Features:**
- **Background Monitoring**: Continuously watches specified ports for conflicts
- **Auto-Resolution**: Automatically kills conflicting processes when detected
- **Process Interception**: Intercepts development commands (npm start, python -m http.server) and checks for port conflicts before execution
- **Port Reservations**: Reserve ports for specific projects with expiration
- **Smart Conflict Detection**: Identifies port collisions and process conflicts
- **Persistent Storage**: Saves port reservations across restarts

**Example Usage:**
```bash
# Start guard mode with auto-resolution
./port-kill-console --guard-mode --auto-resolve

# Watch specific ports
./port-kill-console --guard-mode --guard-ports 3000,8000,5432

# Custom reservation file
./port-kill-console --guard-mode --reservation-file ~/.my-reservations.json

# Enable process interception for development commands
./port-kill-console --guard-mode --auto-resolve --intercept-commands
```

#### **Security Audit Mode**
```bash
./target/release/port-kill-console --audit --security-mode
```
Comprehensive security analysis with suspicious port detection:
```
🔒 SECURITY AUDIT RESULTS
══════════════════════════════════════════════════
📊 Audit Timestamp: 2025-09-18 02:26:06 UTC
🔍 Total Ports Scanned: 4
🛡️  Security Score: 0.0/100

🚨 SUSPICIOUS ACTIVITY DETECTED:
1. Port 8444: suspicious-miner (PID: 12345)
   Risk Level: Critical
   Reason: SuspiciousPort
   Binary Hash: sha256:abc123def456
   Network: 0.0.0.0

💡 SECURITY RECOMMENDATIONS:
1. Investigate Suspicious Processes (Priority: High)
   1 suspicious processes detected
   Action: Review and terminate suspicious processes immediately
```

**Key Features:**
- **Suspicious Port Detection**: Flags crypto miner ports (8444, 4444, 9999, etc.)
- **Process Analysis**: Detects unknown binaries, unexpected locations, high privileges
- **Risk Assessment**: 4-level risk scoring (Low, Medium, High, Critical)
- **Security Scoring**: 0-100 security score based on findings
- **JSON Output**: Perfect for SIEM integration and fleet deployment
- **Baseline Comparison**: Compare against approved port configurations

**Example Usage:**
```bash
# Basic security audit
./port-kill-console --audit

# Enhanced security mode with custom suspicious ports
./port-kill-console --audit --security-mode --suspicious-ports 8444,4444,9999,14444

# Show only suspicious processes (clean output)
./port-kill-console --audit --suspicious-only

# JSON output for SIEM integration
./port-kill-console --audit --json

# Baseline comparison
./port-kill-console --audit --baseline-file /etc/approved-ports.json

# Fleet deployment across multiple servers
for server in $(cat server-list.txt); do
  ssh $server "./port-kill-console --audit --json" > audit-$server.json
done
```

#### Verbose Mode

The `--verbose` flag provides detailed process information to help you identify the right processes to kill:

**What you get in verbose mode:**
- **Command Line**: Full command with arguments (e.g., `Python -m http.server 8000`)
- **Working Directory**: Process working directory (e.g., `- ~/my-project`)
- **Process ID**: Always shown for better identification
- **Debug Logging**: Detailed internal logging for troubleshooting

**Example output:**
```bash
# Basic mode
Port 3000: node server.js (PID 1234)

# Verbose mode
Port 3000: node server.js (PID 1234) - ~/port-kill
Port 8000: Python -m http.server 8000 (PID 5678) - ~/port-kill
```

**Usage:**
```bash
# Console mode with verbose output
./run.sh --console --ports 3000,8000 --verbose

# Tray mode with verbose menu items
./run.sh --ports 3000,8000 --verbose
```

#### Docker Integration
```bash
# Monitor ports including Docker containers
./run.sh --docker --ports 3000,3001,8000,8080

# Monitor port range with Docker support
./run.sh -d -s 3000 -e 8080

# Console mode with Docker monitoring
./run.sh --console --docker --ports 3000,8000,8080
```

#### Ignoring System Processes
```bash
# Ignore common system ports (Chromecast, AirDrop, etc.)
./run.sh --ignore-ports 5353,5000,7000

# Ignore specific process names
./run.sh --ignore-processes Chrome,ControlCe,rapportd

# Combine both ignore options
./run.sh --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe

# Console mode with ignore options
./run.sh --console --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe
```

**Common System Processes to Ignore:**
- **Port 5353**: Google Chromecast service
- **Port 5000**: Apple AirDrop service (ControlCe)
- **Port 7000**: Apple AirDrop service
- **Process "Chrome"**: Google Chrome browser
- **Process "ControlCe"**: Apple Control Center/AirDrop
- **Process "rapportd"**: Apple Rapport service

**Docker Features:**
- Detects processes running inside Docker containers
- Shows container names prominently in the menu and console output (no PID for containers)
- Automatically stops containers when killing processes
- Uses `docker stop` for graceful termination, `docker rm -f` as fallback

#### All Command-Line Options
- `--start-port, -s`: Starting port for range scanning (default: 2000)
- `--end-port, -e`: Ending port for range scanning (default: 6000)
- `--ports, -p`: Specific ports to monitor (comma-separated, overrides start/end range)
- `--ignore-ports`: Ports to ignore (comma-separated, e.g., 5353,5000,7000 for Chromecast/AirDrop)
- `--ignore-processes`: Process names to ignore (comma-separated, e.g., Chrome,ControlCe)
- `--console, -c`: Run in console mode instead of status bar mode
- `--verbose, -v`: Enable verbose mode with detailed process information (command line, working directory, PID) and debug logging
- `--log-level`: Control logging verbosity (info, warn, error, none) (default: info)
- `--docker, -d`: Enable Docker container monitoring (includes containers in process detection)
- `--show-pid, -P`: Show process IDs (PIDs) in the display output
- `--help, -h`: Show help information
- `--version, -V`: Show version information

#### Advanced Command-Line Options

The console application now supports many advanced features for power users:

**Smart Filtering:**
- `--smart-filter`: Enable smart filtering (focuses on development processes)
- `--only-groups`: Filter by process groups (e.g., Node.js,Python,Docker)
- `--ignore-groups`: Ignore specific groups (e.g., Docker,system)
- `--ignore-patterns`: Ignore by regex patterns (e.g., "systemd|kernel")

**Performance Monitoring:**
- `--performance`: Enable CPU and memory usage tracking
- `--show-context`: Show process context (project names, working directories)

**Quick Actions:**
- `--kill-all`: Kill all processes immediately
- `--kill-group`: Kill processes by group (e.g., Node.js)
- `--kill-project`: Kill processes by project name
- `--restart`: Restart processes after killing them
- `--reset`: Reset common development ports (3000, 5000, 8000, 5432, 3306, 6379, 27017, 8080, 9000)
- `--show-tree`: Display hierarchical process relationships

**History Management:**
- `--show-history`: Show process kill history
- `--clear-history`: Clear process history
- `--show-filters`: Show filter information
- `--show-offenders`: Show frequent offenders (processes killed multiple times)
- `--show-patterns`: Show time patterns and statistics
- `--show-suggestions`: Show auto-suggestions for ignore lists
- `--show-stats`: Show detailed history statistics
- `--show-root-cause`: Show smart root cause analysis
- `--guard-mode`: Enable Port Guard Mode (proactive port conflict prevention)
- `--guard-ports`: Ports to watch in guard mode (default: 3000,3001,3002,8000,8080,9000)
- `--auto-resolve`: Auto-resolve conflicts by killing conflicting processes
- `--reservation-file`: Port reservation file path (default: ~/.port-kill/reservations.json)
- `--intercept-commands`: Enable process interception for development commands
- `--audit`: Enable Security Audit Mode (comprehensive security analysis)
- `--security-mode`: Enhanced security mode with suspicious port detection
- `--suspicious-ports`: Suspicious ports to flag (default: 8444,4444,9999,14444,5555,6666,7777)
- `--baseline-file`: Baseline file for approved ports comparison
- `--suspicious-only`: Show only suspicious/unauthorized processes
- `--remote <host>`: Remote mode - connect to remote host via SSH for instant staging/prod management

**JSON Output:**
- `--json`: Output processes as JSON (for API integration)

**Example Advanced Usage:**
```bash
# Smart filtering with performance monitoring
./target/release/port-kill-console --smart-filter --performance --ports 3000,8000

# Kill all Node.js processes
./target/release/port-kill-console --kill-group Node.js --ports 3000,8000

# Reset common development ports (one-command cleanup)
./target/release/port-kill-console --reset

# Show process tree with context
./target/release/port-kill-console --show-tree --show-context --ports 3000,8000

# JSON output for API integration
./target/release/port-kill-console --json --performance --ports 3000,8000

# Enhanced history analysis
./target/release/port-kill-console --show-offenders
./target/release/port-kill-console --show-patterns
./target/release/port-kill-console --show-suggestions
./target/release/port-kill-console --show-stats
./target/release/port-kill-console --show-root-cause

# Port Guard Mode - proactive conflict prevention
./target/release/port-kill-console --guard-mode --auto-resolve
./target/release/port-kill-console --guard-mode --guard-ports 3000,8000,5432

# Security Audit Mode - comprehensive security analysis
./target/release/port-kill-console --audit --security-mode
./target/release/port-kill-console --audit --suspicious-only --json

# Remote Mode - instant staging/prod management
./target/release/port-kill-console --remote user@staging.company.com --ports 3000,8000
./target/release/port-kill-console --remote admin@prod.company.com --audit --json
```

## Technical Details

### Architecture

- **Main Thread**: Handles UI events and menu interactions with winit event loop
- **Process Monitor**: Scans for processes every 2 seconds using `lsof` and `sysinfo`
- **Menu Updates**: Updates context menu every 3 seconds when processes change
- **Process Killing**: Runs in background threads to maintain UI responsiveness
- **Smart Filtering**: Advanced pattern matching and group-based filtering
- **Performance Monitoring**: Real-time CPU and memory usage tracking
- **Process History**: Persistent storage of killed processes with timestamps
- **JSON API**: Machine-readable output for dashboard integration

### Process Detection

The application uses the following command to detect processes:
```bash
lsof -ti :PORT -sTCP:LISTEN
```

### Process Termination

1. **SIGTERM**: First attempts graceful termination
2. **SIGKILL**: If process doesn't terminate within 500ms, forces termination
3. **Error Handling**: Gracefully handles permission errors and already-terminated processes

### Port Range

Monitors ports 2000-6000 (broad range covering common development server ports)

## Dependencies

- `tray-icon`: macOS status bar integration
- `winit`: Event loop management
- `nix`: Signal handling for process termination
- `crossbeam-channel`: Thread communication
- `tokio`: Async runtime
- `anyhow`: Error handling
- `serde`: Data serialization

## Development

### Building for Development

```bash
cargo build
```

### Running with Logging

```bash
RUST_LOG=info cargo run
```

### GitHub Actions

This project uses GitHub Actions for automated building and testing:

- **Build and Test** (`.github/workflows/build.yml`): Runs on pull requests and pushes to main/master
  - Builds binaries for all platforms (macOS, Linux, Windows)
  - Tests that binaries can run and show help
  - Tests console mode functionality

- **Build and Release** (`.github/workflows/release.yml`): Runs when a new release is published
  - Builds optimized binaries for all platforms
  - Creates compressed archives and individual binary files
  - Uploads all assets to the GitHub release

### Local Development

To test the build scripts locally:

```bash
# macOS
./build-macos.sh

# Linux  
./build-linux.sh

# Windows
build-windows.bat
```

## Troubleshooting

### Permission Issues

If you encounter permission errors when trying to kill processes:

1. Ensure the application has the necessary permissions
2. Some system processes may be protected
3. Check if the process is owned by another user

### Process Not Detected

If a process is not being detected:

1. Verify the process is listening on a port in the 2000-6000 range
2. Check if the process is using TCP (not UDP)
3. Ensure the process is in LISTEN state

### Application Not Starting

If the application fails to start:

1. Check if another instance is already running
2. Verify all dependencies are installed
3. Check system logs for error messages

### Full-Screen Mode Issues

If the system tray menu is not accessible when applications are in full-screen mode:

1. **Use Console Mode**: The most reliable solution is to use console mode:
   ```bash
   ./run.sh --console --ports 3000,8000
   ```

2. **Exit Full-Screen**: Temporarily exit full-screen mode to access the system tray

3. **Use Mission Control**: On macOS, use Mission Control (F3 or three-finger swipe up) to access the menu bar

4. **Alternative Access**: The app provides console output with process status updates, so you can monitor activity even when the menu is not accessible

### Linux System Tray Issues

If the Linux system tray is not working or processes are not showing:

1. **Run the debug script** to diagnose issues:
   ```bash
   ./debug_linux.sh
   ```

2. **Install GTK packages** (required for system tray):
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libappindicator3-dev pkg-config
   
   # Fedora/RHEL
   sudo dnf install atk-devel gdk-pixbuf2-devel gtk3-devel libappindicator-gtk3-devel pkg-config
   
   # Arch Linux
   sudo pacman -S atk gdk-pixbuf2 gtk3 libappindicator-gtk3 pkg-config
   ```

3. **Use console mode** (works without GUI dependencies):
   ```bash
   ./run-linux.sh --console --ports 3000,8000 --verbose
   ```

4. **Check display environment**:
   - Ensure you're running in a desktop environment (not SSH without X11 forwarding)
   - Verify `DISPLAY` environment variable is set: `echo $DISPLAY`
   - Try running in a terminal emulator (not pure console)

5. **Common Linux issues**:
   - **Tray icon not showing**: Check if you're in a desktop environment with system tray support
   - **Processes not detected**: Verify `lsof` is installed and working: `lsof -i :3000`
   - **GTK errors**: Install missing GTK packages or use console mode
   - **Permission issues**: Run with appropriate permissions or use `sudo` if needed

6. **For detailed debugging**:
   ```bash
   RUST_LOG=debug ./run-linux.sh --console --ports 3000,8000
   ```

### Docker Issues

If Docker integration is not working:

1. Ensure Docker Desktop is running
2. Verify `docker` command is available in PATH
3. Check Docker permissions and access
4. Ensure containers are running and accessible
5. Try running with `--verbose` flag for detailed logging

## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request
