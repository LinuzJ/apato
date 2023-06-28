#[macro_use]
extern crate rocket;

mod fetch_tokens;

use fetch_tokens::get_tokens;

#[get("/")]
async fn index() -> String {
    let tokens: fetch_tokens::OikotieTokens = get_tokens().await;
    tokens.token
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
