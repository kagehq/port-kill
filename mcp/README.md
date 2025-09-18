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


