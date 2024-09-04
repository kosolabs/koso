import { expect, test } from "@playwright/test";

test("home page presents login header and button", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Koso");
});
