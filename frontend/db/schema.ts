import { openDb } from './index';

export async function initDb() {
  const db = openDb();
  await db.execAsync(/*sql*/ `
    PRAGMA journal_mode = WAL;
    CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY NOT NULL, value TEXT NOT NULL, intValue INTEGER);
    INSERT INTO test (value, intValue) VALUES ('test1', 123);
    INSERT INTO test (value, intValue) VALUES ('test2', 456);
    INSERT INTO test (value, intValue) VALUES ('test3', 789);
  `);

  const allRows = await db.getAllAsync('SELECT * FROM test');
  for (const row of allRows) {
    console.log(row.id, row.value, row.intValue);
  }
}
