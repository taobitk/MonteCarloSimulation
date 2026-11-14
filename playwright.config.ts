import { defineConfig } from '@playwright/test';

export default defineConfig({
	webServer: {
		command: 'npm run build && npm run preview',
		port: 4173,
		reuseExistingServer: true
	},
	use: {
		channel: 'chrome',
		headless: true,
		viewport: { width: 1280, height: 720 }
	},
	testMatch: '**/*.e2e.{ts,js}'
});
