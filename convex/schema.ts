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