use crate::db::ItemRepository;
use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub repo: Arc<dyn ItemRepository>,
}

impl AppState {
    pub fn new(repo: Arc<dyn ItemRepository>) -> Self {
        Self { repo }
    }

    pub fn shared(repo: Arc<dyn ItemRepository>) -> SharedState {
        Arc::new(Self::new(repo))
    }
}
