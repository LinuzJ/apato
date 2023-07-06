#[derive(Debug)]
pub struct Location {
    pub id: u16,
    pub level: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct Apartment {
    pub id: String,
    pub location: Location,
    pub size: f32,
    pub rooms: u16,
    pub price: String,
    pub additional_costs: u16,
}

pub trait MarketplaceClient {
    // Fetch apartments from given location and parse them into vector
    fn get_apartments(self, location: Location) -> Vec<Apartment>;
}
