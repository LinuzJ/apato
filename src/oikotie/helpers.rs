use rand::Rng;
use regex::Regex;

use crate::models::apartment::InsertableApartment;

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

pub fn estimated_rent(
    apartment: &InsertableApartment,
    apartments: Vec<InsertableApartment>,
) -> i32 {
    let size_buffer_percentage = 10.0;
    let similar_size_apartment_rents: Vec<i32> = apartments
        .iter()
        .filter(|ap| {
            is_within_percentage(
                ap.size.unwrap() as f32,
                apartment.size.unwrap() as f32,
                size_buffer_percentage,
            )
        })
        .map(|ap| ap.rent.unwrap())
        .collect();

    let sum: i32 = similar_size_apartment_rents.iter().sum();
    let count = similar_size_apartment_rents.len();

    // If there are no similar size apartments, scale rent by relation to median
    if count == 0 {
        let mut rent_only: Vec<i32> = apartments.iter().map(|ap| ap.rent.unwrap()).collect();
        let mut size_only: Vec<i32> = apartments
            .iter()
            .map(|ap| ap.size.unwrap() as i32)
            .collect();

        let rent_median: f64 = calculate_median(&mut rent_only);
        let size_median: f64 = calculate_median(&mut size_only);

        if apartment.rent.unwrap() as f64 > rent_median {
            let deviation = bigger_than_median(apartment.size.unwrap(), size_median);

            let estimated_rent = rent_median * deviation;
            return estimated_rent as i32;
        } else {
            let deviation = smaller_than_median(apartment.size.unwrap(), size_median);

            let estimated_rent = rent_median * deviation;
            return estimated_rent as i32;
        }
    }

    return (sum as f64 / count as f64) as i32;
}

fn bigger_than_median(size: f64, median: f64) -> f64 {
    ((size - median) / median) + 1.0
}

fn smaller_than_median(size: f64, median: f64) -> f64 {
    1.0 - ((median - size) / median)
}
