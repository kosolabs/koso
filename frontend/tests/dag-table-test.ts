import type { Graph, Status } from "$lib/koso";
import { expect, test, type Page } from "@playwright/test";
import { setupNewProject, tearDown } from "./utils";

test.describe.configure({ mode: "serial" });

test.describe.serial("dag table tests", () => {
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await setupNewProject(browser);
  });

  test.afterAll(async () => {
    await tearDown(page);
  });

  async function getKosoGraph(): Promise<Graph> {
    return page.evaluate("koso.toJSON()");
  }

  async function clear() {
    await page.evaluate(() => {
      window.koso.yGraph.clear();
      window.koso.upsertRoot();
    });
    await page.reload();
  }

  type TaskBuilder = {
    id: string;
    num?: string;
    name?: string;
    children?: string[];
    assignee?: string | null;
    reporter?: string | null;
    status?: Status | null;
  };

  async function init(tasks: TaskBuilder[]) {
    await page.evaluate((tasks) => {
      window.koso.yDoc.transact(() => {
        window.koso.yGraph.clear();
        for (const {
          id = "root",
          num = id,
          name = "",
          children = [],
          assignee = null,
          reporter = null,
          status = null,
        } of tasks) {
          window.koso.upsert({
            id,
            num,
            name,
            children,
            assignee,
            reporter,
            status,
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

  function getTaskNumToTaskIdMap(graph: Graph) {
    const result: { [num: string]: string } = {};
    for (const [id, task] of Object.entries(graph)) {
      result[task.num] = id;
    }
    return result;
  }

  test.describe("creating tasks", () => {
    test("create a task by clicking the Add Task button", async () => {
      await clear();

      await page.getByRole("button", { name: "Add Task" }).click();
      await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

      const graph = await getKosoGraph();
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    });

    test("create a task by presing Shift+Enter on the task", async () => {
      await init([{ id: "root", children: ["1"] }, { id: "1" }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Shift+Enter");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      const graph = await getKosoGraph();
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
    });
  });

  test.describe("deleting tasks", () => {
    test("delete task 2 by clicking the Delete button", async () => {
      await init([
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.getByRole("button", { name: "Delete" }).click();
      expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("delete task 2 by pressing the Delete key on the task", async () => {
      await init([
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "3"] },
        ["1"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("create a task by presing Shift+Enter on the task", async () => {
      await init([{ id: "root", children: ["1"] }, { id: "1" }]);

      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      await page.keyboard.press("Shift+Enter");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      const graph = await getKosoGraph();
      const tasks = getTaskNumToTaskIdMap(graph);
      expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
    });
  });

  test.describe("selecting tasks", () => {
    test("select task 1 by clicking on the drag handle", async () => {
      await init([{ id: "root", children: ["1"] }, { id: "1" }]);
      await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
      expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();
    });
  });

  test.describe("editing tasks", () => {
    test("set task 1's name by clicking Click to edit", async () => {
      await init([{ id: "root", children: ["1"] }, { id: "1" }]);
      await page.getByRole("button", { name: "Task 1 Edit Name" }).click();
      await page.keyboard.type("The 1st Task");
      await page.keyboard.press("Enter");

      await expect(
        page.getByRole("button", { name: "Task 1 Edit Name" }),
      ).toHaveText("The 1st Task");
    });

    test("set task 2's name by pressing Enter", async () => {
      await init([
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
    test("up and down arrows change the selected row", async () => {
      await init([
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();

      await page.keyboard.press("ArrowUp");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("option up and down arrows change the order of rows", async () => {
      await init([
        { id: "root", children: ["1", "2", "3"] },
        { id: "1" },
        { id: "2" },
        { id: "3" },
      ]);

      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      await page.keyboard.press("Alt+ArrowDown");
      await expect(page.getByRole("row", { name: "Task 1" })).toBeFocused();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["2", "1", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("option left and right change row indentation", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: ["2"] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });

      await page.keyboard.press("ArrowUp");
      await expect(page.getByRole("row", { name: "Task 2" })).toBeFocused();
      await page.keyboard.press("Alt+ArrowLeft");

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["3"] },
        ["3"]: { children: [] },
      });
    });
  });

  test.describe("expand and collapse", () => {
    test("expanding task 1 reveals task 2 and collapsing hides it", async () => {
      await init([
        { id: "root", children: ["1"] },
        { id: "1", children: ["2"] },
        { id: "2" },
      ]);
      await expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();
    });
  });

  test.describe("undo and redo", () => {
    test("clicking the undo and redo restores and deletes a task", async () => {
      await init([
        { id: "root", children: ["1", "2"] },
        { id: "1" },
        { id: "2" },
      ]);

      await page.waitForTimeout(50);
      await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
      await page.keyboard.press("Delete");
      await expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();

      await page.waitForTimeout(50);
      await page.getByRole("button", { name: "Undo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "2"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
      });

      await page.waitForTimeout(50);
      await page.getByRole("button", { name: "Redo" }).click();
      await expect(page.getByRole("row", { name: "Task 2" })).not.toBeVisible();

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1"] },
        ["1"]: { children: [] },
      });
    });
  });

  test.describe("drag and drop", () => {
    test("drag task 1 to peer of task 2 shows drop target", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["2", "1", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("drag task 1 to child of task 2 shows drop target", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });
    });

    test("move task 1 to child of task 2 and link as child of task 3", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: ["1"] },
      });
    });

    test("cannot make task 1 child of itself", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: [] },
        ["3"]: { children: [] },
      });
    });

    test("cannot make task 1 a peer of itself", async () => {
      await init([
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

      expect(await getKosoGraph()).toMatchObject({
        root: { children: ["1", "2", "3"] },
        ["1"]: { children: [] },
        ["2"]: { children: ["1"] },
        ["3"]: { children: [] },
      });
    });
  });
});
