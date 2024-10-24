import type { Graph } from "$lib/koso";
import { expect, request, type Page } from "@playwright/test";

export async function getKosoGraph(page: Page): Promise<Graph> {
  return page.evaluate("koso.toJSON()");
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

export function getTaskId(graph: Graph, num: string): string {
  for (const [id, task] of Object.entries(graph)) {
    if (task.num === num) {
      return id;
    }
  }
  throw new Error(`Could not find Task ID for ${num}`);
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
  await page.getByRole("button", { name: "New Project" }).click();
  await expect(
    page.getByRole("button", { name: "Set Project Name" }),
  ).toBeVisible();

  return page;
}

export async function login(page: Page, email: string) {
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
  await expect(res.ok()).toBeTruthy();

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
  await expect(res.ok()).toBeTruthy();
}
