-- This file should undo anything in `up.sql`
ALTER TABLE watchlists
  RENAME COLUMN chat_id TO user_id;