import { expect, test } from "@playwright/test";

test.describe("password reset UI", () => {
  test("prefills email and OTP from reset link query params", async ({ page }) => {
    const email = "prefill+e2e@example.com";
    const code = "654321";

    await page.goto(`/reset-password?email=${encodeURIComponent(email)}&code=${code}`);

    await expect(page.locator("#reset-confirm-email")).toHaveValue(email);
    await expect(page.locator("#reset-confirm-code")).toHaveValue(code);
    await expect(page.getByTestId("password-reset-link-detected")).toBeVisible();
  });

  test("shows mismatch validation and does not submit API request", async ({ page }) => {
    await page.goto("/reset-password");

    let resetRequestCount = 0;
    page.on("request", (request) => {
      if (request.url().includes("/api/auth/reset-password")) {
        resetRequestCount += 1;
      }
    });

    await page.fill("#reset-confirm-email", "mismatch@example.com");
    await page.fill("#reset-confirm-code", "123456");
    await page.fill("#reset-new-password", "ValidPassword123!");
    await page.fill("#reset-confirm-password", "DifferentPassword456!");
    await page.click("button[type='submit']");

    await expect(page.getByTestId("password-reset-confirm-mismatch")).toBeVisible();
    expect(resetRequestCount).toBe(0);
  });
});
