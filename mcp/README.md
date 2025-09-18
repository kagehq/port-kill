# Port Kill MCP Server

Expose Port Kill as MCP tools for Cursor.

## Tools

- `list(ports?, docker?, verbose?, remote?)`
- `kill(ports, remote?)`
- `reset(remote?)`
- `audit(suspiciousOnly?, remote?)`
- `guardStatus(baseUrl?)` (uses dashboard API)

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


