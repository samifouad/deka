import { Mesh, IsolatePool, Isolate } from "deka";

const mesh = new Mesh();
const pool = new IsolatePool(mesh, { workers: 2 });
const isolate = new Isolate(pool, "./worker.ts");

export default async function handler() {
  const res = await isolate.run({
    url: "http://localhost/inner",
    method: "GET"
  });
  const text = await res.text();
  return new Response(`outer: ${text}`);
}
