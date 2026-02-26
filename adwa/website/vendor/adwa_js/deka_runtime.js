export class DekaBrowserRuntime {
    constructor(fs) {
        this.decoder = new TextDecoder();
        this.fs = fs;
    }
    async run(entry) {
        const filename = resolvePath("/", entry);
        const data = await this.fs.readFile(filename);
        const code = this.decoder.decode(data);
        const module = { exports: {} };
        const wrapper = new Function("exports", "module", "__filename", "__dirname", "console", code);
        wrapper(module.exports, module, filename, dirname(filename), console);
        return module.exports;
    }
    async serve(entry, options = {}) {
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
    constructor(handler, port) {
        this.handler = handler;
        this.portValue = port;
    }
    get port() {
        return this.portValue;
    }
    async fetch(request) {
        return await this.handler(request);
    }
}
function resolveHandler(exports) {
    const handler = exports.fetch ??
        exports.default ??
        exports.handler;
    if (typeof handler === "function") {
        return handler;
    }
    return null;
}
function resolvePath(base, path) {
    if (path.startsWith("/")) {
        return normalizePath(path);
    }
    return normalizePath(`${base}/${path}`);
}
function normalizePath(path) {
    const absolute = path.startsWith("/");
    const parts = path.split("/").filter((part) => part.length > 0);
    const stack = [];
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
function dirname(path) {
    if (path === "/") {
        return "/";
    }
    const parts = path.split("/").filter((part) => part.length > 0);
    parts.pop();
    return parts.length === 0 ? "/" : `/${parts.join("/")}`;
}
