mod routes;
use rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = apato::rocket().await.launch().await?;
    Ok(())
}
