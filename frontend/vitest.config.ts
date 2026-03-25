import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
	plugins: [svelte({ hot: false })],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
		globals: true,
		setupFiles: [],
		alias: {
			'$lib': '/src/lib',
			'$lib/*': '/src/lib/*',
			'$app/navigation': '/src/lib/__mocks__/navigation.ts'
		}
	},
	resolve: {
		alias: {
			'$lib': '/src/lib',
			'$app/navigation': '/src/lib/__mocks__/navigation.ts'
		}
	}
});
