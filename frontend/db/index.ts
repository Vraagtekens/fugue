import * as SQLite from 'expo-sqlite';

const DATABASE_NAME = 'app.db';

export function openDb() {
  return SQLite.openDatabaseSync(DATABASE_NAME);
}
