import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	clearScreen: false,
	server: {
		host: true,
		port: 5173,
		strictPort: true,
		allowedHosts: ['mac-mini']
	},
	test: {
		include: ['src/**/*.test.ts']
	}
});
