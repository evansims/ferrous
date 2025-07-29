use serde::{Deserialize, Serialize};
use serde_json::json; // Used in #[schema(example = json!({...}))] attributes
use utoipa::ToSchema;
use validator::Validate;

/// Represents an item in the system
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Example Item",
    "description": "This is an example item",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
}))]
pub struct Item {
    /// Unique identifier for the item
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,

    /// Name of the item
    #[schema(example = "Example Item")]
    pub name: String,

    /// Optional description of the item
    #[schema(example = "This is an example item")]
    pub description: Option<String>,

    /// Timestamp when the item was created
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when the item was last updated
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new item
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "New Item",
    "description": "Description of the new item"
}))]
pub struct CreateItemRequest {
    /// Name of the item (1-255 characters)
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    #[schema(example = "New Item", min_length = 1, max_length = 255)]
    pub name: String,

    /// Optional description (max 1000 characters)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    #[schema(example = "Description of the new item", max_length = 1000)]
    pub description: Option<String>,
}

/// Request to update an existing item
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "Updated Item Name",
    "description": "Updated description"
}))]
pub struct UpdateItemRequest {
    /// New name for the item (1-255 characters)
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    #[schema(example = "Updated Item Name", min_length = 1, max_length = 255)]
    pub name: Option<String>,

    /// New description (max 1000 characters)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    #[schema(example = "Updated description", max_length = 1000)]
    pub description: Option<String>,
}

impl CreateItemRequest {
    /// Sanitize the request data
    pub fn sanitize(mut self) -> Self {
        self.name = self.name.trim().to_string();
        self.description = self
            .description
            .and_then(|d| {
                let trimmed = d.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });
        self
    }
}

impl UpdateItemRequest {
    /// Sanitize the request data
    pub fn sanitize(mut self) -> Self {
        self.name = self.name.map(|n| n.trim().to_string());
        self.description = self
            .description
            .and_then(|d| {
                let trimmed = d.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });
        self
    }
}
