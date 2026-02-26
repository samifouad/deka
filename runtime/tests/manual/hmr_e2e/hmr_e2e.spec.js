const fs = require("node:fs")
const path = require("node:path")
const { test, expect } = require("@playwright/test")

test("hmr preserves state and avoids full reload", async ({ page }) => {
  const baseUrl = process.env.HMR_E2E_URL || "http://127.0.0.1:8530"
  const entryFile = process.env.HMR_E2E_ENTRY_FILE
  if (!entryFile) {
    throw new Error("HMR_E2E_ENTRY_FILE is required")
  }
  const entryPath = path.resolve(entryFile)
  const baseline = fs.readFileSync(entryPath, "utf8")
  fs.writeFileSync(entryPath, baseline.replace("$version = 'v2'", "$version = 'v1'"))

  try {
    await page.goto(baseUrl, { waitUntil: "domcontentloaded" })
    await page.waitForSelector("#version")
    await page.waitForFunction(() => typeof window.__hmrNonce === "string")

    const before = await page.evaluate(() => {
      const input = document.getElementById("name")
      if (!(input instanceof HTMLInputElement)) {
        throw new Error("missing input")
      }
      window.scrollTo(0, 900)
      input.focus()
      input.setSelectionRange(2, 6)
      return {
        nonce: window.__hmrNonce,
        scrollY: window.scrollY,
      }
    })

    fs.writeFileSync(entryPath, baseline.replace("$version = 'v1'", "$version = 'v2'"))

    await page.waitForFunction(() => {
      const node = document.getElementById("version")
      return !!node && node.textContent && node.textContent.includes("v2")
    })

    const after = await page.evaluate(() => {
      const input = document.getElementById("name")
      if (!(input instanceof HTMLInputElement)) {
        throw new Error("missing input after patch")
      }
      return {
        nonce: window.__hmrNonce,
        activeId: document.activeElement ? document.activeElement.id : "",
        selectionStart: input.selectionStart,
        selectionEnd: input.selectionEnd,
        scrollY: window.scrollY,
      }
    })

    expect(after.nonce).toBe(before.nonce)
    expect(after.activeId).toBe("name")
    expect(after.selectionStart).toBe(2)
    expect(after.selectionEnd).toBe(6)
    expect(Math.abs(after.scrollY - before.scrollY)).toBeLessThanOrEqual(2)
  } finally {
    fs.writeFileSync(entryPath, baseline)
  }
})
