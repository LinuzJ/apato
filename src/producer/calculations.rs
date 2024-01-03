use log::error;

use crate::{
    db::{self, establish_connection},
    interest_rate::interest_rate_client,
    models::apartment::Apartment,
    oikotie::oikotie::Oikotie,
};

pub async fn calculate_yields_for_apartments(
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
                db::apartment::insert(&mut establish_connection(), apartment);

                // Get interest rate from Nordea
                let estimated_interest_rate = interest_rate_client::get_interest_rate().await;
                match estimated_interest_rate {
                    Ok(ir) => println!("{}", ir),
                    Err(e) => error!("{}", e),
                }
            }
        }
        None => println!("No apartments added.."),
    }
}
