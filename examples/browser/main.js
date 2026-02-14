import {
  createPhpHostBridge,
  WebContainer,
} from "./vendor/adwa_js/index.js";
import { phpRuntimeRawWasmDataUrl } from "./vendor/adwa_js/php_runtime_raw_wasm_data.js";
import { bundledProjectFiles } from "./vendor/adwa_js/php_project_bundle.js";
import { renderWorkbench } from "./ui/workbench.js";
import { fileIconFor, folderIconFor, icon } from "./ui/file_icons.js";
import { initMonacoEditor } from "./ui/editor_monaco.js";
import { normalizePath, dirname } from "./core/path_utils.js";
import { toBytes, toJsonBytes, fromJsonBytes } from "./core/bytes.js";
import { VirtualFs } from "./core/virtual_fs.js";
import { createResultView } from "./ui/result_view.js";

renderWorkbench(document.getElementById("app"));

let appShellEl = document.querySelector(".appShell");
let logEl = document.getElementById("log");
let sourceEl = document.getElementById("source");
let editorHostEl = document.getElementById("editor");
let runBtn = document.getElementById("runBtn");
let treeBtn = document.getElementById("treeBtn");
let newFileBtn = document.getElementById("newFileBtn");
let newFolderBtn = document.getElementById("newFolderBtn");
let fileTreeEl = document.getElementById("fileTree");
let fileTreeListEl = document.getElementById("fileTreeList");
let explorerPathEl = document.getElementById("explorerPath");
let editorTabsEl = document.getElementById("editorTabs");
let workspacePaneEl = document.querySelector(".workspacePane");
let resultFrame = document.getElementById("resultFrame");
let resultBannerEl = document.getElementById("resultBanner");
let previewInputEl = document.getElementById("previewInput");
let previewGoEl = document.getElementById("previewGo");
let previewStatusEl = document.getElementById("previewStatus");
let termFormEl = document.getElementById("termForm");
let termInputEl = document.getElementById("termInput");
let termPromptEl = document.getElementById("termPrompt");
let rightPaneEl = document.getElementById("rightPane");
let terminalPaneEl = document.getElementById("terminalPane");
let splitXEl = document.getElementById("splitX");
let splitExplorerEl = document.getElementById("splitExplorer");
let splitYEl = document.getElementById("splitY");
let helpModalEl = document.getElementById("helpModal");

let monacoEditor = null;
let monacoApi = null;
let bridgeRef = null;
let lspWasm = null;
let dekaPhpRuntimeReady = false;
let phpStdoutBuffer = "";
let phpStderrBuffer = "";
let phpRuntimeWasmBytesCache = null;
let shellContainer = null;
let terminalVisible = true;
let explorerWidth = 280;
const commandHistory = [];
let historyIndex = 0;
let diagnosticsTimer = null;
let autorunTimer = null;
let runInFlight = false;
let rerunRequested = false;
let lastCliArgCount = 0;
let lastShebangArgCount = 0;
const AUTORUN_DEBOUNCE_MS = 220;
const lspBase = `${window.location.protocol}//${window.location.hostname}:${window.PHPX_LSP_PORT || "8531"}`;
const DEMO_ROOT = "/home/user/demo";
const DEMO_ENTRY = `${DEMO_ROOT}/app/home.phpx`;
const DEFAULT_CWD = DEMO_ROOT;
const BASE_FS_DIRS = [
  "/bin",
  "/usr",
  "/usr/bin",
  "/home",
  "/home/user",
  "/home/user/demo",
  "/home/user/demo/app",
  "/tmp",
  "/etc",
  "/var",
  "/var/tmp",
];
const BASE_FS_FILES = {
  "/etc/hosts": "127.0.0.1 localhost\n",
  "/etc/os-release": "NAME=Adwa\nID=adwa\nPRETTY_NAME=\"Adwa Browser Runtime\"\n",
};
const projectEnv = {
  DEKA_PHPX_ENABLE_DOTENV: "1",
  DEKA_PHPX_DISABLE_CACHE: "1",
  PHPX_MODULE_ROOT: DEMO_ROOT,
  HOME: "/home/user",
  USER: "user",
  PATH: "/usr/bin:/bin",
  TMPDIR: "/tmp",
  PWD: DEFAULT_CWD,
};
const serverState = {
  running: false,
  entry: DEMO_ENTRY,
  mode: "php",
  port: 8530,
  path: "/",
};
const runtimePortState = {
  url: "",
  port: 0,
};
const LOCAL_MODULE_ROOT = "/php_modules";
const GLOBAL_MODULE_ROOT = "/__global/php_modules";
let commandManifestCache = null;
const ADWA_BUILTIN_COMMANDS = new Set([
  "deka",
  "phpx",
]);

const defaultSource = [
  "---",
  "interface HelloProps {",
  "  $name: string",
  "}",
  "function Hello({ $name }: HelloProps): string {",
  "  return $name",
  "}",
  "---",
  "<!doctype html>",
  "<html>",
  "  <body class=\"m-0 bg-slate-950 text-slate-100\">",
  "    <main class=\"min-h-screen flex items-center justify-center p-8\">",
  "      <h1 class=\"text-5xl font-bold tracking-tight\">Hello <Hello name=\"adwa\" /></h1>",
  "    </main>",
  "  </body>",
  "</html>",
].join("\n");

const remapBundledProjectFiles = (files, root) => {
  const out = {};
  for (const [rawPath, content] of Object.entries(files || {})) {
    const rel = String(rawPath || "").replace(/^\/+/, "");
    if (!rel) continue;
    out[normalizePath(`${root}/${rel}`)] = content;
  }
  return out;
};

const resolveProjectPath = (value, fallback = DEMO_ENTRY) => {
  const raw = String(value || "").trim();
  if (!raw) return fallback;
  if (raw.startsWith("/")) return normalizePath(raw);
  return normalizePath(`${DEMO_ROOT}/${raw}`);
};

const parseJsonSafe = (value, fallback = {}) => {
  try {
    const parsed = JSON.parse(String(value ?? ""));
    return parsed && typeof parsed === "object" ? parsed : fallback;
  } catch {
    return fallback;
  }
};

const bundledProjectTree = remapBundledProjectFiles(bundledProjectFiles, DEMO_ROOT);
const bundledDekaConfig = parseJsonSafe(bundledProjectTree[`${DEMO_ROOT}/deka.json`], {});
const bundledServeConfig =
  bundledDekaConfig && typeof bundledDekaConfig.serve === "object" ? bundledDekaConfig.serve : {};
const configuredServeEntry = resolveProjectPath(bundledServeConfig.entry, DEMO_ENTRY);
const configuredServeMode = String(bundledServeConfig.mode || "php");
const configuredServePort = Number(bundledServeConfig.port || 8530);

const vfs = new VirtualFs({
  ...BASE_FS_FILES,
  ...bundledProjectTree,
  "/README.txt": "ADWA browser playground\\nUse terminal: ls, cd, pwd, open, cat, run\\n",
  `${DEMO_ROOT}/main.phpx`: defaultSource,
  [DEMO_ENTRY]: defaultSource,
});
for (const dir of BASE_FS_DIRS) {
  vfs.mkdir(dir);
}

let currentFile = configuredServeEntry;
let cwd = DEFAULT_CWD;
let explorerRoot = DEFAULT_CWD;
const openTabs = [{ path: currentFile, pinned: true }];
const EXPANDED_FOLDERS_STORAGE_KEY = "adwa.explorer.expanded.v1";
const loadExpandedFolders = () => {
  try {
    const raw = localStorage.getItem(EXPANDED_FOLDERS_STORAGE_KEY);
    const arr = JSON.parse(raw || "[]");
    if (!Array.isArray(arr)) return new Set(["/", DEFAULT_CWD]);
    const out = new Set(["/", DEFAULT_CWD]);
    for (const item of arr) {
      if (typeof item === "string" && item.startsWith("/")) out.add(normalizePath(item));
    }
    return out;
  } catch {
    return new Set(["/", DEFAULT_CWD]);
  }
};
const expandedFolders = loadExpandedFolders();
const saveExpandedFolders = () => {
  try {
    localStorage.setItem(EXPANDED_FOLDERS_STORAGE_KEY, JSON.stringify(Array.from(expandedFolders)));
  } catch {}
};

try {
  const stat = vfs.stat(currentFile);
  if (stat.fileType !== "file") {
    currentFile = DEMO_ENTRY;
    openTabs[0].path = currentFile;
  }
} catch {
  currentFile = DEMO_ENTRY;
  openTabs[0].path = currentFile;
}

