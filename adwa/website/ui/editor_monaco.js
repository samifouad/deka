export async function initMonacoEditor({
  editorHostEl,
  sourceEl,
  defaultSource,
  onChange,
  provideCompletionItems,
  provideHover,
}) {
  if (!(editorHostEl instanceof HTMLElement)) {
    return { monacoApi: null, monacoEditor: null };
  }

  try {
    if (!document.getElementById("deka-codicon-css")) {
      const link = document.createElement("link");
      link.id = "deka-codicon-css";
      link.rel = "stylesheet";
      link.href =
        "https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/min/vs/base/browser/ui/codicons/codicon/codicon.css";
      document.head.appendChild(link);
    }

    const monaco = await import("https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/+esm");
    const editor = monaco.editor.create(editorHostEl, {
      value: defaultSource,
      language: "php",
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      lineNumbers: "on",
      roundedSelection: false,
      scrollBeyondLastLine: false,
      tabSize: 2,
      insertSpaces: true,
      fontSize: 14,
      wordWrap: "on",
    });

    editor.onDidChangeModelContent(() => {
      if (typeof onChange === "function") onChange();
    });

    monaco.languages.registerCompletionItemProvider("php", {
      triggerCharacters: [".", "'", "\"", "<", " ", ":"],
      provideCompletionItems: async (model, position) => {
        if (typeof provideCompletionItems !== "function") return { suggestions: [] };
        const suggestions = await provideCompletionItems(monaco, model, position);
        return { suggestions: Array.isArray(suggestions) ? suggestions : [] };
      },
    });

    monaco.languages.registerHoverProvider("php", {
      provideHover: async (model, position) => {
        if (typeof provideHover !== "function") return null;
        return provideHover(monaco, model, position);
      },
    });

    return { monacoApi: monaco, monacoEditor: editor };
  } catch {
    if (sourceEl instanceof HTMLTextAreaElement) {
      sourceEl.style.display = "block";
      sourceEl.value = defaultSource;
    }
    return { monacoApi: null, monacoEditor: null };
  }
}
