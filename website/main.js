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
let termBlockedLineEl = document.getElementById("termBlockedLine");
let termTabsEl = document.getElementById("termTabs");
let termNewTabBtn = document.getElementById("termNewTabBtn");
let termMinimizeBtn = document.getElementById("termMinimizeBtn");
let termStdinFormEl = document.getElementById("termStdinForm");
let termStdinInputEl = document.getElementById("termStdinInput");
let rightPaneEl = document.getElementById("rightPane");
let terminalPaneEl = document.getElementById("terminalPane");
let splitXEl = document.getElementById("splitX");
let splitExplorerEl = document.getElementById("splitExplorer");
let splitYEl = document.getElementById("splitY");
let helpModalEl = document.getElementById("helpModal");
let dataBtnEl = document.getElementById("dataBtn");
let dataModalEl = document.getElementById("dataModal");
let dataCloseBtnEl = document.getElementById("dataCloseBtn");
let settingsBtnEl = document.getElementById("settingsBtn");
let settingsModalEl = document.getElementById("settingsModal");
let settingsCloseBtnEl = document.getElementById("settingsCloseBtn");
let settingsSummaryEl = document.getElementById("settingsSummary");
let settingsListEl = document.getElementById("settingsList");
let settingsRefreshBtnEl = document.getElementById("settingsRefreshBtn");
let settingsResetDbBtnEl = document.getElementById("settingsResetDbBtn");
let settingsResetFsBtnEl = document.getElementById("settingsResetFsBtn");
let settingsResetAllBtnEl = document.getElementById("settingsResetAllBtn");
let dbNameInputEl = document.getElementById("dbNameInput");
let dbTableInputEl = document.getElementById("dbTableInput");
let dbIdInputEl = document.getElementById("dbIdInput");
let dbNameValueInputEl = document.getElementById("dbNameValueInput");
let dbVersionInputEl = document.getElementById("dbVersionInput");
let dbSqlInputEl = document.getElementById("dbSqlInput");
let dbCreateBtnEl = document.getElementById("dbCreateBtn");
let dbEnsureBtnEl = document.getElementById("dbEnsureBtn");
let dbRefreshBtnEl = document.getElementById("dbRefreshBtn");
let dbInsertBtnEl = document.getElementById("dbInsertBtn");
let dbRunSqlBtnEl = document.getElementById("dbRunSqlBtn");
let dbUpdateBtnEl = document.getElementById("dbUpdateBtn");
let dbDeleteBtnEl = document.getElementById("dbDeleteBtn");
let dbClearBtnEl = document.getElementById("dbClearBtn");
let dbStatusEl = document.getElementById("dbStatus");
let dbColsRowEl = document.getElementById("dbColsRow");
let dbRowsBodyEl = document.getElementById("dbRowsBody");

let monacoEditor = null;
let monacoApi = null;
let bridgeRef = null;
let lspWasm = null;
let dekaPhpRuntimeReady = false;
let phpStdoutBuffer = "";
let phpStderrBuffer = "";
let phpRuntimeWasmBytesCache = null;
let shellContainer = null;
let foregroundProcess = null;
let terminalVisible = true;
let terminalMinimized = false;
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
const DB_MIRROR_DIR = `${DEMO_ROOT}/.db`;
const DEFAULT_CWD = DEMO_ROOT;
const BASE_FS_DIRS = [
  "/bin",
  "/usr",
  "/usr/bin",
  "/home",
  "/home/user",
  "/home/user/demo",
  "/home/user/demo/app",
  "/home/user/demo/.db",
  "/php_modules",
  "/__global",
  "/__global/php_modules",
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
  foregroundTerminalId: null,
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
    const key = String(rawPath || "").trim();
    if (!key) continue;
    if (key.startsWith("/php_modules/") || key.startsWith("/__global/php_modules/")) {
      const normalized = normalizePath(key);
      out[normalized] = content;
      // Mirror module trees under the demo root so the explorer and
      // project-local runtime checks can resolve php_modules as expected.
      out[normalizePath(`${root}${normalized}`)] = content;
      continue;
    }
    const rel = key.replace(/^\/+/, "");
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
  [`/main.phpx`]: defaultSource,
  [DEMO_ENTRY]: defaultSource,
});
for (const dir of BASE_FS_DIRS) {
  vfs.mkdir(dir);
}


const EXPANDED_FOLDERS_STORAGE_KEY = "adwa.explorer.expanded.v1";
const VFS_SNAPSHOT_KEY = "adwa.vfs.snapshot.v1";




const bytesToBase64 = (bytes) => {
  let bin = "";
  for (let i = 0; i < bytes.length; i += 1) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
};

const base64ToBytes = (encoded) => {
  const bin = atob(String(encoded || ""));
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
};