const log = (message = "") => {
  if (!logEl) return;
  const raw = String(message);
  const clearPattern = /\u001b\[2J\u001b\[H|\\033\[2J\\033\[H|33\[2J33\[H/g;
  if (clearPattern.test(raw)) {
    const cleaned = raw.replace(clearPattern, "").trim();
    resetLog("");
    if (!cleaned) return;
    logEl.textContent = cleaned;
    logEl.scrollTop = logEl.scrollHeight;
    return;
  }
  logEl.textContent += `${logEl.textContent ? "\n" : ""}${raw}`;
  logEl.scrollTop = logEl.scrollHeight;
};

const resetLog = (message) => {
  if (!logEl) return;
  logEl.textContent = String(message ?? "");
};

const setPrompt = () => {
  if (!termPromptEl) return;
  if (serverState.running) {
    termPromptEl.textContent = "";
    return;
  }
  termPromptEl.textContent = `${cwd} $`;
};

const setTerminalBlocked = (blocked) => {
  if (!(termInputEl instanceof HTMLInputElement)) return;
  termInputEl.disabled = blocked;
  termInputEl.classList.toggle("blocked", blocked);
  if (!blocked) {
    setTimeout(() => {
      termInputEl.focus();
      termInputEl.setSelectionRange(termInputEl.value.length, termInputEl.value.length);
    }, 0);
  }
};

const fileBaseName = (path) => {
  const normalized = normalizePath(path);
  const parts = normalized.split("/").filter(Boolean);
  return parts.length ? parts[parts.length - 1] : "/";
};

const ensureFolderExpandedFor = (filePath) => {
  const normalized = normalizePath(filePath);
  const parts = normalized.split("/").filter(Boolean);
  let current = "";
  expandedFolders.add("/");
  for (let i = 0; i < parts.length - 1; i += 1) {
    current = `${current}/${parts[i]}`;
    expandedFolders.add(current || "/");
  }
  saveExpandedFolders();
};

const syncExplorerToCwd = () => {
  explorerRoot = normalizePath(cwd || DEFAULT_CWD);
  expandedFolders.add(explorerRoot);
  if (explorerPathEl) explorerPathEl.textContent = explorerRoot;
  saveExpandedFolders();
};

const findTabIndexByPath = (path) => openTabs.findIndex((tab) => tab.path === path);
const findPreviewTabIndex = () => openTabs.findIndex((tab) => !tab.pinned);

const setPreviewStatus = (text) => {
  if (previewStatusEl) previewStatusEl.textContent = text;
};

const setPreviewPath = (path) => {
  if (previewInputEl instanceof HTMLInputElement) {
    previewInputEl.value = path;
  }
};

const setServePreviewStatus = (path = null) => {
  const activePath = typeof path === "string" ? path : serverState.path || "/";
  if (runtimePortState.url) {
    setPreviewStatus(`serve ${runtimePortState.url}${activePath}`);
    return;
  }
  setPreviewStatus(`serve :${serverState.port}`);
};

const stripAnsi = (value) => String(value).replace(/\u001b\[[0-9;]*m/g, "");
const resultView = createResultView(resultFrame, resultBannerEl);

const firstErrorLine = (text) => {
  const line = String(text || "")
    .split(/\r?\n/)
    .map((s) => stripAnsi(s).trim())
    .find((s) => s.length > 0);
  return line || "";
};

const looksLikeHtmlOutput = (value) => {
  const text = stripAnsi(String(value || "")).trim();
  if (!text) return false;
  return /<!doctype html|<html[\s>]|<body[\s>]|<\/[a-z][a-z0-9:-]*>/i.test(text);
};


const lspUriForCurrentFile = () => `file:///workspace${currentFile.startsWith("/") ? "" : "/"}${currentFile}`;

const postLsp = async (path, payload) => {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(payload || {})) {
    q.set(k, String(v ?? ""));
  }
  const res = await fetch(`${lspBase}${path}?${q.toString()}`);
  return res.json();
};

const scheduleDiagnostics = () => {
  if (!monacoEditor || !monacoApi) return;
  if (diagnosticsTimer) clearTimeout(diagnosticsTimer);
  diagnosticsTimer = setTimeout(refreshDiagnostics, 180);
};

const refreshDiagnostics = async () => {
  if (!monacoEditor || !monacoApi) return;
  const text = getSource();
  let out = null;

  if (lspWasm?.diagnostics_json) {
    try {
      const raw = lspWasm.diagnostics_json(text, currentFile, "adwa");
      const parsed = JSON.parse(String(raw || "[]"));
      out = { ok: true, items: Array.isArray(parsed) ? parsed : [] };
    } catch {
      out = null;
    }
  }

  if (!out) {
    const uri = lspUriForCurrentFile();
    try {
      out = await postLsp("/diagnostics", { uri, text });
    } catch {
      return;
    }
  }

  if (!out?.ok || !Array.isArray(out.items)) return;
  const model = monacoEditor.getModel();
  if (!model) return;
  const markers = out.items.map((item) => {
    const range = item?.range || {};
    const start = range.start || {};
    const end = range.end || {};
    const severity = Number(item?.severity || 2);
    let sev = monacoApi.MarkerSeverity.Warning;
    if (severity === 1) sev = monacoApi.MarkerSeverity.Error;
    else if (severity === 3) sev = monacoApi.MarkerSeverity.Info;
    else if (severity >= 4) sev = monacoApi.MarkerSeverity.Hint;
    return {
      startLineNumber: Number(start.line || 0) + 1,
      startColumn: Number(start.character || 0) + 1,
      endLineNumber: Number(end.line || start.line || 0) + 1,
      endColumn: Number(end.character || start.character || 0) + 1,
      message: String(item?.message || "PHPX issue"),
      severity: sev,
      source: "phpx_lsp",
    };
  });
  monacoApi.editor.setModelMarkers(model, "phpx_lsp", markers);
};

const initLspWasm = async () => {
  try {
    const mod = await import("./vendor/adwa_js/phpx_lsp_wasm.js");
    const wasmUrl = new URL("./vendor/adwa_js/phpx_lsp_wasm_bg.wasm", import.meta.url).toString();
    await mod.default(wasmUrl);
    lspWasm = mod;
    log("PHPX diagnostics: wasm mode");
  } catch {
    lspWasm = null;
    log("PHPX diagnostics: sidecar fallback");
  }
};

const getWasmCompletion = async (text, line, character) => {
  if (!lspWasm?.completion_json) return null;
  try {
    const raw = lspWasm.completion_json(text, line, character);
    const parsed = JSON.parse(String(raw || "[]"));
    if (!Array.isArray(parsed)) return [];
    return parsed;
  } catch {
    return null;
  }
};

const getWasmHover = async (text, line, character) => {
  if (!lspWasm?.hover_json) return null;
  try {
    const raw = lspWasm.hover_json(text, line, character);
    if (!raw || raw === "null") return null;
    const parsed = JSON.parse(String(raw));
    if (!parsed || typeof parsed.value !== "string") return null;
    return parsed.value;
  } catch {
    return null;
  }
};

const getSource = () => {
  if (monacoEditor) return monacoEditor.getValue();
  if (sourceEl instanceof HTMLTextAreaElement) return sourceEl.value;
  return "";
};

const setSource = (value) => {
  if (monacoEditor) {
    monacoEditor.setValue(value);
    return;
  }
  if (sourceEl instanceof HTMLTextAreaElement) sourceEl.value = value;
};

const syncEditorToFile = () => {
  setSource(new TextDecoder().decode(vfs.readFile(currentFile)));
  renderFileTree();
  renderEditorTabs();
  scheduleDiagnostics();
};

const syncFileFromEditor = () => {
  const bytes = new TextEncoder().encode(getSource());
  vfs.writeFile(currentFile, bytes);
  commandManifestCache = null;
  if (shellContainer) {
    void syncPathToShell(currentFile, bytes);
  }
};

const resolveFromCwd = (path) => {
  if (!path) return cwd;
  return path.startsWith("/") ? normalizePath(path) : normalizePath(`${cwd}/${path}`);
};

const renderFileTree = () => {
  if (!(fileTreeListEl instanceof HTMLElement)) return;
  fileTreeListEl.innerHTML = "";
  const root = { name: "/", path: "/", type: "dir", children: new Map() };
  const targetRoot = normalizePath(explorerRoot || DEFAULT_CWD);

  const ensureTreeNode = (targetPath, typeHint = "dir") => {
    const parts = normalizePath(targetPath).split("/").filter(Boolean);
    let cursor = root;
    for (let i = 0; i < parts.length; i += 1) {
      const part = parts[i];
      const isLeaf = i === parts.length - 1;
      const nextPath = cursor.path === "/" ? `/${part}` : `${cursor.path}/${part}`;
      if (!cursor.children.has(part)) {
        cursor.children.set(part, {
          name: part,
          path: nextPath,
          type: isLeaf ? typeHint : "dir",
          children: isLeaf && typeHint === "file" ? null : new Map(),
        });
      }
      const next = cursor.children.get(part);
      if (!isLeaf && next.type !== "dir") {
        next.type = "dir";
        next.children = new Map();
      }
      cursor = next;
    }
  };

  for (const dir of vfs.listDirs()) {
    if (dir === "/") continue;
    ensureTreeNode(dir, "dir");
  }

  for (const file of vfs.listFiles()) {
    ensureTreeNode(file, "file");
  }

  const renderNode = (node, depth) => {
    const li = document.createElement("li");
    li.className = "treeNode";

    if (node.type === "dir") {
      const isOpen = expandedFolders.has(node.path);
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `treeRow treeFolder${node.path === cwd ? " active" : ""}`;
      btn.style.paddingLeft = `${8 + depth * 14}px`;
      btn.innerHTML = `<span class="treeCaret">${isOpen ? icon("chevronDown") : icon("chevronRight")}</span><span class="treeIcon">${folderIconFor(isOpen)}</span><span class="treeLabel">${node.name}</span>`;
      btn.addEventListener("click", () => {
        if (isOpen) expandedFolders.delete(node.path);
        else expandedFolders.add(node.path);
        saveExpandedFolders();
        renderFileTree();
      });
      li.appendChild(btn);

      if (isOpen && node.children && node.children.size > 0) {
        const ul = document.createElement("ul");
        ul.className = "treeChildren";
        const children = Array.from(node.children.values()).sort((a, b) => {
          if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        for (const child of children) {
          ul.appendChild(renderNode(child, depth + 1));
        }
        li.appendChild(ul);
      }
      return li;
    }

    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `treeRow treeFile${node.path === currentFile ? " active" : ""}`;
    btn.style.paddingLeft = `${8 + depth * 14}px`;
    btn.innerHTML = `<span class="treeCaret"></span><span class="treeIcon">${fileIconFor(node.path)}</span><span class="treeLabel">${node.name}</span>`;
    btn.title = node.path;
    btn.addEventListener("click", () => {
      syncFileFromEditor();
      selectFileTab(node.path, { preview: true });
    });
    btn.addEventListener("dblclick", (event) => {
      event.preventDefault();
      syncFileFromEditor();
      selectFileTab(node.path, { pin: true });
    });
    li.appendChild(btn);
    return li;
  };

  const findNodeByPath = (startNode, path) => {
    if (startNode.path === path) return startNode;
    if (!startNode.children) return null;
    for (const child of startNode.children.values()) {
      const found = findNodeByPath(child, path);
      if (found) return found;
    }
    return null;
  };

  let displayRoot = findNodeByPath(root, targetRoot);
  if (!displayRoot || displayRoot.type !== "dir") {
    displayRoot = root;
  }

  const topLevel = displayRoot.children
    ? Array.from(displayRoot.children.values()).sort((a, b) => {
        if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
        return a.name.localeCompare(b.name);
      })
    : [];

  for (const node of topLevel) {
    fileTreeListEl.appendChild(renderNode(node, 0));
  }

  if (explorerPathEl) explorerPathEl.textContent = targetRoot;
};

const selectFileTab = (path, options = {}) => {

  const file = normalizePath(path);
  ensureFolderExpandedFor(file);
  const existing = findTabIndexByPath(file);
  if (existing >= 0) {
    if (options.pin) openTabs[existing].pinned = true;
    currentFile = file;
    syncEditorToFile();
    return;
  }

  if (options.preview) {
    const previewIdx = findPreviewTabIndex();
    if (previewIdx >= 0) {
      openTabs[previewIdx] = { path: file, pinned: false };
    } else {
      openTabs.push({ path: file, pinned: false });
    }
  } else if (options.replaceCurrent && openTabs.length > 0) {
    const currentIdx = findTabIndexByPath(currentFile);
    if (currentIdx >= 0) {
      openTabs[currentIdx] = { path: file, pinned: Boolean(options.pin) };
    } else {
      openTabs[0] = { path: file, pinned: Boolean(options.pin) };
    }
  } else {
    openTabs.push({ path: file, pinned: Boolean(options.pin) });
  }
  currentFile = file;
  syncEditorToFile();
};

const closeFileTab = (path) => {
  const file = normalizePath(path);
  const idx = findTabIndexByPath(file);
  if (idx === -1) return;
  openTabs.splice(idx, 1);
  if (openTabs.length === 0) {
    const fallback = statPath(`${DEMO_ROOT}/main.phpx`) ? `${DEMO_ROOT}/main.phpx` : (vfs.listFiles()[0] || `${DEMO_ROOT}/README.txt`);
    if (fallback) openTabs.push({ path: fallback, pinned: true });
  }
  if (currentFile === file) {
    const nextIdx = Math.min(idx, openTabs.length - 1);
    currentFile = openTabs[Math.max(0, nextIdx)].path;
    syncEditorToFile();
    return;
  }
  renderEditorTabs();
};

const renderEditorTabs = () => {
  if (!(editorTabsEl instanceof HTMLElement)) return;
  editorTabsEl.innerHTML = "";
  for (const tab of openTabs) {
    const tabPath = tab.path;
    const button = document.createElement("button");
    button.type = "button";
    button.className = `editorTab${tabPath === currentFile ? " active" : ""}${tab.pinned ? "" : " preview"}`;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-selected", tabPath === currentFile ? "true" : "false");
    button.innerHTML = `<span class="editorTabLabel">${fileBaseName(tabPath)}</span><span class="editorTabClose" aria-hidden="true">${icon("close")}</span>`;
    button.title = tabPath;
    button.addEventListener("click", () => {
      if (tabPath === currentFile) return;
      syncFileFromEditor();
      currentFile = tabPath;
      syncEditorToFile();
    });
    button.addEventListener("dblclick", (event) => {
      event.preventDefault();
      selectFileTab(tabPath, { pin: true });
    });
    const close = button.querySelector(".editorTabClose");
    if (close instanceof HTMLElement) {
      close.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        syncFileFromEditor();
        closeFileTab(tabPath);
      });
    }
    editorTabsEl.appendChild(button);
  }
};

const isMacLike = () =>
  typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent || "");

const showFileTree = (open) => {
  if (!(appShellEl instanceof HTMLElement)) return;
  const visible = Boolean(open);
  appShellEl.classList.toggle("explorerCollapsed", !visible);
  if (window.innerWidth > 960 && visible) {
    appShellEl.style.gridTemplateColumns = `56px ${Math.round(explorerWidth)}px 6px 1fr`;
  }
  if (!visible || window.innerWidth <= 960) {
    appShellEl.style.gridTemplateColumns = "";
  }
};

const toggleFileTree = () => {
  if (!(appShellEl instanceof HTMLElement)) return;
  appShellEl.classList.toggle("explorerCollapsed");
};

const setTerminalVisible = (visible) => {
  terminalVisible = visible;
  if (!(rightPaneEl instanceof HTMLElement) || !(terminalPaneEl instanceof HTMLElement)) return;
  rightPaneEl.classList.toggle("terminalHidden", !visible);
  terminalPaneEl.classList.toggle("hidden", !visible);
  if (splitYEl instanceof HTMLElement) splitYEl.classList.toggle("hidden", !visible);
  if (visible) {
    if (!rightPaneEl.style.gridTemplateRows) {
      rightPaneEl.style.gridTemplateRows = "1fr 6px 280px";
    }
  } else {
    rightPaneEl.style.gridTemplateRows = "";
  }
  if (visible && termInputEl instanceof HTMLInputElement) termInputEl.focus();
};

const toggleTerminal = () => setTerminalVisible(!terminalVisible);

const initSplitters = () => {
  const setPreviewPointerEvents = (enabled) => {
    if (resultFrame instanceof HTMLIFrameElement) {
      resultFrame.style.pointerEvents = enabled ? "" : "none";
    }
  };

  const beginDrag = (target, cursor, onMove) => {
    let rafId = 0;
    let lastEvent = null;
    const flush = () => {
      rafId = 0;
      if (!lastEvent) return;
      onMove(lastEvent);
      lastEvent = null;
    };
    const handleMove = (event) => {
      lastEvent = event;
      if (!rafId) rafId = requestAnimationFrame(flush);
    };
    const handleUp = () => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = 0;
      lastEvent = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      setPreviewPointerEvents(true);
      target.releasePointerCapture?.(pointerId);
      target.removeEventListener("pointermove", handleMove);
      target.removeEventListener("pointerup", handleUp);
      target.removeEventListener("pointercancel", handleUp);
      target.removeEventListener("lostpointercapture", handleUp);
    };
    const pointerId = target.__dragPointerId;
    document.body.style.cursor = cursor;
    document.body.style.userSelect = "none";
    setPreviewPointerEvents(false);
    target.setPointerCapture?.(pointerId);
    target.addEventListener("pointermove", handleMove);
    target.addEventListener("pointerup", handleUp);
    target.addEventListener("pointercancel", handleUp);
    target.addEventListener("lostpointercapture", handleUp);
  };

  if (workspacePaneEl instanceof HTMLElement && splitXEl instanceof HTMLElement) {
    const minLeft = 340;
    const minRight = 320;
    splitXEl.addEventListener("pointerdown", (event) => {
      event.preventDefault();
      const rect = workspacePaneEl.getBoundingClientRect();
      splitXEl.__dragPointerId = event.pointerId;
      beginDrag(splitXEl, "col-resize", (moveEvent) => {
        const raw = moveEvent.clientX - rect.left;
        const clamped = Math.max(minLeft, Math.min(raw, rect.width - minRight - 6));
        workspacePaneEl.style.gridTemplateColumns = `${Math.floor(clamped)}px 6px minmax(${minRight}px, 1fr)`;
      });
    });
  }

  if (appShellEl instanceof HTMLElement && splitExplorerEl instanceof HTMLElement) {
    const minExplorer = 180;
    const maxExplorer = 520;
    splitExplorerEl.addEventListener("pointerdown", (event) => {
      if (window.innerWidth <= 960) return;
      event.preventDefault();
      const rect = appShellEl.getBoundingClientRect();
      splitExplorerEl.__dragPointerId = event.pointerId;
      beginDrag(splitExplorerEl, "col-resize", (moveEvent) => {
        const raw = moveEvent.clientX - rect.left - 56;
        explorerWidth = Math.max(minExplorer, Math.min(raw, Math.min(maxExplorer, rect.width - 420)));
        if (!appShellEl.classList.contains("explorerCollapsed")) {
          appShellEl.style.gridTemplateColumns = `56px ${Math.floor(explorerWidth)}px 6px 1fr`;
        }
      });
    });
  }

  if (rightPaneEl instanceof HTMLElement && splitYEl instanceof HTMLElement) {
    const minTop = 180;
    const minBottom = 120;
    splitYEl.addEventListener("pointerdown", (event) => {
      if (!terminalVisible) return;
      event.preventDefault();
      const rect = rightPaneEl.getBoundingClientRect();
      splitYEl.__dragPointerId = event.pointerId;
      beginDrag(splitYEl, "row-resize", (moveEvent) => {
        const raw = moveEvent.clientY - rect.top;
        const clamped = Math.max(minTop, Math.min(raw, rect.height - minBottom - 6));
        rightPaneEl.style.gridTemplateRows = `${Math.floor(clamped)}px 6px minmax(${minBottom}px, 1fr)`;
      });
    });
  }
};

const setHelpOpen = (open) => {
  if (!(helpModalEl instanceof HTMLElement)) return;
  helpModalEl.classList.toggle("open", Boolean(open));
};

const toggleHelp = () => {
  if (!(helpModalEl instanceof HTMLElement)) return;
  setHelpOpen(!helpModalEl.classList.contains("open"));
};

const focusEditor = () => {
  if (monacoEditor) {
    monacoEditor.focus();
    return;
  }
  if (sourceEl instanceof HTMLTextAreaElement) sourceEl.focus();
};

const decodeWasmDataUrl = async () => {
  if (phpRuntimeWasmBytesCache) return phpRuntimeWasmBytesCache;
  const response = await fetch(phpRuntimeRawWasmDataUrl);
  const buffer = await response.arrayBuffer();
  phpRuntimeWasmBytesCache = new Uint8Array(buffer);
  return phpRuntimeWasmBytesCache;
};

const simpleHash64 = (input) => {
  let h1 = 0x811c9dc5;
  let h2 = 0x1b873593;
  for (let i = 0; i < input.length; i += 1) {
    const c = input.charCodeAt(i);
    h1 = Math.imul(h1 ^ c, 0x01000193) >>> 0;
    h2 = Math.imul(h2 ^ c, 0x85ebca6b) >>> 0;
  }
  const base = `${h1.toString(16).padStart(8, "0")}${h2.toString(16).padStart(8, "0")}`;
  return (base + base + base + base).slice(0, 64);
};


const installDenoShim = async () => {
  const wasmBytes = await decodeWasmDataUrl();
  const readDirEntries = (path) =>
    vfs.readdir(path).map((name) => {
      const full = normalizePath(`${path}/${name}`);
      let fileType = "file";
      try {
        fileType = vfs.stat(full).fileType;
      } catch {
        fileType = "file";
      }
      const isFile = fileType === "file";
      const isDir = fileType === "dir";
      return {
        name,
        isFile,
        isDirectory: isDir,
        is_file: isFile,
        is_dir: isDir,
        fileType,
      };
    });

  globalThis.Deno = {
    core: {
      ops: {
        op_php_get_wasm: () => wasmBytes,
        op_php_parse_phpx_types: () => null,
        op_php_read_file_sync: (path) => vfs.readFile(normalizePath(String(path || "/"))),
        op_php_write_file_sync: (path, bytes) => {
          vfs.writeFile(normalizePath(String(path || "/")), toBytes(bytes));
        },
        op_php_mkdirs: (path) => vfs.mkdir(normalizePath(String(path || "/"))),
        op_php_sha256: (source) => simpleHash64(String(source || "")),
        op_php_random_bytes: (n) => {
          const out = new Uint8Array(Math.max(0, Number(n || 0) | 0));
          crypto.getRandomValues(out);
          return out;
        },
        op_php_read_env: () => ({ ...projectEnv }),
        op_php_db_proto_encode: (action, payload) => toJsonBytes({ action, payload }),
        op_php_db_proto_decode: (bytes) => fromJsonBytes(bytes),
        op_php_db_call_proto: () => toJsonBytes({ ok: false, error: "db unsupported in browser demo" }),
        op_php_net_proto_encode: (action, payload) => toJsonBytes({ action, payload }),
        op_php_net_proto_decode: (bytes) => fromJsonBytes(bytes),
        op_php_net_call_proto: () => toJsonBytes({ ok: false, error: "net unsupported in browser demo" }),
        op_php_fs_proto_encode: (action, payload) => toJsonBytes({ action, payload }),
        op_php_fs_proto_decode: (bytes) => fromJsonBytes(bytes),
        op_php_fs_call_proto: (requestBytes) => {
          const req = fromJsonBytes(requestBytes) || {};
          const out = bridgeRef.call({
            kind: "fs",
            action: String(req.action || ""),
            payload: req.payload || {},
          });
          return toJsonBytes(out.ok ? out.value : { ok: false, error: out.error });
        },
        op_php_cwd: () => cwd,
        op_php_file_exists: (path) => {
          const p = normalizePath(String(path || "/"));
          try {
            vfs.stat(p);
            return true;
          } catch {
            return false;
          }
        },
        op_php_path_resolve: (base, path) => normalizePath(`${String(base || "/")}/${String(path || "")}`),
        op_php_read_dir: (path) => readDirEntries(normalizePath(String(path || "/"))),
        op_php_parse_wit: () => ({ worlds: [], interfaces: [], exports: [] }),
      },
      print: (message, isErr) => {
        const text = String(message ?? "");
        if (isErr) phpStderrBuffer += text;
        else phpStdoutBuffer += text;
      },
    },
    env: {
      toObject: () => ({ ...projectEnv }),
    },
  };
};

const ensureDekaPhpRuntime = async () => {
  if (dekaPhpRuntimeReady) return;
  await installDenoShim();
  await import("./vendor/adwa_js/deka_php_runtime.js");
  if (!globalThis.__dekaPhp || typeof globalThis.__dekaPhp.runFile !== "function") {
    throw new Error("deka_php runtime failed to initialize");
  }
  dekaPhpRuntimeReady = true;
};

const executePhpFile = async (entryFile, requestPath = null, envOverrides = null) => {
  if (!bridgeRef) throw new Error("bridge is not ready");

  syncFileFromEditor();
  bridgeRef.call({ kind: "process", action: "chdir", payload: { path: cwd } });
  await ensureDekaPhpRuntime();
  const previousEnv = {};
  const previousBridgeEnv = {};
  if (envOverrides && typeof envOverrides === "object") {
    for (const [key, value] of Object.entries(envOverrides)) {
      previousEnv[key] = Object.prototype.hasOwnProperty.call(projectEnv, key)
        ? projectEnv[key]
        : undefined;
      projectEnv[key] = String(value ?? "");
      const prev = bridgeRef.call({
        kind: "process",
        action: "envGet",
        payload: { key },
      });
      previousBridgeEnv[key] = prev?.ok ? prev.value?.value : null;
      bridgeRef.call({
        kind: "process",
        action: "envSet",
        payload: { key, value: String(value ?? "") },
      });
    }
  }
  if (requestPath) {
    const [pathOnly, query = ""] = String(requestPath).split("?", 2);
    projectEnv.REQUEST_METHOD = "GET";
    projectEnv.REQUEST_URI = query ? `${pathOnly}?${query}` : pathOnly;
    projectEnv.PATH_INFO = pathOnly;
    projectEnv.QUERY_STRING = query;
  } else {
    delete projectEnv.REQUEST_METHOD;
    delete projectEnv.REQUEST_URI;
    delete projectEnv.PATH_INFO;
    delete projectEnv.QUERY_STRING;
  }
  phpStdoutBuffer = "";
  phpStderrBuffer = "";
  let result = null;
  try {
    result = await globalThis.__dekaPhp.runFile(entryFile);
  } finally {
    if (envOverrides && typeof envOverrides === "object") {
      for (const key of Object.keys(envOverrides)) {
        if (previousEnv[key] === undefined) {
          delete projectEnv[key];
        } else {
          projectEnv[key] = previousEnv[key];
        }
        if (previousBridgeEnv[key] === null || previousBridgeEnv[key] === undefined) {
          bridgeRef.call({ kind: "process", action: "envUnset", payload: { key } });
        } else {
          bridgeRef.call({
            kind: "process",
            action: "envSet",
            payload: { key, value: String(previousBridgeEnv[key]) },
          });
        }
      }
    }
  }
  return result;
};

const runPhpxEntry = async (entryFile, requestPath = null) => {
  return executePhpFile(entryFile, requestPath, null);
};

const writeRunResult = (result) => {
  const stdoutText = String(result?.stdout ?? phpStdoutBuffer ?? "");
  const isOk = Boolean(result?.ok);
  if (isOk) {
    resultView.renderResult(stdoutText);
    resultView.clearResultBanner();
    setTimeout(() => attachPreviewFrameNavigation(), 0);
  } else {
    if (!resultView.hasLastGoodResult()) {
      resultView.renderResult(stdoutText);
    }
    const diagMsg =
      Array.isArray(result?.diagnostics) && result.diagnostics.length
        ? `${result.diagnostics[0].severity || "error"}: ${result.diagnostics[0].message || "run failed"}`
        : "";
    const stderrMsg = firstErrorLine(result?.stderr || phpStderrBuffer);
    const msg = diagMsg || stderrMsg || "Run failed. Showing last good result.";
    resultView.showResultBanner(msg);
  }

  if (result?.stdout && !looksLikeHtmlOutput(result.stdout)) {
    log(String(result.stdout).trimEnd());
  }
  if (result?.stderr) log(`[stderr]\n${String(result.stderr).trimEnd()}`);
  if (phpStderrBuffer.trim()) log(`[stderr]\n${phpStderrBuffer.trimEnd()}`);
  if (result.diagnostics?.length) {
    for (const diag of result.diagnostics) log(`[${diag.severity}] ${diag.message}`);
  }
  if (!result.ok && !result.diagnostics?.length) {
    log("[error] runtime returned not-ok without diagnostics");
  }
  scheduleDiagnostics();
};

const runPhpxScript = async () => {
  const result = await runPhpxEntry(currentFile, null);
  writeRunResult(result);
};

const normalizeServePath = (value) => {
  const raw = String(value || "/").trim() || "/";
  const full = raw.startsWith("/") ? raw : `/${raw}`;
  const [pathOnly, query = ""] = full.split("?", 2);
  const normalized = normalizePath(pathOnly);
  return query ? `${normalized}?${query}` : normalized;
};

const buildPreviewHistoryUrl = (path) => {
  const rel = normalizeServePath(path || "/");
  return `?demo=phpx#${rel}`;
};

const readPreviewPathFromLocation = () => {
  const hash = String(window.location.hash || "");
  if (hash.startsWith("#/")) {
    return normalizeServePath(hash.slice(1));
  }
  return "/";
};

const runServedPath = async (path, opts = {}) => {
  if (!serverState.running) return;
  const nextPath = normalizeServePath(path || serverState.path || "/");
  const result = await runPhpxEntry(serverState.entry, nextPath);
  writeRunResult(result);
  serverState.path = nextPath;
  setPreviewPath(nextPath);
  setServePreviewStatus(nextPath);
  if (opts.pushHistory !== false) {
    history.pushState({ previewPath: nextPath }, "", buildPreviewHistoryUrl(nextPath));
  }
};

const startVirtualServer = async (entryFile, port = 8530, mode = "php") => {
  await syncAllVfsToShell();
  const resolved = resolveFromCwd(entryFile || configuredServeEntry || "app/home.phpx");
  try {
    const stat = vfs.stat(resolved);
    if (stat.fileType !== "file") {
      log(`deka serve: not a file: ${resolved}`);
      return;
    }
  } catch {
    log(`deka serve: file not found: ${resolved}`);
    return;
  }
  serverState.running = true;
  serverState.entry = resolved;
  serverState.mode = String(mode || configuredServeMode || "php");
  serverState.port = port;
  serverState.path = readPreviewPathFromLocation();
  runtimePortState.url = "";
  runtimePortState.port = 0;
  setPreviewStatus(`serve :${port} (${serverState.mode})`);
  setPreviewPath(serverState.path);
  history.replaceState(
    { previewPath: serverState.path },
    "",
    buildPreviewHistoryUrl(serverState.path)
  );
  log(`[handler] loaded ${resolved} [mode=${serverState.mode}]`);
  log(`[listen] http://localhost:${port}`);
  await runServedPath(serverState.path, { pushHistory: false });
};

const stopVirtualServer = async (opts = {}) => {
  if (!serverState.running) return;
  const interrupted = Boolean(opts.interrupted);
  if (shellContainer && typeof shellContainer.signalForeground === "function") {
    try {
      await shellContainer.signalForeground(interrupted ? 2 : 15);
    } catch {}
  }
  serverState.running = false;
  runtimePortState.url = "";
  runtimePortState.port = 0;
  setTerminalBlocked(false);
  setPrompt();
  setPreviewStatus("run mode");
  history.replaceState({}, "", "?demo=phpx#/");
  log("[serve] stopped");
};

const runDekaFile = async (entryFile) => {
  const resolved = resolveFromCwd(entryFile || currentFile || "main.phpx");
  try {
    const stat = vfs.stat(resolved);
    if (stat.fileType !== "file") {
      log(`deka run: not a file: ${resolved}`);
      return;
    }
  } catch {
    log(`deka run: file not found: ${resolved}`);
    return;
  }
  const result = await runPhpxEntry(resolved, null);
  log(`PHPX run complete (${resolved}).`);
  writeRunResult(result);
};

const statPath = (path) => {
  try {
    return vfs.stat(path);
  } catch {
    return null;
  }
};

const readJsonFile = (path) => {
  try {
    const raw = new TextDecoder().decode(vfs.readFile(path));
    return JSON.parse(raw);
  } catch {
    return null;
  }
};

const shellSplit = (value) => {
  const text = String(value || "").trim();
  if (!text) return [];
  return text.split(/\s+/).filter(Boolean);
};

const parseDekaShebang = (sourceText) => {
  const firstLine = String(sourceText || "").split(/\r?\n/, 1)[0] || "";
  if (!firstLine.startsWith("#!")) return null;
  const body = firstLine.slice(2).trim();

  const direct = body.match(/^\/usr\/bin\/deka(?:\s+(.*))?$/);
  if (direct) {
    return { runner: "deka", args: shellSplit(direct[1] || "") };
  }

  const env = body.match(/^\/usr\/bin\/env\s+deka(?:\s+(.*))?$/);
  if (env) {
    return { runner: "deka", args: shellSplit(env[1] || "") };
  }

  return null;
};

const listManifestFiles = () => {
  return vfs
    .listFiles()
    .filter((path) => path.endsWith("/deka.json"))
    .sort((a, b) => a.localeCompare(b));
};

const parseBinEntries = (manifest, manifestDir) => {
  const entries = [];
  const bin = manifest?.bin;
  if (typeof bin === "string" && bin.trim()) {
    const name = String(manifest?.name || "").trim();
    if (name) {
      entries.push({
        command: name,
        entry: normalizePath(`${manifestDir}/${bin}`),
      });
    }
  } else if (bin && typeof bin === "object") {
    for (const [command, rel] of Object.entries(bin)) {
      if (!command || typeof rel !== "string" || !rel.trim()) continue;
      entries.push({
        command: String(command).trim(),
        entry: normalizePath(`${manifestDir}/${rel}`),
      });
    }
  }
  return entries;
};

const buildCommandManifestCache = () => {
  const cache = new Map();
  const allManifests = listManifestFiles();
  const passes = [
    { root: LOCAL_MODULE_ROOT, source: "local-bin" },
    { root: GLOBAL_MODULE_ROOT, source: "global-bin" },
  ];

  for (const pass of passes) {
    for (const manifestPath of allManifests) {
      if (!manifestPath.startsWith(`${pass.root}/`)) continue;
      const manifest = readJsonFile(manifestPath);
      if (!manifest || typeof manifest !== "object") continue;
      const manifestDir = dirname(manifestPath);

      for (const entry of parseBinEntries(manifest, manifestDir)) {
        if (!entry.command || !entry.entry) continue;
        if (!statPath(entry.entry)) continue;
        if (!cache.has(entry.command)) {
          cache.set(entry.command, {
            entry: entry.entry,
            manifestPath,
            source: pass.source,
          });
        }
      }
    }
  }

  return cache;
};

const getCommandManifestCache = () => {
  commandManifestCache = buildCommandManifestCache();
  return commandManifestCache;
};

const resolveCliAdwaCommand = (command) => {
  const safe = String(command || "").trim();
  if (!safe) return null;
  const cache = getCommandManifestCache();
  const resolved = cache.get(safe);
  if (!resolved) return null;
  return {
    entry: resolved.entry,
    manifestPath: resolved.manifestPath,
    source: resolved.source,
  };
};

const runCliAdwaCommandDirect = async (command, args, options = {}) => {
  const resolved = resolveCliAdwaCommand(command);
  if (!resolved) return null;
  const processCwd = normalizePath(String(options?.cwd || cwd || "/"));

  const toHex = (text) => {
    const bytes = new TextEncoder().encode(String(text ?? ""));
    let out = "";
    for (let i = 0; i < bytes.length; i += 1) {
      out += bytes[i].toString(16).padStart(2, "0");
    }
    return out;
  };

  const previousCwd = cwd;
  cwd = processCwd;
  projectEnv.PWD = cwd;
  vfs.writeFile("/tmp/.adwa_cwd", new TextEncoder().encode(cwd));
  vfs.writeFile("/tmp/.adwa_args", new TextEncoder().encode(args.join("\u001f")));
  vfs.writeFile("/__adwa_cwd.txt", new TextEncoder().encode(cwd));
  vfs.writeFile("/__adwa_args.txt", new TextEncoder().encode(args.join("\u001f")));
  const envOverrides = {
    ADWA_CMD: command,
    ADWA_CWD: cwd,
    ADWA_ARGC: String(args.length),
    ADWA_ARGS: args.join("\u001f"),
  };
  for (let i = 0; i < args.length; i += 1) {
    envOverrides[`ADWA_ARG_${i}`] = args[i];
    envOverrides[`ADWA_ARGHEX_${i}`] = toHex(args[i]);
  }
  if (bridgeRef && typeof bridgeRef.call === "function") {
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_CWD", value: cwd },
    });
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "PWD", value: cwd },
    });
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_ARGC", value: String(args.length) },
    });
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_ARGS", value: args.join("\u001f") },
    });
    const max = Math.max(lastCliArgCount, args.length);
    for (let i = 0; i < max; i += 1) {
      const key = `ADWA_ARG_${i}`;
      const hexKey = `ADWA_ARGHEX_${i}`;
      if (i < args.length) {
        bridgeRef.call({
          kind: "process",
          action: "envSet",
          payload: { key, value: String(args[i] ?? "") },
        });
        bridgeRef.call({
          kind: "process",
          action: "envSet",
          payload: { key: hexKey, value: toHex(args[i]) },
        });
      } else {
        bridgeRef.call({ kind: "process", action: "envUnset", payload: { key } });
        bridgeRef.call({ kind: "process", action: "envUnset", payload: { key: hexKey } });
      }
    }
    lastCliArgCount = args.length;
  }
  globalThis.__DEKA_ADWA_CLI_CONTEXT = {
    cwd,
    args: [...args],
  };
  let result = null;
  try {
    result = await executePhpFile(resolved.entry, null, envOverrides);
  } finally {
    delete globalThis.__DEKA_ADWA_CLI_CONTEXT;
    cwd = previousCwd;
    projectEnv.PWD = cwd;
  }
  const stdout = String(result?.stdout ?? phpStdoutBuffer ?? "");
  const stderr = String(result?.stderr ?? phpStderrBuffer ?? "");
  const detail = firstErrorLine(
    result?.error || result?.message || result?.diagnostics || ""
  );
  const code = result?.ok ? 0 : 1;
  const finalErr = !result?.ok && !stderr.trim() && detail ? `${detail}\n` : stderr;
  renderFileTree();
  return {
    code,
    stdout,
    stderr: finalErr,
  };
};

