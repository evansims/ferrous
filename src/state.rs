use crate::models::Item;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type SharedState = Arc<AppState>;

#[derive(Default)]
pub struct AppState {
    pub items: RwLock<HashMap<String, Item>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
        }
    }

    pub fn shared() -> SharedState {
        Arc::new(Self::new())
    }
}
