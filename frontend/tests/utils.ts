import type { Graph, Status } from "$lib/yproxy";
import { expect, request, type Page } from "@playwright/test";

export type TaskBuilder = {
  id: string;
  num?: string;
  name?: string;
  children?: string[];
  assignee?: string | null;
  reporter?: string | null;
  status?: Status | null;
  statusTime?: number | null;
  url?: string | null;
  kind?: string | null;
};

export async function getKosoGraph(page: Page): Promise<Graph> {
  return page.evaluate("koso.toJSON()");
}

export async function expectNothingFocused(page: Page) {
  const isFocusOnBody = await page.evaluate(
    () => document.activeElement === document.body,
  );
  expect(isFocusOnBody).toBeTruthy();
}

export function expectKosoGraph(page: Page) {
  return expect.poll(
    async () => {
      return await getKosoGraph(page);
    },
    {
      timeout: 6000,
      intervals: [50],
    },
  );
}

export async function getKosoProjectId(page: Page): Promise<Graph> {
  return page.evaluate("koso.projectId");
}

export async function clear(page: Page) {
  await page.evaluate("koso.clear()");
  await page.reload();
}

export function getTaskNumToTaskIdMap(graph: Graph) {
  const result: { [num: string]: string } = {};
  for (const [id, task] of Object.entries(graph)) {
    result[task.num] = id;
  }
  return result;
}

export function jwt(email: string) {
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

export function generateEmail() {
  return `${Math.random().toString(36).slice(2)}-${Date.now()}-test@test.koso.app`;
}

export async function setupNewProject(page: Page): Promise<Page> {
  await login(page, generateEmail());

  await page.goto("/projects");
  await page.getByRole("button", { name: "New", exact: true }).click();
  // Make sure things are initialized before proceeding
  await expect(page.getByRole("button", { name: "Add Task" })).toBeVisible();

  return page;
}

export async function login(page: Page, email: string, invite: boolean = true) {
  await page.goto("/");
  const loginUrl = `/api/auth/login`;
  const apiContext = await request.newContext({});
  const token = jwt(email);
  const res = await apiContext.post(loginUrl, {
    data: {},
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  expect(res.ok()).toBeTruthy();

  if (invite) {
    const inviteRes = await apiContext.post("/api/dev/invite_test_user", {
      data: {},
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    expect(inviteRes.ok()).toBeTruthy();
  }

  await page.evaluate(
    ([token]) => window.localStorage.setItem("credential", token),
    [token],
  );
}

export async function tearDown() {
  const apiContext = await request.newContext({});
  const token = jwt(`cleanup-test@test.koso.app`);
  const res = await apiContext.post("/api/dev/cleanup_test_data", {
    data: {},
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  expect(res.ok()).toBeTruthy();
}

export async function init(
  page: Page,
  tasks: TaskBuilder[],
  expandAll: boolean = false,
) {
  await page.evaluate(
    ({ tasks, expandAll }) => {
      const koso = window.koso;

      const upsertedTaskIds = new Set<string>(tasks.map((task) => task.id));
      const childTaskIds = new Set<string>(
        tasks.flatMap((task) => task.children ?? []),
      );
      const remainingTaskIds = childTaskIds.difference(upsertedTaskIds);

      koso.doc.transact(() => {
        for (const task of tasks) {
          koso.upsert({
            id: task.id,
            num: task.num ?? task.id,
            name: task.name ?? "",
            children: task.children ?? [],
            assignee: task.assignee ?? null,
            reporter: task.reporter ?? null,
            status: task.status ?? null,
            statusTime: task.statusTime ?? null,
            kind: task.kind ?? null,
            url: task.url ?? null,
          });
        }
        for (const taskId of remainingTaskIds) {
          koso.upsert({
            id: taskId,
            num: taskId,
            name: "",
            children: [],
            assignee: null,
            reporter: null,
            status: null,
            statusTime: null,
            kind: null,
            url: null,
          });
        }
      });
      if (expandAll) {
        koso.expandAll();
      } else {
        koso.collapseAll();
      }
    },
    { tasks, expandAll },
  );
  await page.reload();
  await page.getByLabel("Home").focus();
}
