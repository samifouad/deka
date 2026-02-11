const { test, expect } = require("@playwright/test")

test("wosix playground edit -> run updates output", async ({ page }) => {
  const baseUrl = process.env.WOSIX_E2E_URL || "http://127.0.0.1:5173"
  await page.goto(baseUrl, { waitUntil: "domcontentloaded" })

  const source = page.locator("#source")
  const runBtn = page.locator("#runBtn")
  const log = page.locator("#log")

  await expect(source).toBeVisible()
  await expect(runBtn).toBeVisible()

  await source.fill("process.exit(7)")
  await runBtn.click()

  await expect(log).toContainText("Exit code: 7")
  await expect(source).toHaveValue("process.exit(7)")
})
