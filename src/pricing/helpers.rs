use rand::Rng;

pub fn create_location_string(id: i32, level: i32, name: String) -> String {
    return format!("[[{:?}, {:?}, {}{}{}]]", id, level, '"', name, '"');
}

pub fn generate_random_number() -> String {
    rand::thread_rng().gen_range(15000..70000).to_string()
}
