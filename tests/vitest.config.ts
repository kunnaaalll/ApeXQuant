import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'dist/',
        '**/*.d.ts',
        '**/generated/**',
      ],
    },
    testTimeout: 30000,
    hookTimeout: 30000,
    setupFiles: ['./setup.ts'],
  },
});
