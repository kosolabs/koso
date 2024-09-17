import { expect, request, test, type Page } from "@playwright/test";

test.describe.configure({ mode: "serial" });

let page: Page;

test.beforeAll(async ({ browser }) => {
  page = await browser.newPage();
});

test("home page presents login header and button", async () => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Koso");
});

test("log user in and view projects", async () => {
  await page.goto("/");

  const login_url = `/api/auth/login`;
  const apiContext = await request.newContext({});
  const email = `${Math.random().toString(36).slice(2)}-test@test.koso.app`;
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
  await expect(page.getByText("Create your first Koso project!")).toBeVisible();
});

test("create a project and rename it to Integration Test Project", async () => {
  await page.getByRole("button", { name: "new project" }).click();

  await page.getByTestId("set-project-name-button").click();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.type("Integration Test Project");
  await page.keyboard.press("Enter");

  await expect(page.getByTestId("set-project-name-button")).toHaveText(
    "Integration Test Project",
  );
});

function jwt(email: string) {
  const header = {
    alg: "HS256",
    typ: "JWT",
    kid: "koso-integration-test",
  };
  const encodedHeader = btoa(JSON.stringify(header));
  const expirationEpochSeconds = Math.floor(
    (Date.now() + 24 * 60 * 60 * 1000) / 1000,
  );
  const payload = {
    email: email,
    name: "Pointy-Haired Boss",
    picture: "https://static.wikia.nocookie.net/dilbert/images/6/60/Boss.PNG",
    exp: expirationEpochSeconds,
  };
  const encodedSignature = btoa("test_signature_cannot_validate");
  const encodedPayload = btoa(JSON.stringify(payload));
  return `${encodedHeader}.${encodedPayload}.${encodedSignature}`;
}
