import { test, expect } from "@playwright/test";

test.describe("Theme Toggle", () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
  });

  test("should display theme toggle button on landing page", async ({ page }) => {
    await page.goto("/");
    const themeToggle = page.locator('[data-testid="theme-toggle"]');
    await expect(themeToggle).toBeVisible();
  });

  test("should display theme toggle button on login page", async ({ page }) => {
    await page.goto("/login");
    const themeToggle = page.locator('[data-testid="theme-toggle"]');
    await expect(themeToggle).toBeVisible();
  });

  test("should switch to dark mode when clicking toggle in light mode", async ({ page }) => {
    await page.goto("/");

    const html = page.locator("html");

    // In light mode, only the dark-switch button (moon) is visible
    const darkModeBtn = page.locator('[data-testid="theme-dark"]');
    await expect(darkModeBtn).toBeVisible();
    await darkModeBtn.click();

    await expect(html).toHaveAttribute("data-theme", "dark");
  });

  test("should switch back to light mode after switching to dark", async ({ page }) => {
    await page.goto("/");

    // Switch to dark mode first
    await page.locator('[data-testid="theme-dark"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

    // In dark mode, only the light-switch button (sun) is visible
    const lightModeBtn = page.locator('[data-testid="theme-light"]');
    await expect(lightModeBtn).toBeVisible();
    await lightModeBtn.click();

    const html = page.locator("html");
    const theme = await html.getAttribute("data-theme");
    expect(theme).not.toBe("dark");
  });

  test("should persist theme preference in localStorage", async ({ page }) => {
    await page.goto("/");

    // Switch to dark mode
    await page.locator('[data-testid="theme-dark"]').click();

    const storedTheme = await page.evaluate(() => localStorage.getItem("auth9-theme"));
    expect(storedTheme).toBe("dark");

    await page.reload();

    const html = page.locator("html");
    await expect(html).toHaveAttribute("data-theme", "dark");
  });

  test("should persist light theme preference after reload", async ({ page }) => {
    await page.goto("/");

    // Switch to dark, then back to light
    await page.locator('[data-testid="theme-dark"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
    await page.locator('[data-testid="theme-light"]').click();

    const storedTheme = await page.evaluate(() => localStorage.getItem("auth9-theme"));
    expect(storedTheme).toBe("light");

    await page.reload();

    const html = page.locator("html");
    const theme = await html.getAttribute("data-theme");
    expect(theme).not.toBe("dark");
  });
});
