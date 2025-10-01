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
        extract::{Query, Form, State as AxumState},
        response::Html,
        routing::get,
        Router,
    };
    use std::sync::Arc;
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;

    #[tokio::main]
    pub async fn main() {
        dotenv::dotenv().ok();

        // Initialize logging
        env_logger::init();

        // Create shared session data
        let session_data = Arc::new(tokio::sync::Mutex::new(rsp10::axum_adapter::SessionData {
            auth_data: None,
        }));

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
            .route("/login", get(login_handler).post(login_handler))

            // Logout routes
            .route("/logout", get(logout_handler).post(logout_handler))

            // Test state routes
            .route("/teststate", get(teststate_handler).post(teststate_handler))

            // Sleep routes
            .route("/sleep", get(sleep_handler).post(sleep_handler))

            // Root route (same as teststate)
            .route("/", get(teststate_handler).post(teststate_handler))

            // Static files
            .nest_service("/static", ServeDir::new("staticfiles/"))

            // Shared session state
            .with_state(session_data)
    }

    // Handler functions that call the auto-generated handlers from page modules
    async fn login_handler(
        Query(query): Query<std::collections::HashMap<String, String>>,
        form: Option<Form<std::collections::HashMap<String, String>>>,
        AxumState(session_data): AxumState<Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        simple_pages::login::handler()(Query(query), form, AxumState(session_data)).into_response()
    }

    async fn logout_handler(
        Query(query): Query<std::collections::HashMap<String, String>>,
        form: Option<Form<std::collections::HashMap<String, String>>>,
        AxumState(session_data): AxumState<Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        simple_pages::logout::handler()(Query(query), form, AxumState(session_data)).into_response()
    }

    async fn teststate_handler(
        Query(query): Query<std::collections::HashMap<String, String>>,
        form: Option<Form<std::collections::HashMap<String, String>>>,
        AxumState(session_data): AxumState<Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        simple_pages::teststate::handler()(Query(query), form, AxumState(session_data)).into_response()
    }

    async fn sleep_handler(
        Query(query): Query<std::collections::HashMap<String, String>>,
        form: Option<Form<std::collections::HashMap<String, String>>>,
        AxumState(session_data): AxumState<Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        simple_pages::sleep::handler()(Query(query), form, AxumState(session_data)).into_response()
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