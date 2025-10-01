/// Iron framework adapter
///
/// Implements the HTTP abstraction traits for the Iron web framework

use std::collections::HashMap;
use iron::prelude::*;
use iron::{status, Handler, Plugin};
use iron_sessionstorage::{SessionStorage, Value, SessionRequestExt};
use persistent::State;
use urlencoded::{UrlEncodedBody, UrlEncodedQuery};

use crate::http_adapter::{HttpRequest, HttpResponse, HttpResult, HttpError};
use crate::core::{RspState, RspUserAuth, RspEvent, RspAction, extract_event, extract_json_state, amend_json_value};
use crate::{Rsp10GlobalData, maybe_compile_template};

/// Wrapper to implement HttpRequest for Iron's Request
pub struct IronRequestAdapter<'req, 'a, 'b> {
    req: &'req mut Request<'a, 'b>,
}

impl<'req, 'a, 'b> IronRequestAdapter<'req, 'a, 'b> {
    pub fn new(req: &'req mut Request<'a, 'b>) -> Self {
        IronRequestAdapter { req }
    }

    /// Iron-specific: Get session value from extensions
    pub fn get_iron_session<T: 'static + iron::typemap::Key<Value = T>>(&self) -> Option<&T> {
        self.req.extensions.get::<T>()
    }

    /// Iron-specific: Set session value in extensions
    pub fn set_iron_session<T: 'static + iron::typemap::Key<Value = T>>(&mut self, value: T) {
        self.req.extensions.insert::<T>(value);
    }
}

impl<'req, 'a, 'b> HttpRequest for IronRequestAdapter<'req, 'a, 'b> {
    fn query_params(&mut self) -> Result<HashMap<String, Vec<String>>, String> {
        match self.req.get_ref::<UrlEncodedQuery>() {
            Ok(hashmap) => Ok(hashmap.clone()),
            Err(e) => Err(format!("Failed to get query params: {:?}", e)),
        }
    }

    fn form_data(&mut self) -> Result<HashMap<String, Vec<String>>, String> {
        match self.req.get_ref::<UrlEncodedBody>() {
            Ok(hashmap) => Ok(hashmap.clone()),
            Err(e) => Err(format!("Failed to get form data: {:?}", e)),
        }
    }

    fn get_session<T: 'static>(&mut self) -> Option<&T> {
        // Generic session access doesn't work due to Key trait requirements
        // For Iron-specific code, use extensions directly or get_iron_session()
        None
    }

    fn set_session<T: 'static>(&mut self, _value: T) {
        // Generic session access doesn't work due to Key trait requirements
        // For Iron-specific code, use extensions directly or set_iron_session()
    }

    fn get_state<T: 'static>(&self) -> Option<&T> {
        None // TODO: Implement state support
    }
}

/// Iron response builder
pub struct IronResponseBuilder {
    content: String,
    status: status::Status,
    headers: Vec<(String, String)>,
}

impl HttpResponse for IronResponseBuilder {
    fn html(content: String) -> Self {
        IronResponseBuilder {
            content,
            status: status::Ok,
            headers: vec![
                ("Content-Type".to_string(), "text/html; charset=utf-8".to_string()),
                ("Connection".to_string(), "close".to_string()),
            ],
        }
    }

    fn redirect(location: &str) -> Self {
        IronResponseBuilder {
            content: location.to_string(),
            status: status::Found,
            headers: vec![
                ("Content-Type".to_string(), "text/html; charset=utf-8".to_string()),
                ("Location".to_string(), location.to_string()),
            ],
        }
    }

    fn error(status_code: u16, message: String) -> Self {
        let status = match status_code {
            400 => status::BadRequest,
            401 => status::Unauthorized,
            404 => status::NotFound,
            500 => status::InternalServerError,
            _ => status::InternalServerError,
        };
        IronResponseBuilder {
            content: message,
            status,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        }
    }

    fn set_header(&mut self, name: &str, value: &str) {
        self.headers.push((name.to_string(), value.to_string()));
    }
}

impl IronResponseBuilder {
    pub fn into_iron_response(self) -> Response {
        use iron::headers::{ContentType, Connection, Location};
        use iron::modifiers::Header;

        let mut resp = Response::with((self.status, self.content.clone()));

        for (name, value) in &self.headers {
            match name.as_str() {
                "Content-Type" if value.contains("text/html") => {
                    resp.headers.set(ContentType::html());
                }
                "Connection" if value == "close" => {
                    resp.headers.set(Connection::close());
                }
                "Location" => {
                    resp.headers.set(Location(value.clone()));
                }
                _ => {
                    // Generic header setting
                }
            }
        }

        resp
    }
}

