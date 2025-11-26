import { sqliteTable, AnySQLiteColumn, text, integer, numeric } from 'drizzle-orm/sqlite-core';
import { sql } from 'drizzle-orm';

export const seaqlMigrations = sqliteTable('seaql_migrations', {
  version: text().primaryKey().notNull(),
  appliedAt: integer('applied_at').notNull(),
});

export const sessions = sqliteTable('sessions', {
  id: integer().primaryKey({ autoIncrement: true }).notNull(),
  userId: integer('user_id').notNull(),
  categoryId: integer('category_id'),
  startTime: numeric('start_time').notNull(),
  endTime: numeric('end_time'),
  kind: text().default('pomodoro').notNull(),
  completed: numeric().default(sql`(FALSE)`),
  createdAt: numeric('created_at').default(sql`(CURRENT_TIMESTAMP)`),
  updatedAt: numeric('updated_at').default(sql`(CURRENT_TIMESTAMP)`),
});

export const users = sqliteTable('users', {
  id: integer().primaryKey({ autoIncrement: true }).notNull(),
  email: text().notNull(),
  passwordHash: text('password_hash').notNull(),
  createdAt: numeric('created_at').default(sql`(CURRENT_TIMESTAMP)`),
  updatedAt: numeric('updated_at').default(sql`(CURRENT_TIMESTAMP)`),
});

export const categories = sqliteTable('categories', {
  id: integer().primaryKey({ autoIncrement: true }).notNull(),
  name: text().notNull(),
  createdAt: numeric('created_at').default(sql`(CURRENT_TIMESTAMP)`),
  updatedAt: numeric('updated_at').default(sql`(CURRENT_TIMESTAMP)`),
});
