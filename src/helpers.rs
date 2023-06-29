pub fn create_location_string(id: u16, level: u8, name: String) -> String {
    return format!("[[{:?}, {:?}, {}]]", id, level, name);
}
