import { expect, test, type Page } from "@playwright/test";
import {
  setupNewProject,
  tearDown,
  login,
  generateEmail,
  getTaskNumToTaskIdMap,
  getKosoGraph,
} from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("Collaboration tests", () => {
  let otherPage: Page;

  test.beforeEach(async ({ page, browser }) => {
    await setupNewProject(page);

    otherPage = await browser.newPage();
    const otherEmail = generateEmail();
    await login(otherPage, otherEmail);

    await page.reload();
    await shareProject(page, otherEmail);

    await otherPage.goto("/projects");
    await otherPage.getByRole("link", { name: "My Project!" }).click();
    await expect(
      otherPage.getByRole("button", { name: "Set Project Name" }),
    ).toBeVisible();
  });

  async function shareProject(page: Page, email: string) {
    await page.getByRole("button", { name: "Share Project" }).click();
    await page.getByRole("textbox", { name: "Add people" }).click();
    await page.keyboard.type(email);
    await page.getByText(email).click();
    await page.getByRole("button", { name: "Close" }).click();
  }

  test.afterAll(async () => {
    await tearDown();
  });

  test("Collaborate to create and delete tasks", async ({ page }) => {
    await page.getByRole("button", { name: "Add Task" }).first().click();

    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();
    let graph = await getKosoGraph(page);
    let tasks = getTaskNumToTaskIdMap(graph);
    await expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    await expect(otherPage.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(graph).toStrictEqual(await getKosoGraph(otherPage));

    await otherPage.getByRole("button", { name: "Add Task" }).click();

    await expect(otherPage.getByRole("row", { name: "Task 2" })).toBeVisible();
    graph = await getKosoGraph(otherPage);
    tasks = getTaskNumToTaskIdMap(graph);
    await expect(graph["root"].children).toStrictEqual([
      tasks["2"],
      tasks["1"],
    ]);
    await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();
    await expect(graph).toStrictEqual(await getKosoGraph(otherPage));

    await page.getByRole("button", { name: "Delete" }).click();

    await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
    graph = await getKosoGraph(page);
    tasks = getTaskNumToTaskIdMap(graph);
    await expect(graph["root"].children).toStrictEqual([tasks["2"]]);
    await expect(otherPage.getByRole("row", { name: "Task 1" })).toBeHidden();
    await expect(graph).toStrictEqual(await getKosoGraph(otherPage));

    await otherPage.getByRole("button", { name: "Delete" }).click();

    await expect(otherPage.getByRole("row", { name: "Task 2" })).toBeHidden();
    graph = await getKosoGraph(otherPage);
    tasks = getTaskNumToTaskIdMap(graph);
    await expect(graph["root"].children).toStrictEqual([]);
    await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
    await expect(graph).toStrictEqual(await getKosoGraph(otherPage));
  });
});
