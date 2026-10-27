import { expect, test, type Page } from "@playwright/test";
async function authorize(page: Page) {
  await page
    .getByLabel("Client ID", { exact: true })
    .fill("test.apps.googleusercontent.com");
  await page
    .getByLabel("Client secret", { exact: true })
    .fill("only-a-fixture-secret");
  await page
    .getByRole("button", { name: "Connect with Google", exact: true })
    .click();
  await expect(
    page.getByText(
      "Your Google account is connected. Continue to choose your folder.",
    ),
  ).toBeVisible();
}
async function toAccount(page: Page, scenario = "fresh") {
  await page.goto(`/?scenario=${scenario}`);
  await page.getByRole("button", { name: "Get Started" }).click();
  await page.getByRole("button", { name: "Continue", exact: true }).click();
}
async function toReview(page: Page) {
  await authorize(page);
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await page
    .getByLabel("Drive folder", { exact: true })
    .fill("/home/demo/My Drive 文");
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await page.getByRole("button", { name: "Continue", exact: true }).click();
}
test("complete wizard, persistent connection, dashboard controls, settings and diagnostics", async ({
  page,
}) => {
  await toAccount(page);
  await toReview(page);
  await page
    .getByRole("button", { name: "Connect Drive", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Your Google Drive is ready!" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Go to Dashboard" }).click();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
  await expect(
    page.getByText("/home/demo/My Drive 文", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Disconnect", exact: true }).click();
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(page.getByText("Disconnected", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Reconnect", exact: true }).click();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Connection settings." }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Troubleshooting", exact: true })
    .click();
  await expect(page.getByLabel("Redacted diagnostic report")).toContainText(
    "No real system actions",
  );
  await page.reload();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
});
test("existing manual connection requires consent and preserves external settings", async ({
  page,
}) => {
  await page.goto("/?scenario=existing");
  await page
    .getByRole("button", { name: "Manage existing connection", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Manage existing connection", exact: true })
    .click();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await expect(
    page.getByText(/This connection uses an existing service/),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Renew Google connection" }),
  ).toBeDisabled();
});
test("missing dependencies are installed only after approval", async ({
  page,
}) => {
  await page.goto("/?scenario=missing");
  await page.getByRole("button", { name: "Get Started" }).click();
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeDisabled();
  await page
    .getByRole("button", { name: "Install Required Components" })
    .click();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeDisabled();
  await page
    .getByRole("button", { name: "Install Required Components" })
    .click();
  await page.getByRole("button", { name: "Approve installation" }).click();
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeEnabled();
});
test("OAuth failure has a retry path and cannot advance", async ({ page }) => {
  await toAccount(page, "oauth-failure");
  await page
    .getByLabel("Client ID", { exact: true })
    .fill("test.apps.googleusercontent.com");
  await page
    .getByLabel("Client secret", { exact: true })
    .fill("only-a-fixture-secret");
  await page
    .getByRole("button", { name: "Connect with Google", exact: true })
    .click();
  await expect(
    page.getByText(
      "Google authorization failed. Check your credentials and try again.",
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Connect with Google", exact: true }),
  ).toBeEnabled();
});
test("failed mount never shows success", async ({ page }) => {
  await toAccount(page, "mount-failure");
  await toReview(page);
  await page
    .getByRole("button", { name: "Connect Drive", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText(
    "Google Drive did not appear",
  );
  await expect(
    page.getByRole("heading", { name: "Your Google Drive is ready!" }),
  ).not.toBeVisible();
});
test("offline startup blocks new authorization", async ({ page }) => {
  await page.goto("/?scenario=offline");
  await page.getByRole("button", { name: "Get Started" }).click();
  await expect(
    page.getByText("Needs attention", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Continue", exact: true }),
  ).toBeDisabled();
});
test("pending uploads block disconnect", async ({ page }) => {
  await toAccount(page, "pending");
  await toReview(page);
  await page
    .getByRole("button", { name: "Connect Drive", exact: true })
    .click();
  await page.getByRole("button", { name: "Go to Dashboard" }).click();
  await page.getByRole("button", { name: "Disconnect", exact: true }).click();
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("unfinished uploads");
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
});
test("welcome light and dark layouts fit the supported minimum width", async ({
  page,
}) => {
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: /Bring Google Drive/ }),
  ).toBeVisible();
  await page.screenshot({
    path: "test-results/welcome-light.png",
    fullPage: true,
    animations: "disabled",
  });
  await page.getByRole("button", { name: "Dark theme", exact: true }).click();
  await page.screenshot({
    path: "test-results/welcome-dark.png",
    fullPage: true,
    animations: "disabled",
  });
  await page.setViewportSize({ width: 760, height: 680 });
  await expect(page.getByRole("button", { name: "Get Started" })).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
