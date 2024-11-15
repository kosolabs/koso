import { expect, test, type Page } from "@playwright/test";
import { generateEmail, login, setupNewProject, tearDown } from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("Invite user tests", () => {
  async function shareProject(page: Page, email: string) {
    await page.getByRole("button", { name: "Share Project" }).click();
    await page.getByRole("textbox", { name: "Add people" }).click();
    await page.keyboard.type(email);
    await page.getByText(email).click();
    await expect(
      page.getByRole("button", { name: `Remove ${email}` }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Close" }).click();
  }

  test.afterAll(async () => {
    await tearDown();
  });

  test("Invite a new user to Koso", async ({ page, browser }) => {
    await setupNewProject(page);

    const otherPage = await browser.newPage();
    const otherEmail = generateEmail();
    await login(otherPage, otherEmail, false);

    // The otherEmail user isn't invited and will be booted to login.
    await otherPage.goto("/projects");
    await otherPage.waitForURL("/");
    await expect(
      otherPage.getByText("You don't have access to Koso."),
    ).toBeVisible();

    await page.reload();
    await shareProject(page, otherEmail);

    // Now that the user was invited, they'll have access to the project.
    await login(otherPage, otherEmail, false);
    await otherPage.goto("/projects");
    await otherPage.getByRole("link", { name: "My Project!" }).click();
    await expect(
      otherPage.getByRole("button", { name: "Set Project Name" }),
    ).toBeVisible();
  });
});
