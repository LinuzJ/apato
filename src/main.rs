#[macro_use]
extern crate rocket;

use rand::Rng;
use reqwest;

struct OikotieTokens {
    loaded: String,
    cuid: String,
    token: String,
}

fn fetch_oikotie_tokens() -> OikotieTokens {
    let client: reqwest::blocking::Client = reqwest::blocking::Client::new();

    let binding: String = rand
        ::thread_rng()
        .gen_range(8000..40000)
        .to_string();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &binding)];

    let res: Result<reqwest::blocking::Response, reqwest::Error> = client
        .get("https://asunnot.oikotie.fi/user/get?format=json&rand=7123")
        .send();

    let response = match res {
        Ok(resp) => resp.text().unwrap(),
        Err(err) => panic!("Error: {}", err),
    };

    println!("{:?}", response);

    return OikotieTokens {
        loaded: String::from("a"),
        cuid: String::from("b"),
        token: String::from("c"),
    };
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let tokens: OikotieTokens = fetch_oikotie_tokens();
    let rocket = rocket::build();
    rocket.mount("/", routes![index])
}
