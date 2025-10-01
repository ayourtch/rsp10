mod imports;

#[path = "login.rs"]
mod login;
#[path = "logout.rs"]
mod logout;
#[path = "teststate.rs"]
mod teststate;
// #[path = "sleep.rs"]
// mod sleep; // Temporarily disabled due to serde issues

// Iron router
#[cfg(feature = "iron")]
pub fn get_router() -> router::Router {
    use router::Router;

    let mut r = Router::new();

    // Register routes using auto-generated handler functions
    r.get("/login", login::handler(), "GET/login".to_string());
    r.post("/login", login::handler(), "POST/login".to_string());

    r.get("/logout", logout::handler(), "GET/logout".to_string());
    r.post("/logout", logout::handler(), "POST/logout".to_string());

    r.get("/teststate", teststate::handler(), "GET/teststate".to_string());
    r.post("/teststate", teststate::handler(), "POST/teststate".to_string());

    // r.get("/sleep", sleep::handler(), "GET/sleep".to_string());
    // r.post("/sleep", sleep::handler(), "POST/sleep".to_string());

    // Mount teststate on root as well
    r.get("/", teststate::handler(), "GET/".to_string());
    r.post("/", teststate::handler(), "POST/".to_string());

    r
}

// Axum router
#[cfg(feature = "axum")]
pub fn get_axum_router(
    session_data: std::sync::Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>
) -> axum::Router {
    use axum::routing::any;
    use tower_http::services::ServeDir;

    axum::Router::new()
        // Login routes
        .route("/login", any(login::axum_handler))
        // Logout routes
        .route("/logout", any(logout::axum_handler))
        // Test state routes
        .route("/teststate", any(teststate::axum_handler))
        // Root route (same as teststate)
        .route("/", any(teststate::axum_handler))
        // Static files
        .nest_service("/static", ServeDir::new("staticfiles/"))
        // Shared session state
        .with_state(session_data)
}
