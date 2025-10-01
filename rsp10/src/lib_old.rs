extern crate chrono;
extern crate mount;
extern crate regex;
extern crate staticfile;

// #[macro_use]
extern crate lazy_static;
extern crate mustache;
extern crate rsp10_derive;

// Re-export the derive macro
pub use rsp10_derive::RspState as DeriveRspState;

// #[macro_use]
extern crate iron;
extern crate iron_sessionstorage;

// #[macro_use]
extern crate router;
extern crate urlencoded;

extern crate diesel;
// extern crate time;

#[macro_use]
extern crate serde_derive;
extern crate req2struct;
extern crate serde;
extern crate serde_json;

// #[macro_use]
extern crate rand;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate socket2;

use std::collections::HashMap;
use mustache::MapBuilder;
use mustache::Template;
use std::env;

use std::fmt::Debug;

// New modular architecture
pub mod http_adapter;
pub mod core;
pub mod iron_adapter;

mod html_types;
pub use html_types::*;

pub mod foobuilder;
pub type RspDataBuilder = foobuilder::FooMapBuilder;

// Re-export core types for backward compatibility
pub use core::{RspEvent, RspAction, RspInfo, RspEventHandlerResult, RspFillDataResult, RspUserAuth, RspState};

// Re-export iron adapter for convenience
pub use iron_adapter::{make_iron_handler, IronRequestAdapter, IronResponseBuilder};

extern crate persistent;
use iron::typemap::Key;
use persistent::State;
use std::sync::{Arc, RwLock};
use iron::prelude::*;

#[derive(Clone, Debug)]
pub struct Rsp10GlobalData {
    stop_requested: Arc<RwLock<bool>>,
    test: Arc<RwLock<Option<String>>>,
}

pub fn request_stop(req: &mut Request) {
    let glob = req.get::<State<Rsp10GlobalData>>().unwrap();
    if let Ok(globals) = (*glob).write() {
        println!("Globals: {:?}", &globals);
        // globals.stop_listening();
        globals.request_stop();
        println!("stopped listening!");
    };
}

impl Rsp10GlobalData {
    fn new() -> Self {
        Rsp10GlobalData {
            stop_requested: Arc::new(RwLock::new(false)),
            test: Arc::new(RwLock::new(None)),
        }
    }

    pub fn stop_requested(&self) -> bool {
        if let Ok(mut lock) = self.stop_requested.read() {
            lock.clone()
        } else {
            false
        }
    }
    pub fn request_stop(&self) -> bool {
        if let Ok(mut lock) = self.stop_requested.write() {
            *lock = true;
            true
        } else {
            false
        }
    }

    pub fn set_test(&self, test: String) -> bool {
        if let Ok(mut lock) = self.test.write() {
            *lock = Some(test);
            true
        } else {
            false
        }
    }
    pub fn get_test(&self) -> Option<String> {
        if let Ok(mut lock) = self.test.read() {
            lock.clone()
        } else {
            None
        }
    }
}

impl Key for Rsp10GlobalData {
    type Value = Rsp10GlobalData;
}

#[macro_export]
macro_rules! rsp10_page {
    ($router: ident, $url: expr, $name: ident, $file: expr) => {
        #[path=$file]
        mod $name;
        $router.get($url, $name::PageState::handler, format!("GET/{}", $url));
        $router.post($url, $name::PageState::handler, format!("POST/{}", $url));
        $router.put($url, $name::PageState::handler, format!("PUT/{}", $url));
    };
}

#[macro_export]
macro_rules! rsp10_option_value_container {
    ($gd: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = if $state.$elt.is_some() {
            let myid = format!("{}", stringify!($elt));
            let mut $elt = HtmlValueContainer::new(&myid, &$state.$elt.clone().unwrap());
            let rc = std::rc::Rc::new(std::cell::RefCell::new($elt));
            $gd.push(rc.clone());
            Some(rc)
        } else {
            None
        };
    };
}

#[macro_export]
macro_rules! rsp10_nested_option_value_container {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = if $state.$parent[$idx].$elt.is_some() {
            let myid = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            let mut $elt =
                HtmlValueContainer::new(&myid, &$state.$parent[$idx].$elt.clone().unwrap());
            let rc = std::rc::Rc::new(std::cell::RefCell::new($elt));
            $gd.push(rc.clone());
            Some(rc)
        } else {
            None
        };
    };
}

