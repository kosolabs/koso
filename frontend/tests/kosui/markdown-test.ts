import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });

test.describe("Markdown Component Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/storybook/markdown");
  });

  test("renders without warnings", async ({ page }) => {
    const warnings: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "warning") {
        warnings.push(msg.text());
      }
    });

    await page.goto("/storybook/markdown");
    await expect(page.getByText("EOF")).toBeVisible();

    expect(warnings).toEqual([]);
  });

  test("blockquote", async ({ page }) => {
    await expect(page.getByRole("blockquote")).toBeVisible();
  });

  test("break", async ({ page }) => {
    await expect(page.getByRole("paragraph").locator("br")).toBeAttached();
  });

  test("code", async ({ page }) => {
    await expect(page.getByRole("code").getByText("key")).toBeVisible();
  });

  test("codespan", async ({ page }) => {
    await expect(
      page
        .getByRole("paragraph")
        .getByText("Markdown")
        .getByRole("code")
        .getByText("inline"),
    ).toBeVisible();
  });

  test("del", async ({ page }) => {
    await expect(
      page.getByRole("paragraph").getByText("Markdown").getByRole("deletion"),
    ).toBeVisible();
  });

  test("emphasis", async ({ page }) => {
    await expect(
      page.getByRole("paragraph").getByText("Markdown").getByRole("emphasis"),
    ).toBeVisible();
  });

  test("heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "Heading 1" }),
    ).toBeVisible();

    await expect(
      page.getByRole("heading", { name: "Heading 2" }),
    ).toBeVisible();

    await expect(
      page.getByRole("heading", { name: "Heading 3" }),
    ).toBeVisible();
  });

  test("hr", async ({ page }) => {
    await expect(page.locator("hr")).toHaveCount(2);
  });

  test("html", async ({ page }) => {
    await expect(page.getByRole("paragraph").getByText("HTML")).toBeVisible();
  });

  test("image", async ({ page }) => {
    await expect(page.getByRole("img", { name: "image" })).toBeVisible();
  });

  test("link", async ({ page }) => {
    await expect(page.getByRole("link").getByText("link")).toBeVisible();
  });

  test("list", async ({ page }) => {
    await expect(page.getByRole("list").getByText("ol")).toHaveCount(2);
    await expect(page.getByRole("list").getByText("ul")).toHaveCount(2);
    await expect(page.getByRole("list").getByText("sub")).toHaveCount(2);
  });

  test("paragraph", async ({ page }) => {
    await expect(
      page.getByRole("paragraph").getByText("paragraph"),
    ).toBeVisible();
  });

  test("strong", async ({ page }) => {
    await expect(
      page.getByRole("paragraph").getByText("Markdown").getByRole("strong"),
    ).toBeVisible();
  });

  test("table", async ({ page }) => {
    await expect(page.getByRole("table")).toBeVisible();
  });

  test("text", async ({ page }) => {
    await expect(
      page.getByRole("paragraph").getByText("Markdown"),
    ).toBeVisible();
  });
});
