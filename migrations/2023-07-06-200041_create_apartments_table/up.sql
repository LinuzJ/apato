CREATE TABLE apartments (
    id TEXT PRIMARY KEY,
    location_id INT,
    location_level INT,
    location_name TEXT,
    size FLOAT,
    rooms INT,
    price TEXT,
    additional_costs INT,
    rent INT
)