use rocket::get;

#[get("/")]
pub fn index() -> String {
    String::from("oogalaboogala")
}