const createCliAdwaProcess = (command, args, options = {}) => {
  if (!shellContainer || typeof shellContainer.createVirtualProcess !== "function") {
    return null;
  }
  if (ADWA_BUILTIN_COMMANDS.has(String(command || "").trim())) {
    return null;
  }
  const resolved = resolveCliAdwaCommand(command);
  if (!resolved) return null;

  return shellContainer.createVirtualProcess(async () => runCliAdwaCommandDirect(command, args, options));
};

const runShebangScript = async (scriptPath, argv, shebang) => {
  const shebangArgs = Array.isArray(shebang?.args) ? shebang.args : [];
  const effective = shebangArgs.length ? shebangArgs : ["run"];
  const sub = String(effective[0] || "run");

  // POSIX-like argv passthrough into process env for PHPX script access.
  const fullArgv = [scriptPath, ...argv];
  const toHex = (text) => {
    const bytes = new TextEncoder().encode(String(text ?? ""));
    let out = "";
    for (let i = 0; i < bytes.length; i += 1) {
      out += bytes[i].toString(16).padStart(2, "0");
    }
    return out;
  };
  const envOverrides = {
    ADWA_CMD: scriptPath,
    ADWA_CWD: cwd,
    ADWA_ARGC: String(fullArgv.length),
    ADWA_ARGS: fullArgv.join("\u001f"),
    DEKA_ARGS: JSON.stringify(argv.map((value) => String(value ?? ""))),
  };
  for (let i = 0; i < fullArgv.length; i += 1) {
    envOverrides[`ADWA_ARG_${i}`] = fullArgv[i];
    envOverrides[`ADWA_ARGHEX_${i}`] = toHex(fullArgv[i]);
  }
  if (bridgeRef && typeof bridgeRef.call === "function") {
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_CWD", value: cwd },
    });
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_ARGC", value: String(fullArgv.length) },
    });
    bridgeRef.call({
      kind: "process",
      action: "envSet",
      payload: { key: "ADWA_ARGS", value: fullArgv.join("\u001f") },
    });
    const max = Math.max(lastShebangArgCount, fullArgv.length);
    for (let i = 0; i < max; i += 1) {
      const key = `ADWA_ARG_${i}`;
      const hexKey = `ADWA_ARGHEX_${i}`;
      if (i < fullArgv.length) {
        bridgeRef.call({
          kind: "process",
          action: "envSet",
          payload: { key, value: String(fullArgv[i] ?? "") },
        });
        bridgeRef.call({
          kind: "process",
          action: "envSet",
          payload: { key: hexKey, value: toHex(fullArgv[i]) },
        });
      } else {
        bridgeRef.call({ kind: "process", action: "envUnset", payload: { key } });
        bridgeRef.call({ kind: "process", action: "envUnset", payload: { key: hexKey } });
      }
    }
    lastShebangArgCount = fullArgv.length;
  }

  if (sub !== "run") {
    log(`shebang: unsupported deka subcommand '${sub}' (only 'run' is supported in browser demo)`);
    return true;
  }

  let runPath = scriptPath;
  try {
    const raw = new TextDecoder().decode(vfs.readFile(scriptPath));
    const stripped = raw.replace(/^#![^\r\n]*(?:\r?\n)?/, "");
    if (stripped !== raw) {
      const tempPath = normalizePath(`/tmp/.adwa-shebang-${Date.now()}.phpx`);
      const bytes = new TextEncoder().encode(stripped);
      vfs.writeFile(tempPath, bytes);
      commandManifestCache = null;
      if (shellContainer) {
        await syncPathToShell(tempPath, bytes);
      }
      runPath = tempPath;
    }
  } catch {}

  const result = await executePhpFile(runPath, null, envOverrides);
  log(`[shebang] executed ${scriptPath}`);
  log(`PHPX run complete (${scriptPath}).`);
  writeRunResult(result);
  return true;
};

