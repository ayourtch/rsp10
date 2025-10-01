/// Common authentication types for rsp10
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::{RspUserAuth, HttpRequest};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct NoPageAuth {}

impl RspUserAuth for NoPageAuth {
    fn from_request<Req: HttpRequest>(_req: &mut Req) -> Result<NoPageAuth, String> {
        Ok(NoPageAuth {})
    }
}

#[cfg(feature = "iron")]
impl iron::typemap::Key for NoPageAuth {
    type Value = NoPageAuth;
}

#[cfg(feature = "iron")]
impl iron_sessionstorage::Value for NoPageAuth {
    fn get_key() -> &'static str {
        "rsp10_no_auth"
    }
    fn into_raw(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn from_raw(value: String) -> Option<Self> {
        serde_json::from_str(&value).ok()
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CookiePageAuth {
    pub username: String,
    super_admin_until: Option<NaiveDateTime>,
    groups: HashMap<String, bool>,
}

impl CookiePageAuth {
    pub fn new(username: &str, arg_groups: Option<HashMap<String, bool>>) -> CookiePageAuth {
        let groups = arg_groups.unwrap_or_default();
        CookiePageAuth {
            username: format!("{}", username),
            super_admin_until: None,
            groups,
        }
    }
    pub fn is_admin(&self) -> bool {
        self.groups.contains_key("administrators")
    }
    pub fn is_super_admin(&self) -> bool {
        false
    }
}

impl RspUserAuth for CookiePageAuth {
    fn from_request<Req: HttpRequest>(_req: &mut Req) -> Result<CookiePageAuth, String> {
        // Session checking is handled by the Iron handler
        // If there's no session, redirect to login
        Err("/login".to_string())
    }
}

#[cfg(feature = "iron")]
impl iron::typemap::Key for CookiePageAuth {
    type Value = CookiePageAuth;
}

#[cfg(feature = "iron")]
impl iron_sessionstorage::Value for CookiePageAuth {
    fn get_key() -> &'static str {
        "rsp10_session_cookie"
    }
    fn into_raw(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            serde_json::from_str(&value).ok()
        }
    }
}
