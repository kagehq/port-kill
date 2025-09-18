# Port Kill — Windows Quick Start

A short, practical guide to install and use Port Kill on Windows.

## 1) Install

PowerShell (recommended):

```powershell
# Download + run installer when releases are available
powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/kagehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'" && .\install-release.bat
```

No release yet? Build from source:

```powershell
# Requires Rust (https://rustup.rs/)
git clone https://github.com/kagehq/port-kill.git
cd port-kill
cargo build --release
```

The console binary will be at:
- If using installer: typically `C:\Users\<you>\AppData\Local\port-kill\port-kill-console.exe`
- If building: `target\release\port-kill-console.exe`

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

```powershell
# See what’s using common dev ports
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

- “Command not found” after install: add the install folder to your PATH (see step 2) or open a new shell.
- Access denied / can’t kill a process: run PowerShell/CMD “as Administrator”.
- SmartScreen or AV blocks the exe: open file Properties and “Unblock”, or allow it in your AV.
- Docker processes not showing: ensure Docker Desktop is running and `docker` is on PATH.

## 6) Optional: MCP / HTTP control (automation)

You can drive Port Kill via MCP (Cursor) or plain HTTP:

```powershell
# MCP + HTTP wrapper (from repo root)
cd mcp
$env:PORT_KILL_BIN = "C:\path\to\port-kill-console.exe"    # set if not on PATH
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
