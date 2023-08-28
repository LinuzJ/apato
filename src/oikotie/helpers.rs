use rand::Rng;
use regex::Regex;

use crate::models::apartment::Apartment;

fn is_within_percentage(value: f32, reference: f32, percentage: f32) -> bool {
    let difference = (value - reference).abs();
    let allowed_difference = (percentage / 100.0) * reference;
    difference <= allowed_difference
}

fn calculate_median(numbers: &mut Vec<i32>) -> f64 {
    numbers.sort(); // Sort the vector in ascending order

    let len = numbers.len();
    if len % 2 == 0 {
        // If the length is even, take the average of the middle two values
        let mid = len / 2;
        let median = (numbers[mid - 1] + numbers[mid]) as f64 / 2.0;
        median
    } else {
        // If the length is odd, return the middle value
        numbers[len / 2] as f64
    }
}

pub fn create_location_string(id: i32, level: i32, name: String) -> String {
    return format!("[[{:?}, {:?}, {}{}{}]]", id, level, '"', name, '"');
}

pub fn generate_random_number() -> String {
    rand::thread_rng().gen_range(15000..70000).to_string()
}
pub fn get_rent_regex(rent_string: String) -> i32 {
    // Define a regular expression pattern
    let re = Regex::new(r"(\d+)").unwrap();
    let mut result = -1;

    let rent_without_space = rent_string.replace("\u{a0}", "");
    // Match the pattern against the text
    if let Some(captures) = re.captures(&rent_without_space) {
        // Extract the captured value and convert it to i32
        if let Some(value) = captures.get(1) {
            if let Ok(parsed_value) = value.as_str().parse::<i32>() {
                result = parsed_value;
            }
        }
    } else {
        panic!("lol")
    }

    return result;
}

pub fn estimated_rent(apartment: &Apartment, apartments: Vec<Apartment>) -> i32 {
    let size_buffer_percentage = 10.0;
    let similar_size_apartment_rents: Vec<i32> = apartments
        .iter()
        .filter(|ap| {
            is_within_percentage(
                ap.size as f32,
                apartment.size as f32,
                size_buffer_percentage,
            )
        })
        .map(|ap| ap.rent)
        .collect();

    let sum: i32 = similar_size_apartment_rents.iter().sum();
    let sum_float = sum as f64;
    let count = similar_size_apartment_rents.len() as f64;

    if count == 0.0 {
        let mut rent_only: Vec<i32> = apartments.iter().map(|ap| ap.rent).collect();
        let mut size_only: Vec<i32> = apartments.iter().map(|ap| ap.size as i32).collect();

        let rent_median = calculate_median(&mut rent_only);
        let size_median = calculate_median(&mut size_only);

        if apartment.rent as f64 > rent_median {
            let percentage_bigger_than_median =
                ((apartment.size - size_median) / size_median) + 1.0;
            let estimated_rent = apartment.rent as f64 * percentage_bigger_than_median;
            return estimated_rent as i32;
        } else {
            let percentage_smaller_than_median =
                1.0 - ((size_median - apartment.size) / size_median);
            let estimated_rent = apartment.rent as f64 * percentage_smaller_than_median;
            return estimated_rent as i32;
        }
    }

    return (sum_float / count) as i32;
}
