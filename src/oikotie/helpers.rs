use rand::{seq::SliceRandom, Rng};
use regex::Regex;

use crate::models::apartment::Apartment;

use super::oikotie::Location;

fn is_within_percentage(value: f32, reference: f32, percentage: f32) -> bool {
    let difference = (value - reference).abs();
    let allowed_difference = (percentage / 100.0) * reference;
    difference <= allowed_difference
}

pub fn create_location_string(id: i32, level: i32, name: String) -> String {
    return format!("[[{:?}, {:?}, {}{}{}]]", id, level, '"', name, '"');
}

pub fn generate_random_number() -> String {
    rand::thread_rng().gen_range(15000..70000).to_string()
}
pub fn get_rent_regex(rent_string: String) -> i32 {
    // Define a regular expression pattern
    let re = Regex::new(r"(\d+) € / kk").unwrap();
    let mut result = -1;

    // Match the pattern against the text
    if let Some(captures) = re.captures(&rent_string) {
        // Extract the captured value and convert it to i32
        if let Some(value) = captures.get(1) {
            if let Ok(parsed_value) = value.as_str().parse::<i32>() {
                result = parsed_value;
            }
        }
    }

    return result;
}

pub fn closest_rent(apartment: &Apartment, apartments: Vec<Apartment>) -> i32 {
    let percentage = 10.0;
    let similar_size_apartment_rents: Vec<i32> = apartments
        .iter()
        .filter(|ap| is_within_percentage(ap.size as f32, apartment.size as f32, percentage))
        .map(|ap| ap.rent)
        .collect();

    let sum: i32 = similar_size_apartment_rents.iter().sum();
    let count = similar_size_apartment_rents.len() as i32;

    return sum / count;
}
