# Port Kill — Windows Quick Start

A short, practical guide to install and use Port Kill on Windows.

## 1) Install

PowerShell (recommended):

```powershell
# Download + run installer when releases are available
powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'" && .\install-release.bat
```

Want to build from source?

```powershell
# Requires Rust (https://rustup.rs/)
git clone https://github.com/kagehq/port-kill.git
cd port-kill
./build-windows.bat
```

The binaries will be at:
- If using installer: typically `C:\Users\<you>\AppData\Local\port-kill\`
  - `port-kill.exe` (tray app, can also run in console mode)
  - `port-kill-console.exe` (console-only app)
- If building: `target\release\`
  - `port-kill.exe` (tray app)
  - `port-kill-console.exe` (console app)

Both binaries are included in releases and provide the same functionality.

## 2) Add to PATH (so you can run from anywhere)

Option A — add install folder to user PATH:

```powershell
$p = [Environment]::GetEnvironmentVariable('Path','User')
$new = $p + ';C:\Users\<you>\AppData\Local\port-kill'
[Environment]::SetEnvironmentVariable('Path',$new,'User')
# Restart your terminal after this
```

Option B — copy the binary into a folder already on PATH (your choice).

## 3) Use the console app (recommended on Windows)

**Option A: Use the main binary in console mode (recommended)**
```powershell
# See what's using common dev ports
port-kill.exe --console --ports 3000,8000,8080

# Free up the usual suspects
port-kill.exe --reset

# Kill specific ports
port-kill.exe --kill-all --ports 3000

# JSON output (great for tooling)
port-kill.exe --console --ports 3000,8000 --json

# Verbose details (command line, cwd)
port-kill.exe --console --ports 3000,8000 --verbose
```

**Option B: Use the dedicated console binary**
```powershell
# See what's using common dev ports
port-kill-console.exe --console --ports 3000,8000,8080

# Free up the usual suspects
port-kill-console.exe --reset

# Kill specific ports
port-kill-console.exe --kill-all --ports 3000

# JSON output (great for tooling)
port-kill-console.exe --console --ports 3000,8000 --json

# Verbose details (command line, cwd)
port-kill-console.exe --console --ports 3000,8000 --verbose
```

## 4) About the tray app

You can try the tray binary `port-kill.exe`. On some Windows setups, the tray icon may fail and the app will fall back to console mode with a warning like:

```
Tray mode failed on Windows (Failed to create Windows tray item …). Falling back to console mode…
```

This is harmless. The console app has the same functionality and is the recommended way to use Port Kill on Windows.

## 5) Troubleshooting

- **"Command not found" after install**: add the install folder to your PATH (see step 2) or open a new shell.
- **"port-kill-console.exe not recognized"**: both binaries should be available in releases. If you only see `port-kill.exe`, you can use `port-kill.exe --console` instead.
- **Only one binary installed**: this may indicate an installation issue. Both `port-kill.exe` and `port-kill-console.exe` should be available.
- **Access denied / can't kill a process**: run PowerShell/CMD "as Administrator".
- **SmartScreen or AV blocks the exe**: open file Properties and "Unblock", or allow it in your AV.
- **Docker processes not showing**: ensure Docker Desktop is running and `docker` is on PATH.

## 6) Scripting

Port Kill now supports programmable port management through scripting:

```powershell
# Port Guarding - Auto-kill unauthorized processes!
port-kill.exe --script "guardPort(3000)" --ports 3000
port-kill.exe --script "guardPort(3000, 'my-dev-server')" --ports 3000

# File-Based Process Management - Kill processes with specific files open!
port-kill.exe --script "killFile('package-lock.json')"
port-kill.exe --script "killFileExt('.lock')"
port-kill.exe --script "guardFile('.env')"

# Three Clear Commands:
# 1. kill(pid) - Kill process by PID (one-time action)
# 2. clearPort(port) - Kill all processes on a specific port (one-time action)
# 3. guardPort(port) - Ongoing protection (kill any process that tries to bind to this port)

# Advanced example
port-kill.exe --script "log('Starting'); clearPort(3000); onPort(8080, callback)" --ports 3000,8080
```

See [SCRIPTING.md](SCRIPTING.md) for complete documentation and examples.

## 7) Optional: MCP / HTTP control (automation)

You can drive Port Kill via MCP (Cursor) or plain HTTP:

```powershell
# MCP + HTTP wrapper (from repo root)
cd mcp
$env:PORT_KILL_BIN = "C:\path\to\port-kill.exe"    # set if not on PATH
$env:HTTP_PORT = "8787"
npm run dev    # then POST http://localhost:8787/tool with { name, args }
```

Examples:

```powershell
# List via HTTP
iwr -UseBasicParsing http://localhost:8787/tool -Method POST -ContentType 'application/json' -Body '{"name":"list","args":{"ports":"3000,8000"}}' | % Content

# Reset via HTTP
iwr -UseBasicParsing http://localhost:8787/tool -Method POST -ContentType 'application/json' -Body '{"name":"reset","args":{}}' | % Content
```

That’s it — happy port killing!