/// Iron handler implementation for RspState pages
///
/// This creates a handler that integrates with Iron's routing
#[derive(Clone)]
pub struct RspIronHandler<S, T, TA>
where
    S: RspState<T, TA> + Send + Sync + 'static,
    T: serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned + Send + Sync + 'static,
    TA: RspUserAuth + serde::Serialize + Send + Sync + Value + Clone + iron::typemap::Key<Value = TA> + 'static,
{
    _phantom: std::marker::PhantomData<(S, T, TA)>,
}

impl<S, T, TA> RspIronHandler<S, T, TA>
where
    S: RspState<T, TA> + Send + Sync + 'static,
    T: serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned + Send + Sync + 'static,
    TA: RspUserAuth + serde::Serialize + Send + Sync + Value + Clone + iron::typemap::Key<Value = TA> + 'static,
{
    pub fn new() -> Self {
        RspIronHandler {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, T, TA> Handler for RspIronHandler<S, T, TA>
where
    S: RspState<T, TA> + Send + Sync + 'static,
    T: serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned + Send + Sync + 'static,
    TA: RspUserAuth + serde::Serialize + Send + Sync + Value + Clone + iron::typemap::Key<Value = TA> + 'static,
{
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Load authenticated user from session
        let session_auth: Option<TA> = req.session().get::<TA>().ok().and_then(|x| x);

        let mut adapter = IronRequestAdapter::new(req);

        let auth = if let Some(auth_from_session) = session_auth {
            // User is authenticated via session
            auth_from_session
        } else {
            // No session - call from_request to authenticate
            let auth_res = TA::from_request(&mut adapter);

            match auth_res {
                Ok(a) => a,
                Err(login_url) => {
                    let resp = IronResponseBuilder::redirect(&login_url);
                    return Ok(resp.into_iron_response());
                }
            }
        };

        // Get form data
        let form_data = adapter.form_data().unwrap_or_default();
        let query_params = adapter.query_params().unwrap_or_default();

        // Extract event
        let event = extract_event(&form_data);

        // Get or reconstruct state
        let mut maybe_state_val: Option<serde_json::Value> =
            extract_json_state(&form_data, "state_json");

        let mut maybe_state = if let Some(ref mut state_val) = maybe_state_val {
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

        // Save session if a new auth was provided
        if let Some(new_auth_any) = new_auth {
            // Try to downcast to TA first (same type as page auth)
            if let Some(new_auth_typed) = new_auth_any.downcast_ref::<TA>() {
                // Store in session - will be persisted as signed cookie
                let _ = req.session().set(new_auth_typed.clone());
                // Also store in extensions for immediate access
                req.extensions.insert::<TA>(new_auth_typed.clone());
            } else {
                // If downcast to TA fails, try concrete auth types
                // This allows login pages (with NoPageAuth) to return CookiePageAuth
                if let Some(new_auth_cookie) = new_auth_any.downcast_ref::<crate::CookiePageAuth>() {
                    let _ = req.session().set(new_auth_cookie.clone());
                }
            }
        }

        if !redirect_to.is_empty() {
            let resp = IronResponseBuilder::redirect(&redirect_to);
            return Ok(resp.into_iron_response());
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
                let resp = IronResponseBuilder::error(500, format!("Template error: {}", e));
                return Ok(resp.into_iron_response());
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

        let resp = IronResponseBuilder::html(payload.to_string());
        Ok(resp.into_iron_response())
    }
}

/// Helper factory function for creating handlers
pub fn make_iron_handler<S, T, TA>() -> RspIronHandler<S, T, TA>
where
    S: RspState<T, TA> + Send + Sync + 'static,
    T: serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned + Send + Sync + 'static,
    TA: RspUserAuth + serde::Serialize + Send + Sync + Value + Clone + iron::typemap::Key<Value = TA> + 'static,
{
    RspIronHandler::new()
}

/// Helper to get global data from Iron request
pub fn request_stop(req: &mut Request) {
    let glob = req.get::<State<Rsp10GlobalData>>().unwrap();
    if let Ok(globals) = (*glob).write() {
        globals.request_stop();
    };
}
