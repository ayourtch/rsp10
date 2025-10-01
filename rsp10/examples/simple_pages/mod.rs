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

    // Register routes using web_handler()
    let login_handler = login::web_handler().to_iron();
    r.get("/login", login_handler.clone(), "GET/login".to_string());
    r.post("/login", login_handler, "POST/login".to_string());

    let logout_handler = logout::web_handler().to_iron();
    r.get("/logout", logout_handler.clone(), "GET/logout".to_string());
    r.post("/logout", logout_handler, "POST/logout".to_string());

    let teststate_handler = teststate::web_handler().to_iron();
    r.get("/teststate", teststate_handler.clone(), "GET/teststate".to_string());
    r.post("/teststate", teststate_handler.clone(), "POST/teststate".to_string());
    r.get("/", teststate_handler.clone(), "GET/".to_string());
    r.post("/", teststate_handler, "POST/".to_string());

    r
}

// Axum router
#[cfg(feature = "axum")]
pub fn get_axum_router(
    session_data: std::sync::Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>
) -> axum::Router {
    use axum::routing::{get, post};
    use tower_http::services::ServeDir;

    let login_handler = login::web_handler().to_axum();
    let logout_handler = logout::web_handler().to_axum();
    let teststate_handler = teststate::web_handler().to_axum();

    axum::Router::new()
        .route("/login", get(login_handler.clone()).post(login_handler))
        .route("/logout", get(logout_handler.clone()).post(logout_handler))
        .route("/teststate", get(teststate_handler.clone()).post(teststate_handler.clone()))
        .route("/", get(teststate_handler.clone()).post(teststate_handler))
        .nest_service("/static", ServeDir::new("staticfiles/"))
        .with_state(session_data)
}
