#!/usr/bin/env node
"use strict";
// Import MCP SDK using direct paths (package exports not working properly)
const path = require("node:path");
// have to check potentially up one level for node_modules when installed by npx
const sdkPkgPath = require.resolve("@modelcontextprotocol/sdk/package.json");
let sdkDir = path.dirname(sdkPkgPath).replace(path.join("dist", "cjs"), "");
const fs = require("node:fs");
// check if exists
if (!fs.existsSync(path.join(sdkDir, "dist", "cjs", "server", "mcp.js"))) {
    const altSdkDir = path.join(__dirname, "../node_modules/@modelcontextprotocol/sdk");
    console.log("SDK not found in", sdkDir, "using", altSdkDir);
    sdkDir = altSdkDir;
}
const { McpServer } = require(path.join(sdkDir, "dist", "cjs", "server", "mcp.js"));
const { StdioServerTransport } = require(path.join(sdkDir, "dist", "cjs", "server", "stdio.js"));
const { z } = require("zod");
const { exec } = require("node:child_process");
const activeChildProcesses = new Set();
const isWindows = process.platform === "win32";
let _binPath = null;
function binPath() {
    if (_binPath)
        return _binPath;
    // Assume workspace root is project root; allow override via env
    const bin = process.env.PORT_KILL_BIN || "./target/release/port-kill-console";
    if (!fs.existsSync(path.join(process.cwd(), bin))) {
        console.error("Binary not found, falling back to port-kill-console", bin);
        _binPath = "port-kill-console";
        return "port-kill-console";
    }
    else {
        _binPath = bin;
    }
    return _binPath;
}
function run(cmd) {
    return new Promise((resolve, reject) => {
        const child = exec(cmd, {
            cwd: process.env.PORT_KILL_CWD || process.cwd(),
            maxBuffer: 10 * 1024 * 1024
        });
        activeChildProcesses.add(child);
        let stdoutBuf = "";
        let stderrBuf = "";
        if (child.stdout)
            child.stdout.on("data", (d) => { stdoutBuf += String(d); });
        if (child.stderr)
            child.stderr.on("data", (d) => { stderrBuf += String(d); });
        child.on("error", (err) => {
            activeChildProcesses.delete(child);
            reject(err);
        });
        child.on("close", (code, signal) => {
            activeChildProcesses.delete(child);
            if (code === 0 || signal) {
                resolve(stdoutBuf.trim());
            }
            else {
                const error = new Error(`Command failed with exit code ${code}: ${stderrBuf || stdoutBuf}`);
                error.code = code;
                reject(error);
            }
        });
    });
}
// Forward termination signals to any active child processes.
// Note: SIGKILL cannot be intercepted or forwarded by a Node.js process.
function forwardSignal(signal) {
    for (const child of activeChildProcesses) {
        try {
            if (!isWindows && child.pid) {
                try {
                    process.kill(-child.pid, signal);
                }
                catch { /* best-effort */ }
            }
            child.kill(signal);
        }
        catch {
            // best-effort; ignore
        }
    }
}
process.on("SIGTERM", () => {
    forwardSignal("SIGTERM");
    setTimeout(() => process.exit(143), 50);
});
process.on("SIGINT", () => {
    forwardSignal("SIGINT");
    setTimeout(() => process.exit(130), 50);
});
// Tool handler function
const handler = async (name, args) => {
    switch (name) {
        case "list": {
            const ports = args?.ports ? `--ports ${args.ports}` : "";
            const docker = args?.docker ? "--docker" : "";
            const verbose = args?.verbose ? "--verbose" : "";
            const remote = args?.remote ? `--remote ${args.remote}` : "";
            const cmd = `${binPath()} --console ${ports} ${docker} ${verbose} ${remote} --json`.trim();
            const out = await run(cmd);
            return { content: out };
        }
        case "kill": {
            const remote = args?.remote ? `--remote ${args.remote}` : "";
            const cmd = `${binPath()} --kill-all --ports ${args.ports} ${remote}`.trim();
            const out = await run(cmd);
            return { content: out };
        }
        case "reset": {
            const remote = args?.remote ? `--remote ${args.remote}` : "";
            const cmd = `${binPath()} --reset ${remote}`.trim();
            const out = await run(cmd);
            return { content: out };
        }
        case "audit": {
            const remote = args?.remote ? `--remote ${args.remote}` : "";
            const suspicious = args?.suspiciousOnly ? "--suspicious-only" : "";
            const cmd = `${binPath()} --audit ${suspicious} ${remote} --json`.trim();
            const out = await run(cmd);
            return { content: out };
        }
        case "guardStatus": {
            const baseUrl = args?.baseUrl || "http://localhost:3000";
            const resp = await fetch(`${baseUrl}/api/guard/status`);
            const json = await resp.json();
            return { content: JSON.stringify(json) };
        }
        default:
            throw new Error(`Unknown tool: ${name}`);
    }
};
// Create MCP server with proper tool registration
const server = new McpServer({
    name: "port-kill-mcp",
    version: "0.1.0"
});
// Register each tool individually with proper Zod schemas
server.registerTool("list", {
    description: "List processes on ports. Args: ports (comma), docker(bool), verbose(bool), json(bool)",
    inputSchema: {
        ports: z.string().describe("e.g. 3000,8000,8080, leave blank for all ports"),
        docker: z.boolean().describe("Enable docker support (recommended)"),
        verbose: z.boolean().describe("Enable verbose output (only use for detailed logging)"),
        remote: z.string().describe("user@host for SSH remote")
    }
}, async (args) => {
    const result = await handler("list", args || {});
    return { content: [{ type: "text", text: result.content }] };
});
server.registerTool("kill", {
    description: "Kill processes on given ports. Args: ports (comma)",
    inputSchema: {
        ports: z.string().describe("Comma-separated list of ports"),
        remote: z.string().describe("user@host for SSH remote"),
        required: ["ports", "remote"]
    }
}, async (args) => {
    const result = await handler("kill", args || {});
    return { content: [{ type: "text", text: result.content }] };
});
server.registerTool("reset", {
    description: "Kill common dev ports (3000,5000,8000,5432,3306,6379,27017,8080,9000)",
    inputSchema: {
        remote: z.string().describe("user@host for SSH remote"),
        required: ["remote"]
    }
}, async (args) => {
    const result = await handler("reset", args || {});
    return { content: [{ type: "text", text: result.content }] };
});
server.registerTool("audit", {
    description: "Run security audit. Returns JSON.",
    inputSchema: {
        suspiciousOnly: z.boolean(),
        remote: z.string().describe("user@host for SSH remote"),
        required: ["suspiciousOnly", "remote"]
    }
}, async (args) => {
    const result = await handler("audit", args || {});
    return { content: [{ type: "text", text: result.content }] };
});
server.registerTool("guardStatus", {
    description: "Return Port Guard status if running via dashboard API.",
    inputSchema: {
        baseUrl: z.string().describe("Dashboard base URL"),
        required: ["baseUrl"]
    }
}, async (args) => {
    const result = await handler("guardStatus", args || {});
    return { content: [{ type: "text", text: result.content }] };
});
// Start server with stdio transport
async function startServer() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("Port Kill MCP server running on stdio", binPath());
}
startServer().catch(console.error);
// Optional HTTP wrapper so non-MCP clients can call the same tools
if (process.env.HTTP_PORT) {
    const httpPort = parseInt(process.env.HTTP_PORT, 10) || 8787;
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const http = require("node:http");
    const srv = http.createServer(async (req, res) => {
        try {
            if (!req.url) {
                res.statusCode = 400;
                return res.end("Bad Request");
            }
            const url = new URL(req.url, `http://localhost:${httpPort}`);
            if (req.method === "POST" && url.pathname === "/tool") {
                let body = "";
                req.on("data", (chunk) => body += chunk);
                req.on("end", async () => {
                    try {
                        const { name, args } = JSON.parse(body || "{}");
                        const result = await handler(name, args || {});
                        res.setHeader("content-type", "application/json");
                        res.end(JSON.stringify({ ok: true, result }));
                    }
                    catch (e) {
                        res.statusCode = 500;
                        res.end(JSON.stringify({ ok: false, error: e?.message || String(e) }));
                    }
                });
                return;
            }
            res.statusCode = 404;
            res.end("Not Found");
        }
        catch (e) {
            res.statusCode = 500;
            res.end(JSON.stringify({ ok: false, error: e?.message || String(e) }));
        }
    });
    srv.listen(httpPort, () => {
        console.log(`[port-kill-mcp] HTTP wrapper listening on :${httpPort}`);
    });
}
