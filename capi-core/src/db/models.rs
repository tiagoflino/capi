use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: Option<i64>,
    pub quantization: Option<String>,
    pub context_length: Option<i64>,
    pub created_at: i64,
    pub last_used: Option<i64>,
}

pub fn list_models(conn: &Connection) -> Result<Vec<ModelRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, size_bytes, quantization, context_length, created_at, last_used
         FROM models
         ORDER BY last_used DESC, created_at DESC"
    )?;

    let models = stmt.query_map([], |row| {
        Ok(ModelRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            size_bytes: row.get(3)?,
            quantization: row.get(4)?,
            context_length: row.get(5)?,
            created_at: row.get(6)?,
            last_used: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(models)
}

pub fn get_model(conn: &Connection, id: &str) -> Result<Option<ModelRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, size_bytes, quantization, context_length, created_at, last_used
         FROM models
         WHERE id = ?"
    )?;

    let model = stmt.query_row([id], |row| {
        Ok(ModelRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            size_bytes: row.get(3)?,
            quantization: row.get(4)?,
            context_length: row.get(5)?,
            created_at: row.get(6)?,
            last_used: row.get(7)?,
        })
    }).optional()?;

    Ok(model)
}

pub fn insert_model(conn: &Connection, model: &ModelRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO models (id, name, path, size_bytes, quantization, context_length, created_at, last_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            &model.id,
            &model.name,
            &model.path,
            &model.size_bytes,
            &model.quantization,
            &model.context_length,
            &model.created_at,
            &model.last_used,
        ),
    )?;
    Ok(())
}

pub fn update_last_used(conn: &Connection, id: &str, timestamp: i64) -> Result<()> {
    conn.execute(
        "UPDATE models SET last_used = ? WHERE id = ?",
        (timestamp, id),
    )?;
    Ok(())
}

pub fn delete_model(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM models WHERE id = ?", [id])?;
    Ok(())
}
