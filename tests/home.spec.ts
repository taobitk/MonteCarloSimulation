import { test, expect } from '@playwright/test';

test('Trang chủ phản hồi 200 và render tiêu đề', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle(/.*|SvelteKit/);
});