#[macro_export]
macro_rules! rsp10_gd {
    ( $gd: ident, $elt: ident) => {
        /*
        let $gd = || {
            $gd()
                .insert(stringify!($elt), &$elt.borrow().clone())
                .unwrap()
        };
        */
        $gd.item(stringify!($elt), &$elt);
    };
}

#[macro_export]
macro_rules! rsp10_data {
    ( $elt: ident => $gd: ident) => {
        $gd.insert(stringify!($elt), &$elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_gd {
    ( $gd: ident, $parent: ident, $i: expr, $elt: ident) => {
        $gd.vector(stringify!($parent), |x| {
            x.add_field_at($i, stringify!($elt), &$elt);
        });
    };
}

#[macro_export]
macro_rules! rsp10_select {
    ( $elt: ident, $from: expr , $rinfo: ident => $gd: ident, $modified: ident) => {
        let mut $elt = std::rc::Rc::new(std::cell::RefCell::new($from.clone()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.set_selected_value(&mut $rinfo.state.$elt);
            $elt.highlight = $rinfo.state.$elt != $rinfo.initial_state.$elt;
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        rsp10_gd!($gd, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_select {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $from: expr , $rinfo: ident, $modified: ident) => {
        let mut $elt = std::rc::Rc::new(std::cell::RefCell::new($from.clone()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.set_selected_value(&mut $rinfo.state.$parent[$idx].$elt);
            $elt.highlight =
                $rinfo.state.$parent[$idx].$elt != $rinfo.initial_state.$parent[$idx].$elt;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        rsp10_nested_gd!($gd, $parent, $idx, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_text {
    ($elt: ident, $rinfo: ident => $gd: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlText>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $rinfo.state.$elt != $rinfo.initial_state.$elt;
            $elt.value = $rinfo.state.$elt.clone().to_string();
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        rsp10_gd!($gd, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_text_nogd {
    ($gd: ident, $elt: ident, $rinfo: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlText>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $rinfo.state.$elt != $rinfo.initial_state.$elt;
            $elt.value = $rinfo.state.$elt.clone().to_string();
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
    };
}

#[macro_export]
macro_rules! rsp10_option_text {
    ($gd: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: Option<std::rc::Rc<std::cell::RefCell<HtmlText>>> = if $state.$elt.is_some() {
            let mut $elt: HtmlText = Default::default();
            $elt.highlight = $state.$elt != $default_state.$elt;
            $elt.value = $state.$elt.clone().unwrap().to_string();
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
            let rc = std::rc::Rc::new(std::cell::RefCell::new($elt));
            $gd.push(rc.clone());
            Some(rc)
        } else {
            None
        };
    };
}

#[macro_export]
macro_rules! rsp10_button {
    ($elt: ident, $label: expr => $gd: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlButton>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.id = format!("{}", stringify!($elt));
            $elt.value = $label.into();
        }
        rsp10_gd!($gd, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_button {
    ($parent: ident, $idx: ident,  $elt: ident, $label: expr) => {
        let mut $elt: HtmlButton = Default::default();
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $elt.value = $label.into();
    };
}

#[macro_export]
macro_rules! rsp10_text_escape_backtick {
    ( $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlText = Default::default();
        $elt.highlight = $state.$elt != $default_state.$elt;
        $elt.value = $state.$elt.clone().replace("`", r#"\`"#);
        $elt.id = format!("{}", stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

#[macro_export]
macro_rules! rsp10_nested_text {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $rinfo: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlText>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight =
                $rinfo.state.$parent[$idx].$elt != $rinfo.initial_state.$parent[$idx].$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $elt.value = $rinfo.state.$parent[$idx].$elt.clone().to_string();
        }
        rsp10_nested_gd!($gd, $parent, $idx, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_option_text {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: Option<std::rc::Rc<std::cell::RefCell<HtmlText>>> =
            if $state.$parent[$idx].$elt.is_some() {
                let mut $elt: HtmlText = Default::default();
                $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
                $elt.value = $state.$parent[$idx].$elt.clone().unwrap().to_string();
                $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
                $modified = $modified || $elt.highlight;
                let rc = std::rc::Rc::new(std::cell::RefCell::new($elt));
                $gd.push(rc.clone());
                Some(rc)
            } else {
                None
            };
    };
}

#[macro_export]
macro_rules! rsp10_check {
    ( $elt: ident, $rinfo: ident => $gd: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlCheck>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $rinfo.state.$elt != $rinfo.initial_state.$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}", stringify!($elt));
            $elt.checked = $rinfo.state.$elt;
        }
        rsp10_gd!($gd, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_state {
    ( $gd: ident, $parent: ident, $idx: expr, $rinfo: ident) => {
        $gd.vector(stringify!($parent), |x| {
            x.insert_data_at($idx, "state", &$rinfo.state.$parent[$idx]);
        })
    };
}

#[macro_export]
macro_rules! rsp10_nested_check {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $rinfo: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlCheck>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight =
                $rinfo.state.$parent[$idx].$elt != $rinfo.initial_state.$parent[$idx].$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $elt.checked = $rinfo.state.$parent[$idx].$elt;
        }
        rsp10_nested_gd!($gd, $parent, $idx, $elt);
    };
}

#[macro_export]
macro_rules! rsp10_nested_check_nogd {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $rinfo: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlCheck>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight =
                $rinfo.state.$parent[$idx].$elt != $rinfo.initial_state.$parent[$idx].$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $elt.checked = $rinfo.state.$parent[$idx].$elt;
        }
        // rsp10_gd!($gd, $elt);
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RspEvent {
    pub event: String,
    pub target: String,
}

pub fn req_get_event(req: &mut Request) -> RspEvent {
    use urlencoded::UrlEncodedBody;
    let mut event: String = "unknown".into();
    let mut target: String = "".into();

    match req.get_ref::<UrlEncodedBody>() {
        Ok(ref hashmap) => {
            match hashmap.get("debug") {
                Some(a) => {
                    println!("IN DEBUG: {}", &a[0].clone().to_string());
                }
                _ => {}
            }
            match hashmap.get("event") {
                Some(a) => {
                    event = a[0].clone();
                }
                _ => {}
            }
            match hashmap.get("event_target") {
                Some(a) => {
                    target = a[0].clone();
                }
                _ => {}
            }
            if &event == "unknown" && &target == "" {
                debug!("post hashmap: {:?}", &hashmap);
                match hashmap.keys().find(|x| x.starts_with("submit")) {
                    Some(a) => {
                        event = "submit".into();
                        target = a["submit".len()..].into();
                    }
                    _ => match hashmap.keys().find(|x| x.starts_with("btn")) {
                        Some(a) => {
                            event = "submit".into();
                            target = a.clone();
                        }
                        _ => {}
                    },
                }
                if &event == "submit" {}
            }
        }
        Err(ref e) => {
            debug!("req_get_event err: {:?}", e);
        }
    };
    let retev = RspEvent { event, target };
    debug!("Event: {:?}", &retev);
    retev
}
/*

pub struct RspPage<T, S: RspState<T>> {
    key: T,
    frontend_state: Option<S>,
    initial_state: Option<S>,
    default_state: S,
    event: Option<RspEvent>,
    data: mustache::MapBuilder,
    redirect_to: Option<String>,
    reset_state: bool,
}

*/

fn req_get_initial_state_json_string(req: &mut Request) -> String {
    req_get_post_argument(req, "initial_state_json")
}

fn req_get_state_json_string(req: &mut Request) -> String {
    req_get_post_argument(req, "state_json")
}

fn req_get_post_argument(req: &mut Request, argname: &str) -> String {
    use urlencoded::UrlEncodedBody;
    let arg_state_string = match req.get_ref::<UrlEncodedBody>() {
        Ok(ref hashmap) => {
            // println!("Parsed POST request body:\n {:?}", &hashmap);
            match hashmap.get(argname) {
                Some(a) => a[0].to_string(),
                _ => String::new(),
            }
        }
        Err(ref e) => {
            debug!("req_get_post_argument('{}') err = {:?}", argname, e);
            String::new()
        }
    };
    arg_state_string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RspAction<T> {
    Render,
    SetKey(T),
    ReloadState,
    RedirectTo(String),
}

pub fn maybe_compile_template(name: &str) -> Result<Template, mustache::Error> {
    let fname = format!("./templates/{}.mustache", name);
    debug!("Compiling template: {}", &fname);
    mustache::compile_path(fname)
}

pub fn compile_template(name: &str) -> Template {
    maybe_compile_template(name).expect("Failed to compile")
}

#[macro_export]
macro_rules! get_page_template {
    ( $name: expr) => {
        match maybe_compile_template($name) {
            Ok(t) => t,
            Err(e) => {
                return Ok(Response::with((
                    status::Unauthorized,
                    format!("Error occured: {}", e),
                )));
            }
        }
    };
}

pub fn build_response(template: Template, data: MapBuilder) -> iron::Response {
    use iron::headers::{Connection, ContentType};
    let mut bytes = vec![];
    let data_built = data.build();
    template
        .render_data(&mut bytes, &data_built)
        .expect("Failed to render");
    let payload = std::str::from_utf8(&bytes).unwrap();

    let mut resp = Response::with((status::Ok, payload));
    resp.headers.set(ContentType::html());
    resp.headers.set(Connection::close());
    resp
}

pub fn http_redirect(redirect_to: &str) -> IronResult<Response> {
    use iron::headers::ContentType;
    use iron::headers::Location;
    // let mut resp = Response::with((status::TemporaryRedirect, $redirect_to.clone()));
    let mut resp = Response::with((status::Found, redirect_to));
    resp.headers.set(ContentType::html());
    resp.headers.set(Location(redirect_to.to_string()));
    Ok(resp)
}

fn amend_json_value_from_req(v: &mut serde_json::value::Value, req: &HashMap<String, Vec<String>>) {
    amend_value_x("", v, req);
}

fn amend_value_x(
    name_prefix: &str,
    orig_val: &mut serde_json::value::Value,
    req: &HashMap<String, Vec<String>>,
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
                amend_value_x(&new_prefix, value, req);
            }
        }
        Array(ref mut arr) => {
            for (i, elt) in arr.iter_mut().enumerate() {
                // IDs can't start with number, so always add underscores
                let new_prefix = format!("{}__{}", name_prefix, i);
                amend_value_x(&new_prefix, elt, req);
            }
        }
        ref x => {
            /* overwrite the JSON value from a request hash table */
            if req.contains_key(name_prefix) {
                let new_val_src = req[name_prefix].clone();
                let src = &new_val_src[0];
                match x {
                    Bool(ref val) => {
                        let new_val = match src.as_ref() {
                            "true" => true,
                            "on" => true,
                            "checked" => true,
                            _ => false,
                        };
                        *orig_val = Bool(new_val);
                    }
                    String(ref val) => {
                        *orig_val = String(src.to_string());
                    }
                    _ => {
                        let res: Result<serde_json::Value, _> = serde_json::from_str(src);
                        if res.is_ok() {
                            *orig_val = res.unwrap();
                        } else {
                            println!(
                                "Result not ok: {:#?} on '{:#?}' - val {:#?}",
                                &res, &src, &x
                            );
                        }
                    }
                }
            } else {
                // This is a workaround against checkboxes unchecked values not being passed
                let name_prefix = &format!("{}_sentinel", &name_prefix);
                if req.contains_key(name_prefix) {
                    let new_val_src = req[name_prefix].clone();
                    let src = &new_val_src[0];
                    match x {
                        Bool(ref val) => {
                            let new_val = match src.as_ref() {
                                "true" => true,
                                "on" => true,
                                "checked" => true,
                                _ => false,
                            };
                            *orig_val = Bool(new_val);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub trait RspUserAuth
where
    Self: std::marker::Sized,
{
    fn from_request(req: &mut Request) -> Result<Self, String>;
    // fn has_rights(auth: &Self, rights: &str) -> bool;
}

pub struct RspInfo<'a, 'b, 'c, R, T, TA> {
    pub req: &'a mut Request<'b, 'c>,
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
}

pub struct RspFillDataResult<R> {
    pub state: R,
    pub initial_state: R,
    pub data: MapBuilder,
}

pub trait RspState<T, TA>
where
    Self: std::marker::Sized + serde::Serialize + serde::de::DeserializeOwned + Clone + Debug,
    TA: RspUserAuth + serde::Serialize,
    T: serde::Serialize + Debug + Clone + Default + serde::de::DeserializeOwned,
{
    fn get_state(req: &mut Request, auth: &TA, key: T) -> Self;

    fn event_handler(ri: RspInfo<Self, T, TA>) -> RspEventHandlerResult<Self, T> {
        let mut action = RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;

        RspEventHandlerResult {
            initial_state,
            state,
            action,
        }
    }

    fn get_key(
        auth: &TA,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<Self>,
    ) -> Option<T> {
        None
    }

    fn default_get_key_from_req(auth: &TA, req: &mut Request) -> Option<T> {
        use urlencoded::UrlEncodedQuery;
        let req_res: Result<T, req2struct::Error> = match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => {
                let res: Result<T, _> = req2struct::from_map(&hashmap);
                res
            }
            _ => {
                let hm: HashMap<String, Vec<String>> = HashMap::new();
                req2struct::from_map(&hm)
            }
        };
        req_res.ok()
    }

    fn get_key_from_req(auth: &TA, req: &mut Request) -> Option<T> {
        Self::default_get_key_from_req(auth, req)
    }

    fn default_event_handler_result(ri: RspInfo<Self, T, TA>) -> RspEventHandlerResult<Self, T> {
        let mut action = RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;
        RspEventHandlerResult {
            initial_state,
            state,
            action,
        }
    }

    fn default_fill_data_result_with_data(
        ri: RspInfo<Self, T, TA>,
        data: MapBuilder,
    ) -> RspFillDataResult<Self> {
        let initial_state = ri.initial_state;
        let state = ri.state;
        RspFillDataResult {
            initial_state,
            state,
            data,
        }
    }
    fn default_fill_data_result(ri: RspInfo<Self, T, TA>) -> RspFillDataResult<Self> {
        let data = MapBuilder::new();
        Self::default_fill_data_result_with_data(ri, data)
    }

    fn fill_data_result(ri: RspInfo<Self, T, TA>, gd: RspDataBuilder) -> RspFillDataResult<Self> {
        let data = MapBuilder::new();
        Self::default_fill_data_result_with_data(ri, gd.build(data))
    }

    fn fill_data(ri: RspInfo<Self, T, TA>) -> RspFillDataResult<Self> {
        Self::default_fill_data_result(ri)
    }

    fn get_template_name() -> String {
        return "".into();
    }

    fn get_template_name_auto() -> String {
        use std::any::type_name;
        fn test_type<T: ?Sized>() -> String {
            let full_type_name = type_name::<T>();
            let components: Vec<String> =
                full_type_name.split("::").map(|x| x.to_string()).collect();
            let ret = components[components.len() - 2].to_string();
            ret
        }
        test_type::<Self>()
    }

    fn handler(req: &mut Request) -> IronResult<Response> {
        let auth_res = TA::from_request(req);
        match auth_res {
            Err(login_url) => http_redirect(&login_url),
            Ok(auth) => Self::auth_handler(req, auth),
        }
    }

    fn default_response_finalize(ri: &RspInfo<Self, T, TA>, resp: &mut iron::Response) {
        use iron::headers::{Connection, ContentType};
        resp.headers.set(ContentType::html());
        resp.headers.set(Connection::close());
    }

    fn response_finalize(ri: &RspInfo<Self, T, TA>, resp: &mut iron::Response) {
        Self::default_response_finalize(ri, resp)
    }

    fn build_response(template: Template, data: MapBuilder) -> iron::Response {
        let mut bytes = vec![];
        let data_built = data.build();
        template
            .render_data(&mut bytes, &data_built)
            .expect("Failed to render");
        let payload = std::str::from_utf8(&bytes).unwrap();

        let mut resp = Response::with((status::Ok, payload));
        resp
    }

    fn auth_handler(req: &mut Request, auth: TA) -> IronResult<Response> {
        // use iron::headers::ContentType;
        use urlencoded::{UrlEncodedBody, UrlEncodedQuery};

        let mut redirect_to = "".to_string();
        let mut reload_state = false;

        /* try to get the current state from json variable */
        let mut maybe_res: Result<serde_json::Value, _> =
            serde_json::from_str(&req_get_state_json_string(req));

        let mut maybe_state = if let Ok(mut state_val) = maybe_res {
            if let Ok(ref hashmap) = req.get_ref::<UrlEncodedBody>() {
                /* augment it from request form fields values */
                amend_json_value_from_req(&mut state_val, hashmap);
            }
            let form_state_res: Result<Self, _> = serde_json::from_value(state_val);
            form_state_res.ok()
        } else {
            None
        };

        let maybe_res: Result<Self, serde_json::Error> =
            serde_json::from_str(&req_get_initial_state_json_string(req));

        let mut maybe_initial_state = maybe_res.ok();

        let mut maybe_key = match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => Self::get_key(&auth, &hashmap, &maybe_state),
            Err(ref _e) => {
                let hm = HashMap::new();
                Self::get_key(&auth, &hm, &maybe_state)
            }
        };
        if maybe_key.is_none() {
            maybe_key = Self::get_key_from_req(&auth, req);
        }

        debug!("DESERIALIZED STATE: {:#?}", &maybe_state);
        debug!("DESERIALIZED INITIAL STATE: {:#?}", &maybe_initial_state);
        debug!("DESERIALIZED KEY: {:#?}", &maybe_key);

        let mut key = maybe_key.unwrap_or(Default::default());

        let event = req_get_event(req);

        let mut curr_initial_state = Self::get_state(req, &auth, key.clone());
        let state_none = maybe_state.is_none();
        let initial_state_none = maybe_initial_state.is_none();
        let initial_state = maybe_initial_state.unwrap_or(curr_initial_state.clone());
        let state = maybe_state.unwrap_or(initial_state.clone());
        let ri = RspInfo {
            req: req,
            auth: &auth,
            event: &event,
            key: &key,
            state,
            state_none,
            initial_state,
            initial_state_none,
            curr_initial_state: &curr_initial_state,
        };
        let r = Self::event_handler(ri);
        let mut initial_state = r.initial_state;
        let mut state = r.state;
        let action = r.action;

        match action {
            RspAction::Render => {}
            RspAction::ReloadState => {
                reload_state = true;
            }
            RspAction::RedirectTo(target) => {
                redirect_to = target;
            }
            RspAction::SetKey(k) => {
                key = k;
                reload_state = true;
            }
        };
        if redirect_to.is_empty() {
            if reload_state {
                curr_initial_state = Self::get_state(req, &auth, key.clone());
                let st = curr_initial_state.clone();
                println!("Reload state");
                initial_state = st.clone();
                state = st;
            }
            let template_name = if &Self::get_template_name() != "" {
                Self::get_template_name()
            } else {
                Self::get_template_name_auto()
            };
            let template = get_page_template!(&template_name);
            let ri = RspInfo {
                req: req,
                auth: &auth,
                event: &event,
                key: &key,
                state,
                state_none: false,
                initial_state,
                initial_state_none: false,
                curr_initial_state: &curr_initial_state,
            };
            let r = Self::fill_data(ri);
            let initial_state = r.initial_state;
            let state = r.state;
            let data = r.data;
            let data = data.insert("auth", &auth).unwrap();
            let data = data.insert("state", &state).unwrap();
            let data = data.insert("state_key", &key).unwrap();
            let data = data.insert("initial_state", &initial_state).unwrap();
            let data = data
                .insert("curr_initial_state", &curr_initial_state)
                .unwrap();
            let data = data
                .insert("state_json", &serde_json::to_string(&state).unwrap())
                .unwrap();
            let data = data
                .insert("state_key_json", &serde_json::to_string(&key).unwrap())
                .unwrap();

            let data = data
                .insert(
                    "initial_state_json",
                    &serde_json::to_string(&initial_state).unwrap(),
                )
                .unwrap();
            let data = data
                .insert(
                    "curr_initial_state_json",
                    &serde_json::to_string(&curr_initial_state).unwrap(),
                )
                .unwrap();

            let ri = RspInfo {
                req: req,
                auth: &auth,
                event: &event,
                key: &key,
                state,
                state_none,
                initial_state,
                initial_state_none,
                curr_initial_state: &curr_initial_state,
            };
            let mut resp = Self::build_response(template, data);

            Self::response_finalize(&ri, &mut resp);
            Ok(resp)
        } else {
            http_redirect(&redirect_to)
        }
    }
}

#[derive(Debug)]
pub struct RspServer {
    default_secret: Option<Vec<u8>>,
}

impl RspServer {
    pub fn read_default_secret() -> Result<Vec<u8>, std::io::Error> {
        use std::fs::File;
        use std::io;
        use std::io::prelude::*;

        let mut f = File::open(".secret")?;
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    pub fn new() -> RspServer {
        let secret = Self::read_default_secret().ok();
        RspServer {
            default_secret: secret,
        }
    }

    pub fn set_secret(&mut self, new_secret: Vec<u8>) {
        self.default_secret = Some(new_secret);
    }

    pub fn run<H: Handler>(
        &mut self,
        main_handler: H,
        service_name: &str,
        port: u16,
    ) -> (Rsp10GlobalData, iron::Listening) {
        use mount::Mount;
        use rand::random;
        use staticfile::Static;
        use std::path::Path;

        fn rand_bytes() -> Vec<u8> {
            (0..64).map(|_| random::<u8>()).collect()
        }

        let mut mount = Mount::new();
        mount.mount("/", main_handler);
        mount.mount("/static/", Static::new(Path::new("staticfiles/")));

        let globals = Rsp10GlobalData::new();

        let my_secret = self.default_secret.clone().unwrap_or(rand_bytes());
        let mut ch = Chain::new(mount);

        ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));
        let reuse_s = env::var("IRON_PORT_REUSE").unwrap_or(format!("false"));
        let reuse = reuse_s.parse::<bool>().unwrap_or(false);

        ch.link(State::<Rsp10GlobalData>::both(globals.clone()));

        let listening = if reuse {
            run_http_server_with_reuse(service_name, port, ch)
        } else {
            run_http_server(service_name, port, ch)
        };
        globals.set_test(format!("testing"));
        (globals, listening)
    }
}

fn get_tcp_listener(port: u16, reuse: bool, backlog: i32) -> std::net::TcpListener {
    use socket2::{Domain, Protocol, Socket, Type};
    use std::net::SocketAddr;
    use std::net::TcpListener;

    let bind_ip = env::var("BIND_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port_s = env::var("BIND_PORT").unwrap_or(port.to_string());
    let bind_port = bind_port_s.parse::<u16>().unwrap_or(port);
    let addr: SocketAddr = format!("{}:{}", &bind_ip, &bind_port).parse().unwrap();
    println!(
        "getting TCP listener on {}:{} with{} address/port reuse, backlog {}",
        &bind_ip,
        bind_port,
        if reuse { "" } else { "out" },
        backlog
    );
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .expect("could not make a TCP socket");

    // Set reuse flags
    socket
        .set_reuse_address(reuse)
        .expect("Could not set reuse_address");
    // Note: set_reuse_port is not available on all platforms
    #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
    socket
        .set_reuse_port(reuse)
        .expect("Could not set reuse_port");

    // Bind and convert to std::net::TcpListener
    socket.bind(&addr.into()).expect("Could not bind");
    socket.listen(backlog).expect("Could not set the backlog");
    let listener = std::net::TcpListener::from(socket);
    listener
}

fn run_http_server_with_reuse<H: Handler>(
    service_name: &str,
    port: u16,
    handler: H,
) -> iron::Listening {
    // TODO: Fix hyper compatibility issue
    // For now, fall back to non-reuse version
    run_http_server(service_name, port, handler)
}

fn run_http_server<H: Handler>(service_name: &str, port: u16, handler: H) -> iron::Listening {
    use iron::Timeouts;
    use std::env;
    use std::time::Duration;
    env_logger::init();

    let bind_ip = env::var("BIND_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port_s = env::var("BIND_PORT").unwrap_or(port.to_string());
    let bind_port = bind_port_s.parse::<u16>().unwrap_or(port);
    let mut iron = Iron::new(handler);
    let threads_s = env::var("IRON_HTTP_THREADS").unwrap_or("1".to_string());
    let threads = threads_s.parse::<usize>().unwrap_or(1);
    iron.threads = threads;
    iron.timeouts = Timeouts {
        keep_alive: Some(Duration::from_millis(10)),
        read: Some(Duration::from_secs(10)),
        write: Some(Duration::from_secs(10)),
    };

    println!(
        "HTTP server for {} starting on {}:{} without address/port reuse",
        service_name, &bind_ip, bind_port
    );
    iron.http(&format!("{}:{}", &bind_ip, bind_port)).unwrap()
}
