import type { Graph } from "$lib/koso";
import { expect, request, test, type Page } from "@playwright/test";

test.describe.configure({ mode: "serial" });

test.describe.serial("all tests", () => {
  let page: Page;

  const getKosoGraph = async (): Promise<Graph> =>
    page.evaluate("koso.toJSON()");

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.goto("/");

    const apiContext = await request.newContext({});
    const token = jwt(`cleanup-test@test.koso.app`);
    const res = await apiContext.post("/api/dev/cleanup_test_data", {
      data: {},
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    expect(res.ok()).toBeTruthy();
  });

  test("home page presents login header and button", async () => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Koso");
  });

  test("log user in and view projects", async () => {
    await page.goto("/");

    const login_url = `/api/auth/login`;
    const apiContext = await request.newContext({});
    // Avoid test cross-talk by logging in with a randomly generated user.
    const email = `${Math.random().toString(36).slice(2)}-${Date.now()}-test@test.koso.app`;
    const token = jwt(email);
    const res = await apiContext.post(login_url, {
      data: {},
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    expect(res.ok()).toBeTruthy();

    await page.evaluate(
      ([token]) => window.localStorage.setItem("credential", token),
      [token],
    );

    await page.goto("/projects");
    await expect(
      page.getByText("Create your first Koso project!"),
    ).toBeVisible();
  });

  test("create a project and rename it to Integration Test Project", async () => {
    await page.getByRole("button", { name: "New Project" }).click();

    await page.getByRole("button", { name: "Set Project Name" }).click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type("Integration Test Project");
    await page.keyboard.press("Enter");

    await expect(
      page.getByRole("button", { name: "Set Project Name" }),
    ).toHaveText("Integration Test Project");
  });

  test("insert a task by clicking the Add Task button", async () => {
    await page.getByRole("button", { name: "Add Task" }).click();
    await expect(page.getByRole("row", { name: "Task 1" })).toBeVisible();

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"]]);
  });

  test("set the task name by clicking Click to edit", async () => {
    await page.getByRole("button", { name: "Task 1 Edit Name" }).click();
    await page.keyboard.type("The 1st Task");
    await page.keyboard.press("Enter");

    await expect(
      page.getByRole("button", { name: "Task 1 Edit Name" }),
    ).toHaveText("The 1st Task");
  });

  test("insert a task by pressing Shift+Enter on the task", async () => {
    await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
    await page.keyboard.press("Shift+Enter");
    await expect(page.getByRole("row", { name: "Task 2" })).toBeVisible();

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
  });

  test("set the task name by pressing Enter", async () => {
    await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
    await page.keyboard.press("Enter");
    await page.keyboard.type("The 2nd Task");
    await page.keyboard.press("Enter");

    await expect(
      page.getByRole("button", { name: "Task 2 Edit Name" }),
    ).toHaveText("The 2nd Task");
  });

  test("insert a child task by clicking on a row and clicking add child", async () => {
    await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
    await page.getByRole("button", { name: "Add Child" }).click();

    await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph[tasks["1"]].children).toContain(tasks["3"]);
  });

  test("collapsing task 1 hides task 3 and expanding reveals it again", async () => {
    await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
    await expect(page.getByRole("row", { name: "Task 3" })).not.toBeVisible();

    await page.getByRole("button", { name: "Task 1 Toggle Expand" }).click();
    await expect(page.getByRole("row", { name: "Task 3" })).toBeVisible();
  });

  test("undent task 3 using the mouse", async () => {
    await page.getByRole("button", { name: "Task 3 Drag Handle" }).click();
    await page.getByRole("button", { name: "Undent" }).click();

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([
      tasks["1"],
      tasks["3"],
      tasks["2"],
    ]);
  });

  test("indent task 2 using the mouse", async () => {
    await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
    await page.getByRole("button", { name: "Indent" }).click();

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["3"]]);
    expect(graph[tasks["3"]].children).toStrictEqual([tasks["2"]]);
  });

  test("undent task 2 using the keyboard", async () => {
    await page.getByRole("button", { name: "Task 2 Drag Handle" }).click();
    await page.keyboard.press("Alt+ArrowLeft");

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([
      tasks["1"],
      tasks["3"],
      tasks["2"],
    ]);
  });

  test("select and indent task 3 using the keyboard", async () => {
    await page.keyboard.press("ArrowUp");
    await page.keyboard.press("Alt+ArrowRight");

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"], tasks["2"]]);
    expect(graph[tasks["1"]].children).toStrictEqual([tasks["3"]]);
  });

  test("move task 1 down using the keyboard", async () => {
    await page.getByRole("button", { name: "Task 1 Drag Handle" }).click();
    await page.keyboard.press("Alt+ArrowDown");

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["2"], tasks["1"]]);
  });

  test("make task 2 the child of task 3 using the mouse", async () => {
    page.getByRole("button", { name: "Task 3 Child Dropzone" });
    await page
      .getByRole("button", { name: "Task 2 Drag Handle" })
      .dragTo(page.getByRole("button", { name: "Task 3 Child Dropzone" }));

    const graph = await getKosoGraph();
    const tasks = getTaskNumToTaskIdMap(graph);
    expect(graph["root"].children).toStrictEqual([tasks["1"]]);
    expect(graph[tasks["1"]].children).toStrictEqual([tasks["3"]]);
    expect(graph[tasks["3"]].children).toStrictEqual([tasks["2"]]);
  });
});

function getTaskNumToTaskIdMap(graph: Graph) {
  const result: { [num: string]: string } = {};
  for (const [id, task] of Object.entries(graph)) {
    result[task.num] = id;
  }
  return result;
}

function jwt(email: string) {
  const base64 = (s: string) => Buffer.from(s).toString("base64url");
  const header = {
    alg: "HS256",
    typ: "JWT",
    kid: "koso-integration-test",
  };
  const encodedHeader = base64(JSON.stringify(header));
  const expirationEpochSeconds = Math.floor(
    (Date.now() + 24 * 60 * 60 * 1000) / 1000,
  );
  const payload = {
    email: email,
    name: "Pointy-Haired Boss",
    picture: "https://static.wikia.nocookie.net/dilbert/images/6/60/Boss.PNG",
    exp: expirationEpochSeconds,
  };
  const encodedSignature = base64("test_signature_cannot_validate");
  const encodedPayload = base64(JSON.stringify(payload));
  return `${encodedHeader}.${encodedPayload}.${encodedSignature}`;
}