const attachPreviewFrameNavigation = () => {
  if (!(resultFrame instanceof HTMLIFrameElement)) return;
  const doc = resultFrame.contentDocument;
  if (!doc || doc.__adwaNavBound) return;
  doc.__adwaNavBound = true;
  doc.addEventListener("click", (event) => {
    if (!serverState.running) return;
    const target = event.target;
    if (!(target instanceof Element)) return;
    const anchor = target.closest("a[href]");
    if (!(anchor instanceof HTMLAnchorElement)) return;
    const href = anchor.getAttribute("href");
    if (!href || href.startsWith("http://") || href.startsWith("https://") || href.startsWith("mailto:")) {
      return;
    }
    event.preventDefault();
    const current = `http://localhost${serverState.path || "/"}`;
    const next = new URL(href, current);
    void runServedPath(`${next.pathname}${next.search}`, { pushHistory: true });
  });
};

const executeRun = async (fromAuto = false) => {
  if (runInFlight) {
    rerunRequested = true;
    return;
  }
  runInFlight = true;
  if (runBtn instanceof HTMLButtonElement) runBtn.disabled = true;
  try {
    if (serverState.running) {
      await runServedPath(serverState.path || "/", { pushHistory: false });
    } else {
      await runPhpxScript();
    }
  } catch (err) {
    resetLog(fromAuto ? "Auto-run failed." : "Run failed.");
    const message = err instanceof Error ? err.message : String(err);
    log(message);
    resultView.showResultBanner(message || "Run failed. Showing last good result.");
  } finally {
    runInFlight = false;
    if (runBtn instanceof HTMLButtonElement) runBtn.disabled = false;
  }
  if (rerunRequested) {
    rerunRequested = false;
    await executeRun(true);
  }
};

