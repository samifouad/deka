export type RuntimeFs = {
    readFile(path: string): Uint8Array | Promise<Uint8Array>;
};
export type ServeOptions = {
    port?: number;
    publishPort?: (port: number) => void;
};
export type Handler = (request: Request) => Response | Promise<Response>;
export declare class DekaBrowserRuntime {
    private readonly fs;
    private readonly decoder;
    constructor(fs: RuntimeFs);
    run(entry: string): Promise<Record<string, unknown>>;
    serve(entry: string, options?: ServeOptions): Promise<DekaBrowserServer>;
}
export declare class DekaBrowserServer {
    private readonly handler;
    private readonly portValue;
    constructor(handler: Handler, port: number);
    get port(): number;
    fetch(request: Request): Promise<Response>;
}
