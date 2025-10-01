#[macro_use]
extern crate rsp10;
extern crate chrono;
extern crate dotenv;

mod simple_pages;

// Put all Axum-specific code in a module
#[cfg(feature = "axum")]
mod axum_impl {
    use super::*;
    use axum::{
        routing::any,
        Router,
    };
    use std::sync::Arc;
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;
    use rsp10::axum_adapter::SessionData;

    #[tokio::main]
    pub async fn main() {
        dotenv::dotenv().ok();

        // Initialize logging
        env_logger::init();

        // Create shared session data
        let session_data = Arc::new(tokio::sync::Mutex::new(rsp10::axum_adapter::SessionData::default()));

        // Build the router
        let app = create_router(session_data);

        // Run the server
        let addr = SocketAddr::from(([127, 0, 0, 1], 4480));
        println!("HTTP server for Simple Example (Axum) starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    fn create_router(session_data: Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>) -> Router {
        Router::new()
            // Login routes
            .route("/login", any(simple_pages::login_bridge))

            // Logout routes
            .route("/logout", any(simple_pages::logout_bridge))

            // Test state routes
            .route("/teststate", any(simple_pages::teststate_bridge))

            // Sleep routes (temporarily disabled)
            // .route("/sleep", get(sleep_get_handler).post(sleep_post_handler))

            // Root route (same as teststate)
            .route("/", any(simple_pages::teststate_bridge))

            // Static files
            .nest_service("/static", ServeDir::new("staticfiles/"))

            // Shared session state
            .with_state(session_data)
    }

}

// Fallback for when axum feature is not enabled
#[cfg(not(feature = "axum"))]
fn main() {
    eprintln!("Error: This example requires the 'axum' feature to be enabled.");
    eprintln!("Run with: cargo run --example simple_axum --features axum");
    std::process::exit(1);
}

#[cfg(feature = "axum")]
fn main() {
    axum_impl::main();
}