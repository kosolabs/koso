import { expect, test, type Page } from "@playwright/test";
import { generateEmail, login, tearDown } from "./utils";

test.describe("project tests", () => {
  test.describe.configure({ mode: "serial" });

  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test("home page presents login header and button", async () => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Koso");
  });

  test("log user in and view projects", async () => {
    await page.goto("/");

    const email = generateEmail();
    await login(page, email);

    await page.goto("/projects");
    await expect(
      page.getByText("Create your first Koso project!"),
    ).toBeVisible();
  });

  test("create a project and rename it to Integration Test Project", async () => {
    await page.getByRole("button", { name: "New" }).click();

    await page.getByRole("button", { name: "Set Project Name" }).click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type("Integration Test Project");
    await page.keyboard.press("Enter");

    await expect(
      page.getByRole("button", { name: "Set Project Name" }),
    ).toHaveText("Integration Test Project");
  });

  test("return to projects view and delete Integration Test Project", async () => {
    await page.getByRole("link", { name: "Home" }).click();

    await page
      .getByRole("button", { name: "Delete Integration Test Project" })
      .click();

    await expect(
      page.getByText("Create your first Koso project!"),
    ).toBeVisible();
  });
});
