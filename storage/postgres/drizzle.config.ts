import type { Config } from 'drizzle-kit';

export default {
  schema: './src/db/schema.ts',
  out: './drizzle',
  driver: 'pg',
  dbCredentials: {
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT || '5432'),
    user: process.env.DB_USER || 'apex',
    password: process.env.DB_PASSWORD || 'apex',
    database: process.env.DB_NAME || 'apex_v3',
  },
  verbose: true,
  strict: true,
} satisfies Config;
