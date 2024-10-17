import type { Status } from "$lib/koso";
import { expect, test, type Page } from "@playwright/test";
import {
  getKosoGraph,
  getTaskNumToTaskIdMap,
  setupNewProject,
  tearDown,
} from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("dag table tests", () => {
  test.beforeEach(async ({ page }) => {
    await setupNewProject(page);
  });

  test.afterAll(async () => {
    await tearDown();
  });

  type TaskBuilder = {
    id: string;
    num?: string;
    name?: string;
    children?: string[];
    assignee?: string | null;
    reporter?: string | null;
    status?: Status | null;
    statusTime?: number | null;
  };

  async function init(page: Page, tasks: TaskBuilder[]) {
    await page.evaluate((tasks) => {
      window.koso.yDoc.transact(() => {
        for (const {
          id,
          num = id,
          name = "",
          children = [],
          assignee = null,
          reporter = null,
          status = null,
          statusTime = null,
        } of tasks) {
          window.koso.upsert({
            id,
            num,
            name,
            children,
            assignee,
            reporter,
            status,
            statusTime,
          });
        }
      });
      for (let i = 0; i < window.localStorage.length; i++) {
        const key = window.localStorage.key(i);
        if (key && key.startsWith("expanded-nodes-")) {
          window.localStorage.removeItem(key);
        }
      }
    }, tasks);
    await page.reload();
    await page.getByLabel("Home").focus();
  }

  test.describe("creating tasks", () => {
    test("create a task by clicking the Add Task button", async ({ page }) => {
      await page.getByRole("button", { name: "Add Task" }).last().click();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    });

    test("create a task by clicking the Add Task button and then edit", async ({
      page,
    }) => {
      await page.getByRole("button", { name: "Add Task" }).last().click();

      await expect(
        page.getByRole("textbox", { name: "Task 1 Edit Name" }),
      ).toBeVisible();
      await page.keyboard.type("New task 1");
      await page.keyboard.press("Enter");

      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    });

    test("create a task by presing Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Shift+Enter");

      await expect(
        page.getByRole("textbox", { name: "Task 2 Edit Name" }),
      ).toBeVisible();
      await page.keyboard.type("New task 2");
      await page.keyboard.press("Enter");

      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([
        tasks["1"],
        tasks["2"],
      ]);
    });

    test("create a child task by presing Option+Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Alt+Shift+Enter");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      let graph = await getKosoGraph(page);
      let tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([tasks["1"]]);
      await expect(graph[tasks["1"]].children).toStrictEqual([tasks["2"]]);

      await page.keyboard.press("Alt+Shift+Enter");
      await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();

      graph = await getKosoGraph(page);
      tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([tasks["1"]]);
      await expect(graph[tasks["1"]].children).toStrictEqual([tasks["2"]]);
      await expect(graph[tasks["2"]].children).toStrictEqual([tasks["3"]]);
    });
  });

  test.describe("deleting tasks", () => {
    test("delete task 2 by clicking the Delete button", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.getByRole("button", { name: "Delete" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();

      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("delete task 2 by pressing the Delete key on the task", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2", children: ["4"] },
        { id: "3" },
        { id: "4" },
      ]);
      await page.getByRole("button", { name: "Task 2 Toggle Expand" }).click();

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();

      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("delete last task by pressing the Delete key on the task", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2"] },
        { id: "1", children: ["4"] },
        { id: "2", children: ["3"] },
        { id: "3" },
        { id: "4" },
      ]);
      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await page.getByRole("button", { name: "Task 2 Toggle Expand" }).click();

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeHidden();

      await expect(page.getByRole("row", { name: "Task 4" })).toBeFocused();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["4"] },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 4" })).toBeHidden();

      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: {},
      });
    });

    test("create a task by presing Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Shift+Enter");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      await expect(graph["root"].children).toStrictEqual([
        tasks["1"],
        tasks["2"],
      ]);
    });
  });

  test.describe("selecting tasks", () => {
    test("select task 1 by clicking on the drag handle", async ({ page }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);
      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
    });
  });

  test.describe("editing tasks", () => {
    test("set task 1's name by clicking Click to edit", async ({ page }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);
      await page.getByRole("button", { name: "Task 1 Edit Name" }).click();
      await page.keyboard.type("The 1st Task");
      await page.keyboard.press("Enter");

      await expect(
        page.getByRole("button", { name: "Task 1 Edit Name" }),
      ).toHaveText("The 1st Task");
    });

    test("set task 2's name by pressing Enter", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2"] },
        { id: "1" },
        { id: "2" },
      ]);
      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Enter");
      await page.keyboard.type("The 2nd Task");
      await page.keyboard.press("Enter");

      await expect(
        page.getByRole("button", { name: "Task 2 Edit Name" }),
      ).toHaveText("The 2nd Task");
    });
  });

  test.describe("movement using keyboard bindings", () => {
    test("up and down arrows change the selected row", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await expect(
        page.getByRole("button", { name: "Task 1 Drag Handle" }),
      ).toBeVisible();

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();

      await page.keyboard.press("ArrowUp");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("option up and down arrows change the order of rows", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await expect(
        page.getByRole("button", { name: "Task 1 Drag Handle" }),
      ).toBeVisible();

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await page.keyboard.press("Alt+ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "1", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("option left and right change row indentation", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowRight");

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowRight");
      await page.keyboard.press("Alt+ArrowRight");

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2"] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });

      await page.keyboard.press("ArrowUp");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowLeft");

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });
    });

    test.describe("option+shift up and down arrows change the order of rows", async () => {
      test("option+shift skips past collapsed nodes", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1", "2", "3"] },
          { id: "1" },
          { id: "2", children: ["4", "5"] },
          { id: "3" },
          { id: "4" },
          { id: "5", children: ["6"] },
          { id: "6" },
        ]);

        await expect(
          page.getByRole("button", { name: "Task 1 Drag Handle" }),
        ).toBeVisible();

        await page.keyboard.press("ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });
      });

      test("option+shift moves row by row", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1", "2", "3"] },
          { id: "1" },
          { id: "2", children: ["4", "5"] },
          { id: "3" },
          { id: "4" },
          { id: "5", children: ["6"] },
          { id: "6" },
        ]);

        await page
          .getByRole("button", { name: "Task 2 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 5 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["1", "4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "1", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1", "6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6", "1"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5", "1"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });
      });

      test("option+shift moves ignores target descendants", async ({
        page,
      }) => {
        await init(page, [
          { id: "root", children: ["1", "2", "3"] },
          { id: "1", children: ["11", "12", "13"] },
          { id: "2", children: ["4", "5"] },
          { id: "3" },
          { id: "4" },
          { id: "5", children: ["6"] },
          { id: "6" },
          { id: "11" },
          { id: "12", children: ["14", "16"] },
          { id: "13" },
          { id: "14", children: ["15"] },
          { id: "15" },
          { id: "16" },
        ]);

        await page
          .getByRole("button", { name: "Task 2 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 5 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 1 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 12 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["1", "4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "1", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1", "6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6", "1"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5", "1"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5", "1"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6", "1"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1", "6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "1", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["1", "4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1", "2", "3"] },
          ["1"]: { children: ["11", "12", "13"] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
          ["11"]: { children: [] },
          ["12"]: { children: ["14", "16"] },
          ["13"]: { children: [] },
          ["14"]: { children: ["15"] },
          ["15"]: { children: [] },
          ["16"]: { children: [] },
        });
      });

      test("move row past invalid to start", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1", "2"] },
          { id: "1", children: ["2"] },
          { id: "2" },
        ]);

        await page
          .getByRole("button", { name: "Task 1 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 2" }).nth(1),
        ).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 2" }).first(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 2" }).nth(1),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1", "2"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: [] },
        });
      });

      test("move row past multiple invalid locations", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1"] },
          { id: "1", children: ["2", "99"] },
          { id: "2", children: ["99", "3", "4"] },
          { id: "3", children: ["99"] },
          { id: "4" },
          { id: "99" },
        ]);

        await page
          .getByRole("button", { name: "Task 1 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 2 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 3 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).nth(2),
        ).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["99", "2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["99", "1"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["99", "2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).last(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["2", "99"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).last(),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1", "99"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });
      });

      test("single row remains unchanged", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1"] },
          { id: "1", children: ["2", "3"] },
          { id: "2" },
          { id: "3", children: ["4"] },
          { id: "4", children: ["5", "6"] },
          { id: "5" },
          { id: "6" },
        ]);

        await page
          .getByRole("button", { name: "Task 1 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 3 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 4 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: ["5", "6"] },
          ["5"]: { children: [] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: ["5", "6"] },
          ["5"]: { children: [] },
          ["6"]: { children: [] },
        });
      });

      test("invalid locations are skipped", async ({ page }) => {
        await init(page, [
          { id: "root", children: ["1", "5", "6"] },
          { id: "1", children: ["2", "3"] },
          { id: "2" },
          { id: "3", children: ["4"] },
          { id: "4" },
          { id: "5", children: ["1"] },
          { id: "6", children: ["7"] },
          { id: "7" },
        ]);

        await page
          .getByRole("button", { name: "Task 1 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 3 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 5 Toggle Expand" })
          .click();
        await page
          .getByRole("button", { name: "Task 6 Toggle Expand" })
          .click();

        await page.keyboard.press("ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).first(),
        ).toBeFocused();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "1", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["7"] },
          ["7"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["1", "7"] },
          ["7"]: { children: [] },
        });

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["7", "1"] },
          ["7"]: { children: [] },
        });
        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        await expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "6", "1"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["7"] },
          ["7"]: { children: [] },
        });
      });
    });
  });

  test.describe("expand and collapse", () => {
    test("expanding task 1 reveals task 2 and collapsing hides it", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2" },
      ]);
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
    });
  });

  test.describe("undo and redo", () => {
    test("clicking the undo and redo restores and deletes a task", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2"] },
        { id: "1" },
        { id: "2" },
      ]);
      page.evaluate(() => {
        window.koso.undoManager.captureTimeout = 0;
      });

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });

      await page.getByRole("button", { name: "Undo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
      });

      await page.getByRole("button", { name: "Redo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });
    });

    test("clicking undo restores selected node", async ({ page }) => {
      await init(page, [{ id: "root", children: ["1"] }, { id: "1" }]);
      page.evaluate(() => {
        window.koso.undoManager.captureTimeout = 0;
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: [] },
      });

      await page.getByRole("button", { name: "Undo" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });

      await page.getByRole("button", { name: "Redo" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: [] },
      });
    });
  });

  test.describe("drag and drop", () => {
    test("drag task 1 to peer of task 2 shows drop target", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).hover();
      await page.mouse.down();
      await page.getByRole("button", { name: "Task 2 Peer Dropzone" }).hover();
      await page.getByRole("button", { name: "Task 2 Peer Dropzone" }).hover();
      await expect(
        page.getByRole("button", { name: "Task 2 Peer Drop Indicator" }),
      ).toBeVisible();
      await page.mouse.up();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "1", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("drag task 1 to child of task 2 shows drop target", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).hover();
      await page.mouse.down();
      await page.getByRole("button", { name: "Task 2 Child Dropzone" }).hover();
      await page.getByRole("button", { name: "Task 2 Child Dropzone" }).hover();
      await expect(
        page.getByRole("button", { name: "Task 2 Child Drop Indicator" }),
      ).toBeVisible();
      await page.mouse.up();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });
    });

    test("move task 1 to child of task 2 and link as child of task 3", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      await page.keyboard.down("Alt");
      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 3 Child Dropzone" }));
      await page.keyboard.up("Alt");
      await expect(page.getByRole("row", { name: "Task 1" })).toHaveCount(2);

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: ["1"] },
      });
    });

    test("cannot make task 1 child of itself", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.keyboard.down("Alt");
      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 1 Child Dropzone" }));
      await page.keyboard.up("Alt");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("cannot make task 1 a peer of itself", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.keyboard.down("Alt");
      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));
      await page.keyboard.up("Alt");
      await expect(page.getByRole("row", { name: "Task 1" })).toHaveCount(2);

      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .first()
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));

      await expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });
    });
  });

  test.describe("status icon", () => {
    const now = Date.now();

    test("all tasks not started shows Not Started", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1"] },
        { id: "1", children: ["2", "3", "4", "5"] },
        { id: "2", status: null, statusTime: now },
        { id: "3", status: null, statusTime: now },
        { id: "4", status: null, statusTime: now },
        { id: "5", status: null, statusTime: now },
      ]);

      await expect(
        page.getByRole("row", { name: "Task 1" }).getByLabel("task-status"),
      ).toHaveText("Not Started");
    });

    test("one out of four tasks in-progress shows 0% In Progress", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1"] },
        { id: "1", children: ["2", "3", "4", "5"] },
        { id: "2", status: null, statusTime: now },
        { id: "3", status: null, statusTime: now },
        { id: "4", status: null, statusTime: now },
        { id: "5", status: "In Progress", statusTime: now },
      ]);

      await expect(
        page.getByRole("row", { name: "Task 1" }).getByLabel("task-status"),
      ).toHaveText("0% In Progress");
    });

    test("two out of four tasks complete shows 50% In Progress", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", children: ["1"] },
        { id: "1", children: ["2", "3", "4", "5"] },
        { id: "2", status: null, statusTime: now },
        { id: "3", status: null, statusTime: now },
        { id: "4", status: "Done", statusTime: now },
        { id: "5", status: "Done", statusTime: now },
      ]);

      await expect(
        page.getByRole("row", { name: "Task 1" }).getByLabel("task-status"),
      ).toHaveText("50% In Progress");
    });

    test("four out of four tasks complete shows Done", async ({ page }) => {
      await init(page, [
        { id: "root", children: ["1"] },
        { id: "1", children: ["2", "3", "4", "5"] },
        { id: "2", status: "Done", statusTime: now },
        { id: "3", status: "Done", statusTime: now },
        { id: "4", status: "Done", statusTime: now },
        { id: "5", status: "Done", statusTime: now },
      ]);

      await expect(
        page.getByRole("row", { name: "Task 1" }).getByLabel("task-status"),
      ).toHaveText("Done");
    });
  });
});