const scheduleAutoRun = () => {
  if (autorunTimer) clearTimeout(autorunTimer);
  autorunTimer = setTimeout(() => {
    executeRun(true);
  }, AUTORUN_DEBOUNCE_MS);
};

const openFile = (path) => {
  const resolved = resolveFromCwd(path);
  try {
    const stat = vfs.stat(resolved);
    if (stat.fileType !== "file") {
      log(`open: not a file: ${resolved}`);
      return;
    }
    syncFileFromEditor();
    selectFileTab(resolved, { pin: true });
    log(`opened ${resolved}`);
  } catch (err) {
    log(err instanceof Error ? err.message : String(err));
  }
};

const createFileAt = async (inputPath) => {
  const raw = String(inputPath || "").trim();
  if (!raw) {
    log("new file: path required");
    return null;
  }
  const resolved = resolveFromCwd(raw);
  try {
    const stat = vfs.stat(resolved);
    if (stat.fileType === "dir") {
      log(`new file: path is a directory: ${resolved}`);
      return null;
    }
  } catch {}

  // Keep file creation behavior unified with terminal commands.
  const bytes = new TextEncoder().encode("");
  vfs.writeFile(resolved, bytes);
  commandManifestCache = null;
  if (shellContainer) {
    await syncPathToShell(resolved, bytes);
  }

  try {
    const stat = vfs.stat(resolved);
    if (stat.fileType !== "file") {
      log(`new file: failed to create ${resolved}`);
      return null;
    }
  } catch {
    log(`new file: failed to create ${resolved}`);
    return null;
  }
  selectFileTab(resolved, { pin: true });
  log(`created ${resolved}`);
  return resolved;
};

