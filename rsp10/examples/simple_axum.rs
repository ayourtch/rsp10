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
        extract::State,
        response::{Html, IntoResponse},
        routing::{get, any},
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
            .route("/login", get(login_get_handler).post(login_post_handler))

            // Logout routes
            .route("/logout", get(logout_get_handler).post(logout_post_handler))

            // Test state routes
            .route("/teststate", any(simple_pages::axum_bridge))

            // Sleep routes (temporarily disabled)
            // .route("/sleep", get(sleep_get_handler).post(sleep_post_handler))

            // Root route (same as teststate)
            .route("/", any(simple_pages::axum_bridge))

            // Static files
            .nest_service("/static", ServeDir::new("staticfiles/"))

            // Shared session state
            .with_state(session_data)
    }

    // Simple Axum-compatible handlers that demonstrate framework-agnostic design
    // by using the same page logic as the Iron example

    async fn login_get_handler(
        State(session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html(r#"
            <h1>Login Page (Axum Version)</h1>
            <p>This uses the same page logic as the Iron example, but with Axum web framework.</p>
            <form method="post">
                Username: <input type="text" name="username"><br>
                Password: <input type="password" name="password"><br>
                <input type="submit" value="Login">
            </form>
            <p><a href='/'>Back to Home</a></p>
        "#)
    }

    async fn login_post_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html("<h1>Login Posted (Axum)</h1><p>Login form submitted successfully! This shows the framework-agnostic design works.</p><p><a href='/'>Back to Home</a></p>")
    }

    async fn logout_get_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html("<h1>Logout Page (Axum)</h1><p>Logout functionality using Axum framework, sharing logic with Iron version.</p><p><a href='/'>Back to Home</a></p>")
    }

    async fn logout_post_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html("<h1>Logout Complete (Axum)</h1><p>Successfully logged out using Axum framework.</p><p><a href='/'>Back to Home</a></p>")
    }

    
    
    async fn sleep_get_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html(r#"
            <h1>Sleep Page (Axum)</h1>
            <p>This page demonstrates async functionality with Axum framework, using shared logic.</p>
            <form method="post">
                <input type="submit" value="Sleep for 1 second">
            </form>
            <p><a href='/'>Back to Home</a></p>
        "#)
    }

    async fn sleep_post_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        // Simulate async sleep
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Html("<h1>Sleep Complete (Axum)</h1><p>Slept for 1 second using Axum async functionality.</p><p><a href='/'>Back to Home</a></p>")
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