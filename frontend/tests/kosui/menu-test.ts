import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });

test.describe("Menu Component Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/storybook/menus");
  });

  test("menus should have correct roles", async ({ page }) => {
    const trigger = page.getByRole("button", {
      name: "Open Menu",
    });

    // Initial state
    await expect(trigger).toHaveAttribute("aria-haspopup", "menu");
    await expect(trigger).toHaveAttribute("aria-expanded", "false");

    // After opening
    await trigger.click();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  test("menu should have correct role and structure", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    const menu = page.getByRole("menu");
    await expect(menu).toBeVisible();
    await expect(menu).toHaveAttribute("role", "menu");

    // Menu items should have correct role
    const menuItems = page.getByRole("menuitem");
    await expect(menuItems).toHaveCount(3);
  });

  test("keyboard navigation should work correctly", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    // Arrow down should focus first item
    await page.keyboard.press("ArrowDown");
    await expect(page.getByRole("menuitem").first()).toBeFocused();

    // Arrow down should move to next item
    await page.keyboard.press("ArrowDown");
    await expect(page.getByRole("menuitem").nth(1)).toBeFocused();

    // Arrow up should move to previous item
    await page.keyboard.press("ArrowUp");
    await expect(page.getByRole("menuitem").first()).toBeFocused();

    // End should focus last item
    await page.keyboard.press("End");
    await expect(page.getByRole("menuitem").last()).toBeFocused();

    // Home should focus first item
    await page.keyboard.press("Home");
    await expect(page.getByRole("menuitem").first()).toBeFocused();
  });

  test("menu should close on escape and return focus", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    await page.keyboard.press("Escape");
    await expect(page.getByRole("menu")).not.toBeVisible();
    await expect(trigger).toBeFocused();
  });

  test("menu should support character search", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    // Type first character of menu item
    await page.keyboard.type("3");
    await expect(page.getByRole("menuitem").last()).toBeFocused();
  });

  test("menu should maintain focus within menu", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    // Tab should cycle through menu items
    await page.keyboard.press("Tab");
    await expect(page.getByRole("menuitem").first()).toBeFocused();

    // Shift+Tab should reverse cycle
    await page.keyboard.press("Shift+Tab");
    await expect(page.getByRole("menuitem").last()).toBeFocused();
  });

  test("menu items should be selectable by enter/space", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Open Menu" });
    await trigger.click();

    await page.getByRole("menuitem").first().press("Enter");
    await expect(page.getByRole("menu")).not.toBeVisible();
    await expect(trigger).toBeFocused();

    // Repeat with Space
    await trigger.click();
    await page.getByRole("menuitem").first().press(" ");
    await expect(page.getByRole("menu")).not.toBeVisible();
    await expect(trigger).toBeFocused();
  });
});
