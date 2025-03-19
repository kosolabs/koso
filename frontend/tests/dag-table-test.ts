import { expect, test } from "@playwright/test";
import {
  expectKosoGraph,
  getKosoGraph,
  getTaskNumToTaskIdMap,
  init,
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

  test.describe("creating tasks", () => {
    test("create a task by clicking the Add button", async ({ page }) => {
      await page.getByRole("button", { name: "Add" }).last().click();
      await page.keyboard.press("Escape");

      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    });

    test("create a task by clicking the Add button and then edit", async ({
      page,
    }) => {
      await page.getByRole("button", { name: "Add" }).last().click();

      await expect(
        page.getByRole("textbox", { name: "Task 1 Edit Name" }),
      ).toBeVisible();
      await page.keyboard.type("New task 1");
      await page.keyboard.press("Enter");

      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    });

    test("create a task by presing Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);

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
      expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
    });

    test("create a child task by presing Option+Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Alt+Shift+Enter");
      await page.keyboard.type("Task 2");

      await expect(page.getByRole("row", { name: "Task 2" })).not.toBeFocused();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      let graph = await getKosoGraph(page);
      let tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"]]);
      expect(graph[tasks["1"]].children).toStrictEqual([tasks["2"]]);

      await page.keyboard.press("Alt+Shift+Enter");
      await page.keyboard.type("Task 3");
      await page.keyboard.press("Enter");

      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();

      graph = await getKosoGraph(page);
      tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"]]);
      expect(graph[tasks["1"]].children).toStrictEqual([tasks["2"]]);
      expect(graph[tasks["2"]].children).toStrictEqual([tasks["3"]]);
    });
  });

  test.describe("deleting tasks", () => {
    test("delete task 2 by clicking the Delete button", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.getByRole("button", { name: "Delete" }).click();

      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("delete task 2 by pressing the Delete key on the task", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
        { id: "2", children: ["4"] },
      ]);
      await page.getByRole("button", { name: "Task 2 Toggle Expand" }).click();

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");

      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("delete last task by pressing the Delete key on the task", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2"] },
        { id: "1", children: ["4"] },
        { id: "2", children: ["3"] },
      ]);
      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await page.getByRole("button", { name: "Task 2 Toggle Expand" }).click();

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");

      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 4" })).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["4"] },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Delete");

      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 4" })).toBeHidden();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();

      expect(await getKosoGraph(page)).toMatchObject({ root: {} });
    });

    test("create a task by presing Shift+Enter on the task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Shift+Enter");

      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      const graph = await getKosoGraph(page);
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
    });
  });

  test.describe("selecting tasks", () => {
    test("select task 1 by clicking on the drag handle", async ({ page }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);
      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();

      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
    });

    test("up and down arrows change the selected row", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
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

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("meta+up and down arrows select next/prev linked task", async ({
      page,
    }) => {
      await init(
        page,
        [
          { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
          { id: "2", children: ["3"] },
          { id: "4", children: ["5"] },
          { id: "5", children: ["3"] },
        ],
        true,
      );

      page
        .getByTestId("Row 3")
        .getByRole("button", { name: "Task 3 Drag Handle" })
        .click();
      await expect(page.getByTestId("Row 3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 4/5/3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 2/3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 2/3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 4/5/3")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 3")).toBeFocused();
    });

    test("meta+up and down arrows select next/prev linked task and wraps around", async ({
      page,
    }) => {
      await init(
        page,
        [
          { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
          { id: "2", children: ["1"] },
          { id: "4", children: ["5"] },
          { id: "5", children: ["1"] },
        ],
        true,
      );

      page
        .getByTestId("Row 1")
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .click();
      await expect(page.getByTestId("Row 1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 2/1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 4/5/1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowDown");
      await expect(page.getByTestId("Row 2/1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 4/5/1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 2/1")).toBeFocused();

      await page.keyboard.press("Meta+ArrowUp");
      await expect(page.getByTestId("Row 1")).toBeFocused();
    });
  });

  test.describe("editing tasks", () => {
    test("set task 1's name by clicking Click to edit", async ({ page }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);
      await page.getByRole("button", { name: "Task 1 Edit Name" }).click();
      await page.keyboard.type("The 1st Task");
      await page.keyboard.press("Enter");

      await expect(
        page.getByRole("button", { name: "Task 1 Edit Name" }),
      ).toHaveText("The 1st Task");
    });

    test("set task 2's name by pressing Enter", async ({ page }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1", "2"] }]);
      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Enter");
      await page.keyboard.type("The 2nd Task");
      await page.keyboard.press("Enter");

      await expect(
        page.getByRole("button", { name: "Task 2 Edit Name" }),
      ).toHaveText("The 2nd Task");
    });
  });

  test.describe("moving tasks", () => {
    test.describe("option up and down arrows change the order of rows", async () => {
      test("option skips past collapsed nodes", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3"] },
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });
      });

      test("option moves row by row", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3"] },
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["1", "4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "1", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1", "6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6", "1"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5", "1"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: ["4", "5"] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
          ["5"]: { children: ["6"] },
          ["6"]: { children: [] },
        });
      });

      test("option moves ignores target descendants", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3"] },
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 1", exact: true }),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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
          { id: "root", name: "Root", children: ["1", "2"] },
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 2" }).first(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 2" }).nth(1),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1", "2"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: [] },
        });
      });

      test("move row past multiple invalid locations", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1"] },
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

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["99", "2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowUp");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["99", "1"] },
          ["1"]: { children: ["2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).first(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["99", "2"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).last(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["2", "99"] },
          ["2"]: { children: ["99", "3", "4"] },
          ["3"]: { children: ["99"] },
          ["4"]: { children: [] },
          ["99"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 99" }).last(),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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
          { id: "root", name: "Root", children: ["1"] },
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["1"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: ["5", "6"] },
          ["5"]: { children: [] },
          ["6"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowUp");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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
          { id: "root", name: "Root", children: ["1", "5", "6"] },
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

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "1", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["7"] },
          ["7"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["1", "7"] },
          ["7"]: { children: [] },
        });

        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["5", "6"] },
          ["1"]: { children: ["2", "3"] },
          ["2"]: { children: [] },
          ["3"]: { children: ["4"] },
          ["4"]: { children: [] },
          ["5"]: { children: ["1"] },
          ["6"]: { children: ["7", "1"] },
          ["7"]: { children: [] },
        });
        await page.keyboard.press("Alt+ArrowDown");
        await expect(
          page.getByRole("row", { name: "Task 1" }).nth(1),
        ).toBeFocused();
        expect(await getKosoGraph(page)).toMatchObject({
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

    test.describe("within contiguous groups", () => {
      const now = Date.now();

      test("all tasks same status moves task to the bottom", async ({
        page,
      }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3"] },
        ]);

        await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1"] },
          ["1"]: { children: [] },
          ["2"]: { children: [] },
          ["3"]: { children: [] },
        });
      });

      test("all tasks same status moves task to the top", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3"] },
        ]);

        await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();

        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["3", "1", "2"] },
        });
      });

      test("moves task to the top of the next group", async ({ page }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
          { id: "4", status: "Done", statusTime: now },
        ]);

        await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();

        await page.keyboard.press("Alt+Shift+ArrowDown");
        await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "3", "1", "4"] },
          ["1"]: { children: [] },
          ["2"]: { children: [] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
        });
      });

      test("moves task on the edge to the top of the group", async ({
        page,
      }) => {
        await init(page, [
          { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
          { id: "1", status: "In Progress", statusTime: now },
          { id: "4", status: "Done", statusTime: now },
        ]);

        await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();

        await page.keyboard.press("Alt+Shift+ArrowUp");
        await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();

        expect(await getKosoGraph(page)).toMatchObject({
          root: { children: ["2", "1", "3", "4"] },
          ["1"]: { children: [] },
          ["2"]: { children: [] },
          ["3"]: { children: [] },
          ["4"]: { children: [] },
        });
      });
    });
  });

  test.describe("indenting tasks", () => {
    test("option left and right change row indentation", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowRight");
      await expect(page.getByTestId("Row 1/2")).toBeFocused();

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowRight");
      await expect(page.getByTestId("Row 1/3")).toBeFocused();
      await page.keyboard.press("Alt+ArrowRight");
      await expect(page.getByTestId("Row 1/2/3")).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2"] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });

      await page.keyboard.press("ArrowUp");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowLeft");
      await expect(page.getByTestId("Row 2")).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });
    });
  });

  test.describe("expand and collapse", () => {
    test("expanding task 1 reveals task 2 and collapsing hides it", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2"] },
      ]);
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
    });

    test("expand all reveals all children", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2", children: ["3"] },
        { id: "3" },
      ]);
      await expect(page.getByTestId("Row 1/2/3")).not.toBeVisible();

      await page.getByRole("button", { name: "Palette" }).click();
      await expect(page.getByRole("dialog")).toBeVisible();

      await page.keyboard.type("Expand All");
      await page.keyboard.press("Enter");
      await expect(page.getByTestId("Row 1/2/3")).toBeVisible();
      await expect(page.getByTestId("Row 1/2")).toBeVisible();
    });

    test("collapse all hides all children", async ({ page }) => {
      await init(
        page,
        [
          { id: "root", name: "Root", children: ["1"] },
          { id: "1", children: ["2"] },
          { id: "2", children: ["3"] },
          { id: "3" },
        ],
        true,
      );
      await expect(page.getByTestId("Row 1/2/3")).toBeVisible();

      await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();
      await expect(page.getByRole("row", { name: "Task 3" })).toBeFocused();

      await page.getByRole("button", { name: "Palette" }).click();
      await expect(page.getByRole("dialog")).toBeVisible();

      await page.keyboard.type("Collapse All");
      await page.keyboard.press("Enter");
      await expect(page.getByTestId("Row 1/2/3")).not.toBeVisible();
      await expect(page.getByTestId("Row 1/2")).not.toBeVisible();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
    });

    test("collapse unselects selected, hidden child node and selects ancestor", async ({
      page,
    }) => {
      await init(
        page,
        [
          { id: "root", name: "Root", children: ["1", "7"] },
          { id: "1", children: ["2", "6"] },
          { id: "2", children: ["5", "3", "4"] },
          { id: "3" },
          { id: "4" },
          { id: "5" },
          { id: "6" },
          { id: "7" },
        ],
        true,
      );
      await expect(page.getByTestId("Row 1/2/3")).toBeVisible();
      await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();

      await page.getByRole("button", { name: "Task 2 Toggle Expand" }).click();
      await expect(page.getByTestId("Row 1/2/3")).not.toBeVisible();
      await expect(page.getByTestId("Row 1/2/4")).not.toBeVisible();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByTestId("Row 1/2")).not.toBeVisible();
      await expect(page.getByTestId("Row 1/6")).not.toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
    });
  });

  test.describe("changing task status", () => {
    test("change status to in progress with space key", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", children: [] },
        { id: "2", children: [] },
        { id: "3", children: [] },
        { id: "4", children: [] },
      ]);

      await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();
      await page.keyboard.press("Space");

      const statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("In Progress");
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["3", "1", "2", "4"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [], status: "In Progress" },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Space");
      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["3", "2", "1", "4"] },
        ["1"]: { children: [], status: "In Progress" },
        ["2"]: { children: [], status: "In Progress" },
        ["3"]: { children: [], status: "In Progress" },
        ["4"]: { children: [] },
      });
    });

    test("change status to done with space key", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", children: [], status: "In Progress" },
        { id: "2", children: [], status: "In Progress" },
        { id: "3", children: [] },
        { id: "4", children: [] },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "3", "4", "2"] },
        ["1"]: { children: [], status: "In Progress" },
        ["2"]: { children: [], status: "Done" },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["3", "4", "1", "2"] },
        ["1"]: { children: [], status: "Done" },
        ["2"]: { children: [], status: "Done" },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });

      // Clicking space doesn't toggle back to not started
      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["3", "4", "1", "2"] },
        ["1"]: { children: [], status: "Done" },
        ["2"]: { children: [], status: "Done" },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });
    });

    test("change status from blocked with space key", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", children: ["2"], status: "Blocked", kind: "Juggled" },
        { id: "2", children: [] },
        { id: "3", children: [] },
        { id: "4", children: [] },
      ]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3", "4"] },
        ["1"]: { children: ["2"], status: "Not Started" },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3", "4"] },
        ["1"]: { children: ["2"], status: "In Progress" },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Space");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "3", "4", "1"] },
        ["1"]: { children: ["2"], status: "Done" },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
        ["4"]: { children: [] },
      });
    });

    test("change status to in progress by clicking", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", children: [] },
        { id: "2", children: [] },
        { id: "3", children: [] },
        { id: "4", children: [] },
      ]);

      let statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 3" })
        .getByRole("menuitem", { name: "In Progress" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("In Progress");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["3", "1", "2", "4"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [], status: "In Progress" },
        ["4"]: { children: [] },
      });
    });

    test("change status to done by clicking", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["3", "1", "2", "4"] },
        { id: "1", children: [] },
        { id: "2", children: [] },
        { id: "3", children: [], status: "In Progress" },
        { id: "4", children: [] },
      ]);
      let statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("In Progress");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 3" })
        .getByRole("menuitem", { name: "Done" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Done");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "4", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [], status: "Done" },
        ["4"]: { children: [] },
      });
    });

    test("change status to not started by clicking", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3", "4"] },
        { id: "1", children: [] },
        { id: "2", children: [] },
        { id: "3", children: [], status: "Done", statusTime: Date.now() },
        { id: "4", children: [] },
      ]);
      let statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Done");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 3" })
        .getByRole("menuitem", { name: "Not Started" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3", "4"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [], status: "Not Started" },
        ["4"]: { children: [] },
      });
    });
  });

  test.describe("undo and redo", () => {
    test("clicking the undo and redo restores and deletes a task", async ({
      page,
    }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1", "2"] }]);
      page.evaluate(() => {
        window.koso.undoManager.captureTimeout = 0;
      });

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });

      await page.getByRole("button", { name: "Undo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
      });

      await page.getByRole("button", { name: "Redo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeHidden();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });
    });

    test("clicking undo restores selected node", async ({ page }) => {
      await init(page, [{ id: "root", name: "Root", children: ["1"] }]);
      page.evaluate(() => {
        window.koso.undoManager.captureTimeout = 0;
      });

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: [] },
      });

      await page.getByRole("button", { name: "Undo" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
      await expect(page.getByRole("button", { name: "Delete" })).toBeVisible();
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });

      await page.getByRole("button", { name: "Redo" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeHidden();
      await expect(page.getByRole("button", { name: "Delete" })).toBeHidden();
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: [] },
      });
    });
  });

  test.describe("drag and drop", () => {
    test.describe.configure({ retries: 3 });

    test("drag task 1 to peer of task 2 shows drop target", async ({
      page,
    }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).hover();
      await page.mouse.down();
      await page.getByRole("button", { name: "Task 2 Peer Dropzone" }).hover();
      await page.getByRole("button", { name: "Task 2 Peer Dropzone" }).hover();
      await expect(
        page.getByRole("tooltip", { name: "Task 2 Peer Drop Indicator" }),
      ).toBeVisible();
      await page.mouse.up();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      await expectKosoGraph(page).toMatchObject({
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
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).hover();
      await page.mouse.down();
      await page.getByRole("button", { name: "Task 2 Child Dropzone" }).hover();
      await page.getByRole("button", { name: "Task 2 Child Dropzone" }).hover();
      await expect(
        page.getByRole("tooltip", { name: "Task 2 Child Drop Indicator" }),
      ).toBeVisible();
      await page.mouse.up();
      await expect(page.getByTestId("Row 2/1")).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      expect(await getKosoGraph(page)).toMatchObject({
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
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));
      await expect(page.getByTestId("Row 2/1")).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });

      await page.keyboard.down("Alt");
      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 3 Child Dropzone" }));
      await page.keyboard.up("Alt");
      await expect(page.getByTestId("Row 3/1")).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toHaveCount(2);

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: ["1"] },
      });
    });

    test("cannot make task 1 child of itself", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1", "2", "3"] },
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
        { id: "root", name: "Root", children: ["1", "2", "3"] },
      ]);

      await page.keyboard.down("Alt");
      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));
      await page.keyboard.up("Alt");
      await expect(page.getByTestId("Row 2/1")).toBeVisible();
      await expect(page.getByRole("row", { name: "Task 1" })).toHaveCount(2);
      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });

      await page
        .getByRole("button", { name: "Task 1 Drag Handle" })
        .first()
        .dragTo(page.getByRole("button", { name: "Task 2 Child Dropzone" }));

      expect(await getKosoGraph(page)).toMatchObject({
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
        { id: "root", name: "Root", children: ["1"] },
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
        { id: "root", name: "Root", children: ["1"] },
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
        { id: "root", name: "Root", children: ["1"] },
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
        { id: "root", name: "Root", children: ["1"] },
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

  test.describe("search panel", () => {
    test("search panel expands and selects task", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["m1", "m2", "c1", "c2"] },
        { id: "m1", name: "Milestone 1", children: ["f1", "f2"] },
        { id: "m2", name: "Milestone 2", children: [] },
        { id: "c1", name: "Component 1", children: [] },
        { id: "c2", name: "Component 2", children: [] },
        { id: "f1", name: "Feature 1", children: [] },
        { id: "f2", name: "Feature 2", children: [] },
      ]);

      await page.keyboard.press("Meta+p");
      await expect(page.getByRole("dialog")).toBeVisible();
      await page.getByRole("option", { name: "Task f2 Search Item" }).click();

      await expect(page.getByRole("row", { name: "Task f2" })).toBeFocused();
    });
  });

  test.describe("link panel", () => {
    test("link panel adds a link to task by name", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["m1", "m2", "c1", "c2"] },
        { id: "m1", name: "Milestone 1", children: ["f1", "f2"] },
        { id: "m2", name: "Milestone 2", children: [] },
        { id: "c1", name: "Component 1", children: [] },
        { id: "c2", name: "Component 2", children: [] },
        { id: "f1", name: "Feature 1", children: [] },
        { id: "f2", name: "Feature 2", children: [] },
      ]);

      await page.getByRole("button", { name: "Task m1 Toggle Expand" }).click();
      await page.getByRole("button", { name: "Task f1 Drag Handle" }).click();

      await page.keyboard.press("Meta+/");
      await page.getByRole("option", { name: "Task c2 Command Item" }).click();

      await page.getByRole("button", { name: "Task c2 Toggle Expand" }).click();
      await expect(page.getByTestId("Row c2/f1")).toContainText("Milestone 1");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["m1", "m2", "c1", "c2"] },
        m1: { children: ["f1", "f2"] },
        m2: { children: [] },
        c1: { children: [] },
        c2: { children: ["f1"] },
        f1: { children: [] },
        f2: { children: [] },
      });
    });

    test("link panel adds a link to task by ID", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["m1", "m2", "c1", "c2"] },
        { id: "m1", name: "Milestone 1", children: ["f1", "f2"] },
        { id: "m2", name: "Milestone 2", children: [] },
        { id: "c1", name: "Component 1", children: [] },
        { id: "c2", name: "Component 2", children: [] },
        { id: "f1", name: "Feature 1", children: [] },
        { id: "f2", name: "Feature 2", children: [] },
      ]);

      await page.getByRole("button", { name: "Task m1 Toggle Expand" }).click();
      await page.getByTestId("Row m1/f2").click();

      await page.keyboard.press("Meta+/");
      await page.getByRole("option", { name: "Task c1 Command Item" }).click();

      await page.getByRole("button", { name: "Task c1 Toggle Expand" }).click();
      await expect(page.getByTestId("Row c1/f2")).toContainText("Milestone 1");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["m1", "m2", "c1", "c2"] },
        m1: { children: ["f1", "f2"] },
        m2: { children: [] },
        c1: { children: ["f2"] },
        c2: { children: [] },
        f1: { children: [] },
        f2: { children: [] },
      });
    });

    test("link panel adds a link root", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["m1", "m2", "c1", "c2"] },
        { id: "m1", name: "Milestone 1", children: ["f1", "f2"] },
        { id: "m2", name: "Milestone 2", children: [] },
        { id: "c1", name: "Component 1", children: [] },
        { id: "c2", name: "Component 2", children: [] },
        { id: "f1", name: "Feature 1", children: [] },
        { id: "f2", name: "Feature 2", children: [] },
      ]);

      await page.getByRole("button", { name: "Task m1 Toggle Expand" }).click();
      await page.getByRole("button", { name: "Task f1 Drag Handle" }).click();

      await page.keyboard.press("Meta+/");
      await page
        .getByRole("option", { name: "Task root Command Item" })
        .click();
      await expect(page.getByTestId("Row f1")).toContainText("Milestone 1");

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["m1", "m2", "c1", "c2", "f1"] },
        m1: { children: ["f1", "f2"] },
        m2: { children: [] },
        c1: { children: [] },
        c2: { children: [] },
        f1: { children: [] },
        f2: { children: [] },
      });
    });
  });

  test.describe("task tags", () => {
    test("clicking tags jump to the corresponding node", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["m1", "m2", "c1", "c2", "f3"] },
        { id: "m1", name: "Milestone 1", children: ["f1", "f2", "f3"] },
        { id: "m2", name: "Milestone 2", children: ["f3"] },
        { id: "c1", name: "Component 1", children: ["f1", "f2", "c2"] },
        { id: "c2", name: "Component 2", children: ["f2", "f3"] },
        { id: "f1", name: "Feature 1", children: [] },
        { id: "f2", name: "Feature 2", children: [] },
        { id: "f3", name: "Feature 3", children: [] },
      ]);

      await page
        .getByTestId("Row f3")
        .getByRole("button", { name: "Milestone 1" })
        .click();
      await expect(page.getByTestId("Row m1/f3")).toBeVisible();
      await expect(page.getByTestId("Row m1/f3")).toBeFocused();

      await page
        .getByTestId("Row m1/f3")
        .getByRole("button", { name: "Root" })
        .click();
      await expect(page.getByTestId("Row f3")).toBeVisible();
      await expect(page.getByTestId("Row f3")).toBeFocused();

      await page
        .getByTestId("Row m1/f2")
        .getByRole("button", { name: "Component 2" })
        .click();
      await expect(page.getByTestId("Row c2/f2")).toBeVisible();
      await expect(page.getByTestId("Row c2/f2")).toBeFocused();
      await expect(
        page
          .getByTestId("Row c2/f2")
          .getByRole("button", { name: "Milestone 1" }),
      ).toBeVisible();
      await expect(
        page
          .getByTestId("Row c2/f2")
          .getByRole("button", { name: "Component 1" }),
      ).toBeVisible();
    });

    test("clicking the delete button removes tags", async ({ page }) => {
      await init(
        page,
        [
          { id: "root", name: "Root", children: ["m1", "m2", "f3"] },
          { id: "m1", name: "Milestone 1", children: ["f1", "f2", "f3"] },
          { id: "m2", name: "Milestone 2", children: ["f3"] },
          { id: "f1", name: "Feature 1", children: [] },
          { id: "f2", name: "Feature 2", children: [] },
          { id: "f3", name: "Feature 3", children: [] },
        ],
        true,
      );

      await page
        .getByTestId("Row m2/f3")
        .getByRole("option")
        .filter({ hasText: "Milestone 1" })
        .getByRole("button", { name: "Delete chip" })
        .click();
      await expect(page.getByTestId("Row m1/f3")).not.toBeVisible();
    });
  });

  test.describe("changing assignee", () => {
    test("assign and unassign", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [] },
      ]);

      await page.getByRole("button", { name: "Unassigned" }).first().click();
      await page.getByRole("menuitem", { name: "-test" }).click();

      expect((await getKosoGraph(page))["1"].assignee).toContain(
        "-test@test.koso.app",
      );

      await page
        .getByRole("button", { name: "Pointy-Haired Boss" })
        .first()
        .click();
      await page.getByRole("menuitem", { name: "-test" }).isHidden();
      await page.getByRole("menuitem", { name: "Unassigned" }).click();

      expect((await getKosoGraph(page))["1"].assignee).toBeNull();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [], assignee: null },
      });
    });

    test("search and assign", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [] },
      ]);

      await page.getByRole("button", { name: "Unassigned" }).first().click();
      await page.getByRole("menuitem", { name: "Unassigned" }).isVisible();
      await page.getByRole("menuitem", { name: "-test" }).isVisible();
      // Query a user that doesn't exist.
      const notExistentQuery = "not";
      await page.keyboard.type(notExistentQuery);
      await page.getByRole("menuitem", { name: "Unassigned" }).isVisible();
      await page.getByRole("menuitem", { name: "-test" }).isHidden();
      // Clear the query
      [...notExistentQuery].forEach(() => page.keyboard.press("Backspace"));
      await page.getByRole("menuitem", { name: "Unassigned" }).isVisible();
      await page.getByRole("menuitem", { name: "-test" }).isVisible();

      // Query for the user that does exist
      await page.keyboard.type("pointy");
      await page.getByRole("menuitem", { name: "Unassigned" }).isVisible();
      await page.getByRole("menuitem", { name: "-test" }).isVisible();
    });
  });

  test.describe("changing reporter", () => {
    test("assign and unassign", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: [] },
      ]);

      await page.getByRole("button", { name: "Unassigned" }).nth(1).click();
      await page.getByRole("menuitem", { name: "-test" }).click();

      expect((await getKosoGraph(page))["1"].reporter).toContain(
        "-test@test.koso.app",
      );

      await page
        .getByRole("button", { name: "Pointy-Haired Boss" })
        .first()
        .click();
      await page.getByRole("menuitem", { name: "-test" }).isHidden();
      await page.getByRole("menuitem", { name: "Unassigned" }).click();

      expect((await getKosoGraph(page))["1"].reporter).toBeNull();

      expect(await getKosoGraph(page)).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [], reporter: null },
      });
    });
  });

  test.describe("juggling tasks", () => {
    test("block and unblock a task by resolving children", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", children: [] },
        { id: "3", children: [] },
      ]);

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();

      let statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 1" })
        .getByRole("menuitem", { name: "Juggled" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Blocked");
      let graph = await getKosoGraph(page);
      expect(graph).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "3"], status: "Blocked" },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
      expect(graph["1"].assignee).toContain("-test@test.koso.app");

      statusButton = page
        .getByRole("row", { name: "Task 2" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 2" })
        .getByRole("menuitem", { name: "Done" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 2" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Done");

      statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Blocked");

      statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 3" })
        .getByRole("menuitem", { name: "Done" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 3" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Done");

      statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      graph = await getKosoGraph(page);
      expect(graph).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["3", "2"], status: "Not Started" },
        ["2"]: { children: [], status: "Done" },
        ["3"]: { children: [], status: "Done" },
      });
      expect(graph["1"].assignee).toContain("-test@test.koso.app");
    });

    test("block and unblock a task by removing children", async ({ page }) => {
      await init(page, [
        { id: "root", name: "Root", children: ["1"] },
        { id: "1", children: ["2", "3"] },
        { id: "2", children: [], status: "Done" },
        { id: "3", children: [] },
      ]);

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();

      let statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("50% In Progress");
      statusButton.click();
      page
        .getByRole("row", { name: "Task 1" })
        .getByRole("menuitem", { name: "Juggled" })
        .click();
      statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Blocked");
      let graph = await getKosoGraph(page);
      expect(graph).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2", "3"], status: "Blocked" },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
      expect(graph["1"].assignee).toContain("-test@test.koso.app");

      await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();
      await page.getByRole("button", { name: "Delete" }).click();

      statusButton = page
        .getByRole("row", { name: "Task 1" })
        .getByRole("button", { name: "task-status" });
      await expect(statusButton).toBeVisible();
      await expect(statusButton).toHaveText("Not Started");
      graph = await getKosoGraph(page);
      expect(graph["1"].assignee).toContain("-test@test.koso.app");

      await expectKosoGraph(page).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2"], status: "Not Started" },
        ["2"]: { children: [], status: "Done" },
      });
    });
  });
});
