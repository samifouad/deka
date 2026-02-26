const UTILITY_MARKER = "__deka_utility_css";

function stripAnsi(value) {
  return String(value).replace(/\u001b\[[0-9;]*m/g, "");
}

function looksLikeHtml(value) {
  const text = stripAnsi(value).trim();
  if (!text) return false;
  return /<!doctype html|<html[\s>]|<body[\s>]|<\/[a-z][a-z0-9:-]*>/i.test(text);
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function collectUtilityClasses(html) {
  const classes = new Set();
  const classRegex = /\bclass\s*=\s*"([^"]*)"/gi;
  let match = null;
  while ((match = classRegex.exec(html)) !== null) {
    for (const token of String(match[1] || "").split(/\s+/)) {
      const name = token.trim();
      if (name) classes.add(name);
    }
  }
  return classes;
}

function utilityRuleFor(name) {
  switch (name) {
    case "m-0":
      return "margin:0;";
    case "p-8":
      return "padding:2rem;";
    case "flex":
      return "display:flex;";
    case "items-center":
      return "align-items:center;";
    case "justify-center":
      return "justify-content:center;";
    case "min-h-screen":
      return "min-height:100vh;";
    case "font-bold":
      return "font-weight:700;";
    case "tracking-tight":
      return "letter-spacing:-0.025em;";
    case "text-5xl":
      return "font-size:3rem;line-height:1;";
    case "bg-slate-950":
      return "background-color:#020617;";
    case "text-slate-100":
      return "color:#f1f5f9;";
    default:
      return "";
  }
}

function generateUtilityCss(classes) {
  const rules = [];
  for (const name of classes) {
    const body = utilityRuleFor(name);
    if (!body) continue;
    const selector = `.${name.replaceAll(":", "\\:").replaceAll("/", "\\/")}`;
    rules.push(`${selector}{${body}}`);
  }
  if (!rules.length) return "";
  return `/* ${UTILITY_MARKER} */\n` + rules.join("\n");
}

function injectUtilityCss(html) {
  if (!html || html.includes(UTILITY_MARKER)) return html;
  const classes = collectUtilityClasses(html);
  const css = generateUtilityCss(classes);
  if (!css) return html;
  const styleTag = `<style data-deka-utility="1">${css}</style>`;
  if (/<\/head>/i.test(html)) {
    return html.replace(/<\/head>/i, `${styleTag}\n</head>`);
  }
  if (/<body[^>]*>/i.test(html)) {
    return html.replace(/<body[^>]*>/i, (m) => `${m}\n${styleTag}\n`);
  }
  return `${styleTag}\n${html}`;
}

export function createResultView(resultFrame, resultBannerEl) {
  let lastGoodResultDoc = "";

  function renderResult(stdout) {
    if (!(resultFrame instanceof HTMLIFrameElement)) return;
    const text = stripAnsi(String(stdout || "")).trim();
    if (looksLikeHtml(text)) {
      resultFrame.srcdoc = injectUtilityCss(text);
      lastGoodResultDoc = resultFrame.srcdoc;
      return;
    }
    resultFrame.srcdoc = `<pre style=\"font:14px/1.45 ui-monospace,monospace;padding:12px;white-space:pre-wrap;\">${escapeHtml(text || "(no output)")}</pre>`;
  }

  function clearResultBanner() {
    if (!(resultBannerEl instanceof HTMLElement)) return;
    resultBannerEl.classList.remove("show");
    resultBannerEl.textContent = "";
  }

  function showResultBanner(message) {
    if (!(resultBannerEl instanceof HTMLElement)) return;
    resultBannerEl.textContent = String(message || "Run failed.");
    resultBannerEl.classList.add("show");
  }

  return {
    renderResult,
    clearResultBanner,
    showResultBanner,
    hasLastGoodResult() {
      return Boolean(lastGoodResultDoc);
    },
  };
}
