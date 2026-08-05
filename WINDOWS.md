# Port Kill — Windows Quick Start

A short, practical guide to install and use Port Kill on Windows.

## 1) Install

PowerShell or CMD (simple):

```powershell
curl.exe -L "https://raw.githubusercontent.com/treadiehq/port-kill/main/install-release.bat" -o install-release.bat
.\\install-release.bat
```

**⚠️ IMPORTANT:** After installation completes, you MUST completely restart your terminal (close and reopen) for PATH changes to take effect. This is not optional!

Alternative (PowerShell with cache-bypass):

```powershell
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -UseBasicParsing -Headers @{Pragma='no-cache'; 'Cache-Control'='no-cache'} -Uri "https://raw.githubusercontent.com/treadiehq/port-kill/main/install-release.bat" -OutFile "install-release.bat"
.\install-release.bat
```

Want to build from source?

```powershell
# Requires Rust (https://rustup.rs/)
git clone https://github.com/treadiehq/port-kill.git
cd port-kill
./build-windows.bat
```

Manual fallback (direct download of release assets):

```powershell
$tag = (Invoke-RestMethod "https://api.github.com/repos/treadiehq/port-kill/releases/latest").tag_name
$dir = "$env:USERPROFILE\AppData\Local\port-kill"; New-Item -ItemType Directory -Force -Path $dir | Out-Null
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/treadiehq/port-kill/releases/download/$tag/port-kill" -OutFile "$dir\port-kill.exe"
Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/treadiehq/port-kill/releases/download/$tag/port-kill-console" -OutFile "$dir\port-kill-console.exe"
[Environment]::SetEnvironmentVariable('PATH', ([Environment]::GetEnvironmentVariable('PATH','User') + ";$dir"), 'User')
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

### Dead-simple one-liners (new)
```powershell
# Kill whatever is blocking a port
port-kill 3000

# Kill multiple ports
port-kill 3000 5000

# List ports in use (one-time snapshot)
port-kill --list

# Confirm before killing
port-kill 3000 --safe
```

### CLI quick reference (new)
```powershell
# Positional ports imply clearPort on each
port-kill <port> [<port> ...]

# Thin aliases
--clear <port>          # clearPort(port)
--guard <port>          # guardPort(port)
--allow <name>          # allow only this process name in guard
--kill <pid>            # kill(pid)
--kill-file <path>      # kill processes holding this file
--kill-ext <ext>        # kill processes holding files with this extension
--list-file <pattern>   # list processes by file path/pattern
--list                  # list current ports in use (one-shot)
--safe                  # ask for confirmation before killing
```

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

# Cache management (NEW!)
port-kill-console.exe cache --list
port-kill-console.exe cache --clean --safe-delete
port-kill-console.exe cache --doctor --json
```

## 4) About the tray app

Running `port-kill.exe` without `--console` starts the system tray UI (same experience as macOS):

```powershell
# Start the tray app monitoring specific ports
port-kill.exe --ports 3000,8000,8080

# Or a range
port-kill.exe --start-port 3000 --end-port 9000
```

- A badge icon appears in the notification area (near the clock) showing the number of detected processes; its color follows the count: green (0), orange (1–9), red (10+). If you don't see it, check the hidden icons overflow (`^` arrow).
- No console window opens when launched from Explorer — the app runs entirely in the tray. When run from a terminal, output still prints there.
- Note: because `port-kill.exe` is a GUI app, Windows PowerShell 5.1 won't capture its output into a variable or pipeline (`$out = port-kill.exe --list` comes back empty). Redirection from `cmd` works fine (`port-kill.exe --list > out.txt`), and for scripting use `port-kill-console.exe`, which is a regular console app.
- **Click (left or right)** the icon to open the menu:
  - **Kill All Processes** — kills everything on the monitored ports
  - Processes grouped by type (`Docker (7)`, `Node (2)`, …) as submenus; single processes listed directly (`Kill: Port 3000: node`) — click to kill just that one
  - **Start with Windows** — toggle to launch Port Kill at login with the same options (user-scoped `HKCU` Run key, no admin needed)
  - **Quit** — exits the app
- The menu refreshes automatically as processes come and go (scans every 5 seconds).
- `--show-pid` and `--verbose` add PIDs / command line details to the menu entries.

On some Windows setups (e.g. headless sessions), the tray icon may fail and the app will fall back to console mode with a warning like:

```
Tray mode failed on Windows (…). Falling back to console mode…
```

This is harmless. The console app has the same functionality.

## 5) Troubleshooting

### Common Issue: "port-kill is not recognized" or "port-kill-console is not recognized"

This is the #1 issue Windows users face. Here's how to solve it:

**Root Cause:** The installer adds binaries to your PATH, but your current terminal session doesn't see the updated PATH yet.

**Solution (pick one):**

1. **Restart your terminal completely** (RECOMMENDED)
   - Close all terminal windows completely
   - Open a new terminal window
   - Try: `port-kill --list`

2. **Use full path** (temporary workaround - works immediately):
   ```powershell
   "%USERPROFILE%\AppData\Local\port-kill\port-kill.exe" --list
   "%USERPROFILE%\AppData\Local\port-kill\port-kill-console.exe" --version
   ```

3. **Run diagnostics** to identify what's wrong:
   ```powershell
   powershell -Command "Invoke-WebRequest -UseBasicParsing -Uri 'https://raw.githubusercontent.com/treadiehq/port-kill/main/diagnose-installation.bat' -OutFile 'diagnose.bat'"; .\diagnose.bat
   ```

4. **Reinstall** (if binaries are missing):
   ```powershell
   powershell -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing -Headers @{Pragma='no-cache'; 'Cache-Control'='no-cache'} -Uri 'https://raw.githubusercontent.com/treadiehq/port-kill/main/install-release.bat' -OutFile 'install-release.bat'"; .\install-release.bat
   ```

### Other Common Issues

- **Only one binary installed**: Both `port-kill.exe` and `port-kill-console.exe` should be downloaded. Run diagnostics above.
- **Access denied / can't kill a process**: run PowerShell/CMD "as Administrator".
- **SmartScreen or AV blocks the exe**: open file Properties and "Unblock", or allow it in your AV.
- **Docker processes not showing**: ensure Docker Desktop is running and `docker` is on PATH.

### Verify Installation

Check if binaries exist and PATH is set correctly:

```powershell
# Check if binaries exist
dir "%USERPROFILE%\AppData\Local\port-kill"

# Check if PATH contains the install directory
echo %PATH% | findstr "port-kill"

# Test binaries directly (works without PATH)
"%USERPROFILE%\AppData\Local\port-kill\port-kill.exe" --version
"%USERPROFILE%\AppData\Local\port-kill\port-kill-console.exe" --version
```

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
