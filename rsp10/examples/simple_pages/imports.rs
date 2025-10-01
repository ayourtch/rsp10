pub use rsp10::RspState;
pub use rsp10::*;

pub use mustache::MapBuilder;
pub use mustache::Template;
pub use rsp10::RspAction;
pub use rsp10::RspEvent;

pub use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct NoPageAuth {}
impl rsp10::RspUserAuth for NoPageAuth {
    fn from_request<Req: rsp10::HttpRequest>(_req: &mut Req) -> Result<NoPageAuth, String> {
        Ok(NoPageAuth {})
    }
}

pub use chrono::NaiveDateTime;

pub use iron_sessionstorage::backends::SignedCookieBackend;
pub use iron_sessionstorage::traits::*;
pub use iron_sessionstorage::SessionStorage;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CookiePageAuth {
    pub username: String,
    super_admin_until: Option<NaiveDateTime>,
    groups: HashMap<String, bool>,
}

#[allow(dead_code)]
impl CookiePageAuth {
    pub fn new(username: &str, arg_groups: Option<HashMap<String, bool>>) -> CookiePageAuth {
        let groups = if arg_groups.is_some() {
            arg_groups.unwrap()
        } else {
            HashMap::new()
        };
        CookiePageAuth {
            username: format!("{}", username),
            super_admin_until: None,
            groups: groups,
        }
    }
    pub fn is_admin(&self) -> bool {
        self.groups.contains_key("administrators")
    }
    pub fn is_super_admin(&self) -> bool {
        false
    }
}

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
            let maybe_res: Result<CookiePageAuth, serde_json::Error> = serde_json::from_str(&value);
            maybe_res.ok()
        }
    }
}

impl rsp10::RspUserAuth for CookiePageAuth {
    fn from_request<Req: rsp10::HttpRequest>(_req: &mut Req) -> Result<CookiePageAuth, String> {
        // TODO: Implement proper session-based authentication once SessionStorage is fixed
        // For now, allow all requests with a default user
        Ok(CookiePageAuth::new("user", None))
    }
}
