function svg(path, viewBox = "0 0 16 16") {
  return `<svg class="iconSvg" viewBox="${viewBox}" aria-hidden="true" focusable="false"><path d="${path}"></path></svg>`;
}

const ICONS = {
  chevronRight: svg("M6 3.4L10.6 8 6 12.6l-1.2-1.2L8.2 8 4.8 4.6z"),
  chevronDown: svg("M3.4 6L8 10.6 12.6 6l-1.2-1.2L8 8.2 4.6 4.8z"),
  folderClosed: svg("M1.75 3.5A1.75 1.75 0 0 1 3.5 1.75h2.6l1.1 1.2h5.3A1.75 1.75 0 0 1 14.25 4.7v6.8A1.75 1.75 0 0 1 12.5 13.25h-9A1.75 1.75 0 0 1 1.75 11.5z"),
  folderOpen: svg("M1.75 4.75A1.75 1.75 0 0 1 3.5 3h2.2l1.05 1.1h5.75A1.75 1.75 0 0 1 14.25 5.85v5.65a1.75 1.75 0 0 1-1.75 1.75h-9A1.75 1.75 0 0 1 1.75 11.5z"),
  file: svg("M3.5 1.75h5L12.25 5.5v8.75a1 1 0 0 1-1 1h-7.5a1 1 0 0 1-1-1v-11.5a1 1 0 0 1 1-1zm4.25 1.5v2.5h2.5z"),
  code: svg("M6 4.2L2.2 8 6 11.8l1.1-1.1L4.4 8l2.7-2.7zm4 0L8.9 5.3 11.6 8l-2.7 2.7 1.1 1.1L13.8 8z"),
  database: svg("M8 1.75c3.18 0 5.75 1.3 5.75 2.9v6.7c0 1.6-2.57 2.9-5.75 2.9s-5.75-1.3-5.75-2.9v-6.7c0-1.6 2.57-2.9 5.75-2.9zm0 1.5c-2.58 0-4.25.95-4.25 1.4 0 .45 1.67 1.4 4.25 1.4s4.25-.95 4.25-1.4c0-.45-1.67-1.4-4.25-1.4z"),
  settings: svg("M8.95 1.9l.3 1.2a4.9 4.9 0 0 1 1.14.66l1.15-.48.9 1.56-.95.8c.06.2.1.4.13.62l1.15.4v1.8l-1.15.4a3.2 3.2 0 0 1-.13.62l.95.8-.9 1.56-1.15-.48a4.9 4.9 0 0 1-1.14.66l-.3 1.2h-1.8l-.3-1.2a4.9 4.9 0 0 1-1.14-.66l-1.15.48-.9-1.56.95-.8a3.2 3.2 0 0 1-.13-.62l-1.15-.4v-1.8l1.15-.4c.03-.22.07-.42.13-.62l-.95-.8.9-1.56 1.15.48a4.9 4.9 0 0 1 1.14-.66l.3-1.2zM8 5.9A2.1 2.1 0 1 0 8 10.1 2.1 2.1 0 0 0 8 5.9z"),
  lock: svg("M4.25 6V4.75A3.75 3.75 0 0 1 8 1a3.75 3.75 0 0 1 3.75 3.75V6h.5a1 1 0 0 1 1 1v6.25a1 1 0 0 1-1 1h-8.5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1zm1.5 0h4.5V4.75A2.25 2.25 0 0 0 8 2.5a2.25 2.25 0 0 0-2.25 2.25z"),
  markdown: svg("M2 3h12v10H2zm1.5 2v6H5V7.2L6.7 9 8.4 7.2V11H10V5H8.4L6.7 6.9 5 5zm7.2 3.3L12.4 11H11l-1.7-2.7H10.7z"),
  close: svg("M4.3 3.2L8 6.9l3.7-3.7 1.1 1.1L9.1 8l3.7 3.7-1.1 1.1L8 9.1l-3.7 3.7-1.1-1.1L6.9 8 3.2 4.3z"),
};

function ext(path) {
  const i = path.lastIndexOf(".");
  return i >= 0 ? path.slice(i + 1).toLowerCase() : "";
}

export function icon(name) {
  return ICONS[name] || ICONS.file;
}

export function fileIconFor(path) {
  const name = path.split("/").pop() || path;
  if (name === "deka.lock") return ICONS.lock;
  if (name === "deka.json") return ICONS.settings;
  const e = ext(name);
  if (e === "phpx" || e === "php" || e === "ts" || e === "js") return ICONS.code;
  if (e === "json" || e === "sql" || e === "db") return ICONS.database;
  if (e === "md" || e === "mdx") return ICONS.markdown;
  return ICONS.file;
}

export function folderIconFor(open) {
  return open ? ICONS.folderOpen : ICONS.folderClosed;
}
