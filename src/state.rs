use crate::database::Database;
use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub db: Arc<dyn Database>,
}

impl AppState {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub fn shared(db: Arc<dyn Database>) -> SharedState {
        Arc::new(Self::new(db))
    }
}
