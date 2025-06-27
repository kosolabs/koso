import { expect, test, type Page } from "@playwright/test";
import { generateEmail, login, tearDown } from "./utils";

test.describe("subscription tests", () => {
  test.describe.configure({ mode: "serial" });

  let page: Page;
  let email: string;
  let otherEmail: string;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
    email = generateEmail();
    otherEmail = generateEmail();
  });

  test.afterAll(async () => {
    await tearDown();
  });

  test("log user in and view profile", async () => {
    await page.goto("/");

    await login(page, email, false);

    await page.goto("/profile");
    await expect(
      page.getByText("Profile Settings for Pointy-Haired Boss"),
    ).toBeVisible();
  });

  test("start a new subscription and navigate back to profile", async () => {
    // Navigate to the stripe portal
    await page.getByRole("button", { name: "Subscribe" }).click();
    await expect(page.getByText("Koso Labs sandbox").first()).toBeVisible({
      timeout: 30 * 1000,
    });
    await expect(page.getByText(email)).toBeVisible();

    // Set the quantity to 5.
    await page.getByTestId("line-item-adjustable-qty").click();
    await page.getByRole("textbox", { name: "quantity" }).fill("5");
    await page.getByRole("button", { name: "Update" }).click();

    // Fill the form
    await page.getByTestId("card-accordion-item").click();
    await page
      .getByRole("textbox", { name: "Card number" })
      .fill("4242424242424242");
    await page.getByRole("textbox", { name: "Expiration" }).fill("03/45");
    await page.getByRole("textbox", { name: "CVC" }).fill("333");
    await page
      .getByRole("textbox", { name: "Cardholder name" })
      .fill("SubscriptionTest User");
    await page.getByRole("textbox", { name: "Zip" }).fill("84109");
    await page
      .getByRole("checkbox", { name: "Save my information for" })
      .click();

    // Submit
    await page.getByRole("button", { name: "Subscribe" }).click();

    // Wait to be redirected back to the profile page
    await expect(
      page.getByText("Profile Settings for Pointy-Haired Boss"),
    ).toBeVisible({ timeout: 30 * 1000 });
  });

  test("verify subscription status", async () => {
    await expect(
      page.getByText("You have a premium subscription"),
    ).toBeVisible();
    await expect(page.getByText("You have 4 remaining seats.")).toBeVisible();
    await expect(
      page.getByRole("button", {
        name: email,
        exact: true,
      }),
    ).toBeVisible();
  });

  test("add a new member", async () => {
    // Add a new member
    await page
      .getByRole("textbox", { name: "List of members" })
      .fill(otherEmail);
    await page.keyboard.press("Enter");
    // Verify the subscription was updated
    await expect(page.getByText("Subscription members updated.")).toBeVisible();
    await expect(
      page.getByRole("button", {
        name: otherEmail,
        exact: true,
      }),
    ).toBeVisible();
    await expect(page.getByText("You have 3 remaining seats.")).toBeVisible();
  });

  test("add members and fill all seats", async () => {
    // Add 3 additional members to fill all seats.
    // other-1
    await page
      .getByRole("textbox", { name: "List of members" })
      .fill("other-1");
    await page.keyboard.press("Enter");
    await expect(page.getByText("You have 2 remaining seats.")).toBeVisible();
    // other-2
    await page
      .getByRole("textbox", { name: "List of members" })
      .fill("other-2");
    await page.keyboard.press("Enter");
    await expect(page.getByText("You have 1 remaining seat.")).toBeVisible();
    // other-3
    await page
      .getByRole("textbox", { name: "List of members" })
      .fill("other-3");
    await page.keyboard.press("Enter");

    // Verify that all seats are filled.
    await expect(page.getByText("All seats (5) are in use")).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "List of members" }),
    ).toBeDisabled();
  });

  test("remove a member", async () => {
    await page
      .getByRole("option", { name: `${otherEmail} Delete chip` })
      .getByRole("button", { name: "Delete chip" })
      .click();

    await expect(
      page.getByRole("button", {
        name: otherEmail,
        exact: true,
      }),
    ).toBeHidden();
    await expect(page.getByText("You have 1 remaining seat.")).toBeVisible();
  });

  test("manage the existing subscription and decrease quantity by 1", async () => {
    // Navigate back to the stripe portal
    await page.getByRole("button", { name: "Manage" }).click();
    await expect(page.getByText("Koso Labs sandbox").first()).toBeVisible({
      timeout: 30 * 1000,
    });
    await expect(page.getByText(email)).toBeVisible();

    // Update the quantity of seats from 5 to 4.
    // There should already be 4 members, leaving all seats filled.
    await page.locator('[data-test="update-subscription"]').click();
    await page.getByTestId("portal-quantity-editor").fill("4");
    await page.getByTestId("continue-button").click();
    await page.getByTestId("confirm").click();
    await expect(
      page.locator('[data-test="update-subscription"]'),
    ).toBeVisible();

    // Return to Koso
    await page.getByTestId("return-to-business-link").click();

    // Verify the modified subscription.
    await expect(
      page.getByText("Profile Settings for Pointy-Haired Boss"),
    ).toBeVisible({ timeout: 30 * 1000 });
    await expect(page.getByText("All seats (4) are in use")).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "List of members" }),
    ).toBeDisabled();
  });
});
