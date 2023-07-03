#[get("/")]
pub async fn index() -> String {
    String::from("oogalaboogala")
}
