use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

#[derive(Debug)]
pub struct DbConnection {
    conn: Connection,
}

impl DbConnection {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = DbConnection { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn get_connection(&mut self) -> &mut Connection {
        &mut self.conn
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS category_rules (
                id INTEGER PRIMARY KEY,
                category_id INTEGER NOT NULL,
                pattern TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY(category_id) REFERENCES categories(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transaction_categories (
                id INTEGER PRIMARY KEY,
                transaction_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(category_id) REFERENCES categories(id)
            )",
            [],
        )?;

        Ok(())
    }
}