CREATE TABLE apartments (
    id SERIAL PRIMARY KEY,
    card_id TEXT,
    location_id INT,
    location_level INT,
    location_name TEXT,
    size FLOAT,
    rooms INT,
    price TEXT,
    additional_costs INT,
    rent INT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
)