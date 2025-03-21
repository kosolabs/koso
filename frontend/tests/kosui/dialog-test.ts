import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });

test.describe("Dialog Component Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/storybook/dialogs");
  });

  test("notice dialog should open and close with OK", async ({ page }) => {
    // Initial state
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();

    // Open dialog
    await page.getByRole("button", { name: "Show Notice" }).click();
    await expect(page.getByText("Notice Dialog Open!").first()).toBeVisible();

    // Check dialog content
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText("Lookout")).toBeVisible();
    await expect(
      dialog.getByText("Heads up. A thing just happened!"),
    ).toBeVisible();

    // Close with OK button
    await dialog.getByRole("button", { name: "OK" }).click();
    await expect(dialog).not.toBeVisible();
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();
  });

  test("notice dialog should open and close with Escape", async ({ page }) => {
    // Initial state
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();

    // Close by pressing escape
    await page.getByRole("button", { name: "Show Notice" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();
  });

  test("notice dialog should open and close with click outside", async ({
    page,
  }) => {
    // Initial state
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();

    // Close by clicking outside
    await page.getByRole("button", { name: "Show Notice" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.mouse.click(10, 10);
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(page.getByText("Notice Dialog Closed!").first()).toBeVisible();
  });

  test("confirm dialog should return correct values", async ({ page }) => {
    // Open confirm dialog
    await page.getByRole("button", { name: "Show Confirm" }).click();
    const dialog = page.getByRole("dialog");

    // Check dialog content
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText("Do Something Destructive?")).toBeVisible();

    // Confirm and check result
    await dialog.getByRole("button", { name: "Accept" }).click();
    await expect(page.getByText("Confirm Dialog Accepted!")).toBeVisible();

    // Open again and cancel
    await page.getByRole("button", { name: "Show Confirm" }).click();
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(page.getByText("Cancelled!")).toBeVisible();
  });

  test("confirm dialog should cancel with Escape", async ({ page }) => {
    // Initial state
    await expect(
      page.getByText("Confirm Dialog Closed!").first(),
    ).toBeVisible();

    // Close by pressing escape
    await page.getByRole("button", { name: "Show Confirm" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(
      page.getByText("Confirm Dialog Cancelled!").first(),
    ).toBeVisible();
  });

  test("confirm dialog should cancel with click outside", async ({ page }) => {
    // Initial state
    await expect(
      page.getByText("Confirm Dialog Closed!").first(),
    ).toBeVisible();

    // Close by pressing escape
    await page.getByRole("button", { name: "Show Confirm" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.mouse.click(10, 10);
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(
      page.getByText("Confirm Dialog Cancelled!").first(),
    ).toBeVisible();
  });

  test("select dialog should show all options and return selection", async ({
    page,
  }) => {
    // Open select dialog
    await page.getByRole("button", { name: "Show Select" }).click();
    const dialog = page.getByRole("dialog");

    // Check all options are present
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole("button", { name: "North" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "South" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "East" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "West" })).toBeVisible();

    // Select and check result
    await dialog.getByRole("button", { name: "East" }).click();
    await expect(page.getByText("Select Dialog East")).toBeVisible();
  });

  test("select dialog should cancel with Escape", async ({ page }) => {
    // Initial state
    await expect(page.getByText("Select Dialog Closed!").first()).toBeVisible();

    // Close by pressing escape
    await page.getByRole("button", { name: "Show Select" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(
      page.getByText("Select Dialog Cancelled!").first(),
    ).toBeVisible();
  });

  test("select dialog should cancel with click outside", async ({ page }) => {
    // Initial state
    await expect(page.getByText("Select Dialog Closed!").first()).toBeVisible();

    // Close by pressing escape
    await page.getByRole("button", { name: "Show Select" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.mouse.click(10, 10);
    await expect(page.getByRole("dialog")).not.toBeVisible();
    await expect(
      page.getByText("Select Dialog Cancelled!").first(),
    ).toBeVisible();
  });

  test("custom dialog should support custom actions", async ({ page }) => {
    // Open custom dialog
    await page.getByRole("button", { name: "Show Custom" }).click();
    const dialog = page.getByRole("dialog");

    // Check custom content
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText("Cool Title Batman!")).toBeVisible();
    await expect(dialog.getByText("This is a custom dialog.")).toBeVisible();
    await expect(dialog.getByText("where")).toBeVisible();
    await expect(dialog.getByText("...anything can happen!")).toBeVisible();

    // Check all buttons are present
    await expect(dialog.getByRole("button", { name: "One" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "Two" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "Three" })).toBeVisible();
    await expect(dialog.getByRole("button", { name: "Four" })).toBeVisible();

    // Select and check result
    await dialog.getByRole("button", { name: "One" }).click();
    await expect(page.getByText("Selected: one")).toBeVisible();
  });

  test("dialog should be accessible via keyboard", async ({ page }) => {
    // Open dialog
    await page.getByRole("button", { name: "Show Custom" }).click();
    const dialog = page.getByRole("dialog");

    // One should be focused by default
    await expect(dialog.getByRole("button", { name: "One" })).toBeFocused();

    // Tab navigation should work
    await page.keyboard.press("Tab");
    await expect(dialog.getByRole("button", { name: "Two" })).toBeFocused();

    await page.keyboard.press("Tab");
    await expect(dialog.getByRole("button", { name: "Three" })).toBeFocused();

    // Close with Escape key
    await page.keyboard.press("Escape");
    await expect(dialog).not.toBeVisible();
  });

  test("nested dialogs should work correctly", async ({ page }) => {
    // Open custom dialog
    await page.getByRole("button", { name: "Show Custom" }).click();

    // Click Four to open nested dialog
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Four" })
      .click();

    // Check new dialog is visible
    await expect(page.getByText("Confirmation Title!")).toBeVisible();

    // Click confirmation
    await page.getByRole("button", { name: "Absolutely!" }).click();

    // Check result
    await expect(page.getByText("Selected: four")).toBeVisible();
  });

  test("dialog should have proper ARIA attributes", async ({ page }) => {
    // Open dialog
    await page.getByRole("button", { name: "Show Notice" }).click();
    const dialog = page.getByRole("dialog");

    // Check ARIA attributes
    await expect(dialog).toHaveAttribute("role", "dialog");
    await expect(dialog).toHaveAttribute("aria-modal", "true");

    // Close dialog
    await dialog.getByRole("button", { name: "OK" }).click();
  });
});
