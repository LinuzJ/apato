#[macro_use]
extern crate rocket;

use rand::Rng;
use reqwest::Client;
use reqwest::Response;

struct OikotieTokens {
    loaded: String,
    cuid: String,
    token: String,
}

async fn fetch_oikotie_tokens() -> &'static str {
    let client: Client = Client::builder().build().unwrap();

    // let random_number: String = rand
    //     ::thread_rng()
    //     .gen_range(8000..40000)
    //     .to_string();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", "7123")];

    let client = Client::new();
    let response = client.get("https://asunnot.oikotie.fi/user/get")
                            .query(&params)
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await;

    println!("{:?}", response);
    "asd"
}

#[get("/")]
async fn index() -> &'static str {
    let tokens = fetch_oikotie_tokens().await;
    "heh"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}