const createFolderAt = async (inputPath) => {
  const raw = String(inputPath || "").trim();
  if (!raw) {
    log("new folder: path required");
    return null;
  }
  const resolved = resolveFromCwd(raw);
  vfs.mkdir(resolved);
  commandManifestCache = null;
  if (shellContainer) {
    try {
      await shellContainer.fs.mkdir(resolved, { recursive: true });
    } catch {}
  }
  expandedFolders.add(resolved);
  saveExpandedFolders();
  renderFileTree();
  log(`created dir ${resolved}`);
  return resolved;
};

const syncPathToShell = async (path, bytes) => {
  if (!shellContainer) return;
  const normalized = normalizePath(path);
  const parent = dirname(normalized);
  try {
    await shellContainer.fs.mkdir(parent, { recursive: true });
  } catch {}
  await shellContainer.fs.writeFile(normalized, bytes);
};

const syncAllVfsToShell = async () => {
  if (!shellContainer) return;
  for (const file of vfs.listFiles()) {
    await syncPathToShell(file, vfs.readFile(file));
  }
};

syncExplorerToCwd();

const initShellContainer = async () => {
  const bindings = await import("./vendor/adwa_wasm/adwa_wasm.js");
  shellContainer = await WebContainer.boot(bindings, {
    init: bindings.default,
  });
  shellContainer.setSpawnInterceptor(({ program, args, options }) =>
    createCliAdwaProcess(program, args, options)
  );
  await syncAllVfsToShell();
};

