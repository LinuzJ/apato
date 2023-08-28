use crate::{
    db::{self, establish_connection},
    models::apartment::Apartment,
    oikotie::oikotie::Oikotie,
};

pub async fn calculate_yields_for_apartments(
    potential_apartments: Option<Vec<Apartment>>,
    mut oikotie: Oikotie,
) {
    match potential_apartments {
        Some(apartments) => {
            for apartment in apartments {
                /*
                   Calculate yield here
                   - Get rent for similar apartments close by
                   - Get interest rate from Nordea
                   - Calculate
                */

                let rent = oikotie.get_expected_rent(&apartment).await;

                println!("RENT: {:?}", rent);
                // Get expected rent for this apartment
                db::apartment::insert(&mut establish_connection(), apartment);
            }
        }
        None => println!("No apartments added.."),
    }
}
