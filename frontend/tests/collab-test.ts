import {
  test as base,
  expect,
  type Browser,
  type Page,
} from "@playwright/test";
import {
  expectNothingFocused,
  generateEmail,
  getKosoGraph,
  getTaskNumToTaskIdMap,
  init,
  login,
  setupNewProject,
  tearDown,
} from "./utils";

type CollabFixtures = {
  page1: Page;
  page2: Page;
  page3: Page;
};

async function shareProject(page: Page, email: string) {
  await page.getByRole("button", { name: "Project menu" }).click();
  await page.getByRole("menuitem", { name: "Share project" }).click();
  await page.getByRole("textbox", { name: "Add people" }).click();
  await page.keyboard.type(email);
  await page.getByText(email).click();
  await expect(
    page.getByRole("button", { name: `Remove ${email}` }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();
}

async function createSharedPage(page: Page, browser: Browser) {
  const otherPage = await browser.newPage();

  const otherEmail = generateEmail();
  await login(otherPage, otherEmail);
  await shareProject(page, otherEmail);

  await otherPage.goto("/projects");
  await otherPage.getByRole("link", { name: "Collab Test Project" }).click();
  await expect(
    otherPage.getByRole("button", { name: "Set Project Name" }),
  ).toBeVisible();

  return otherPage;
}

export const test = base.extend<CollabFixtures>({
  page1: async ({ page }, use) => {
    await login(page, generateEmail());
    await setupNewProject(page);

    await page.getByRole("button", { name: "Set Project Name" }).click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type("Collab Test Project");
    await page.keyboard.press("Enter");

    await use(page);
  },

  page2: async ({ page1, browser }, use) => {
    const otherPage = await createSharedPage(page1, browser);
    await use(otherPage);
  },

  page3: async ({ page1, browser }, use) => {
    const otherPage = await createSharedPage(page1, browser);
    await use(otherPage);
  },
});

test.describe.configure({ mode: "parallel" });

test.describe("Collaboration tests", () => {
  test.afterAll(async () => {
    await tearDown();
  });

  test("Collaborate to create and delete tasks", async ({ page1, page2 }) => {
    await page1.getByRole("button", { name: "Add" }).first().click();

    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();
    let graph = await getKosoGraph(page1);
    let tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeVisible();
    expect(graph).toStrictEqual(await getKosoGraph(page2));

    await page2.getByRole("button", { name: "Add" }).click();

    await expect(page2.getByRole("row", { name: "Task 2" })).toBeVisible();
    graph = await getKosoGraph(page2);
    tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["2"], tasks["1"]]);
    await expect(page1.getByRole("row", { name: "Task 2" })).toBeVisible();
    expect(graph).toStrictEqual(await getKosoGraph(page2));

    await page1
      .getByRole("row", { name: "Task 1" })
      .getByRole("button", { name: "Task Actions" })
      .click();
    await page1.getByRole("menuitem", { name: "Delete" }).click();

    await expect(page1.getByRole("row", { name: "Task 1" })).toBeHidden();
    graph = await getKosoGraph(page1);
    tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["2"]]);
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeHidden();
    expect(graph).toStrictEqual(await getKosoGraph(page2));

    await page2
      .getByRole("row", { name: "Task 2" })
      .getByRole("button", { name: "Task Actions" })
      .click();
    await page2.getByRole("menuitem", { name: "Delete" }).click();

    await expect(page2.getByRole("row", { name: "Task 2" })).toBeHidden();
    graph = await getKosoGraph(page2);
    tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([]);
    await expect(page1.getByRole("row", { name: "Task 2" })).toBeHidden();
    expect(graph).toStrictEqual(await getKosoGraph(page2));
  });

  test("Adjacent node selected when task deleted by other user", async ({
    page1,
    page2,
  }) => {
    await page1.getByRole("button", { name: "Add" }).first().click();
    await page1.getByRole("button", { name: "Add" }).first().click();
    await page1.getByRole("button", { name: "Add" }).first().click();

    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(page1.getByRole("row", { name: "Task 2" })).toBeVisible();
    await expect(page1.getByRole("row", { name: "Task 3" })).toBeVisible();

    await page2.getByRole("button", { name: "Task 2 Drag Handle" }).click();
    await page1.getByRole("button", { name: "Task 2 Drag Handle" }).click();
    await page1.keyboard.press("Delete");
    await expect(page2.getByRole("row", { name: "Task 2" })).toBeHidden();
    await expect(page1.getByRole("row", { name: "Task 3" })).toBeFocused();
    await expect(page2.getByRole("row", { name: "Task 3" })).toBeFocused();

    await page1.keyboard.press("Delete");
    await expect(page2.getByRole("row", { name: "Task 3" })).toBeHidden();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeFocused();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeFocused();

    await page1.keyboard.press("Delete");
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeHidden();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeHidden();
    console.log(
      await page1.evaluate(() => document.activeElement === document.body),
    );
    await expectNothingFocused(page1);
    await expectNothingFocused(page2);
    await expect(page1.getByRole("button", { name: "Add Task" })).toBeVisible();
    await expect(page1.getByRole("button", { name: "Add Task" })).toBeVisible();
  });

  test("Undo ignores tasks inserted by other user", async ({
    page1,
    page2,
  }) => {
    page1.evaluate(() => {
      window.koso.undoManager.captureTimeout = 0;
    });
    page2.evaluate(() => {
      window.koso.undoManager.captureTimeout = 0;
    });

    await page1.getByRole("button", { name: "Add" }).first().click();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeVisible();

    await page2.getByRole("button", { name: "Undo" }).click();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();

    await page1.getByRole("button", { name: "Undo" }).click();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeHidden();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeHidden();
    await page1.getByRole("button", { name: "Redo" }).click();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();
  });

  test("Undo ignores task edits by other user", async ({ page1, page2 }) => {
    page1.evaluate(() => {
      window.koso.undoManager.captureTimeout = 0;
    });
    page2.evaluate(() => {
      window.koso.undoManager.captureTimeout = 0;
    });

    await page1.getByRole("button", { name: "Add" }).first().click();
    await expect(page1.getByRole("row", { name: "Task 1" })).toBeVisible();
    await expect(page2.getByRole("row", { name: "Task 1" })).toBeVisible();
    await page1.keyboard.type("EditedTask");
    await page1.keyboard.press("Enter");
    await expect(
      page1.getByRole("button", { name: "Task 1 Edit Name" }),
    ).toHaveText("EditedTask");
    await expect(
      page2.getByRole("button", { name: "Task 1 Edit Name" }),
    ).toHaveText("EditedTask");

    await page2.getByRole("button", { name: "Undo" }).click();
    await expect(
      page2.getByRole("button", { name: "Task 1 Edit Name" }),
    ).toHaveText("EditedTask");
    await expect(
      page1.getByRole("button", { name: "Task 1 Edit Name" }),
    ).toHaveText("EditedTask");

    await page1.getByRole("button", { name: "Undo" }).click();
    await expect(
      page1.getByRole("button", { name: "Task 1 Edit Name" }),
    ).not.toHaveText("EditedTask");
    await expect(
      page2.getByRole("button", { name: "Task 1 Edit Name" }),
    ).not.toHaveText("EditedTask");
  });

  test("Awareness shows other users", async ({ page1, page2, page3 }) => {
    await init(page1, [
      { id: "root", name: "Root", children: ["1", "2", "3"] },
      { id: "1", name: "Task 1" },
      { id: "2", name: "Task 2" },
      { id: "3", name: "Task 3" },
    ]);

    await page2.getByRole("button", { name: "Task 1 Drag Handle" }).click();
    await expect(
      page1.getByRole("note", { name: "Pointy-Haired Boss selected" }),
    ).toBeVisible();

    await page3.getByRole("button", { name: "Task 1 Drag Handle" }).click();
    await expect(
      page1.getByRole("note", {
        name: "Pointy-Haired Boss and 1 more selected",
      }),
    ).toBeVisible();
  });
});
