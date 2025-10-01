/// Axum framework adapter
///
/// Implements the HTTP abstraction traits for the Axum web framework

#[cfg(feature = "axum")]
use std::collections::HashMap;
#[cfg(feature = "axum")]
use std::sync::Arc;
#[cfg(feature = "axum")]
use axum::{
    extract::{Query, Form, State as AxumState},
    http::{StatusCode, header},
    response::{Html, Response, IntoResponse},
    body::Body,
};
#[cfg(feature = "axum")]
use serde::Deserialize;
#[cfg(feature = "axum")]
use tower::ServiceBuilder;
#[cfg(feature = "axum")]
use tower_http::services::ServeDir;

#[cfg(feature = "axum")]
use crate::http_adapter::{HttpRequest, HttpResponse, HttpResult, HttpError};
#[cfg(feature = "axum")]
use crate::core::{RspState, RspUserAuth, RspEvent, RspAction, extract_event, extract_json_state, amend_json_value};
#[cfg(feature = "axum")]
use crate::core::RspKey;
#[cfg(feature = "axum")]
use crate::{maybe_compile_template};

#[cfg(feature = "axum")]
#[derive(Debug, Clone)]
pub struct SessionData {
    pub auth_data: Option<String>, // Serialized auth data
}

#[cfg(feature = "axum")]
/// Axum request adapter
pub struct AxumRequestAdapter {
    pub query_params: HashMap<String, Vec<String>>,
    pub form_data: HashMap<String, Vec<String>>,
    pub session: Arc<tokio::sync::Mutex<SessionData>>,
}

#[cfg(feature = "axum")]
impl AxumRequestAdapter {
    pub fn new(
        query: Query<HashMap<String, String>>,
        form: Option<Form<HashMap<String, String>>>,
        session: Arc<tokio::sync::Mutex<SessionData>>,
    ) -> Self {
        let mut query_params = HashMap::new();
        for (key, value) in query.0 {
            query_params.insert(key, vec![value]);
        }

        let mut form_data = HashMap::new();
        if let Some(Form(form_map)) = form {
            for (key, value) in form_map {
                form_data.insert(key, vec![value]);
            }
        }

        Self {
            query_params,
            form_data,
            session,
        }
    }
}

#[cfg(feature = "axum")]
impl HttpRequest for AxumRequestAdapter {
    fn query_params(&mut self) -> Result<HashMap<String, Vec<String>>, String> {
        Ok(self.query_params.clone())
    }

    fn form_data(&mut self) -> Result<HashMap<String, Vec<String>>, String> {
        Ok(self.form_data.clone())
    }

    fn get_session<T: 'static>(&mut self) -> Option<&T> {
        // For simplicity, we'll store session data as a single type for now
        // In a more complete implementation, this would use a type map
        None
    }

    fn set_session<T: 'static>(&mut self, value: T) {
        // For simplicity, we'll store session data as a single type for now
        // In a more complete implementation, this would use a type map
    }

    fn get_state<T: 'static>(&self) -> Option<&T> {
        // For Axum, this would get state from the State extractor
        None
    }
}

#[cfg(feature = "axum")]
/// Axum response builder
pub struct AxumResponseBuilder {
    content: String,
    status_code: u16,
    headers: Vec<(String, String)>,
}

#[cfg(feature = "axum")]
impl HttpResponse for AxumResponseBuilder {
    fn html(content: String) -> Self {
        AxumResponseBuilder {
            content,
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html; charset=utf-8".to_string()),
            ],
        }
    }

    fn redirect(location: &str) -> Self {
        AxumResponseBuilder {
            content: location.to_string(),
            status_code: 302,
            headers: vec![
                ("Location".to_string(), location.to_string()),
            ],
        }
    }

    fn error(status_code: u16, message: String) -> Self {
        AxumResponseBuilder {
            content: message,
            status_code,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        }
    }

    fn set_header(&mut self, name: &str, value: &str) {
        self.headers.push((name.to_string(), value.to_string()));
    }
}

#[cfg(feature = "axum")]
impl AxumResponseBuilder {
    pub fn into_axum_response(self) -> impl IntoResponse {
        let mut response = axum::response::Html(self.content).into_response();

        // Set status code
        let status = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        *response.status_mut() = status;

        // Set headers
        for (name, value) in self.headers {
            response.headers_mut().insert(
                header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                header::HeaderValue::from_str(&value).unwrap(),
            );
        }

        response
    }
}

