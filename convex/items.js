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