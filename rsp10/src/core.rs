/// Core rsp10 logic - framework agnostic
///
/// This module contains the core state management and page handling logic
/// without any dependencies on specific HTTP frameworks

use std::collections::HashMap;
use std::fmt::Debug;

use serde;
use serde_json;

use crate::http_adapter::{HttpRequest, HttpResponse, HttpResult, HttpError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RspEvent {
    pub event: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RspAction<T> {
    Render,
    SetKey(T),
    ReloadState,
    RedirectTo(String),
}

/// Extract event from form data
pub fn extract_event(form_data: &HashMap<String, Vec<String>>) -> RspEvent {
    let mut event: String = "unknown".into();
    let mut target: String = "".into();

    match form_data.get("event") {
        Some(a) => {
            event = a[0].clone();
        }
        _ => {}
    }
    match form_data.get("event_target") {
        Some(a) => {
            target = a[0].clone();
        }
        _ => {}
    }

    if &event == "unknown" && &target == "" {
        // Look for submit buttons
        match form_data.keys().find(|x| x.starts_with("submit")) {
            Some(a) => {
                event = "submit".into();
                target = a["submit".len()..].into();
            }
            _ => match form_data.keys().find(|x| x.starts_with("btn")) {
                Some(a) => {
                    event = "submit".into();
                    target = a.clone();
                }
                _ => {}
            },
        }
    }

    RspEvent { event, target }
}

/// Extract JSON state from form field
pub fn extract_json_state<S>(form_data: &HashMap<String, Vec<String>>, field_name: &str) -> Option<S>
where
    S: serde::de::DeserializeOwned,
{
    form_data
        .get(field_name)
        .and_then(|v| v.first())
        .and_then(|json_str| serde_json::from_str(json_str).ok())
}

/// Amend JSON value with form data
pub fn amend_json_value(
    orig_val: &mut serde_json::Value,
    form_data: &HashMap<String, Vec<String>>,
) {
    amend_value_recursive("", orig_val, form_data);
}

fn amend_value_recursive(
    name_prefix: &str,
    orig_val: &mut serde_json::Value,
    form_data: &HashMap<String, Vec<String>>,
) {
    use serde_json::Value::*;
    match orig_val {
        Object(ref mut obj) => {
            for (key, mut value) in obj.iter_mut() {
                let new_prefix = if name_prefix == "" {
                    format!("{}", key)
                } else {
                    format!("{}__{}", name_prefix, key)
                };
                amend_value_recursive(&new_prefix, value, form_data);
            }
        }
        Array(ref mut arr) => {
            for (i, elt) in arr.iter_mut().enumerate() {
                let new_prefix = format!("{}__{}", name_prefix, i);
                amend_value_recursive(&new_prefix, elt, form_data);
            }
        }
        ref x => {
            if form_data.contains_key(name_prefix) {
                let new_val_src = form_data[name_prefix].clone();
                let src = &new_val_src[0];
                match x {
                    Bool(ref _val) => {
                        let new_val = match src.as_ref() {
                            "true" | "on" | "checked" => true,
                            _ => false,
                        };
                        *orig_val = Bool(new_val);
                    }
                    String(ref _val) => {
                        *orig_val = String(src.to_string());
                    }
                    _ => {
                        if let Ok(parsed) = serde_json::from_str(src) {
                            *orig_val = parsed;
                        }
                    }
                }
            } else {
                // Workaround for unchecked checkboxes
                let sentinel_key = format!("{}_sentinel", name_prefix);
                if form_data.contains_key(&sentinel_key) {
                    let new_val_src = form_data[&sentinel_key].clone();
                    let src = &new_val_src[0];
                    if let Bool(_) = x {
                        let new_val = matches!(src.as_ref(), "true" | "on" | "checked");
                        *orig_val = Bool(new_val);
                    }
                }
            }
        }
    }
}

/// Core state management info passed to handlers
pub struct RspInfo<'a, R, T, TA> {
    pub auth: &'a TA,
    pub event: &'a RspEvent,
    pub key: &'a T,
    pub state_none: bool,
    pub state: R,
    pub initial_state: R,
    pub initial_state_none: bool,
    pub curr_initial_state: &'a R,
}

pub struct RspEventHandlerResult<R, T> {
    pub state: R,
    pub initial_state: R,
    pub action: RspAction<T>,
    pub new_auth: Option<Box<dyn std::any::Any>>,  // Optional new auth to store in session
}

pub struct RspFillDataResult<R> {
    pub state: R,
    pub initial_state: R,
    pub data: mustache::MapBuilder,
}

/// User authentication trait
pub trait RspUserAuth
where
    Self: std::marker::Sized,
{
    fn from_request<Req: HttpRequest>(req: &mut Req) -> Result<Self, String>;
}

/// Core state trait - framework agnostic
pub trait RspState<T, TA>
where
    Self: std::marker::Sized + serde::Serialize + serde::de::DeserializeOwned + Clone + Debug,
    TA: RspUserAuth + serde::Serialize,
    T: serde::Serialize + Debug + Clone + Default + serde::de::DeserializeOwned,
{
    /// Get initial state based on key - no HTTP framework dependency
    fn get_state(auth: &TA, key: T) -> Self;

    /// Fill data result helper
    fn fill_data_result(ri: RspInfo<Self, T, TA>, gd: crate::RspDataBuilder) -> RspFillDataResult<Self> {
        let data = mustache::MapBuilder::new();
        let initial_state = ri.initial_state;
        let state = ri.state;
        RspFillDataResult {
            initial_state,
            state,
            data: gd.build(data),
        }
    }

    /// Event handler - pure function
    fn event_handler(ri: RspInfo<Self, T, TA>) -> RspEventHandlerResult<Self, T> {
        let action = RspAction::Render;
        let initial_state = ri.initial_state;
        let state = ri.state;

        RspEventHandlerResult {
            initial_state,
            state,
            action,
            new_auth: None,
        }
    }

    /// Get key from query parameters
    fn get_key(
        auth: &TA,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<Self>,
    ) -> Option<T> {
        None
    }

    /// Default key extraction using req2struct
    fn default_get_key_from_args(
        _auth: &TA,
        args: &HashMap<String, Vec<String>>,
    ) -> Option<T> {
        req2struct::from_map(&args).ok()
    }

    /// Get key from query args (framework agnostic)
    fn get_key_from_args(auth: &TA, args: &HashMap<String, Vec<String>>) -> Option<T> {
        Self::default_get_key_from_args(auth, args)
    }

    /// Fill data for template rendering
    fn fill_data(ri: RspInfo<Self, T, TA>) -> RspFillDataResult<Self> {
        let data = mustache::MapBuilder::new();
        let initial_state = ri.initial_state;
        let state = ri.state;
        RspFillDataResult {
            initial_state,
            state,
            data,
        }
    }

    /// Get template name (override if needed)
    fn get_template_name() -> String {
        "".into()
    }

    /// Auto-generate template name from type
    fn get_template_name_auto() -> String {
        use std::any::type_name;
        fn test_type<T: ?Sized>() -> String {
            let full_type_name = type_name::<T>();
            let components: Vec<String> =
                full_type_name.split("::").map(|x| x.to_string()).collect();
            components[components.len() - 2].to_string()
        }
        test_type::<Self>()
    }
}
