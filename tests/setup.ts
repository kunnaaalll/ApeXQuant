import { vi } from 'vitest';

// Global test setup
globalThis.vi = vi;

// Mock environment variables
process.env.NODE_ENV = 'test';
process.env.LOG_LEVEL = 'error';
process.env.DATABASE_URL = 'postgres://test:test@localhost:5433/apex_test';
process.env.REDIS_URL = 'redis://localhost:6380';

// Clean up after all tests
afterAll(async () => {
  // Add any global cleanup here
});

// Reset mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
});
