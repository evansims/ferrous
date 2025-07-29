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

impl Default for InMemoryDatabase {
    fn default() -> Self {
        Self::new()
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

impl Default for InMemoryItemRepository {
    fn default() -> Self {
        Self::new()
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
        items.get(id).cloned().ok_or(DatabaseError::NotFound)
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

        let item = items.get_mut(id).ok_or(DatabaseError::NotFound)?;

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

        items.remove(id).ok_or(DatabaseError::NotFound)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_item() {
        let repo = InMemoryItemRepository::new();

        let request = CreateItemRequest {
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
        };

        let created = repo.create(request).await.unwrap();
        assert_eq!(created.name, "Test Item");
        assert_eq!(created.description, Some("Test Description".to_string()));

        let retrieved = repo.get(&created.id).await.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);
        assert_eq!(retrieved.description, created.description);
    }

    #[tokio::test]
    async fn test_get_nonexistent_item() {
        let repo = InMemoryItemRepository::new();
        let result = repo.get("nonexistent").await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }

    #[tokio::test]
    async fn test_update_item() {
        let repo = InMemoryItemRepository::new();

        let request = CreateItemRequest {
            name: "Original Name".to_string(),
            description: Some("Original Description".to_string()),
        };

        let created = repo.create(request).await.unwrap();
        let original_updated_at = created.updated_at;

        // Sleep briefly to ensure time difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let update_request = UpdateItemRequest {
            name: Some("Updated Name".to_string()),
            description: None,
        };

        let updated = repo.update(&created.id, update_request).await.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(
            updated.description,
            Some("Original Description".to_string())
        );
        assert!(updated.updated_at > original_updated_at);
    }

    #[tokio::test]
    async fn test_update_nonexistent_item() {
        let repo = InMemoryItemRepository::new();

        let update_request = UpdateItemRequest {
            name: Some("New Name".to_string()),
            description: None,
        };

        let result = repo.update("nonexistent", update_request).await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }

    #[tokio::test]
    async fn test_delete_item() {
        let repo = InMemoryItemRepository::new();

        let request = CreateItemRequest {
            name: "To Delete".to_string(),
            description: None,
        };

        let created = repo.create(request).await.unwrap();

        // Verify item exists
        assert!(repo.get(&created.id).await.is_ok());

        // Delete item
        repo.delete(&created.id).await.unwrap();

        // Verify item no longer exists
        let result = repo.get(&created.id).await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }

    #[tokio::test]
    async fn test_delete_nonexistent_item() {
        let repo = InMemoryItemRepository::new();
        let result = repo.delete("nonexistent").await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }

    #[tokio::test]
    async fn test_list_items() {
        let repo = InMemoryItemRepository::new();

        // Create multiple items
        for i in 0..5 {
            let request = CreateItemRequest {
                name: format!("Item {}", i),
                description: None,
            };
            repo.create(request).await.unwrap();
        }

        // Test listing all items
        let all_items = repo.list(10, 0).await.unwrap();
        assert_eq!(all_items.len(), 5);

        // Test pagination
        let page1 = repo.list(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.list(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = repo.list(2, 4).await.unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn test_count_items() {
        let repo = InMemoryItemRepository::new();

        assert_eq!(repo.count().await.unwrap(), 0);

        for i in 0..3 {
            let request = CreateItemRequest {
                name: format!("Item {}", i),
                description: None,
            };
            repo.create(request).await.unwrap();
        }

        assert_eq!(repo.count().await.unwrap(), 3);

        // Delete one item
        let items = repo.list(1, 0).await.unwrap();
        repo.delete(&items[0].id).await.unwrap();

        assert_eq!(repo.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_database_health_check() {
        let db = InMemoryDatabase::new();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_database_items_repository() {
        let db = InMemoryDatabase::new();
        let items_repo = db.items();

        let request = CreateItemRequest {
            name: "Test Item".to_string(),
            description: None,
        };

        let created = items_repo.create(request).await.unwrap();
        assert!(items_repo.get(&created.id).await.is_ok());
    }
}
