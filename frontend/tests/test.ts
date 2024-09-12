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
  const token =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9zdGF0aWMud2lraWEubm9jb29raWUubmV0L2RpbGJlcnQvaW1hZ2VzLzYvNjAvQm9zcy5QTkciLCJleHAiOjIwMjQ3ODgwMTR9.3btheBY5h0nQRpWNODfYWQ_mMc26551178jrSDmpv_c";
  const res = await apiContext.post(login_url, {
    data: {},
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  expect(res.ok()).toBeTruthy();

  await page.evaluate(() =>
    window.localStorage.setItem(
      "credential",
      "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9zdGF0aWMud2lraWEubm9jb29raWUubmV0L2RpbGJlcnQvaW1hZ2VzLzYvNjAvQm9zcy5QTkciLCJleHAiOjIwMjQ3ODgwMTR9.3btheBY5h0nQRpWNODfYWQ_mMc26551178jrSDmpv_c",
    ),
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
