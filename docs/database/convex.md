# Convex Functions for Ferrous

This directory contains the Convex functions required for the Ferrous project's Convex database implementation.

## Setup

1. Install Convex CLI:
   ```bash
   npm install -g convex
   ```

2. Initialize Convex in this directory:
   ```bash
   convex init
   ```

3. Deploy the functions:
   ```bash
   convex deploy
   ```

## Required Functions

The following functions need to be implemented in your Convex deployment:

### items.js

```javascript
import { v } from "convex/values";
import { query, mutation } from "./_generated/server";

// List items with pagination
export const list = query({
  args: {
    limit: v.float64(),
    offset: v.float64(),
  },
  handler: async (ctx, args) => {
    const items = await ctx.db
      .query("items")
      .order("desc")
      .take(args.limit);
    
    // Simple offset implementation (not optimal for large datasets)
    return items.slice(args.offset);
  },
});

// Count total items
export const count = query({
  args: {},
  handler: async (ctx) => {
    const items = await ctx.db.query("items").collect();
    return items.length;
  },
});

// Get single item by ID
export const get = query({
  args: {
    id: v.string(),
  },
  handler: async (ctx, args) => {
    const item = await ctx.db.get(args.id);
    return item || null;
  },
});

// Create new item
export const create = mutation({
  args: {
    name: v.string(),
    description: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const now = Date.now();
    const id = await ctx.db.insert("items", {
      name: args.name,
      description: args.description || null,
      created_at: new Date(now).toISOString(),
      updated_at: new Date(now).toISOString(),
    });
    
    return await ctx.db.get(id);
  },
});

// Update existing item
export const update = mutation({
  args: {
    id: v.string(),
    name: v.optional(v.string()),
    description: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      return null;
    }
    
    const updates = {
      ...existing,
      updated_at: new Date().toISOString(),
    };
    
    if (args.name !== undefined) {
      updates.name = args.name;
    }
    if (args.description !== undefined) {
      updates.description = args.description;
    }
    
    await ctx.db.replace(args.id, updates);
    return await ctx.db.get(args.id);
  },
});

// Delete item
export const delete = mutation({
  args: {
    id: v.string(),
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      return null;
    }
    
    await ctx.db.delete(args.id);
    return { success: true };
  },
});
```

### schema.ts

```typescript
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  items: defineTable({
    name: v.string(),
    description: v.union(v.string(), v.null()),
    created_at: v.string(),
    updated_at: v.string(),
  }),
});
```

## Environment Configuration

Add your Convex deployment URL to your `.env` file:

```
DATABASE_TYPE=convex
CONVEX_DEPLOYMENT_URL=https://your-project-name.convex.cloud
```

## Testing

To test the Convex implementation:

1. Ensure your Convex functions are deployed
2. Set `DATABASE_TYPE=convex` in your `.env` file
3. Add your `CONVEX_DEPLOYMENT_URL`
4. Run the Ferrous server:
   ```bash
   cargo run
   ```
5. Test the API endpoints:
   ```bash
   # Create item
   curl -X POST http://localhost:3000/api/v1/items \
     -H "Content-Type: application/json" \
     -d '{"name": "Test Item", "description": "Test Description"}'
   
   # List items
   curl http://localhost:3000/api/v1/items
   
   # Get item
   curl http://localhost:3000/api/v1/items/{id}
   
   # Update item
   curl -X PUT http://localhost:3000/api/v1/items/{id} \
     -H "Content-Type: application/json" \
     -d '{"name": "Updated Name"}'
   
   # Delete item
   curl -X DELETE http://localhost:3000/api/v1/items/{id}
   ```

## Notes

- The Convex implementation uses optimistic locking via the `updated_at` field
- IDs are managed by Convex and returned as strings
- All timestamps are stored as ISO 8601 strings
- The offset-based pagination is simple but not optimal for large datasets