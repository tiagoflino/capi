use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: Option<String>,
    pub model_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

pub fn list_sessions(conn: &Connection) -> Result<Vec<ChatSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, model_id, created_at, updated_at
         FROM chat_sessions
         ORDER BY updated_at DESC"
    )?;

    let sessions = stmt.query_map([], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            model_id: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(sessions)
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Option<ChatSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, model_id, created_at, updated_at
         FROM chat_sessions
         WHERE id = ?"
    )?;

    let session = stmt.query_row([id], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            model_id: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).optional()?;

    Ok(session)
}

pub fn create_session(conn: &Connection, session: &ChatSession) -> Result<()> {
    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &session.id,
            &session.title,
            &session.model_id,
            &session.created_at,
            &session.updated_at,
        ),
    )?;
    Ok(())
}

pub fn update_session(conn: &Connection, session: &ChatSession) -> Result<()> {
    conn.execute(
        "UPDATE chat_sessions
         SET title = ?, model_id = ?, updated_at = ?
         WHERE id = ?",
        (
            &session.title,
            &session.model_id,
            &session.updated_at,
            &session.id,
        ),
    )?;
    Ok(())
}

pub fn delete_session(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM chat_messages WHERE session_id = ?", [id])?;
    conn.execute("DELETE FROM chat_sessions WHERE id = ?", [id])?;
    Ok(())
}

pub fn get_messages(conn: &Connection, session_id: &str) -> Result<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, created_at
         FROM chat_messages
         WHERE session_id = ?
         ORDER BY created_at ASC"
    )?;

    let messages = stmt.query_map([session_id], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub fn add_message(conn: &Connection, message: &ChatMessage) -> Result<()> {
    conn.execute(
        "INSERT INTO chat_messages (id, session_id, role, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &message.id,
            &message.session_id,
            &message.role,
            &message.content,
            &message.created_at,
        ),
    )?;
    Ok(())
}
