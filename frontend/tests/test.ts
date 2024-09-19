import { expect, request, test, type Page } from "@playwright/test";

test.describe.configure({ mode: "serial" });

test.describe.serial("all tests", () => {
  let page: Page;

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
    await page.getByRole("button", { name: "new project" }).click();

    await page.getByRole("button", { name: "set project name" }).click();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.type("Integration Test Project");
    await page.keyboard.press("Enter");

    await expect(
      page.getByRole("button", { name: "set project name" }),
    ).toHaveText("Integration Test Project");
  });
});

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
