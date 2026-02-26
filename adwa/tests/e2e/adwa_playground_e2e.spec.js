const { test, expect } = require("@playwright/test")

test("adwa playground edit -> run preserves interactive shell state", async ({ page }) => {
  const baseUrl = process.env.ADWA_E2E_URL || "http://127.0.0.1:5173"
  await page.goto(baseUrl, { waitUntil: "domcontentloaded" })

  const runBtn = page.locator("#runBtn")
  const log = page.locator("#log")

  await expect(runBtn).toBeVisible()
  await expect(log).toContainText("[foreground] process running (Ctrl+C to stop)")

  const nextSource = "/*__DEKA_PHPX__*/\necho 'updated from e2e\\n'"
  await page.evaluate(async (value) => {
    if (!window.__adwaTest || typeof window.__adwaTest.setSource !== "function") {
      throw new Error("missing __adwaTest.setSource hook")
    }
    if (typeof window.__adwaTest.run !== "function") {
      throw new Error("missing __adwaTest.run hook")
    }
    window.__adwaTest.setSource(value)
    await window.__adwaTest.run()
  }, nextSource)

  const currentSource = await page.evaluate(() =>
    window.__adwaTest && typeof window.__adwaTest.getSource === "function"
      ? window.__adwaTest.getSource()
      : ""
  )
  expect(currentSource).toBe(nextSource)

  await expect(log).toContainText("[foreground] process running (Ctrl+C to stop)")
})
