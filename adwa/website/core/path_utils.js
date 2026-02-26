export function normalizePath(path) {
  const raw = String(path || "/");
  const absolute = raw.startsWith("/");
  const parts = raw.split("/").filter(Boolean);
  const stack = [];
  for (const part of parts) {
    if (part === ".") continue;
    if (part === "..") {
      stack.pop();
      continue;
    }
    stack.push(part);
  }
  return `${absolute ? "/" : ""}${stack.join("/")}` || "/";
}

export function dirname(path) {
  const normalized = normalizePath(path);
  if (normalized === "/") return "/";
  const idx = normalized.lastIndexOf("/");
  return idx <= 0 ? "/" : normalized.slice(0, idx);
}
