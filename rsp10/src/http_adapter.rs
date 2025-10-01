/// HTTP abstraction layer - framework agnostic
///
/// This module defines traits that abstract away the specific HTTP framework,
/// allowing rsp10 core logic to work with any HTTP server (Iron, Actix, Axum, etc.)

use std::collections::HashMap;

/// Abstract HTTP request interface
pub trait HttpRequest {
    /// Get query parameters as a map
    fn query_params(&mut self) -> Result<HashMap<String, Vec<String>>, String>;

    /// Get POST form data as a map
    fn form_data(&mut self) -> Result<HashMap<String, Vec<String>>, String>;

    /// Get a specific form field value
    fn get_form_field(&mut self, name: &str) -> Option<String> {
        self.form_data()
            .ok()
            .and_then(|map| map.get(name).and_then(|v| v.first().cloned()))
    }

    /// Get session data by type
    fn get_session<T: 'static>(&mut self) -> Option<&T>;

    /// Set session data
    fn set_session<T: 'static>(&mut self, value: T);

    /// Get global state by type
    fn get_state<T: 'static>(&self) -> Option<&T>;
}

/// Abstract HTTP response builder
pub trait HttpResponse {
    /// Create a new response with HTML content
    fn html(content: String) -> Self;

    /// Create a redirect response
    fn redirect(location: &str) -> Self;

    /// Create an error response
    fn error(status: u16, message: String) -> Self;

    /// Set a header on the response
    fn set_header(&mut self, name: &str, value: &str);
}

/// Result type for HTTP handlers
pub type HttpResult<R> = Result<R, HttpError>;

/// HTTP error type
#[derive(Debug)]
pub enum HttpError {
    BadRequest(String),
    Unauthorized(String),
    InternalError(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            HttpError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            HttpError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

// Note: HttpAdapter trait removed - we use concrete types instead
// Each framework adapter provides its own concrete request wrapper
