const { test, expect } = require("@playwright/test")

async function readHydrated(page, componentSuffix) {
  return page.evaluate((suffix) => {
    const node = document.querySelector(`[data-deka-island][data-component*="${suffix}"]`)
    if (!(node instanceof HTMLElement)) return null
    return node.dataset.dekaIslandHydrated === "1"
  }, componentSuffix)
}

test("islands directives hydrate on expected schedule", async ({ page }) => {
  const baseUrl = process.env.HMR_E2E_URL || "http://127.0.0.1:8530"

  await page.goto(baseUrl, { waitUntil: "domcontentloaded" })
  await page.waitForFunction(() => typeof window.__dekaHydrateIslands === "function")

  await page.waitForFunction((suffix) => {
    const node = document.querySelector(`[data-deka-island][data-component*="${suffix}"]`)
    return node instanceof HTMLElement && node.dataset.dekaIslandHydrated === "1"
  }, "LoadCard")

  await page.waitForFunction((suffix) => {
    const node = document.querySelector(`[data-deka-island][data-component*="${suffix}"]`)
    return node instanceof HTMLElement && node.dataset.dekaIslandHydrated === "1"
  }, "IdleCard")

  await page.waitForFunction((suffix) => {
    const node = document.querySelector(`[data-deka-island][data-component*="${suffix}"]`)
    return node instanceof HTMLElement && node.dataset.dekaIslandHydrated === "1"
  }, "MediaCard")

  const visibleBefore = await readHydrated(page, "VisibleCard")
  expect(visibleBefore).toBe(false)

  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight))

  await page.waitForFunction((suffix) => {
    const node = document.querySelector(`[data-deka-island][data-component*="${suffix}"]`)
    return node instanceof HTMLElement && node.dataset.dekaIslandHydrated === "1"
  }, "VisibleCard")
})
