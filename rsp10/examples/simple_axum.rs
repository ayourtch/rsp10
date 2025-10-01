extern crate dotenv;

mod simple_pages;

#[cfg(feature = "axum")]
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let server = rsp10::axum_adapter::RspAxumServer::new();
    let router = simple_pages::get_axum_router(server.session_data());
    server.run(router, "Simple Example", 4480).await;
}

// Fallback for when axum feature is not enabled
#[cfg(not(feature = "axum"))]
fn main() {
    eprintln!("Error: This example requires the 'axum' feature to be enabled.");
    eprintln!("Run with: cargo run --example simple_axum --features axum");
    std::process::exit(1);
}