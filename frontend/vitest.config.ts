import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: { conditions: ['browser'] },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: [],
    exclude: ['**/node_modules/**', '**/*.stories.svelte', '**/.storybook/**'],
    alias: {
      $lib: new URL('./src/lib', import.meta.url).pathname,
    },
  },
});
