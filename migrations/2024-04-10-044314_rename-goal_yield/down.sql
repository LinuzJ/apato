-- This file should undo anything in `up.sql`
ALTER TABLE watchlists
  RENAME COLUMN target_yield TO goal_yield;
