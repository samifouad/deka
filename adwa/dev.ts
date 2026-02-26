import { readFile } from "node:fs/promises";
import path from "node:path";

const adwaRoot = process.env.ADWA_ROOT || process.cwd();
const staticRoot = path.resolve(adwaRoot, "website");
const lspHost = process.env.PHPX_LSP_HOST || "127.0.0.1";
const lspPort = Number(process.env.PHPX_LSP_PORT || 8531);
const lspBase = process.env.PHPX_LSP_BASE || `http://${lspHost}:${lspPort}`;

const MIME: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".mjs": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".map": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
};

const proxyLsp = async (incoming: URL, endpoint: string) => {
  const target = new URL(endpoint, lspBase);
  target.search = incoming.search;
  const res = await fetch(target.toString(), {
    method: "GET",
    headers: { accept: "application/json" },
  });
  const body = await res.text();
  return new Response(body, {
    status: res.status,
    headers: {
      "content-type": res.headers.get("content-type") || "application/json; charset=utf-8",
    },
  });
};

const safePath = (pathname: string) => {
  const normalized = pathname === "/" ? "/index.html" : pathname;
  const decoded = decodeURIComponent(normalized);
  if (decoded.includes("\0")) return null;
  if (decoded.includes("..")) return null;
  const absolute = path.resolve(staticRoot, `.${decoded}`);
  if (!absolute.startsWith(staticRoot)) return null;
  return absolute;
};

export default {
  async fetch(req: Request) {
    const url = new URL(req.url);
    if (url.pathname === "/_lsp/ping") {
      try {
        return await proxyLsp(url, "/ping");
      } catch (err) {
        return Response.json(
          { ok: false, error: `lsp sidecar unreachable: ${err instanceof Error ? err.message : String(err)}` },
          { status: 500 }
        );
      }
    }
    if (url.pathname === "/_lsp/diagnostics") {
      try {
        return await proxyLsp(url, "/diagnostics");
      } catch (err) {
        return Response.json(
          { ok: false, error: `lsp diagnostics failed: ${err instanceof Error ? err.message : String(err)}` },
          { status: 500 }
        );
      }
    }
    if (url.pathname === "/_lsp/completion") {
      try {
        return await proxyLsp(url, "/completion");
      } catch (err) {
        return Response.json(
          { ok: false, error: `lsp completion failed: ${err instanceof Error ? err.message : String(err)}` },
          { status: 500 }
        );
      }
    }
    if (url.pathname === "/_lsp/hover") {
      try {
        return await proxyLsp(url, "/hover");
      } catch (err) {
        return Response.json(
          { ok: false, error: `lsp hover failed: ${err instanceof Error ? err.message : String(err)}` },
          { status: 500 }
        );
      }
    }

    const filePath = safePath(url.pathname);
    if (!filePath) {
      return new Response("Bad Request\n", {
        status: 400,
        headers: { "content-type": "text/plain; charset=utf-8" },
      });
    }

    try {
      const bytes = await readFile(filePath);
      const ext = path.extname(filePath).toLowerCase();
      return new Response(bytes, {
        status: 200,
        headers: {
          "content-type": MIME[ext] || "application/octet-stream",
        },
      });
    } catch {
      return new Response("Not Found\n", {
        status: 404,
        headers: { "content-type": "text/plain; charset=utf-8" },
      });
    }
  },
};
