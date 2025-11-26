use crate::db::{Database, ModelRecord, models};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct Registry {
    db: Arc<Database>,
    active_model: Arc<RwLock<Option<String>>>,
}

impl Registry {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            active_model: Arc::new(RwLock::new(None)),
        }
    }

    pub fn list_models(&self) -> Result<Vec<ModelRecord>> {
        self.db.with_connection(|conn| models::list_models(conn))
    }

    pub fn get_model(&self, id: &str) -> Result<Option<ModelRecord>> {
        self.db.with_connection(|conn| models::get_model(conn, id))
    }

    pub fn add_model(&self, model: ModelRecord) -> Result<()> {
        self.db.with_connection(|conn| models::insert_model(conn, &model))
    }

    pub fn remove_model(&self, id: &str) -> Result<()> {
        self.db.with_connection(|conn| models::delete_model(conn, id))
    }

    pub fn set_active_model(&self, id: String) -> Result<()> {
        if self.get_model(&id)?.is_none() {
            return Err(anyhow::anyhow!("Model not found: {}", id));
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.db.with_connection(|conn| models::update_last_used(conn, &id, timestamp))?;

        let mut active = self.active_model.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        *active = Some(id);

        Ok(())
    }

    pub fn get_active_model(&self) -> Option<String> {
        self.active_model.read().ok()?.clone()
    }

    pub fn get_active_model_path(&self) -> Result<Option<PathBuf>> {
        if let Some(id) = self.get_active_model() {
            if let Some(model) = self.get_model(&id)? {
                return Ok(Some(PathBuf::from(model.path)));
            }
        }
        Ok(None)
    }
}
