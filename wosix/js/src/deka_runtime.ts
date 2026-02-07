export type RuntimeFs = {
  readFile(path: string): Uint8Array | Promise<Uint8Array>;
};

export type ServeOptions = {
  port?: number;
  publishPort?: (port: number) => void;
};

export type Handler = (request: Request) => Response | Promise<Response>;

export class DekaBrowserRuntime {
  private readonly fs: RuntimeFs;
  private readonly decoder = new TextDecoder();

  constructor(fs: RuntimeFs) {
    this.fs = fs;
  }

  async run(entry: string): Promise<Record<string, unknown>> {
    const filename = resolvePath("/", entry);
    const data = await this.fs.readFile(filename);
    const code = this.decoder.decode(data);
    const module = { exports: {} as Record<string, unknown> };
    const wrapper = new Function(
      "exports",
      "module",
      "__filename",
      "__dirname",
      "console",
      code
    );
    wrapper(module.exports, module, filename, dirname(filename), console);
    return module.exports;
  }

  async serve(entry: string, options: ServeOptions = {}): Promise<DekaBrowserServer> {
    const exports = await this.run(entry);
    const handler = resolveHandler(exports);
    if (!handler) {
      throw new Error("No handler exported. Expected fetch/default/handler.");
    }
    const port = options.port ?? 3000;
    if (options.publishPort) {
      options.publishPort(port);
    }
    return new DekaBrowserServer(handler, port);
  }
}

export class DekaBrowserServer {
  private readonly handler: Handler;
  private readonly portValue: number;

  constructor(handler: Handler, port: number) {
    this.handler = handler;
    this.portValue = port;
  }

  get port() {
    return this.portValue;
  }

  async fetch(request: Request): Promise<Response> {
    return await this.handler(request);
  }
}

function resolveHandler(exports: Record<string, unknown>): Handler | null {
  const handler =
    (exports.fetch as Handler | undefined) ??
    (exports.default as Handler | undefined) ??
    (exports.handler as Handler | undefined);
  if (typeof handler === "function") {
    return handler;
  }
  return null;
}

function resolvePath(base: string, path: string): string {
  if (path.startsWith("/")) {
    return normalizePath(path);
  }
  return normalizePath(`${base}/${path}`);
}

function normalizePath(path: string): string {
  const absolute = path.startsWith("/");
  const parts = path.split("/").filter((part) => part.length > 0);
  const stack: string[] = [];
  for (const part of parts) {
    if (part === ".") {
      continue;
    }
    if (part === "..") {
      stack.pop();
      continue;
    }
    stack.push(part);
  }
  return `${absolute ? "/" : ""}${stack.join("/")}` || "/";
}

function dirname(path: string): string {
  if (path === "/") {
    return "/";
  }
  const parts = path.split("/").filter((part) => part.length > 0);
  parts.pop();
  return parts.length === 0 ? "/" : `/${parts.join("/")}`;
}
