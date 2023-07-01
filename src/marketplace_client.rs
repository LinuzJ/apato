pub struct Location {
    pub id: u16,
    pub level: u8,
    pub name: String,
}

pub struct Apartment {
    pub id: String,
    pub location: Location,
    pub size: u16,
    pub rooms: u16,
    pub price: u16,
    pub additional_costs: u16,
}

pub trait MarketplaceClient {
    // Fetch apartments from given location and parse them into vector
    fn get_apartments(self, location: Location) -> Vec<Apartment>;
}