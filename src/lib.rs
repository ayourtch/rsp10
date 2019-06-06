extern crate chrono;
extern crate mount;
extern crate regex;
extern crate staticfile;

// #[macro_use]
extern crate lazy_static;
extern crate mustache;

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
extern crate params;
extern crate rand;

use router::Router;

use iron::prelude::*;
use iron::status;

use iron_sessionstorage::backends::SignedCookieBackend;
use iron_sessionstorage::SessionStorage;

use iron::Handler;
// use std::cell::RefCell;
use std::collections::HashMap;

use mustache::MapBuilder;
use mustache::Template;

use std::fmt::Debug;
mod html_types;
pub use html_types::*;

#[macro_export]
macro_rules! html_select {
    ( $gd: ident, $elt: ident, $from: expr , $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = std::rc::Rc::new(std::cell::RefCell::new($from.clone()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.set_selected_value(&mut $state.$elt);
            $elt.highlight = $state.$elt != $default_state.$elt;
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        // let $gd = || $gd().insert(stringify!($elt), &$elt).unwrap();
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_nested_select {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $from: expr , $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = std::rc::Rc::new(std::cell::RefCell::new($from.clone()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.set_selected_value(&mut $state.$parent[$idx].$elt);
            $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_text {
    ($gd: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlText>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $state.$elt != $default_state.$elt;
            $elt.value = $state.$elt.clone();
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
        }

        // let $gd = || $gd().insert(stringify!($elt), &$elt).unwrap();
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_option_text {
    ($gd: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: Option<std::rc::Rc<std::cell::RefCell<HtmlText>>> = 
        if $state.$elt.is_some() {
            let mut $elt: HtmlText = Default::default();
            $elt.highlight = $state.$elt != $default_state.$elt;
            $elt.value = $state.$elt.clone().unwrap().to_string();
            $elt.id = format!("{}", stringify!($elt));
            $modified = $modified || $elt.highlight;
            let rc =  std::rc::Rc::new(std::cell::RefCell::new($elt));
            $gd.push(rc.clone());
            Some(rc)
        } else {
            None
        };

    };
}

#[macro_export]
macro_rules! html_button {
    ($gd: ident, $elt: ident, $label: expr) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlButton>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.id = format!("{}", stringify!($elt));
            $elt.value = $label.into();
        }
        // let $gd = || $gd().insert(stringify!($elt), &$elt).unwrap();
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_nested_button {
    ($parent: ident, $idx: ident,  $elt: ident, $label: expr) => {
        let mut $elt: HtmlButton = Default::default();
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $elt.value = $label.into();
    };
}

#[macro_export]
macro_rules! html_text_escape_backtick {
    ( $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlText = Default::default();
        $elt.highlight = $state.$elt != $default_state.$elt;
        $elt.value = $state.$elt.clone().replace("`", r#"\`"#);
        $elt.id = format!("{}", stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

#[macro_export]
macro_rules! html_nested_text {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlText>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
            $elt.value = $state.$parent[$idx].$elt.clone();
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $modified = $modified || $elt.highlight;
        }
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_nested_option_text {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: Option<std::rc::Rc<std::cell::RefCell<HtmlText>>> = 
        if $state.$elt.is_some() {
            let mut $elt: HtmlText = Default::default();
            $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
            $elt.value = $state.$parent[$idx].$elt.clone();
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $modified = $modified || $elt.highlight;
            let rc =  std::rc::Rc::new(std::cell::RefCell::new($elt));
            $gd.push(rc.clone());
            Some(rc)
        } else {
            None
        };

    };
}

#[macro_export]
macro_rules! html_check {
    ( $gd: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlCheck>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $state.$elt != $default_state.$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}", stringify!($elt));
            $elt.checked = $state.$elt;
        }
        // let $gd = || $gd().insert(stringify!($elt), &$elt).unwrap();
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! html_nested_check {
    ( $gd: ident, $parent: ident, $idx: expr, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<HtmlCheck>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
            $modified = $modified || $elt.highlight;
            $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
            $elt.checked = $state.$parent[$idx].$elt;
        }
        $gd.push($elt.clone());
    };
}

#[macro_export]
macro_rules! data_insert {
    ($data: ident, $elt: ident) => {
        $data = $data.insert(stringify!($elt), &$elt).unwrap();
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
                println!("post hashmap: {:?}", &hashmap);
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
            println!("req_get_event err: {:?}", e);
        }
    };
    let retev = RspEvent { event, target };
    println!("Event: {:?}", &retev);
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

fn req_get_initial_state_string(req: &mut Request) -> String {
    req_get_post_argument(req, "initial_state")
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
            println!("req_get_post_argument('{}') err = {:?}", argname, e);
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
    println!("Compiling: {}", &fname);
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
    use iron::headers::ContentType;
    let mut bytes = vec![];
    let data_built = data.build();
    template
        .render_data(&mut bytes, &data_built)
        .expect("Failed to render");
    let payload = std::str::from_utf8(&bytes).unwrap();

    let mut resp = Response::with((status::Ok, payload));
    resp.headers.set(ContentType::html());
    resp
}

fn http_redirect(redirect_to: &str) -> IronResult<Response> {
    use iron::headers::ContentType;
    use iron::headers::Location;
    // let mut resp = Response::with((status::TemporaryRedirect, $redirect_to.clone()));
    let mut resp = Response::with((status::Found, redirect_to));
    resp.headers.set(ContentType::html());
    resp.headers.set(Location(redirect_to.to_string()));
    Ok(resp)
}

pub trait RspStateName {
    fn get_template_name() -> String;
}

pub trait RspUserAuth
where
    Self: std::marker::Sized,
{
    fn from_request(req: &mut Request) -> Result<Self, String>;
    // fn has_rights(auth: &Self, rights: &str) -> bool;
}

pub trait RspState<T, TA>
where
    Self: std::marker::Sized
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + Debug
        + RspStateName,
    TA: RspUserAuth,
    T: serde::Serialize + Clone,
{
    fn get_key(auth: &TA, args: &HashMap<String, Vec<String>>, maybe_state: &Option<Self>) -> T;
    fn get_state(auth: &TA, key: T) -> Self;

    fn event_handler(
        req: &mut Request,
        auth: &TA,
        ev: &RspEvent,
        curr_key: &T,
        maybe_state: &mut Option<Self>,
        maybe_initial_state: &Option<Self>,
        curr_initial_state: &Self,
    ) -> RspAction<T>;

    fn fill_data(
        _auth: &TA,
        _data: MapBuilder,
        _ev: &RspEvent,
        _curr_key: &T,
        _state: &mut Self,
        _initial_state: &Self,
        _curr_initial_state: &Self,
    ) -> MapBuilder {
        _data
    }

    fn handler(req: &mut Request) -> IronResult<Response> {
        let auth_res = TA::from_request(req);
        match auth_res {
            Err(login_url) => http_redirect(&login_url),
            Ok(auth) => Self::auth_handler(req, auth),
        }
    }

    fn auth_handler(req: &mut Request, auth: TA) -> IronResult<Response> {
        // use iron::headers::ContentType;
        use urlencoded::{UrlEncodedBody, UrlEncodedQuery};

        let mut redirect_to = "".to_string();
        let mut reload_state = false;
        let form_state_res: Result<Self, req2struct::Error> = match req.get_ref::<UrlEncodedBody>()
        {
            Ok(ref hashmap) => {
                let res: Result<Self, _> = req2struct::from_map(&hashmap);
                res
            }
            _ => {
                let hm: HashMap<String, Vec<String>> = HashMap::new();
                req2struct::from_map(&hm)
            }
        };
        println!("form_state_res: {:#?}", &form_state_res);

        let mut maybe_state = match form_state_res {
            Ok(s) => Some(s),
            Err(e) => {
                println!("Error deserializing state: {:?}", e);
                None
            }
        };

        let maybe_res: Result<Self, serde_json::Error> =
            serde_json::from_str(&req_get_initial_state_string(req));

        let mut maybe_initial_state = maybe_res.ok();

        let mut key = match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => Self::get_key(&auth, &hashmap, &maybe_state),
            Err(ref _e) => {
                let hm = HashMap::new();
                Self::get_key(&auth, &hm, &maybe_state)
            }
        };

        let event = req_get_event(req);

        let curr_initial_state = Self::get_state(&auth, key.clone());
        let action = Self::event_handler(
            req,
            &auth,
            &event,
            &key,
            &mut maybe_state,
            &maybe_initial_state,
            &curr_initial_state,
        );

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
            if maybe_state.is_none() || maybe_initial_state.is_none() || reload_state {
                let st = Self::get_state(&auth, key.clone());
                println!("Reload state");
                maybe_initial_state = Some(st.clone());
                maybe_state = Some(st);
            }
            let mut data = MapBuilder::new();
            let template = get_page_template!(&Self::get_template_name());
            let mut state = maybe_state.unwrap();
            let initial_state = maybe_initial_state.unwrap();
            data = Self::fill_data(
                &auth,
                data,
                &event,
                &key,
                &mut state,
                &initial_state,
                &curr_initial_state,
            );
            data = data.insert("state", &state).unwrap();
            data = data.insert("state_key", &key).unwrap();
            data = data.insert("initial_state", &initial_state).unwrap();
            data = data
                .insert("state_json", &serde_json::to_string(&state).unwrap())
                .unwrap();
            data = data
                .insert("state_key_json", &serde_json::to_string(&key).unwrap())
                .unwrap();

            data = data
                .insert(
                    "initial_state_json",
                    &serde_json::to_string(&initial_state).unwrap(),
                )
                .unwrap();

            let resp = build_response(template, data);
            Ok(resp)
        } else {
            http_redirect(&redirect_to)
        }
    }
}

pub struct RspServer {
    default_secret: Option<Vec<u8>>,
}

impl RspServer {
    pub fn new() -> RspServer {
        RspServer {
            default_secret: None,
        }
    }

    pub fn set_secret(&mut self, new_secret: Vec<u8>) {
        self.default_secret = Some(new_secret);
    }

    pub fn run(&mut self, router: Router, service_name: &str, port: u16) {
        use mount::Mount;
        use rand::random;
        use staticfile::Static;
        use std::path::Path;

        fn rand_bytes() -> Vec<u8> {
            (0..64).map(|_| random::<u8>()).collect()
        }

        let mut mount = Mount::new();
        mount.mount("/", router);
        mount.mount("/static/", Static::new(Path::new("staticfiles/")));

        let my_secret = self.default_secret.clone().unwrap_or(rand_bytes());
        let mut ch = Chain::new(mount);
        ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));

        run_http_server(service_name, port, ch);
    }
}

fn run_http_server<H: Handler>(service_name: &str, port: u16, handler: H) {
    use iron::Timeouts;
    use std::env;
    use std::time::Duration;
    let mut iron = Iron::new(handler);
    iron.threads = 1;
    iron.timeouts = Timeouts {
        keep_alive: Some(Duration::from_millis(10)),
        read: Some(Duration::from_secs(10)),
        write: Some(Duration::from_secs(10)),
    };

    let bind_ip = env::var("BIND_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    println!(
        "HTTP server for {} starting on {}:{}",
        service_name, &bind_ip, port
    );
    iron.http(&format!("{}:{}", &bind_ip, port)).unwrap();
}
