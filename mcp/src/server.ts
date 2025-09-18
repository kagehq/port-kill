// The SDK exports differ by version; import minimal types dynamically
// eslint-disable-next-line @typescript-eslint/no-var-requires
// Resolve SDK CJS server entry without relying on package exports
// eslint-disable-next-line @typescript-eslint/no-var-requires
const path = require("node:path");
const fs = require("node:fs");
// eslint-disable-next-line @typescript-eslint/no-var-requires
const sdkPkgPath = require.resolve("@modelcontextprotocol/sdk/package.json");
const sdkDir = path.dirname(sdkPkgPath);
function resolveSdkServerEntry() {
  const candidates = [
    path.join(sdkDir, "server", "index.js"),
    path.join(sdkDir, "../server/index.js"),
    path.join(sdkDir, "../..", "dist", "cjs", "server", "index.js"),
    path.join(sdkDir, "dist", "cjs", "server", "index.js")
  ];
  for (const p of candidates) {
    try {
      const full = path.resolve(p);
      if (fs.existsSync(full)) return full;
    } catch {}
  }
  throw new Error("Could not locate @modelcontextprotocol/sdk CJS server entry");
}
// eslint-disable-next-line @typescript-eslint/no-var-requires
const SDK = require(resolveSdkServerEntry());
const { exec } = require("node:child_process");
const { promisify } = require("node:util");

const execAsync = promisify(exec);

function binPath() {
  // Assume workspace root is project root; allow override via env
  return process.env.PORT_KILL_BIN || "./target/release/port-kill-console";
}

async function run(cmd: string) {
  const { stdout } = await execAsync(cmd, { cwd: process.env.PORT_KILL_CWD || process.cwd(), maxBuffer: 10 * 1024 * 1024 });
  return stdout.trim();
}

const tools: Record<string, any> = {
  list: {
    description: "List processes on ports. Args: ports (comma), docker(bool), verbose(bool), json(bool)",
    inputSchema: {
      type: "object",
      properties: {
        ports: { type: "string", description: "e.g. 3000,8000,8080" },
        docker: { type: "boolean" },
        verbose: { type: "boolean" },
        remote: { type: "string", description: "user@host for SSH remote" }
      }
    }
  },
  kill: {
    description: "Kill processes on given ports. Args: ports (comma)",
    inputSchema: {
      type: "object",
      required: ["ports"],
      properties: { ports: { type: "string" }, remote: { type: "string" } }
    }
  },
  reset: {
    description: "Kill common dev ports (3000,5000,8000,5432,3306,6379,27017,8080,9000)",
    inputSchema: { type: "object", properties: { remote: { type: "string" } } }
  },
  audit: {
    description: "Run security audit. Returns JSON.",
    inputSchema: { type: "object", properties: { suspiciousOnly: { type: "boolean" }, remote: { type: "string" } } }
  },
  guardStatus: {
    description: "Return Port Guard status if running via dashboard API.",
    inputSchema: { type: "object", properties: { baseUrl: { type: "string" } } }
  }
};

const handler = async (name: string, args: any) => {
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

const create = (SDK as any).createServer || ((options: any) => new (SDK as any).Server(options));
const server = create({ name: "port-kill-mcp", version: "0.1.0", tools, handler });
(server as any).listen?.();

// Optional HTTP wrapper so non-MCP clients can call the same tools
if (process.env.HTTP_PORT) {
  const httpPort = parseInt(process.env.HTTP_PORT, 10) || 8787;
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const http = require("node:http");
  const srv = http.createServer(async (req: any, res: any) => {
    try {
      if (!req.url) { res.statusCode = 400; return res.end("Bad Request"); }
      const url = new URL(req.url, `http://localhost:${httpPort}`);
      if (req.method === "POST" && url.pathname === "/tool") {
        let body = "";
        req.on("data", (chunk: any) => body += chunk);
        req.on("end", async () => {
          try {
            const { name, args } = JSON.parse(body || "{}");
            const result = await (handler as any)(name, args || {});
            res.setHeader("content-type", "application/json");
            res.end(JSON.stringify({ ok: true, result }));
          } catch (e: any) {
            res.statusCode = 500;
            res.end(JSON.stringify({ ok: false, error: e?.message || String(e) }));
          }
        });
        return;
      }
      res.statusCode = 404; res.end("Not Found");
    } catch (e: any) {
      res.statusCode = 500; res.end(JSON.stringify({ ok: false, error: e?.message || String(e) }));
    }
  });
  srv.listen(httpPort, () => {
    console.log(`[port-kill-mcp] HTTP wrapper listening on :${httpPort}`);
  });
}