#[cfg(feature = "axum")]
/// Generic Axum handler that works with any RspState implementation
pub async fn axum_handler_fn<S, T, TA>(
    args: (
        axum::extract::Query<HashMap<String, String>>,
        Option<axum::extract::Form<HashMap<String, String>>>,
        axum::extract::State<Arc<tokio::sync::Mutex<SessionData>>>,
    ),
) -> axum::http::Response<axum::body::Body>
where
    S: RspState<T, TA> + 'static,
    T: RspKey + Send + Sync + 'static,
    TA: RspUserAuth + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Default + 'static,
{
    let (query, form, session_state) = args;
    // Extract the session data from AxumState
    let session = session_state.0.clone();

    // Create adapter for request processing
    let mut adapter = AxumRequestAdapter::new(query, form, session);

    // For now, use NoPageAuth as default authentication
    // TODO: Implement proper session-based authentication
    let auth = TA::from_request(&mut adapter).unwrap_or_else(|_| {
        // Default to no authentication for now
        TA::default()
    });

    // Get form data and query parameters
    let form_data = adapter.form_data().unwrap_or_default();
    let query_params = adapter.query_params().unwrap_or_default();

    // Extract event
    let event = extract_event(&form_data);

    // Get or reconstruct state
    let mut maybe_state_val: Option<serde_json::Value> =
        extract_json_state(&form_data, "state_json");

    let maybe_state = if let Some(ref mut state_val) = maybe_state_val {
        amend_json_value(state_val, &form_data);
        serde_json::from_value(state_val.clone()).ok()
    } else {
        None
    };

    let maybe_initial_state: Option<S> =
        extract_json_state(&form_data, "initial_state_json");

    // Get key
    let mut maybe_key = S::get_key(&auth, &query_params, &maybe_state);
    if maybe_key.is_none() {
        maybe_key = S::get_key_from_args(&auth, &query_params);
    }
    let mut key = maybe_key.unwrap_or_default();

    // Get current initial state
    let mut curr_initial_state = S::get_state(&auth, key.clone());
    let state_none = maybe_state.is_none();
    let initial_state_none = maybe_initial_state.is_none();
    let initial_state = maybe_initial_state.unwrap_or(curr_initial_state.clone());
    let state = maybe_state.unwrap_or(initial_state.clone());

    // Handle event
    let ri = crate::core::RspInfo {
        auth: &auth,
        event: &event,
        key: &key,
        state,
        state_none,
        initial_state,
        initial_state_none,
        curr_initial_state: &curr_initial_state,
    };

    let r = S::event_handler(ri);
    let mut initial_state = r.initial_state;
    let mut state = r.state;
    let action = r.action;
    let new_auth = r.new_auth;

    // Process action
    let mut redirect_to = String::new();
    match action {
        RspAction::Render => {}
        RspAction::ReloadState => {
            curr_initial_state = S::get_state(&auth, key.clone());
            initial_state = curr_initial_state.clone();
            state = curr_initial_state.clone();
        }
        RspAction::RedirectTo(target) => {
            redirect_to = target;
        }
        RspAction::SetKey(k) => {
            key = k;
            curr_initial_state = S::get_state(&auth, key.clone());
            initial_state = curr_initial_state.clone();
            state = curr_initial_state.clone();
        }
    }

    // Save session if a new auth was provided (simplified for now)
    if let Some(_new_auth_any) = new_auth {
        // TODO: Implement proper session management for Axum
    }

    if !redirect_to.is_empty() {
        let resp = AxumResponseBuilder::redirect(&redirect_to);
        return resp.into_axum_response().into_response();
    }

    // Render template
    let template_name = if S::get_template_name() != "" {
        S::get_template_name()
    } else {
        S::get_template_name_auto()
    };

    let template = match maybe_compile_template(&template_name) {
        Ok(t) => t,
        Err(e) => {
            let resp = AxumResponseBuilder::error(500, format!("Template error: {}", e));
            return resp.into_axum_response().into_response();
        }
    };

    // Fill data
    let ri = crate::core::RspInfo {
        auth: &auth,
        event: &event,
        key: &key,
        state,
        state_none: false,
        initial_state,
        initial_state_none: false,
        curr_initial_state: &curr_initial_state,
    };

    let r = S::fill_data(ri);
    let initial_state = r.initial_state;
    let state = r.state;
    let data = r.data;

    // Build template data
    let data = data.insert("auth", &auth).unwrap();
    let data = data.insert("state", &state).unwrap();
    let data = data.insert("state_key", &key).unwrap();
    let data = data.insert("initial_state", &initial_state).unwrap();
    let data = data.insert("curr_initial_state", &curr_initial_state).unwrap();
    let data = data
        .insert("state_json", &serde_json::to_string(&state).unwrap())
        .unwrap();
    let data = data
        .insert("state_key_json", &serde_json::to_string(&key).unwrap())
        .unwrap();
    let data = data
        .insert("initial_state_json", &serde_json::to_string(&initial_state).unwrap())
        .unwrap();
    let data = data
        .insert("curr_initial_state_json", &serde_json::to_string(&curr_initial_state).unwrap())
        .unwrap();

    // Render
    let mut bytes = vec![];
    let data_built = data.build();
    template
        .render_data(&mut bytes, &data_built)
        .expect("Failed to render");
    let payload = std::str::from_utf8(&bytes).unwrap();

    let resp = AxumResponseBuilder::html(payload.to_string());
    resp.into_axum_response().into_response()
}

#[cfg(feature = "axum")]
/// Generic Axum handler factory that returns a proper Handler implementation
pub fn make_axum_handler<S, T, TA>() -> impl Fn(
    axum::extract::Query<HashMap<String, String>>,
    Option<axum::extract::Form<HashMap<String, String>>>,
    axum::extract::State<Arc<tokio::sync::Mutex<SessionData>>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::http::Response<axum::body::Body>> + Send>>
where
    S: RspState<T, TA> + 'static,
    T: RspKey + Send + Sync + 'static,
    TA: RspUserAuth + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Default + 'static,
{
    move |query, form, session_state| {
        Box::pin(axum_handler_fn::<S, T, TA>((query, form, session_state)))
    }
}