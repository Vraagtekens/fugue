// drizzle.config.ts
import { defineConfig } from 'drizzle-kit';
import path from 'path';

export default defineConfig({
  dialect: 'sqlite',
  dbCredentials: {
    url: `file://${path.resolve('db/app.db')}`, // absolute path
  },
  out: './db/drizzle',
});
