#[macro_use]
extern crate rsp10;
extern crate chrono;
extern crate iron;
extern crate iron_sessionstorage;
extern crate mustache;
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;

mod simple_pages;

fn main() {
    dotenv::dotenv().ok();

    let router = simple_pages::get_router();
    let mut s = rsp10::RspServer::new();
    s.run(router, "Simple Example", 4480);
    println!("Listening!");
}