const readProcessOutput = async (proc) => {
  const decoder = new TextDecoder();
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const MAX_IDLE_POLLS = 4;
  const IDLE_POLL_MS = 8;
  let combined = "";
  let outputIdle = 0;
  while (true) {
    const chunk = await proc.readOutput(65536);
    if (!chunk) {
      if (outputIdle >= MAX_IDLE_POLLS) break;
      outputIdle += 1;
      await sleep(IDLE_POLL_MS);
      continue;
    }
    outputIdle = 0;
    combined += decoder.decode(chunk, { stream: true });
  }
  if (combined.trim().length > 0) {
    return { stdout: combined, stderr: "" };
  }

  let out = "";
  let err = "";
  let stdoutIdle = 0;
  while (true) {
    const chunk = await proc.readStdout(65536);
    if (!chunk) {
      if (stdoutIdle >= MAX_IDLE_POLLS) break;
      stdoutIdle += 1;
      await sleep(IDLE_POLL_MS);
      continue;
    }
    stdoutIdle = 0;
    out += decoder.decode(chunk, { stream: true });
  }
  let stderrIdle = 0;
  while (true) {
    const chunk = await proc.readStderr(65536);
    if (!chunk) {
      if (stderrIdle >= MAX_IDLE_POLLS) break;
      stderrIdle += 1;
      await sleep(IDLE_POLL_MS);
      continue;
    }
    stderrIdle = 0;
    err += decoder.decode(chunk, { stream: true });
  }
  return { stdout: out, stderr: err };
};

const readProcessOutputAfterExit = async (proc) => {
  // ADWA virtual processes don't currently close output streams on exit.
  // Read buffered stdout/stderr queues instead of waiting on stream EOF.
  return readProcessOutput(proc);
};

const runShellCommand = async (line) => {
  const cmdLine = line.trim();
  if (!cmdLine) return;

  const [cmd, ...rest] = cmdLine.split(/\s+/);
  const arg = rest.join(" ");
  const runProcessAndRender = async (proc, cmdName, argv) => {
    const status = await proc.exit();
    let { stdout, stderr } = await readProcessOutputAfterExit(proc);
    if (stdout.trim()) log(stdout.trimEnd());
    if (stderr.trim()) log(`[stderr]\n${stderr.trimEnd()}`);
    if (status.code !== 0 && !stderr.trim()) {
      log(`[exit ${status.code}]`);
    }
    renderFileTree();
  };

  if (serverState.running && cmd !== "deka") {
    log("[busy] deka serve is running (Ctrl+C to stop)");
    return;
  }

  if (cmd === "clear") {
    resetLog("");
    return;
  }

  log(`${cwd} $ ${cmdLine}`);

  if (cmd === "cd") {
    const next = resolveFromCwd(arg || "/");
    try {
      const stat = vfs.stat(next);
      if (stat.fileType !== "dir") {
        log(`cd: not a directory: ${next}`);
        return;
      }
      cwd = next;
      projectEnv.PWD = cwd;
      syncExplorerToCwd();
      setPrompt();
      renderFileTree();
      return;
    } catch {
      log(`cd: no such directory: ${next}`);
      return;
    }
  }
  if (cmd === "open") {
    if (!arg) {
      log("open: path required");
      return;
    }
    openFile(arg);
    return;
  }
  if (cmd === "run") {
    await executeRun(false);
    return;
  }
  if (cmd === "history") {
    for (let i = 0; i < commandHistory.length; i += 1) {
      log(`${i + 1}  ${commandHistory[i]}`);
    }
    return;
  }
  if (cmd === "deka") {
    const sub = rest[0] || "";
    const printDekaHelp = () => {
      log("deka commands:");
      log("  deka help");
      log("  deka init");
      log("  deka run <file>");
      log("  deka serve [entry] [--port N] [--mode php|phpx]");
      log("  deka stop");
      log("  deka status");
    };
    if (!sub || sub === "help" || sub === "--help" || sub === "-h") {
      printDekaHelp();
      return;
    }
    if (sub === "init") {
      const force = rest.includes("--force");
      const target = "/deka.json";
      let exists = false;
      try {
        exists = vfs.stat(target).fileType === "file";
      } catch {
        exists = false;
      }
      if (exists && !force) {
        log("deka init: /deka.json already exists (use --force to overwrite)");
        return;
      }
      const template = {
        serve: {
          entry: "app/home.phpx",
          mode: "php",
        },
        scripts: {
          dev: "deka serve --dev",
        },
      };
      const bytes = new TextEncoder().encode(`${JSON.stringify(template, null, 2)}\n`);
      vfs.writeFile(target, bytes);
      commandManifestCache = null;
      if (shellContainer) {
        await syncPathToShell(target, bytes);
      }
      renderFileTree();
      log("created /deka.json");
      return;
    }
    if (sub === "run") {
      const entry = rest[1] || currentFile;
      await runDekaFile(entry);
      return;
    }
    if (sub === "serve") {
      if (serverState.running) {
        log("[serve] already running (Ctrl+C to stop)");
        return;
      }
      let entry = configuredServeEntry || currentFile;
      let port = 8530;
      let mode = configuredServeMode || "php";
      for (let i = 1; i < rest.length; i += 1) {
        const token = rest[i];
        if (!token) continue;
        if (token === "--port") {
          const val = Number(rest[i + 1] || "8530");
          if (Number.isFinite(val) && val > 0) port = Math.floor(val);
          i += 1;
          continue;
        }
        if (token === "--mode") {
          mode = String(rest[i + 1] || mode);
          i += 1;
          continue;
        }
        if (!token.startsWith("-")) {
          entry = token;
        }
      }
      if (!shellContainer) {
        log("terminal runtime not ready");
        return;
      }
      const proc = await shellContainer.spawn(
        "deka",
        ["serve", entry, "--port", String(port), "--mode", String(mode)],
        { cwd }
      );
      const { stdout, stderr } = await readProcessOutput(proc);
      if (stdout.trim()) log(stdout.trimEnd());
      if (stderr.trim()) log(`[stderr]\n${stderr.trimEnd()}`);
      if (shellContainer && typeof shellContainer.setForegroundPid === "function") {
        await shellContainer.setForegroundPid(proc.pid());
      }
      setTerminalBlocked(true);
      setPrompt();
      log("[serve] foreground process running (Ctrl+C to stop)");
      await startVirtualServer(entry, port, mode);
      return;
    }
    if (sub === "stop") {
      await stopVirtualServer({ interrupted: false });
      return;
    }
    if (sub === "status") {
      if (serverState.running) {
        log(`[serve] running entry=${serverState.entry} port=${serverState.port} path=${serverState.path}`);
      } else {
        log("[serve] stopped");
      }
      return;
    }
    log(`deka: unsupported subcommand '${sub}'`);
    printDekaHelp();
    return;
  }

  const maybeScript = resolveFromCwd(cmd);
  const scriptStat = statPath(maybeScript);
  if (scriptStat && scriptStat.fileType === "file") {
    try {
      const text = new TextDecoder().decode(vfs.readFile(maybeScript));
      const shebang = parseDekaShebang(text);
      if (shebang?.runner === "deka") {
        await runShebangScript(maybeScript, rest, shebang);
        return;
      }
    } catch {}
  }

  if (!shellContainer) {
    log("terminal runtime not ready");
    return;
  }
  const latest = new TextEncoder().encode(getSource());
  vfs.writeFile(currentFile, latest);
  commandManifestCache = null;
  await syncPathToShell(currentFile, latest);
  if (!ADWA_BUILTIN_COMMANDS.has(cmd)) {
    const direct = await runCliAdwaCommandDirect(cmd, rest, { cwd });
    if (direct) {
      if (direct.stdout.trim()) log(direct.stdout.trimEnd());
      if (direct.stderr.trim()) log(`[stderr]\n${direct.stderr.trimEnd()}`);
      if (direct.code !== 0 && !direct.stderr.trim()) {
        log(`[exit ${direct.code}]`);
      }
      return;
    }
  }
  const intercepted = createCliAdwaProcess(cmd, rest, { cwd });
  if (intercepted) {
    await runProcessAndRender(intercepted, cmd, rest);
    return;
  }
  const args = rest;
  const proc = await shellContainer.spawn(cmd, args, { cwd });
  await runProcessAndRender(proc, cmd, args);
};

const initMonaco = async () => {
  const out = await initMonacoEditor({
    editorHostEl,
    sourceEl,
    defaultSource,
    onChange: () => {
      scheduleDiagnostics();
      scheduleAutoRun();
    },
    provideCompletionItems: async (monaco, model, position) => {
      const wasmItems = await getWasmCompletion(
        model.getValue(),
        position.lineNumber - 1,
        position.column - 1
      );
      if (Array.isArray(wasmItems) && wasmItems.length > 0) {
        return wasmItems.map((item) => ({
          label: item.label || "",
          kind: monaco.languages.CompletionItemKind.Text,
          insertText: item.insert_text || item.label || "",
          detail: item.detail || "",
          range: undefined,
        }));
      }
      const out = await postLsp("/completion", {
        uri: lspUriForCurrentFile(),
        text: model.getValue(),
        line: position.lineNumber - 1,
        character: position.column - 1,
      });
      if (!out?.ok || !Array.isArray(out.items)) return [];
      return out.items.map((item) => ({
        label: item.label || "",
        kind: monaco.languages.CompletionItemKind.Text,
        insertText: item.insertText || item.label || "",
        detail: item.detail || "",
        documentation:
          typeof item.documentation === "string"
            ? item.documentation
            : item.documentation?.value || "",
        range: undefined,
      }));
    },
    provideHover: async (monaco, model, position) => {
      const wasmHover = await getWasmHover(
        model.getValue(),
        position.lineNumber - 1,
        position.column - 1
      );
      if (wasmHover) {
        return {
          range: new monaco.Range(
            position.lineNumber,
            position.column,
            position.lineNumber,
            position.column
          ),
          contents: [{ value: "```php\n" + wasmHover + "\n```" }],
        };
      }

      const out = await postLsp("/hover", {
        uri: lspUriForCurrentFile(),
        text: model.getValue(),
        line: position.lineNumber - 1,
        character: position.column - 1,
      });
      if (!out?.ok || !out.hover) return null;
      const contents = out.hover.contents;
      let value = "";
      if (typeof contents === "string") value = contents;
      else if (Array.isArray(contents))
        value = contents
          .map((entry) => (typeof entry === "string" ? entry : entry?.value || ""))
          .filter(Boolean)
          .join("\n\n");
      else value = contents?.value || "";
      if (!value) return null;
      return {
        range: new monaco.Range(
          position.lineNumber,
          position.column,
          position.lineNumber,
          position.column
        ),
        contents: [{ value }],
      };
    },
  });
  monacoApi = out.monacoApi;
  monacoEditor = out.monacoEditor;
};

