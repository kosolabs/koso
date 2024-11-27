import { expect, request, test } from "@playwright/test";
import { generateEmail, jwt, login, tearDown } from "./utils";

test.describe.configure({ mode: "parallel" });

test.describe("poll github tests", () => {
  test.beforeEach(async ({ page }) => {
    await login(page, generateEmail());
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test("poll github prs", async () => {
    const apiContext = await request.newContext({});
    const token = jwt(`cleanup-test@test.koso.app`);
    const res = await apiContext.get("/api/poll/github/prs", {
      data: {},
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    expect(res.ok()).toBeTruthy();
    expect(await res.json()).toBeDefined();
  });
});
