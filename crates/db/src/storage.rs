use chrono::Utc;
use rusqlite::{Connection, Result, params};

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS lists (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       name TEXT NOT NULL,
                       created_at TEXT NOT NULL
                   );
                   CREATE TABLE IF NOT EXISTS saved_words (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       list_id INTEGER NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
                       entry_seq INTEGER NOT NULL,
                       added_at TEXT NOT NULL
                   );",
        )?;

        Ok(Self { connection })
    }

    pub fn create_list(&self, name: &str) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO lists (name, created_at) VALUES (?1, ?2)",
            params![name, now()],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn save_word(&self, list_id: i64, entry_seq: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO saved_words (list_id, entry_seq, added_at) VALUES (?1, ?2, ?3)",
            params![list_id, entry_seq, now()],
        )?;
        Ok(())
    }
}

fn now() -> String {
    Utc::now().to_string()
}