const boot = async () => {
  resetLog("Booting PHPX runtime...");
  await initShellContainer();
  if (shellContainer) {
    const onServerReady = (event) => {
      if (!serverState.running) return;
      const eventPort = Number(event?.port || 0);
      if (!eventPort || eventPort !== Number(serverState.port || 0)) return;
      runtimePortState.port = eventPort;
      runtimePortState.url = String(event?.url || "");
      setServePreviewStatus(serverState.path || "/");
    };
    const onPortClosed = (event) => {
      const eventPort = Number(event?.port || 0);
      const activePort = Number(runtimePortState.port || serverState.port || 0);
      if (!eventPort || !activePort || eventPort !== activePort) return;
      runtimePortState.url = "";
      runtimePortState.port = 0;
      if (serverState.running) {
        setServePreviewStatus(serverState.path || "/");
      }
    };
    shellContainer.on("server-ready", onServerReady);
    shellContainer.on("port-closed", onPortClosed);
  }
  await initMonaco();
  await initLspWasm();

  // Lightweight e2e hooks for deterministic playground tests.
  window.__adwaTest = {
    setSource: (source) => {
      const next = String(source ?? "");
      if (monacoEditor) {
        monacoEditor.setValue(next);
      } else if (sourceEl instanceof HTMLTextAreaElement) {
        sourceEl.value = next;
      }
      vfs.writeFile(currentFile, new TextEncoder().encode(next));
      commandManifestCache = null;
      syncEditorToFile();
      scheduleDiagnostics();
    },
    getSource: () => {
      if (monacoEditor) return monacoEditor.getValue();
      if (sourceEl instanceof HTMLTextAreaElement) return sourceEl.value;
      return "";
    },
    run: async () => {
      await executeRun(false);
    },
    setFile: (path, source) => {
      const filePath = normalizePath(String(path || `${DEMO_ROOT}/main.phpx`));
      const next = String(source ?? "");
      const bytes = new TextEncoder().encode(next);
      vfs.writeFile(filePath, bytes);
      commandManifestCache = null;
      if (shellContainer) {
        void syncPathToShell(filePath, bytes);
      }
    },
    openFile: (path) => {
      const filePath = normalizePath(String(path || `${DEMO_ROOT}/main.phpx`));
      const stat = statPath(filePath);
      if (!stat || stat.fileType !== "file") return false;
      syncFileFromEditor();
      selectFileTab(filePath, { pin: true });
      return true;
    },
  };

  bridgeRef = createPhpHostBridge({
    fs: {
      readFile: (path) => vfs.readFile(path),
      writeFile: (path, data) => vfs.writeFile(path, data),
      readdir: (path) => vfs.readdir(path),
      mkdir: (path, options) => vfs.mkdir(path, options),
      rm: (path, options) => vfs.rm(path, options),
      rename: (from, to) => vfs.rename(from, to),
      stat: (path) => vfs.stat(path),
    },
    target: "adwa",
    projectRoot: "/",
    cwd,
    capabilities: {
      fs: true,
      net: false,
      processEnv: true,
      db: false,
      wasmImports: true,
    },
    stdio: {
      writeStdout: (chunk) => {
        phpStdoutBuffer += new TextDecoder().decode(chunk);
      },
      writeStderr: (chunk) => {
        phpStderrBuffer += new TextDecoder().decode(chunk);
      },
      readStdin: () => null,
    },
  });

  syncEditorToFile();
  setPrompt();
  setPreviewStatus("run mode");
  setPreviewPath("/");
  initSplitters();
  scheduleDiagnostics();

  if (runBtn instanceof HTMLButtonElement) {
    runBtn.addEventListener("click", async () => {
      await executeRun(false);
    });
  }

  if (treeBtn instanceof HTMLButtonElement) {
    treeBtn.addEventListener("click", () => toggleFileTree());
  }

  if (newFileBtn instanceof HTMLButtonElement) {
    newFileBtn.addEventListener("click", async () => {
      const seed = dirname(currentFile || `${DEMO_ROOT}/main.phpx`);
      const suggested = `${seed === "/" ? "" : seed}/untitled.phpx`;
      const input = window.prompt("New file path", suggested);
      if (input == null) return;
      await createFileAt(input);
    });
  }

  if (newFolderBtn instanceof HTMLButtonElement) {
    newFolderBtn.addEventListener("click", async () => {
      const input = window.prompt("New folder path", cwd || "/");
      if (input == null) return;
      await createFolderAt(input);
    });
  }

  const previewNavigate = async () => {
    const raw = previewInputEl instanceof HTMLInputElement ? previewInputEl.value : "/";
    const next = normalizeServePath(raw || "/");
    if (!serverState.running) {
      log("preview navigation requires `deka serve`");
      return;
    }
    await runServedPath(next, { pushHistory: true });
  };

  if (previewGoEl instanceof HTMLButtonElement) {
    previewGoEl.addEventListener("click", async () => {
      await previewNavigate();
    });
  }

  if (previewInputEl instanceof HTMLInputElement) {
    previewInputEl.addEventListener("keydown", async (event) => {
      if (event.key !== "Enter") return;
      event.preventDefault();
      await previewNavigate();
    });
  }

  window.addEventListener("popstate", async (event) => {
    const path = event?.state?.previewPath || readPreviewPathFromLocation();
    if (!path || !serverState.running) return;
    await runServedPath(path, { pushHistory: false });
  });

  if (termFormEl instanceof HTMLFormElement && termInputEl instanceof HTMLInputElement) {
    termFormEl.addEventListener("submit", async (event) => {
      event.preventDefault();
      const value = termInputEl.value;
      termInputEl.value = "";
      if (value.trim()) commandHistory.push(value.trim());
      historyIndex = commandHistory.length;
      await runShellCommand(value);
    });

    termInputEl.addEventListener("keydown", (event) => {
      if (termInputEl.disabled) return;
      if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
      if (!commandHistory.length) return;
      event.preventDefault();
      if (event.key === "ArrowUp") {
        historyIndex = Math.max(0, historyIndex - 1);
      } else {
        historyIndex = Math.min(commandHistory.length, historyIndex + 1);
      }
      if (historyIndex >= commandHistory.length) {
        termInputEl.value = "";
      } else {
        termInputEl.value = commandHistory[historyIndex] ?? "";
      }
      termInputEl.setSelectionRange(termInputEl.value.length, termInputEl.value.length);
    });
  }

  if (helpModalEl instanceof HTMLElement) {
    helpModalEl.addEventListener("click", (event) => {
      if (event.target === helpModalEl) setHelpOpen(false);
    });
  }

  if (sourceEl instanceof HTMLTextAreaElement) {
    sourceEl.addEventListener("input", () => {
      scheduleDiagnostics();
      scheduleAutoRun();
    });
  }

  document.addEventListener("keydown", (event) => {
    const ctrlLike = isMacLike() ? event.metaKey : event.ctrlKey;
    const target = event.target;
    const typingInInput =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      (target instanceof HTMLElement && target.isContentEditable);

    if (serverState.running && event.key.toLowerCase() === "c" && ctrlLike) {
      event.preventDefault();
      log("^C");
      void stopVirtualServer({ interrupted: true });
      return;
    }

    if (event.key === "Escape") {
      showFileTree(false);
      setHelpOpen(false);
      return;
    }

    if (!ctrlLike || event.altKey || event.shiftKey) return;
    if (typingInInput && target === termInputEl) return;

    const key = event.key.toLowerCase();
    if (key === "b") {
      event.preventDefault();
      toggleFileTree();
      return;
    }
    if (key === "j") {
      event.preventDefault();
      toggleTerminal();
      return;
    }
    if (key === "k") {
      event.preventDefault();
      focusEditor();
      return;
    }
    if (key === "h") {
      event.preventDefault();
      toggleHelp();
    }
  });

  const shouldAutoServe = bundledServeConfig && Object.keys(bundledServeConfig).length > 0;
  if (shouldAutoServe) {
    const port = Number.isFinite(configuredServePort) && configuredServePort > 0
      ? Math.floor(configuredServePort)
      : 8530;
    try {
      const cmd = `deka serve ${configuredServeEntry} --port ${port} --mode ${configuredServeMode}`;
      await runShellCommand(cmd);
    } catch (err) {
      log(
        `auto-serve failed, falling back to run mode: ${
          err instanceof Error ? err.message : String(err)
        }`
      );
      await executeRun(false);
    }
  } else {
    await executeRun(false);
  }
};

boot().catch((err) => {
  resetLog("Boot failed.");
  log(err instanceof Error ? err.message : String(err));
  if (phpStderrBuffer.trim()) {
    log(`[stderr]\n${phpStderrBuffer.trimEnd()}`);
  }
});
