use crate::{
    database::{Database, DatabaseError, DatabaseResult, ItemRepository},
    models::{CreateItemRequest, Item, UpdateItemRequest},
};
use async_trait::async_trait;
use convex::{ConvexClient, FunctionResult, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ConvexDatabase {
    client: Arc<Mutex<ConvexClient>>,
    items: Arc<ConvexItemRepository>,
}

impl ConvexDatabase {
    pub async fn new(deployment_url: &str) -> DatabaseResult<Self> {
        let client = ConvexClient::new(deployment_url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        let client = Arc::new(Mutex::new(client));

        let items = Arc::new(ConvexItemRepository {
            client: client.clone(),
        });

        Ok(Self { client, items })
    }
}

#[async_trait]
impl Database for ConvexDatabase {
    fn items(&self) -> Arc<dyn ItemRepository> {
        self.items.clone()
    }

    async fn health_check(&self) -> DatabaseResult<()> {
        // Perform a simple query to check connection
        let mut args = BTreeMap::new();
        args.insert("limit".to_string(), Value::Float64(1.0));

        let mut client = self.client.lock().await;
        client
            .query("items:list", args)
            .await
            .map_err(|e| DatabaseError::ConnectionError(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

pub struct ConvexItemRepository {
    client: Arc<Mutex<ConvexClient>>,
}

#[async_trait]
impl ItemRepository for ConvexItemRepository {
    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>> {
        let mut args = BTreeMap::new();
        args.insert("limit".to_string(), Value::Float64(limit as f64));
        args.insert("offset".to_string(), Value::Float64(offset as f64));

        let mut client = self.client.lock().await;
        let result = client
            .query("items:list", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Convert Convex FunctionResult to Vec<Item>
        match result {
            FunctionResult::Value(value) => {
                let json_value = convex_value_to_json(&value);
                let items: Vec<Item> = serde_json::from_value(json_value)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
                Ok(items)
            }
            _ => Err(DatabaseError::SerializationError("Unexpected result format".to_string())),
        }
    }

    async fn count(&self) -> DatabaseResult<usize> {
        let args = BTreeMap::new();

        let mut client = self.client.lock().await;
        let result = client
            .query("items:count", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Extract count from result
        match result {
            FunctionResult::Value(Value::Float64(n)) => Ok(n as usize),
            _ => Err(DatabaseError::SerializationError("Invalid count format".to_string())),
        }
    }

    async fn get(&self, id: &str) -> DatabaseResult<Item> {
        let mut args = BTreeMap::new();
        args.insert("id".to_string(), Value::String(id.to_string()));

        let mut client = self.client.lock().await;
        let result = client
            .query("items:get", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Convert Convex FunctionResult to Item
        match result {
            FunctionResult::Value(Value::Null) => Err(DatabaseError::NotFound),
            FunctionResult::Value(value) => {
                let json_value = convex_value_to_json(&value);
                let item: Item = serde_json::from_value(json_value)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
                Ok(item)
            }
            _ => Err(DatabaseError::SerializationError("Unexpected result format".to_string())),
        }
    }

    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item> {
        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String(request.name));
        if let Some(desc) = request.description {
            args.insert("description".to_string(), Value::String(desc));
        }

        let mut client = self.client.lock().await;
        let result = client
            .mutation("items:create", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Convert Convex FunctionResult to Item
        match result {
            FunctionResult::Value(value) => {
                let json_value = convex_value_to_json(&value);
                let item: Item = serde_json::from_value(json_value)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
                Ok(item)
            }
            _ => Err(DatabaseError::SerializationError("Unexpected result format".to_string())),
        }
    }

    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item> {
        let mut args = BTreeMap::new();
        args.insert("id".to_string(), Value::String(id.to_string()));
        if let Some(name) = request.name {
            args.insert("name".to_string(), Value::String(name));
        }
        if let Some(desc) = request.description {
            args.insert("description".to_string(), Value::String(desc));
        }

        let mut client = self.client.lock().await;
        let result = client
            .mutation("items:update", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Convert Convex FunctionResult to Item
        match result {
            FunctionResult::Value(Value::Null) => Err(DatabaseError::NotFound),
            FunctionResult::Value(value) => {
                let json_value = convex_value_to_json(&value);
                let item: Item = serde_json::from_value(json_value)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
                Ok(item)
            }
            _ => Err(DatabaseError::SerializationError("Unexpected result format".to_string())),
        }
    }

    async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let mut args = BTreeMap::new();
        args.insert("id".to_string(), Value::String(id.to_string()));

        let mut client = self.client.lock().await;
        let result = client
            .mutation("items:delete", args)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Convert result to check if deletion was successful
        match result {
            FunctionResult::Value(Value::Null) => Err(DatabaseError::NotFound),
            FunctionResult::Value(_) => Ok(()),
            _ => Err(DatabaseError::SerializationError("Unexpected result format".to_string())),
        }
    }
}

// Helper function to convert Convex Value to serde_json::Value
fn convex_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Float64(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap()),
        Value::Int64(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Bytes(_) => serde_json::Value::String("<binary>".to_string()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(convex_value_to_json).collect())
        }
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), convex_value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
    }
}
