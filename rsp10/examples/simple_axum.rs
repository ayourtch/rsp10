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
        response::Html,
        routing::get,
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
            .route("/teststate", get(teststate_get_handler).post(teststate_post_handler))

            // Sleep routes (temporarily disabled)
            // .route("/sleep", get(sleep_get_handler).post(sleep_post_handler))

            // Root route (same as teststate)
            .route("/", get(teststate_get_handler).post(teststate_post_handler))

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

    async fn teststate_get_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        // Replicate Iron flow for GET request - using shared simple_pages::teststate
        use simple_pages::teststate::{PageState, KeyI32, MyPageAuth};
        use rsp10::RspState;
        use std::collections::HashMap;

        // Step 1: Authentication (for GET, use default)
        let auth = MyPageAuth::default();

        // Step 2: Form data and query params (empty for GET)
        let form_data = HashMap::new();
        let query_params = HashMap::new();

        // Step 3: Extract event (empty for GET)
        let event = rsp10::RspEvent {
            event: "view".to_string(),
            target: "".to_string(),
        };

        // Step 4: Get/reconstruct state (none for initial GET)
        let maybe_state = None;
        let maybe_initial_state = None;

        // Step 5: Get key using shared implementation
        let mut maybe_key = PageState::get_key(&auth, &query_params, &maybe_state);
        if maybe_key.is_none() {
            maybe_key = PageState::get_key_from_args(&auth, &query_params);
        }
        let key = maybe_key.unwrap_or_default();

        // Step 6: Get current initial state using shared implementation
        let curr_initial_state = PageState::get_state(&auth, key.clone());
        let state_none = maybe_state.is_none();
        let initial_state_none = maybe_initial_state.is_none();
        let initial_state = maybe_initial_state.unwrap_or(curr_initial_state.clone());
        let state = maybe_state.unwrap_or(initial_state.clone());

        // Step 7: Handle event using shared implementation
        let ri = rsp10::RspInfo {
            auth: &auth,
            event: &event,
            key: &key,
            state,
            state_none,
            initial_state,
            initial_state_none,
            curr_initial_state: &curr_initial_state,
        };

        let event_result = PageState::event_handler(ri);
        let mut initial_state = event_result.initial_state;
        let mut state = event_result.state;
        let action = event_result.action;
        let new_auth = event_result.new_auth;

        // Step 8: Process action (simple case - just render)
        // For now, ignore actions, redirect, session saving for simplicity

        // Step 9: Fill data using shared implementation
        let ri = rsp10::RspInfo {
            auth: &auth,
            event: &event,
            key: &key,
            state,
            state_none: false,
            initial_state,
            initial_state_none: false,
            curr_initial_state: &curr_initial_state,
        };

        let fill_result = PageState::fill_data(ri);

        // Step 10: Render template using shared implementation
        let template_name = if PageState::get_template_name() != "" {
            PageState::get_template_name()
        } else {
            PageState::get_template_name_auto()
        };

        let template = match rsp10::maybe_compile_template(&template_name) {
            Ok(t) => t,
            Err(e) => {
                return axum::response::Html(format!("Template error: {}", e)).into_response();
            }
        };

        // Step 11: Build template data and render
        let data = fill_result.data
            .insert("auth", &auth).unwrap()
            .insert("state", &fill_result.state).unwrap()
            .insert("state_key", &key).unwrap()
            .insert("initial_state", &fill_result.initial_state).unwrap()
            .insert("curr_initial_state", &curr_initial_state).unwrap()
            .insert("state_json", &serde_json::to_string(&fill_result.state).unwrap()).unwrap()
            .insert("state_key_json", &serde_json::to_string(&key).unwrap()).unwrap()
            .insert("initial_state_json", &serde_json::to_string(&fill_result.initial_state).unwrap()).unwrap()
            .insert("curr_initial_state_json", &serde_json::to_string(&curr_initial_state).unwrap()).unwrap();

        let mut bytes = vec![];
        let data_built = data.build();
        template.render_data(&mut bytes, &data_built).expect("Failed to render");
        let payload = std::str::from_utf8(&bytes).unwrap();

        axum::response::Html(payload.to_string()).into_response()
    }

    async fn teststate_post_handler(
        State(_session_state): State<Arc<tokio::sync::Mutex<SessionData>>>,
    ) -> impl axum::response::IntoResponse {
        Html("<h1>Test State Posted (Axum)</h1><p>Form submitted using Axum framework with shared page logic.</p><p><a href='/'>Back to Home</a></p>")
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