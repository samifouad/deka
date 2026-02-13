const { test, expect } = require("@playwright/test")

test("wosix playground edit -> run updates output", async ({ page }) => {
  const baseUrl = process.env.WOSIX_E2E_URL || "http://127.0.0.1:5173"
  await page.goto(baseUrl, { waitUntil: "domcontentloaded" })

  const source = page.locator("#source")
  const runBtn = page.locator("#runBtn")
  const log = page.locator("#log")

  await expect(source).toBeVisible()
  await expect(runBtn).toBeVisible()
  await expect(log).toContainText("PHPX run complete")
  await expect(log).toContainText("hello from phpx in wosix")

  await source.fill("/*__DEKA_PHPX__*/\necho 'updated from e2e\\n'")
  await runBtn.click()

  await expect(log).toContainText("updated from e2e")
  await expect(source).toHaveValue("/*__DEKA_PHPX__*/\necho 'updated from e2e\\n'")
})
