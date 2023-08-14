CREATE TABLE watchlists (
    id SERIAL PRIMARY KEY,
    location_id INT,
    location_level INT,
    location_name TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
)