const loadVfsSnapshot = () => {
  try {
    const raw = localStorage.getItem(VFS_SNAPSHOT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return null;
    const dirs = Array.isArray(parsed.dirs) ? parsed.dirs.filter((d) => typeof d === "string") : [];
    const files = parsed.files && typeof parsed.files === "object" ? parsed.files : {};
    return { dirs, files };
  } catch {
    return null;
  }
};

const applyVfsSnapshot = (snap) => {
  if (!snap) return false;
  const dirs = new Set(["/"]);
  for (const d of snap.dirs || []) {
    if (typeof d === "string" && d.startsWith("/")) dirs.add(normalizePath(d));
  }
  const files = new Map();
  for (const [path, encoded] of Object.entries(snap.files || {})) {
    if (typeof path !== "string" || !path.startsWith("/")) continue;
    if (typeof encoded !== "string") continue;
    const normalized = normalizePath(path);
    let decoded;
    try {
      decoded = base64ToBytes(encoded);
    } catch {
      decoded = new TextEncoder().encode(String(encoded));
    }
    files.set(normalized, decoded);
    const parent = dirname(normalized);
    let current = "";
    for (const part of parent.split("/").filter(Boolean)) {
      current = current + "/" + part;
      dirs.add(current || "/");
    }
  }
  vfs.files = files;
  vfs.dirs = dirs;
  return true;
};

const ensureBundledProjectSeed = () => {
  const pathExists = (path) => {
    try {
      vfs.stat(path);
      return true;
    } catch {
      return false;
    }
  };
  const mergedFiles = {
    ...BASE_FS_FILES,
    ...bundledProjectTree,
    "/README.txt": "ADWA browser playground\\nUse terminal: ls, cd, pwd, open, cat, run\\n",
    "/main.phpx": defaultSource,
    [DEMO_ENTRY]: defaultSource,
  };
  for (const dir of BASE_FS_DIRS) {
    if (!pathExists(dir)) vfs.mkdir(dir);
  }
  for (const [path, content] of Object.entries(mergedFiles)) {
    if (!pathExists(path)) {
      vfs.writeFile(path, new TextEncoder().encode(String(content)));
    }
  }
  // Force-refresh lock/config from bundled project to avoid stale browser
  // snapshots causing lock drift errors after runtime/module updates.
  for (const pinned of ["/deka.lock", `${DEMO_ROOT}/deka.lock`, "/deka.json", `${DEMO_ROOT}/deka.json`]) {
    if (Object.prototype.hasOwnProperty.call(mergedFiles, pinned)) {
      vfs.writeFile(pinned, new TextEncoder().encode(String(mergedFiles[pinned])));
    }
  }
  // Older snapshots may not include the mirrored local module root.
  const localModuleDir = normalizePath(`${DEMO_ROOT}/php_modules`);
  if (!pathExists(localModuleDir) && pathExists("/php_modules")) {
    vfs.mkdir(localModuleDir);
  }
};

const saveVfsSnapshot = () => {
  try {
    const files = {};
    for (const filePath of vfs.listFiles()) {
      files[filePath] = bytesToBase64(vfs.readFile(filePath));
    }
    const payload = {
      dirs: vfs.listDirs(),
      files,
    };
    localStorage.setItem(VFS_SNAPSHOT_KEY, JSON.stringify(payload));
  } catch {}
};

const wrapVfsPersistence = () => {
  let timer = null;
  const schedule = () => {
    if (timer != null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      saveVfsSnapshot();
    }, 120);
  };
  const bindWrap = (name) => {
    const original = vfs[name].bind(vfs);
    vfs[name] = (...args) => {
      const out = original(...args);
      schedule();
      return out;
    };
  };
  bindWrap("writeFile");
  bindWrap("mkdir");
  bindWrap("rm");
  bindWrap("rename");
  window.addEventListener("beforeunload", () => {
    if (timer != null) clearTimeout(timer);
    saveVfsSnapshot();
  });
};

applyVfsSnapshot(loadVfsSnapshot());
ensureBundledProjectSeed();
wrapVfsPersistence();

let currentFile = configuredServeEntry;
let cwd = DEFAULT_CWD;
let explorerRoot = DEFAULT_CWD;
const openTabs = [{ path: currentFile, pinned: true }];

