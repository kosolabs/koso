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
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9kcml2ZS5nb29nbGUuY29tL2ZpbGUvZC8xM0xiUFRfT3I1dUFVQnc0b1ZkNi1ja1pidVRxNkQ3Sy0vcHJldmlldyIsImV4cCI6MjAyNDc4ODAxNH0.6xthU8Bv-2BftYts0jKDJIscyy0ZqQ6RzaJ0W_-wvgo";
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
      "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9kcml2ZS5nb29nbGUuY29tL2ZpbGUvZC8xM0xiUFRfT3I1dUFVQnc0b1ZkNi1ja1pidVRxNkQ3Sy0vcHJldmlldyIsImV4cCI6MjAyNDc4ODAxNH0.6xthU8Bv-2BftYts0jKDJIscyy0ZqQ6RzaJ0W_-wvgo",
    ),
  );

  await page.goto("/projects");
  await expect(page.getByText("Create your first Koso project!")).toBeVisible();
});
