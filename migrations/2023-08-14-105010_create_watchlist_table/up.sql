CREATE TABLE watchlists (
    id SERIAL PRIMARY KEY,
    location_id INT NOT NULL,
    location_level INT NOT NULL,
    location_name TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    goal_yield FLOAT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
)