const createTerminalSession = (cwdPath, index = 1) => ({
  id: `t${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
  label: `Terminal ${index}`,
  cwd: normalizePath(cwdPath || DEFAULT_CWD),
  history: [],
  historyIndex: 0,
  logText: "",
});

let terminalSessions = [createTerminalSession(DEFAULT_CWD, 1)];
let activeTerminalSessionId = terminalSessions[0].id;
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

const getActiveTerminalSession = () => {
  const found = terminalSessions.find((tab) => tab.id === activeTerminalSessionId);
  return found || terminalSessions[0] || null;
};

const isActiveTerminalBlocked = () => {
  if (!foregroundProcess) return false;
  if (!serverState.foregroundTerminalId) return true;
  return serverState.foregroundTerminalId === activeTerminalSessionId;
};

const syncActiveTerminalSessionState = () => {
  const active = getActiveTerminalSession();
  if (!active) return;
  active.cwd = normalizePath(cwd || DEFAULT_CWD);
  active.history = [...commandHistory];
  active.historyIndex = historyIndex;
  if (logEl) active.logText = logEl.textContent || "";
};

const applyTerminalSession = (tabId) => {
  syncActiveTerminalSessionState();
  const next = terminalSessions.find((tab) => tab.id === tabId);
  if (!next) return;
  activeTerminalSessionId = next.id;
  cwd = normalizePath(next.cwd || DEFAULT_CWD);
  projectEnv.PWD = cwd;
  commandHistory.length = 0;
  commandHistory.push(...(next.history || []));
  historyIndex = Number.isFinite(next.historyIndex) ? next.historyIndex : commandHistory.length;
  if (logEl) logEl.textContent = next.logText || "";
  setPrompt();
  setTerminalBlocked(isActiveTerminalBlocked(), false);
  renderTerminalTabs();
};

const addTerminalSession = (cwdPath) => {
  syncActiveTerminalSessionState();
  const nextIndex = terminalSessions.length + 1;
  const tab = createTerminalSession(cwdPath || cwd || DEFAULT_CWD, nextIndex);
  terminalSessions.push(tab);
  applyTerminalSession(tab.id);
};

const closeTerminalSession = (tabId) => {
  if (terminalSessions.length <= 1) return;
  syncActiveTerminalSessionState();
  const idx = terminalSessions.findIndex((tab) => tab.id === tabId);
  if (idx < 0) return;
  const wasActive = terminalSessions[idx].id === activeTerminalSessionId;
  terminalSessions.splice(idx, 1);
  const removedOwnedForeground = Boolean(foregroundProcess) && serverState.foregroundTerminalId === tabId;
  if (wasActive) {
    const fallback = terminalSessions[Math.max(0, idx - 1)] || terminalSessions[0];
    if (fallback) {
      if (removedOwnedForeground) serverState.foregroundTerminalId = fallback.id;
      applyTerminalSession(fallback.id);
    }
  } else if (removedOwnedForeground) {
    serverState.foregroundTerminalId = terminalSessions[0]?.id || null;
  }
  renderTerminalTabs();
};

const renderTerminalTabs = () => {
  if (!(termTabsEl instanceof HTMLElement)) return;
  termTabsEl.innerHTML = "";
  terminalSessions.forEach((tab) => {
    const wrap = document.createElement("div");
    wrap.className = `termTab${tab.id === activeTerminalSessionId ? " active" : ""}`;
    wrap.title = tab.cwd;

    const openBtn = document.createElement("button");
    openBtn.type = "button";
    openBtn.className = "termTabOpen";
    openBtn.textContent = tab.label;
    openBtn.addEventListener("click", () => applyTerminalSession(tab.id));

    const closeBtn = document.createElement("button");
    closeBtn.type = "button";
    closeBtn.className = "termTabClose";
    closeBtn.textContent = "x";
    closeBtn.setAttribute("aria-label", `Close ${tab.label}`);
    closeBtn.disabled = terminalSessions.length <= 1;
    closeBtn.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      closeTerminalSession(tab.id);
    });

    wrap.appendChild(openBtn);
    wrap.appendChild(closeBtn);
    termTabsEl.appendChild(wrap);
  });
};

renderTerminalTabs();

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
  syncActiveTerminalSessionState();
};

const setPrompt = () => {
  if (!termPromptEl) return;
  if (termInputEl instanceof HTMLInputElement && termInputEl.disabled) {
    termPromptEl.textContent = "";
    return;
  }
  termPromptEl.textContent = `${cwd} $`;
  const active = getActiveTerminalSession();
  if (active) active.cwd = normalizePath(cwd || DEFAULT_CWD);
};

const setTerminalBlocked = (blocked, allowStdin = false) => {
  const isBlocked = Boolean(blocked);
  const showStdin = isBlocked && Boolean(allowStdin);
  const showBlockedLine = isBlocked && !showStdin;

  if (termInputEl instanceof HTMLInputElement) {
    termInputEl.disabled = isBlocked;
    termInputEl.classList.toggle("blocked", isBlocked);
  }
  if (termFormEl instanceof HTMLFormElement) {
    termFormEl.classList.toggle("hidden", isBlocked);
    termFormEl.hidden = isBlocked;
  }
  if (termStdinFormEl instanceof HTMLFormElement) {
    termStdinFormEl.classList.toggle("hidden", !showStdin);
    termStdinFormEl.hidden = !showStdin;
  }
  if (termBlockedLineEl instanceof HTMLElement) {
    termBlockedLineEl.classList.toggle("hidden", !showBlockedLine);
    termBlockedLineEl.hidden = !showBlockedLine;
  }
  if (termStdinInputEl instanceof HTMLInputElement) {
    termStdinInputEl.disabled = !showStdin;
  }
  setPrompt();
  setTimeout(() => {
    if (showStdin && termStdinInputEl instanceof HTMLInputElement) {
      termStdinInputEl.focus();
      termStdinInputEl.setSelectionRange(termStdinInputEl.value.length, termStdinInputEl.value.length);
      return;
    }
    if (!isBlocked && termInputEl instanceof HTMLInputElement) {
      termInputEl.focus();
      termInputEl.setSelectionRange(termInputEl.value.length, termInputEl.value.length);
    }
  }, 0);
};

const setTerminalMinimized = (minimized) => {
  terminalMinimized = Boolean(minimized);
  if (rightPaneEl instanceof HTMLElement) {
    rightPaneEl.classList.toggle("terminalMinimized", terminalMinimized);
  }
  if (termMinimizeBtn instanceof HTMLButtonElement) {
    termMinimizeBtn.textContent = terminalMinimized ? "▴" : "▾";
    termMinimizeBtn.title = terminalMinimized ? "Expand terminal" : "Minimize terminal";
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
  setPreviewStatus("run mode");
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
    const mod = await import("./vendor/adwa_editor/phpx_lsp_wasm.js");
    const wasmUrl = new URL("./vendor/adwa_editor/phpx_lsp_wasm_bg.wasm", import.meta.url).toString();
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
  const desktop = window.innerWidth > 960;
  appShellEl.classList.toggle("explorerCollapsed", !visible);

  if (desktop) {
    if (visible) {
      appShellEl.style.gridTemplateColumns = `56px ${Math.round(explorerWidth)}px 6px 1fr`;
    } else {
      appShellEl.style.gridTemplateColumns = "56px 0 0 1fr";
    }
    return;
  }

  appShellEl.style.gridTemplateColumns = "56px 1fr";
};

const toggleFileTree = () => {
  if (!(appShellEl instanceof HTMLElement)) return;
  const nextVisible = appShellEl.classList.contains("explorerCollapsed");
  showFileTree(nextVisible);
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

const setDataOpen = (open) => {
  if (!(dataModalEl instanceof HTMLElement)) return;
  dataModalEl.classList.toggle("open", Boolean(open));
};

const toggleData = () => {
  if (!(dataModalEl instanceof HTMLElement)) return;
  setDataOpen(!dataModalEl.classList.contains("open"));
};

const setSettingsOpen = (open) => {
  if (!(settingsModalEl instanceof HTMLElement)) return;
  settingsModalEl.classList.toggle("open", Boolean(open));
};

const settingsSetInfo = (summary, lines = []) => {
  if (settingsSummaryEl instanceof HTMLElement) settingsSummaryEl.textContent = String(summary || "");
  if (settingsListEl instanceof HTMLElement) {
    settingsListEl.innerHTML = "";
    for (const line of lines) {
      const li = document.createElement("li");
      li.textContent = String(line);
      settingsListEl.appendChild(li);
    }
  }
};

const refreshSettings = () => {
  const dbNames = (() => {
    try { return dbListDatabases(); } catch { return []; }
  })();
  const snapshotRaw = localStorage.getItem(VFS_SNAPSHOT_KEY);
  const snapshotBytes = snapshotRaw ? snapshotRaw.length : 0;
  const lines = [
    "databases: " + (dbNames.length ? dbNames.join(", ") : "none"),
    "open db handles: " + String((() => { try { return Number(dbBridgeCall("stats", {}).active_handles || 0); } catch { return 0; } })()),
    "vfs files: " + vfs.listFiles().length,
    "vfs dirs: " + vfs.listDirs().length,
    "vfs snapshot: " + (snapshotRaw ? (snapshotBytes + " bytes") : "none"),
  ];
  settingsSetInfo("Browser-backed runtime state.", lines);
};

const dbInputValue = (el, fallback = "") => {
  if (!(el instanceof HTMLInputElement) && !(el instanceof HTMLTextAreaElement)) return String(fallback || "");
  return String(el.value || fallback || "").trim();
};

const dbSafeIdent = (value, fallback) => {
  const raw = String(value || fallback || "").trim();
  if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(raw)) return raw;
  throw new Error("invalid SQL identifier: " + raw);
};

const dbSetStatus = (message, isError = false) => {
  if (!(dbStatusEl instanceof HTMLElement)) return;
  dbStatusEl.textContent = String(message || "");
  dbStatusEl.classList.toggle("error", Boolean(isError));
};

const dbEscape = (value) =>
  String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

const dbRenderRows = (rows) => {
  if (!(dbRowsBodyEl instanceof HTMLElement)) return;
  dbRowsBodyEl.innerHTML = "";
  const list = Array.isArray(rows) ? rows : [];
  const cols = list.length
    ? Array.from(list.reduce((set, row) => {
        Object.keys(row || {}).forEach((k) => set.add(k));
        return set;
      }, new Set()))
    : ["id", "name", "version"];

  if (dbColsRowEl instanceof HTMLElement) {
    dbColsRowEl.innerHTML = cols.map((col) => "<th>" + dbEscape(col) + "</th>").join("");
  }

  if (!list.length) {
    dbRowsBodyEl.innerHTML = "<tr><td colspan=\"" + cols.length + "\" class=\"empty\">no rows</td></tr>";
    return;
  }
  for (const row of list) {
    const tr = document.createElement("tr");
    tr.innerHTML = cols.map((col) => "<td>" + dbEscape(row?.[col]) + "</td>").join("");
    dbRowsBodyEl.appendChild(tr);
  }
};

const dbBridgeCall = (action, payload = {}) => {
  if (!bridgeRef || typeof bridgeRef.call !== "function") throw new Error("bridge is not ready");
  const out = bridgeRef.call({ kind: "db", action, payload });
  if (!out || !out.ok) {
    throw new Error(out?.error || ("db " + action + " failed"));
  }
  const value = out.value;
  if (value && typeof value === "object" && value.ok === false) {
    throw new Error(String(value.error || ("db " + action + " error")));
  }
  return value || {};
};

const warmSqliteEngine = () => {
  try {
    const opened = dbBridgeCall("open", {
      driver: "sqlite",
      config: { database: "__adwa_warmup__" },
    });
    if (opened.handle) {
      try { dbBridgeCall("close", { handle: opened.handle }); } catch {}
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (message.includes("initializing")) {
      setTimeout(warmSqliteEngine, 80);
    }
  }
};

const dbListDatabases = () => {
  const result = dbBridgeCall("list", {});
  const names = Array.isArray(result.databases) ? result.databases : [];
  return names.map((name) => String(name));
};

const dbToFileName = (dbName) =>
  String(dbName || "db")
    .replace(/[^A-Za-z0-9._-]+/g, "_")
    .replace(/^_+|_+$/g, "") || "db";

const syncDbMirrorFiles = (dbNames) => {
  const names = Array.isArray(dbNames) ? dbNames : [];
  try {
    vfs.mkdir(DB_MIRROR_DIR, { recursive: true });
  } catch {}

  let existing = [];
  try {
    existing = vfs.readdir(DB_MIRROR_DIR).map((name) => normalizePath(DB_MIRROR_DIR + "/" + name));
  } catch {
    existing = [];
  }

  const desired = new Set(names.map((name) => normalizePath(DB_MIRROR_DIR + "/" + dbToFileName(name) + ".sqlite")));

  for (const file of existing) {
    if (!desired.has(file)) {
      try { vfs.rm(file, { recursive: false, force: true }); } catch {}
    }
  }

  for (const file of desired) {
    if (!statPath(file)) {
      const body = "-- ADWA sqlite mirror placeholder\n-- real storage: browser localStorage\n";
      vfs.writeFile(file, new TextEncoder().encode(body));
    }
  }
};

const withDbHandle = (fn) => {
  const dbName = dbInputValue(dbNameInputEl, "adwa_demo") || "adwa_demo";
  const opened = dbBridgeCall("open", {
    driver: "sqlite",
    config: { database: dbName },
  });
  const handle = opened.handle;
  if (!handle) throw new Error("db open returned no handle");
  try {
    return fn(handle);
  } finally {
    try { dbBridgeCall("close", { handle }); } catch {}
  }
};

const dbCreateDatabase = () => {
  const dbName = dbInputValue(dbNameInputEl, "adwa_demo") || "adwa_demo";
  const opened = dbBridgeCall("open", {
    driver: "sqlite",
    config: { database: dbName },
  });
  if (opened.handle) {
    try { dbBridgeCall("close", { handle: opened.handle }); } catch {}
  }
  dbSetStatus("database " + dbName + " created");
  const names = dbListDatabases();
  syncDbMirrorFiles(names);
  renderFileTree();
  dbInitModal();
};

const dbEnsureTable = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  withDbHandle((handle) => {
    dbBridgeCall("exec", {
      handle,
      sql: "create table if not exists " + table + " (id int, name text, version text)",
      params: [],
    });
  });
  dbSetStatus("table " + table + " ready");
};

const dbRefreshRows = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  let rows = [];
  try {
    withDbHandle((handle) => {
      const result = dbBridgeCall("query", {
        handle,
        sql: "select id, name, version from " + table + " order by id asc limit 200",
        params: [],
      });
      rows = Array.isArray(result.rows) ? result.rows : [];
    });
    dbRenderRows(rows);
    dbSetStatus("loaded " + rows.length + " row(s) from " + table);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (message.toLowerCase().includes("no such table")) {
      dbRenderRows([]);
      dbSetStatus("table " + table + " not found. Click Ensure Table.");
      return;
    }
    throw err;
  }
};

const dbInsertRow = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  const idRaw = dbInputValue(dbIdInputEl, "");
  const id = idRaw === "" ? null : Number(idRaw);
  const name = dbInputValue(dbNameValueInputEl, "item");
  const version = dbInputValue(dbVersionInputEl, "0.1.0");
  withDbHandle((handle) => {
    dbBridgeCall("exec", {
      handle,
      sql: "insert into " + table + " (id, name, version) values (?, ?, ?)",
      params: [Number.isFinite(id) ? id : null, name, version],
    });
  });
  dbSetStatus("inserted row");
  dbRefreshRows();
};

const dbUpdateRow = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  const id = Number(dbInputValue(dbIdInputEl, ""));
  if (!Number.isFinite(id)) throw new Error("ID is required for update");
  const name = dbInputValue(dbNameValueInputEl, "item");
  const version = dbInputValue(dbVersionInputEl, "0.1.0");
  withDbHandle((handle) => {
    dbBridgeCall("exec", {
      handle,
      sql: "update " + table + " set name = ?, version = ? where id = ?",
      params: [name, version, id],
    });
  });
  dbSetStatus("updated id=" + id);
  dbRefreshRows();
};

const dbDeleteRow = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  const id = Number(dbInputValue(dbIdInputEl, ""));
  if (!Number.isFinite(id)) throw new Error("ID is required for delete");
  withDbHandle((handle) => {
    dbBridgeCall("exec", {
      handle,
      sql: "delete from " + table + " where id = ?",
      params: [id],
    });
  });
  dbSetStatus("deleted id=" + id);
  dbRefreshRows();
};

const dbClearRows = () => {
  const table = dbSafeIdent(dbInputValue(dbTableInputEl, "packages"), "packages");
  withDbHandle((handle) => {
    dbBridgeCall("exec", {
      handle,
      sql: "delete from " + table,
      params: [],
    });
  });
  dbSetStatus("cleared " + table);
  dbRefreshRows();
};

const dbRunSql = () => {
  const sql = dbInputValue(dbSqlInputEl, "");
  if (!sql) throw new Error("SQL is required");
  let rows = [];
  let affectedRows = 0;
  withDbHandle((handle) => {
    if (/^\s*select\b/i.test(sql)) {
      const result = dbBridgeCall("query", { handle, sql, params: [] });
      rows = Array.isArray(result.rows) ? result.rows : [];
      return;
    }
    const execResult = dbBridgeCall("exec", { handle, sql, params: [] });
    affectedRows = Number(execResult.affected_rows || 0);
  });
  if (/^\s*select\b/i.test(sql)) {
    dbRenderRows(rows);
    dbSetStatus("query returned " + rows.length + " row(s)");
  } else {
    dbSetStatus("statement applied; affected_rows=" + affectedRows);
    try { dbRefreshRows(); } catch {}
  }
};

const dbInitModal = () => {
const resetFsSnapshot = () => {
  localStorage.removeItem(VFS_SNAPSHOT_KEY);
  location.reload();
};

const resetAllBrowserState = () => {
  try { dbBridgeCall("reset", {}); } catch {}
  localStorage.removeItem(VFS_SNAPSHOT_KEY);
  location.reload();
};

  const names = dbListDatabases();
  syncDbMirrorFiles(names);
  renderFileTree();
  const dbName = dbInputValue(dbNameInputEl, "adwa_demo") || "adwa_demo";
  if (!names.length) {
    dbRenderRows([]);
    dbSetStatus("no databases found. Click Create DB to start.");
    return;
  }
  if (!names.includes(dbName) && dbNameInputEl instanceof HTMLInputElement) {
    dbNameInputEl.value = names[0];
  }
  try {
    dbRefreshRows();
  } catch (err) {
    dbSetStatus(err instanceof Error ? err.message : String(err), true);
  }
};

const runDbAction = (action) => {
  try {
    action();
  } catch (err) {
    dbSetStatus(err instanceof Error ? err.message : String(err), true);
  }
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

const sha256Hex = (input) => {
  const K = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
    0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
    0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
    0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
    0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
    0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
  ];
  const rotr = (x, n) => ((x >>> n) | (x << (32 - n))) >>> 0;
  const bytes = Array.from(new TextEncoder().encode(String(input ?? "")));
  const bitLen = bytes.length * 8;
  bytes.push(0x80);
  while ((bytes.length % 64) !== 56) bytes.push(0);
  for (let i = 7; i >= 0; i -= 1) bytes.push((bitLen >>> (i * 8)) & 0xff);

  let h0 = 0x6a09e667;
  let h1 = 0xbb67ae85;
  let h2 = 0x3c6ef372;
  let h3 = 0xa54ff53a;
  let h4 = 0x510e527f;
  let h5 = 0x9b05688c;
  let h6 = 0x1f83d9ab;
  let h7 = 0x5be0cd19;

  for (let i = 0; i < bytes.length; i += 64) {
    const w = new Array(64);
    for (let j = 0; j < 16; j += 1) {
      const k = i + (j * 4);
      w[j] = ((bytes[k] << 24) | (bytes[k + 1] << 16) | (bytes[k + 2] << 8) | bytes[k + 3]) >>> 0;
    }
    for (let j = 16; j < 64; j += 1) {
      const s0 = (rotr(w[j - 15], 7) ^ rotr(w[j - 15], 18) ^ (w[j - 15] >>> 3)) >>> 0;
      const s1 = (rotr(w[j - 2], 17) ^ rotr(w[j - 2], 19) ^ (w[j - 2] >>> 10)) >>> 0;
      w[j] = (((w[j - 16] + s0) >>> 0) + ((w[j - 7] + s1) >>> 0)) >>> 0;
    }
    let a = h0, b = h1, c = h2, d = h3, e = h4, f = h5, g = h6, h = h7;
    for (let j = 0; j < 64; j += 1) {
      const s1 = (rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)) >>> 0;
      const ch = ((e & f) ^ ((~e) & g)) >>> 0;
      const t1 = (((((h + s1) >>> 0) + ch) >>> 0) + ((K[j] + w[j]) >>> 0)) >>> 0;
      const s0 = (rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)) >>> 0;
      const maj = ((a & b) ^ (a & c) ^ (b & c)) >>> 0;
      const t2 = (s0 + maj) >>> 0;
      h = g; g = f; f = e; e = (d + t1) >>> 0; d = c; c = b; b = a; a = (t1 + t2) >>> 0;
    }
    h0 = (h0 + a) >>> 0;
    h1 = (h1 + b) >>> 0;
    h2 = (h2 + c) >>> 0;
    h3 = (h3 + d) >>> 0;
    h4 = (h4 + e) >>> 0;
    h5 = (h5 + f) >>> 0;
    h6 = (h6 + g) >>> 0;
    h7 = (h7 + h) >>> 0;
  }
  return [h0, h1, h2, h3, h4, h5, h6, h7]
    .map((word) => (word >>> 0).toString(16).padStart(8, "0"))
    .join("");
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
        op_php_sha256: (source) => sha256Hex(String(source || "")),
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
  const nextPath = normalizeServePath(path || serverState.path || "/");
  serverState.path = nextPath;
  setPreviewPath(nextPath);
  setServePreviewStatus(nextPath);
  if (opts.pushHistory !== false) {
    history.pushState({ previewPath: nextPath }, "", buildPreviewHistoryUrl(nextPath));
  }
  if (!serverState.entry) return;
  const result = await runPhpxEntry(serverState.entry, nextPath);
  writeRunResult(result);
};

const interruptForegroundProcess = async (opts = {}) => {
  if (!foregroundProcess) return;
  const interrupted = Boolean(opts.interrupted);
  const proc = foregroundProcess;
  if (shellContainer && typeof shellContainer.signalForeground === "function") {
    try {
      await shellContainer.signalForeground(interrupted ? 2 : 15);
    } catch {}
  }

  // Fallback teardown when host runtime does not emit port-closed promptly.
  // Keeps terminal/process UX unblocked under Ctrl+C semantics.
  if (foregroundProcess === proc) {
    foregroundProcess = null;
    serverState.foregroundTerminalId = null;
    if (runtimePortState.url) {
      runtimePortState.url = "";
      runtimePortState.port = 0;
    }
    serverState.running = false;
    setPreviewStatus("run mode");
    setTerminalBlocked(false, false);
    setPrompt();
  }
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
    if (!runtimePortState.url) return;
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
    if (runtimePortState.url) {
      await runServedPath(serverState.path || "/", { pushHistory: false, reload: true });
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
    const finalize = async (status) => {
      const early = await readProcessOutput(proc);
      const late = await readProcessOutputAfterExit(proc);
      const stdout = `${early.stdout || ""}${late.stdout || ""}`;
      const stderr = `${early.stderr || ""}${late.stderr || ""}`;
      if (stdout.trim()) log(stdout.trimEnd());
      if (stderr.trim()) log(`[stderr]
${stderr.trimEnd()}`);
      if (status && status.code !== 0 && !stderr.trim() && !stdout.trim()) {
        log(`[exit ${status.code}]`);
      }
      if (foregroundProcess === proc) {
        foregroundProcess = null;
        if (!runtimePortState.url) {
          serverState.running = false;
          serverState.foregroundTerminalId = null;
          setPreviewStatus("run mode");
        }
        setTerminalBlocked(isActiveTerminalBlocked(), false);
        setPrompt();
      }
      renderFileTree();
    };

    foregroundProcess = proc;
    serverState.foregroundTerminalId = activeTerminalSessionId;
    setTerminalBlocked(isActiveTerminalBlocked(), false);
    setPrompt();

    try {
      const status = await proc.exit();
      await finalize(status);
      return;
    } catch (err) {
      const message = String(err instanceof Error ? err.message : err || "");
      if (!message.toLowerCase().includes("busy") && !message.toLowerCase().includes("still running")) {
        log(`[stderr]
${message}`);
        await finalize({ code: 1 });
        return;
      }
    }

    log("[foreground] process running (Ctrl+C to stop)");
    const monitor = async () => {
      while (true) {
        await new Promise((resolve) => setTimeout(resolve, 120));
        try {
          const status = await proc.exit();
          await finalize(status);
          return;
        } catch (err) {
          const message = String(err instanceof Error ? err.message : err || "").toLowerCase();
          if (message.includes("busy") || message.includes("still running")) {
            continue;
          }
          log(`[stderr]
${String(err instanceof Error ? err.message : err)}`);
          await finalize({ code: 1 });
          return;
        }
      }
    };
    void monitor();
  };

  if (foregroundProcess && isActiveTerminalBlocked()) {
    log("[busy] foreground process is running in this terminal (Ctrl+C to stop)");
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
    if (!shellContainer) {
      log("terminal runtime not ready");
      return;
    }
    const proc = await shellContainer.spawn("deka", rest, { cwd });
    await runProcessAndRender(proc, "deka", rest);
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
      const eventPort = Number(event?.port || 0);
      if (!eventPort) return;
      runtimePortState.port = eventPort;
      runtimePortState.url = `http://localhost:${eventPort}`;
      serverState.running = true;
      serverState.port = eventPort;
      if (!serverState.path) {
        serverState.path = readPreviewPathFromLocation();
      }
      setPreviewPath(serverState.path || "/");
      setServePreviewStatus(serverState.path || "/");
      void runServedPath(serverState.path || "/", { pushHistory: false, reload: true });
    };
    const onPortClosed = (event) => {
      const eventPort = Number(event?.port || 0);
      if (!eventPort) return;
      if (runtimePortState.port && eventPort !== Number(runtimePortState.port)) return;
      runtimePortState.url = "";
      runtimePortState.port = 0;
      serverState.running = false;
      serverState.foregroundTerminalId = null;
      foregroundProcess = null;
      setTerminalBlocked(false, false);
      setPrompt();
      setPreviewStatus("run mode");
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
      db: true,
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
  warmSqliteEngine();

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

  if (termNewTabBtn instanceof HTMLButtonElement) {
    termNewTabBtn.addEventListener("click", () => {
      addTerminalSession(cwd || DEFAULT_CWD);
      renderTerminalTabs();
      setTerminalBlocked(isActiveTerminalBlocked(), false);
    });
  }

  if (termMinimizeBtn instanceof HTMLButtonElement) {
    termMinimizeBtn.addEventListener("click", () => {
      setTerminalMinimized(!terminalMinimized);
    });
  }

  const previewNavigate = async () => {
    const raw = previewInputEl instanceof HTMLInputElement ? previewInputEl.value : "/";
    const next = normalizeServePath(raw || "/");
    if (!runtimePortState.url) {
      log("preview navigation requires an active server endpoint");
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
    if (!path || !runtimePortState.url) return;
    await runServedPath(path, { pushHistory: false });
  });

  if (termFormEl instanceof HTMLFormElement && termInputEl instanceof HTMLInputElement) {
    termFormEl.addEventListener("submit", async (event) => {
      event.preventDefault();
      const value = termInputEl.value;
      termInputEl.value = "";
      if (value.trim()) commandHistory.push(value.trim());
      historyIndex = commandHistory.length;
      syncActiveTerminalSessionState();
      await runShellCommand(value);
      syncActiveTerminalSessionState();
      renderTerminalTabs();
    });

    termInputEl.addEventListener("keydown", (event) => {
      const interruptChord = event.ctrlKey;
      if (interruptChord && ((event.key || "").toLowerCase() === "c" || event.code === "KeyC") && foregroundProcess) {
        event.preventDefault();
        log("^C");
        void interruptForegroundProcess({ interrupted: true });
        return;
      }
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
      syncActiveTerminalSessionState();
    });
  }

  if (termStdinFormEl instanceof HTMLFormElement && termStdinInputEl instanceof HTMLInputElement) {
    termStdinFormEl.addEventListener("submit", async (event) => {
      event.preventDefault();
      const value = termStdinInputEl.value;
      termStdinInputEl.value = "";
      if (!value.trim()) return;
      if (foregroundProcess && typeof foregroundProcess.write === "function") {
        try {
          await foregroundProcess.write(`${value}
`);
        } catch {}
      }
      log(value);
      syncActiveTerminalSessionState();
    });

    termStdinInputEl.addEventListener("keydown", (event) => {
      const interruptChord = event.ctrlKey;
      if (interruptChord && ((event.key || "").toLowerCase() === "c" || event.code === "KeyC") && foregroundProcess) {
        event.preventDefault();
        log("^C");
        void interruptForegroundProcess({ interrupted: true });
      }
    });
  }

  setTerminalBlocked(isActiveTerminalBlocked(), false);
  setTerminalMinimized(false);
  renderTerminalTabs();

  if (helpModalEl instanceof HTMLElement) {
    helpModalEl.addEventListener("click", (event) => {
      if (event.target === helpModalEl) setHelpOpen(false);
    });
  }

  if (dataBtnEl instanceof HTMLButtonElement) {
    dataBtnEl.addEventListener("click", () => {
      const wasOpen = dataModalEl instanceof HTMLElement && dataModalEl.classList.contains("open");
      toggleData();
      if (!wasOpen) runDbAction(dbInitModal);
    });
  }
  if (dataCloseBtnEl instanceof HTMLButtonElement) {
    dataCloseBtnEl.addEventListener("click", () => setDataOpen(false));
  }
  if (dataModalEl instanceof HTMLElement) {
    dataModalEl.addEventListener("click", (event) => {
      if (event.target === dataModalEl) setDataOpen(false);
    });
  }
  if (settingsBtnEl instanceof HTMLButtonElement) {
    settingsBtnEl.addEventListener("click", () => {
      setSettingsOpen(true);
      refreshSettings();
    });
  }
  if (settingsCloseBtnEl instanceof HTMLButtonElement) {
    settingsCloseBtnEl.addEventListener("click", () => setSettingsOpen(false));
  }
  if (settingsModalEl instanceof HTMLElement) {
    settingsModalEl.addEventListener("click", (event) => {
      if (event.target === settingsModalEl) setSettingsOpen(false);
    });
  }
  if (settingsRefreshBtnEl instanceof HTMLButtonElement) settingsRefreshBtnEl.addEventListener("click", refreshSettings);
  if (settingsResetDbBtnEl instanceof HTMLButtonElement) {
    settingsResetDbBtnEl.addEventListener("click", () => {
      runDbAction(() => {
        dbBridgeCall("reset", {});
        syncDbMirrorFiles([]);
        renderFileTree();
        refreshSettings();
        dbSetStatus("all databases reset");
      });
    });
  }
  if (settingsResetFsBtnEl instanceof HTMLButtonElement) {
    settingsResetFsBtnEl.addEventListener("click", () => {
      if (!window.confirm("Reset filesystem snapshot and reload?")) return;
      resetFsSnapshot();
    });
  }
  if (settingsResetAllBtnEl instanceof HTMLButtonElement) {
    settingsResetAllBtnEl.addEventListener("click", () => {
      if (!window.confirm("Reset all browser runtime state and reload?")) return;
      resetAllBrowserState();
    });
  }
  if (dbCreateBtnEl instanceof HTMLButtonElement) dbCreateBtnEl.addEventListener("click", () => runDbAction(dbCreateDatabase));
  if (dbEnsureBtnEl instanceof HTMLButtonElement) dbEnsureBtnEl.addEventListener("click", () => runDbAction(dbEnsureTable));
  if (dbRefreshBtnEl instanceof HTMLButtonElement) dbRefreshBtnEl.addEventListener("click", () => runDbAction(dbRefreshRows));
  if (dbInsertBtnEl instanceof HTMLButtonElement) dbInsertBtnEl.addEventListener("click", () => runDbAction(dbInsertRow));
  if (dbUpdateBtnEl instanceof HTMLButtonElement) dbUpdateBtnEl.addEventListener("click", () => runDbAction(dbUpdateRow));
  if (dbDeleteBtnEl instanceof HTMLButtonElement) dbDeleteBtnEl.addEventListener("click", () => runDbAction(dbDeleteRow));
  if (dbClearBtnEl instanceof HTMLButtonElement) dbClearBtnEl.addEventListener("click", () => runDbAction(dbClearRows));
  if (dbRunSqlBtnEl instanceof HTMLButtonElement) dbRunSqlBtnEl.addEventListener("click", () => runDbAction(dbRunSql));
  if (dbSqlInputEl instanceof HTMLTextAreaElement) {
    dbSqlInputEl.addEventListener("keydown", (event) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
        event.preventDefault();
        runDbAction(dbRunSql);
      }
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

    const interruptChord = event.ctrlKey;
    if (foregroundProcess && interruptChord && (((event.key || "").toLowerCase() === "c") || event.code === "KeyC")) {
      event.preventDefault();
      log("^C");
      void interruptForegroundProcess({ interrupted: true });
      return;
    }

    if (event.key === "Escape") {
      showFileTree(false);
      setHelpOpen(false);
      setDataOpen(false);
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
