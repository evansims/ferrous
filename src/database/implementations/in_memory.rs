use crate::{
    database::{Database, DatabaseError, DatabaseResult, ItemRepository},
    models::{CreateItemRequest, Item, UpdateItemRequest},
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct InMemoryDatabase {
    items: Arc<InMemoryItemRepository>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            items: Arc::new(InMemoryItemRepository::new()),
        }
    }
}

#[async_trait]
impl Database for InMemoryDatabase {
    fn items(&self) -> Arc<dyn ItemRepository> {
        self.items.clone()
    }
    
    async fn health_check(&self) -> DatabaseResult<()> {
        // In-memory database is always healthy
        Ok(())
    }
}

pub struct InMemoryItemRepository {
    data: RwLock<HashMap<String, Item>>,
}

impl InMemoryItemRepository {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ItemRepository for InMemoryItemRepository {
    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;
        
        let mut all_items: Vec<Item> = items.values().cloned().collect();
        // Sort by created_at for consistent ordering
        all_items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        Ok(all_items.into_iter().skip(offset).take(limit).collect())
    }
    
    async fn count(&self) -> DatabaseResult<usize> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;
        Ok(items.len())
    }
    
    async fn get(&self, id: &str) -> DatabaseResult<Item> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;
        items
            .get(id)
            .cloned()
            .ok_or(DatabaseError::NotFound)
    }
    
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let item = Item {
            id: id.clone(),
            name: request.name,
            description: request.description,
            created_at: now,
            updated_at: now,
        };
        
        items.insert(id, item.clone());
        Ok(item)
    }
    
    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;
        
        let item = items
            .get_mut(id)
            .ok_or(DatabaseError::NotFound)?;
        
        if let Some(name) = request.name {
            item.name = name;
        }
        if request.description.is_some() {
            item.description = request.description;
        }
        item.updated_at = chrono::Utc::now();
        
        Ok(item.clone())
    }
    
    async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;
        
        items
            .remove(id)
            .ok_or(DatabaseError::NotFound)?;
        
        Ok(())
    }
}