use log::error;

use crate::{
    db::{self, establish_connection},
    interest_rate::interest_rate_client,
    models::apartment::Apartment,
    oikotie::oikotie::Oikotie,
};

pub async fn process_apartment_calculations(
    potential_apartments: Option<Vec<Apartment>>,
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

                // Get expected rent for this apartment
                let expected_rent = oikotie.get_estimated_rent(&apartment).await;

                match expected_rent {
                    Ok(rent) => apartment.rent = rent,
                    Err(e) => error!("{}", e),
                }
                db::apartment::insert(&mut establish_connection(), apartment.clone());

                // Get interest rate from Nordea
                let interest_rate_option = interest_rate_client::get_interest_rate().await;
                let apartment_yield = match interest_rate_option {
                    Ok(interest_rate) => calculate_rental_yield(
                        apartment.price.clone(),
                        apartment.rent,
                        apartment.additional_costs,
                        interest_rate,
                    ),
                    Err(e) => {
                        error!("{}", e);
                        0.0
                    }
                };

                println!("YIELD: {:?}", apartment_yield)
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

    println!("MORTAGE {:?}", mortgage_payment);

    // Calculate net cash flow (rent - mortgage payment)
    let net_cash_flow = rent as f64 - additional_cost as f64 - mortgage_payment;

    // Calculate rental yield (net cash flow / initial investment)
    let initial_investment = price as f64 * 0.2; // For simpolicity for now, assume 20% downpayments
    let rental_yield = net_cash_flow / initial_investment;

    // Convert to yearly yield (multiply by 12)
    let yearly_yield = rental_yield * 12.0;

    yearly_yield * 100.0 // Convert to percentage
}
