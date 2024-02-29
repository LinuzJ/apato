CREATE TABLE apartments (
    id SERIAL NOT NULL PRIMARY KEY,
    card_id TEXT,
    location_id INT,
    location_level INT,
    location_name TEXT,
    size FLOAT,
    rooms INT,
    price INT,
    additional_costs INT,
    rent INT,
    estimated_yield FLOAT,
    watchlist_id INTEGER NOT NULL REFERENCES watchlists ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
)