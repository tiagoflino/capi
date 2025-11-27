pub mod models;
pub mod chats;

pub use models::ModelRecord;
pub use chats::{ChatSession, ChatMessage};

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size_bytes INTEGER,
                quantization TEXT,
                context_length INTEGER,
                created_at INTEGER NOT NULL,
                last_used INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                model_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id)
            )",
            [],
        )?;

        // Add new columns if they don't exist (migration)
        let has_estimated_memory = conn
            .prepare("SELECT estimated_memory_bytes FROM models LIMIT 1")
            .is_ok();

        if !has_estimated_memory {
            conn.execute("ALTER TABLE models ADD COLUMN estimated_memory_bytes INTEGER", [])?;
            conn.execute("ALTER TABLE models ADD COLUMN context_override INTEGER", [])?;
        }

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn with_connection<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R>,
    {
        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;
        f(&conn)
    }
}
