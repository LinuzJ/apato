-- This file should undo anything in `up.sql`
ALTER TABLE watchlists
    DROP COLUMN target_size_min,
    DROP COLUMN target_size_max
