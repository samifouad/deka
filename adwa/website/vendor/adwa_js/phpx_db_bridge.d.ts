export type DbBridgePayload = Record<string, unknown>;
type SqlJsModule = {
    Database: new (data?: Uint8Array) => SqlJsDatabase;
};
type SqlJsStatement = {
    bind(values?: unknown[] | Record<string, unknown>): boolean;
    step(): boolean;
    getAsObject(params?: unknown[] | Record<string, unknown>): Record<string, unknown>;
    free(): void;
};
type SqlJsDatabase = {
    prepare(sql: string): SqlJsStatement;
    run(sql: string, params?: unknown[] | Record<string, unknown>): void;
    exec(sql: string, params?: unknown[] | Record<string, unknown>): Array<{
        columns: string[];
        values: unknown[][];
    }>;
    export(): Uint8Array;
    close(): void;
};
declare global {
    interface Window {
        initSqlJs?: (opts: {
            locateFile: (file: string) => string;
        }) => Promise<SqlJsModule>;
        __adwaSqlJsPromise?: Promise<SqlJsModule>;
        __adwaSqlJsModule?: SqlJsModule;
    }
    var initSqlJs: ((opts: {
        locateFile: (file: string) => string;
    }) => Promise<SqlJsModule>) | undefined;
    var __adwaSqlJsPromise: Promise<SqlJsModule> | undefined;
    var __adwaSqlJsModule: SqlJsModule | undefined;
}
export declare function handleDbBridge(kind: string, action: string, payload: DbBridgePayload): unknown;
export {};
