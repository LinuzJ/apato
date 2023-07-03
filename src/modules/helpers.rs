use rand::Rng;

pub fn create_location_string(id: u16, level: u8, name: String) -> String {
    return format!("[[{:?}, {:?}, {}{}{}]]", id, level, '"', name, '"');
}

pub fn generate_random_number() -> String {
    rand::thread_rng().gen_range(5000..10000).to_string()
}
