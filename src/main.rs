use apato;
use rocket;

#[rocket::main]
async fn main() {
    let _ = apato::rocket().await.launch().await;
}
