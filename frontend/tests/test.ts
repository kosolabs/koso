import { expect, test, request } from "@playwright/test";

test("home page presents login header and button", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Koso");
});

test("log user in and view projects", async ({ page }) => {
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
