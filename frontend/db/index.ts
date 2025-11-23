// import { drizzle } from "drizzle-orm/expo-sqlite";
// import { openDatabaseSync } from "expo-sqlite";

// const DATABASE_NAME = 'app.db';

// export function openDb() {
//   return SQLite.openDatabaseSync(DATABASE_NAME);
// }

import { drizzle } from 'drizzle-orm/expo-sqlite';
import { openDatabaseSync, useSQLiteContext } from 'expo-sqlite';
import * as schema from './schema';

const DATABASE_NAME = 'sqlite://./db/app.db';

// export const db = drizzle(openDatabaseSync(DATABASE_NAME));

export function openDb() {
  const expoDb = openDatabaseSync('db.db');
  const db = drizzle(expoDb, { schema });
  return db;
}
