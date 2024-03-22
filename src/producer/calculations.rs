use log::error;

use crate::{
    db, interest_rate::interest_rate_client, models::apartment::InsertableApartment,
    oikotie::oikotie::Oikotie,
};

pub async fn process_apartment_calculations(
    potential_apartments: Option<Vec<InsertableApartment>>,
    mut oikotie: Oikotie,
) {
    match potential_apartments {
        Some(apartments) => {
            for mut apartment in apartments {
                /*
                   Calculate yield here
                   - Get rent for similar apartments close by
                   - Get interest rate from Nordea
                   - Calculate
                */

                let estimated_rent = oikotie.get_estimated_rent(&apartment).await;

                match estimated_rent {
                    Ok(rent) => apartment.rent = Some(rent),
                    Err(e) => error!("{}", e),
                }

                let interest_rate = interest_rate_client::get_interest_rate().await;

                let apartment_yield = match interest_rate {
                    Ok(interest_rate) => calculate_rental_yield(
                        apartment.price.clone().unwrap(),
                        apartment.rent.unwrap(),
                        apartment.additional_costs.unwrap(),
                        interest_rate,
                    ),
                    Err(e) => {
                        error!("Error while calculating rental yield: {}", e);
                        0.0
                    }
                };

                let insertable_apartment = InsertableApartment {
                    card_id: apartment.card_id,
                    location_id: apartment.location_id,
                    location_level: apartment.location_level,
                    location_name: apartment.location_name,
                    size: apartment.size,
                    rooms: apartment.rooms,
                    price: apartment.price,
                    additional_costs: apartment.additional_costs,
                    rent: apartment.rent,
                    estimated_yield: Some(apartment_yield),
                    watchlist_id: apartment.watchlist_id,
                };
                db::apartment::insert(insertable_apartment);
            }
        }
        None => println!("No apartments added.."),
    }
}

pub fn calculate_rental_yield(
    price: i32,
    rent: i32,
    additional_cost: i32,
    interest_rate: f64,
) -> f64 {
    // Calculate annual mortgage payment using the loan term and interest rate

    let annual_interest_rate = interest_rate / 100.0;
    let loan_term_months = 25 * 12;
    let monthly_interest_rate = annual_interest_rate / 12.0;
    let discount_factor = ((1.0 + monthly_interest_rate).powi(loan_term_months as i32) - 1.0)
        / (monthly_interest_rate * (1.0 + monthly_interest_rate).powi(loan_term_months as i32));

    let mortgage_payment = price as f64
        * (monthly_interest_rate
            / (1.0 - 1.0 / (1.0 + monthly_interest_rate).powi(loan_term_months as i32)))
        / discount_factor;

    // Calculate net cash flow (rent - mortgage payment)
    let net_cash_flow = rent as f64 - additional_cost as f64 - mortgage_payment;

    // Calculate rental yield (net cash flow / initial investment)
    let initial_investment = price as f64 * 0.2; // For simplicity for now, assume 20% down-payments
    let rental_yield = net_cash_flow / initial_investment;

    // Convert to yearly yield (multiply by 12)
    let yearly_yield = rental_yield * 12.0;

    yearly_yield * 100.0 // Convert to percentage
}
