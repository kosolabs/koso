import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });

test.describe("Button Component Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/storybook/buttons");
  });

  test("buttons should have correct roles", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: "Elevated Primary Button" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Elevated Primary Icon" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Elevated Primary Disabled" }),
    ).toBeVisible();
  });

  test("buttons should activate when enter is pressed", async ({ page }) => {
    await page.getByTestId("result").isHidden();
    await page.getByRole("button", { name: "Elevated Primary Button" }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByTestId("result")).toBeVisible();
  });

  test("buttons should activate when space is pressed", async ({ page }) => {
    await page.getByTestId("result").isHidden();
    await page.getByRole("button", { name: "Elevated Primary Button" }).focus();
    await page.keyboard.press("Space");
    await expect(page.getByTestId("result")).toBeVisible();
  });

  test("buttons should activate when clicked", async ({ page }) => {
    await page.getByTestId("result").isHidden();
    await page.getByRole("button", { name: "Elevated Primary Button" }).click();
    await expect(page.getByTestId("result")).toBeVisible();
  });

  test("disabled buttons should be disabled", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: "Elevated Primary Disabled" }),
    ).toBeDisabled();
  });

  test("disabled buttons should not be focusable", async ({ page }) => {
    await page
      .getByRole("button", { name: "Elevated Primary Disabled" })
      .focus();
    const focusedElement = await page.evaluate(
      () => document.activeElement?.tagName,
    );
    expect(focusedElement).not.toBe("BUTTON");
  });

  // TODO: Add a test for focus rings
});
