# Port-Kill Scripting Guide

Port-kill now supports **programmable port management** through a simple scripting interface. You can write custom automation scripts to monitor, manage, and react to port events in real-time.

## 🚀 Quick Start

### Basic Usage

```bash
# Execute inline script
./port-kill-console --script "listPorts()"

# Execute script file
./port-kill-console --script-file my-script.js

# Specify ports to monitor
./port-kill-console --script "onPort(3000, callback)" --ports 3000,8080
```

### Example Script

```javascript
// Simple port monitoring
log("Starting port monitoring")
listPorts()
onPort(3000, callback)
```

## Available Commands

### Port Monitoring
- `onPort(port, callback)` - Monitor a specific port for process changes
- `listPorts()` - List all monitored ports

### Port Guarding (NEW!)
- `guardPort(port)` - Automatically kill any process that binds to this port
- `guardPort(port, allowedName)` - Only allow a specific process name on this port, kill everything else
- `killOnPort(port)` - Alternative syntax for `guardPort(port)` - kill any process on this port

### File-Based Process Management (NEW!)
- `killFile("filename.ext")` - Kill all processes that have a specific file open
- `killFileExt(".extension")` - Kill all processes that have files with a specific extension open
- `guardFile("filename.ext")` - Guard a file - kill any process that opens it
- `guardFile("filename.ext", "allowedProcess")` - Only allow a specific process to open the file
- `listFileProcesses("filename.ext")` - List all processes that have a specific file open

### Process Management
- `kill(pid)` - Kill process by PID
- `killPort(port)` - Kill all processes on a specific port
- `getProcess(port)` - Get process information for a port

### Utility Commands
- `log("message")` - Log a message to console
- `wait(seconds)` - Wait for specified seconds

## Use Cases

### 1. Development Port Guard
```javascript
// Auto-kill any process that steals your dev port
guardPort(3000)

// Or only allow your specific dev server
guardPort(3000, "my-react-app")
```

### 2. Port Cleanup
```javascript
// Clean up common development ports
killPort(3000)
killPort(3001)
killPort(5000)
killPort(8000)
killPort(8080)
```

### 3. Multi-Port Guard System
```javascript
// Guard multiple ports with different policies
guardPort(3000)                    // Kill any process on port 3000
guardPort(8080, "nginx")           // Only allow nginx on port 8080
killOnPort(9000)                   // Kill any process on port 9000
```

### 4. File-Based Process Management
```javascript
// Kill processes with problematic files open
killFile("package-lock.json")      // Kill processes with package-lock.json open
killFileExt(".lock")               // Kill processes with any .lock files open
guardFile(".env")                  // Guard .env file - kill any process that opens it
guardFile("config.json", "npm")    // Only allow npm to open config.json
```

### 5. Development Environment Protection
```javascript
// Comprehensive development environment guard
guardFile("package.json")          // Protect package.json
guardFile("package-lock.json")     // Protect package-lock.json
killFileExt(".lock")               // Clear all lock files
guardPort(3000)                    // Guard development port
```

### 6. Resource Monitoring
```javascript
// Monitor high-memory processes
onPort(8080, callback)
// In callback: if (process.memory > 500MB) kill(process.pid)
```

### 7. Security Monitoring
```javascript
// Monitor suspicious ports
onPort(4444, callback)
// In callback: log("Suspicious process detected: " + process.name)
```

## Example Scripts

### Basic Monitoring
```javascript
// examples/simple-script.js
onPort(3000, callback)
listPorts()
kill(1234)
```

### Advanced Port Management
```javascript
// examples/advanced-script.js
log("Starting advanced port management script")
listPorts()
wait(2)
onPort(3000, callback)
killPort(8080)
getProcess(5000)
log("Script setup complete - monitoring active")
```

### Port Cleanup
```javascript
// examples/port-cleanup.js
log("Starting port cleanup script")
killPort(3000)
killPort(3001)
killPort(5000)
killPort(8000)
killPort(8080)
killPort(9000)
log("Port cleanup completed")
onPort(3000, callback)
onPort(8080, callback)
log("Monitoring active - press Ctrl+C to stop")
```

### Port Guarding
```javascript
// examples/port-guard-simple.js
log("Starting simple port guard for port 3000")
guardPort(3000)
log("Port guard activated. Any process on port 3000 will be killed.")
```

### Port Guard with Whitelist
```javascript
// examples/port-guard-whitelist.js
log("Starting port guard with whitelist for port 3000")
guardPort(3000, "my-dev-server")
log("Port guard activated. Only 'my-dev-server' is allowed on port 3000.")
```

### Multi-Port Guard System
```javascript
// examples/port-guard-multi.js
log("Starting multi-port guard system")
guardPort(3000)                    // Kill any process on port 3000
guardPort(8080, "nginx")           // Only allow nginx on port 8080
killOnPort(9000)                   // Kill any process on port 9000
log("Multi-port guard activated")
```

### File-Based Process Management
```javascript
// examples/file-cleanup.js
log("Starting file cleanup script...")
killFileExt(".lock")               // Kill processes with lock files
killFileExt(".log")                // Kill processes with log files
killFile("package-lock.json")      // Kill processes with specific files
killFile("yarn.lock")
log("File cleanup completed")
```

### File Guarding
```javascript
// examples/file-guard-simple.js
log("Starting simple file guard for package.json")
guardFile("package.json")          // Guard package.json
log("File guard activated")
```

### Development Environment Guard
```javascript
// examples/development-guard.js
log("Starting development environment guard...")
guardFile("package.json")          // Protect package.json
guardFile("package-lock.json")     // Protect package-lock.json
guardFile(".env")                  // Protect .env
killFileExt(".lock")               // Clear lock files
guardPort(3000)                    // Guard development port
log("Development environment guard activated")
```

## Script Syntax

### Multiple Commands
Commands can be separated by semicolons:
```javascript
log("Hello"); listPorts(); wait(2); log("Done")
```

### Comments
Use `//` for single-line comments:
```javascript
// This is a comment
onPort(3000, callback) // Monitor port 3000
```

### Parameters
- **Ports**: Use numeric values (e.g., `3000`, `8080`)
- **PIDs**: Use numeric values (e.g., `1234`, `5678`)
- **Messages**: Use quotes (e.g., `"Hello World"`)
- **Seconds**: Use numeric values (e.g., `5`, `10`)

## Event System

The scripting engine provides **event-driven** port monitoring:

- **🟢 NEW**: Process started on port
- **🔄 CHANGED**: Process changed on port
- **🔴 REMOVED**: Process stopped on port

## Advanced Features

### Real-time Monitoring
Scripts can run continuously and react to port events in real-time:

```bash
# Start monitoring (runs until Ctrl+C)
./port-kill-console --script "onPort(3000, callback)" --ports 3000
```

### Port Ranges
Monitor multiple ports:
```bash
./port-kill-console --script "onPort(3000, callback)" --ports 3000,3001,8080,9000
```

## Getting Started

1. **Try a simple script**:
   ```bash
   ./port-kill-console --script "log('Hello World!')"
   ```

2. **Monitor a port**:
   ```bash
   ./port-kill-console --script "onPort(3000, callback)" --ports 3000
   ```

3. **Create your own script**:
   ```bash
   echo 'log("My first script"); listPorts()' > my-script.js
   ./port-kill-console --script-file my-script.js
   ```

4. **Explore examples**:
   ```bash
   ./port-kill-console --script-file examples/advanced-script.js --ports 3000,8080
   ```

---
