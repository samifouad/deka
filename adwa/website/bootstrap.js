const source = String(window.ADWA_UI_SOURCE || "legacy").toLowerCase()

if (source === "phpx") {
  import("./generated/phpx_ui_entry.js")
    .then((mod) => mod.mountPhpXUi())
    .catch((err) => {
      // Hard fallback to legacy path to avoid boot regressions.
      console.error("[adwa] phpx ui mount failed; falling back to legacy", err)
      return import("./main.js")
    })
} else {
  import("./main.js")
}
