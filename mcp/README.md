# Port Kill MCP Server

Expose Port Kill as MCP tools for Cursor.

## Tools

- `list(ports?, docker?, verbose?, remote?)`
- `kill(ports, remote?)`
- `reset(remote?)`
- `audit(suspiciousOnly?, remote?)`
- `guardStatus(baseUrl?)` (uses dashboard API)


## Quick Install

Add the following to your MCP config e.g. `.cursor/mcp.json`

```json
{
  "mcpServers": {
    "port-kill-mcp": {
      "command": "npx",
      "args": ["-y", "https://gitpkg.vercel.app/kagehq/port-kill/mcp?main"]
    }
  }
}
```

## Installation From Source

Alternatively, if you don't want to use `npx 'https://gitpkg.vercel.app/kagehq/port-kill/mcp?main'`.

Checkout the repo, build the server & install `port-kill-mcp` globally:
```bash
npm install
npm run build
npm install -g .
```

Then add to your config e.g. `.cursor/mcp.json`

```json
{
  "mcpServers": {
    "port-kill-mcp": {
      "command": "port-kill-mcp"
    }
  }
}
```

## Usage

Ask your AI to use the tools.

```text
What process is running on port 3000?
```

```text
Kill the process running on port 3000
```

```text
Kill all processes using dev ports
```


## Tools Extended

### 1. **`list`** - List Processes on Ports
**Purpose**: List all processes running on specified ports with detailed information

**Example Usage**:
```text
"What processes are running on port 3000?"
"List all processes on development ports"
"Show me what's running on ports 3000, 8000, and 8080"
```

### 2. **`kill`** - Kill Processes on Ports
**Purpose**: Kill all processes running on specified ports

**Example Usage**:
```text
"Kill the process running on port 3000"
"Kill all processes on ports 3000, 8000, and 8080"
"Free up port 5432"
```

### 3. **`reset`** - Reset Common Dev Ports
**Purpose**: Kill processes on common development ports (3000, 5000, 8000, 5432, 3306, 6379, 27017, 8080, 9000)

**Example Usage**:
```text
"Reset all development ports"
"Kill all processes using common dev ports"
"Clean up my development environment"
```

### 4. **`audit`** - Security Audit
**Purpose**: Run security audit on processes and ports to identify suspicious or unauthorized processes

**Example Usage**:
```text
"Run a security audit on all ports"
"Check for suspicious processes on development ports"
"Audit port 3000 for security issues"
```

### 5. **`guardStatus`** - Port Guard Status
**Purpose**: Check the status of Port Guard if running via dashboard API

**Example Usage**:
```text
"Check the port guard status"
"What's the current guard configuration?"
"Show me the dashboard guard status"
```


## Dev

```bash
cd mcp
npm install
npm run dev
```

Cursor config: `.cursor/mcp.json` is included. Restart Cursor to detect the server.

Environment variables:

```bash
# Override binary path (defaults to ./target/release/port-kill-console)
export PORT_KILL_BIN=/abs/path/to/port-kill-console

# Override working directory for commands (defaults to repo root)
export PORT_KILL_CWD=/abs/path/for/commands

# Override per-tool timeout in seconds (default 300 = 5 minutes)
export PORT_KILL_MCP_TOOL_TIMEOUT_SECONDS=60
```


### Optional HTTP wrapper (use outside MCP/Cursor)

```bash
# Start the MCP server with an HTTP wrapper for tools (POST /tool)
HTTP_PORT=8787 npm run dev

# Call a tool over HTTP
curl -s -X POST \
  -H 'content-type: application/json' \
  --data '{
    "name": "list",
    "args": { "ports": "3000,8000" }
  }' \
  http://localhost:8787/tool

# Override binary and working dir if needed
PORT_KILL_BIN=/abs/path/to/port-kill-console \
PORT_KILL_CWD=/abs/path/to/project \
PORT_KILL_MCP_TOOL_TIMEOUT_SECONDS=60 \
HTTP_PORT=8787 npm run dev
```


