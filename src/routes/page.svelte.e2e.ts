import { expect, test } from '@playwright/test';

test.describe('DCA CLI Results Viewer E2E Tests', () => {
	test('Tự động nạp latest_suite.json, lọc bảng so sánh, vẽ Fan Chart và hiển thị chỉ số', async ({ page }) => {
		const errors: string[] = [];
		page.on('pageerror', (err) => errors.push(err.message));

		await page.goto('/');

		await expect(page.getByRole('heading', { name: /DCA & Chaos Monkey/ })).toBeVisible();
		await expect(page.getByText(/Bảng So Sánh Kịch Bản/)).toBeVisible({ timeout: 10000 });

		// Kiểm tra bộ lọc kỳ hạn (40 năm) và chiến lược (Chỉ Dynamic DCA)
		const selects = page.locator('select');
		await selects.nth(0).selectOption('40');
		await selects.nth(1).selectOption('dynamic');

		const rows = page.locator('tbody tr');
		await expect(rows.first()).toBeVisible();
		const rowCount = await rows.count();
		expect(rowCount).toBeGreaterThan(0);

		// Click dòng đầu tiên sau khi lọc và kiểm tra Fan Chart + thẻ chỉ số cập nhật
		await rows.first().click();
		await expect(page.getByRole('heading', { name: /Biểu Đồ Fan Chart — 40 Năm/ })).toBeVisible();
		await expect(page.getByText('Trung vị P50 (MOIC)')).toBeVisible();
		await expect(page.getByText('Phân Rã Sụt Giảm Lúc Chạm Đáy')).toBeVisible();

		// Kiểm tra canvas có kích thước hợp lệ
		const canvasBox = await page.locator('canvas').boundingBox();
		expect(canvasBox).not.toBeNull();
		expect(canvasBox!.width).toBeGreaterThan(200);
		expect(canvasBox!.height).toBeGreaterThan(200);

		expect(errors).toEqual([]);
		await page.screenshot({
			path: 'C:/Users/8hung/.gemini/antigravity/brain/03bb1f6e-1b52-4ef2-b364-31252af45e66/cli_viewer_screenshot.png',
			fullPage: true
		});
	});
});
