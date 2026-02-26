export function toBytes(input) {
  if (input instanceof Uint8Array) return input;
  if (typeof input === "string") return new TextEncoder().encode(input);
  if (ArrayBuffer.isView(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }
  if (input instanceof ArrayBuffer) return new Uint8Array(input);
  if (Array.isArray(input)) {
    const out = new Uint8Array(input.length);
    for (let i = 0; i < input.length; i += 1) out[i] = Number(input[i]) & 0xff;
    return out;
  }
  return new Uint8Array(0);
}

export function toJsonBytes(value) {
  return new TextEncoder().encode(JSON.stringify(value ?? null));
}

export function fromJsonBytes(value) {
  const text = new TextDecoder().decode(toBytes(value));
  if (!text.trim()) return null;
  return JSON.parse(text);
}
