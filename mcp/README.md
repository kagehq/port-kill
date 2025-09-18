# Port Kill MCP Server

Expose Port Kill as MCP tools for Cursor.

## Tools

- `list(ports?, docker?, verbose?, remote?)`
- `kill(ports, remote?)`
- `reset(remote?)`
- `audit(suspiciousOnly?, remote?)`
- `guardStatus(baseUrl?)` (uses dashboard API)

## Quick Install

```bash
# One-liner (from anywhere)
curl -fsSL https://raw.githubusercontent.com/kagehq/port-kill/main/install-mcp.sh | bash

# From port-kill root directory
./install-mcp.sh
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
HTTP_PORT=8787 npm run dev
```


