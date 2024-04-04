extern crate chrono;
// extern crate mount;
// extern crate procinfo;
// extern crate quickersort;
extern crate regex;
// extern crate staticfile;

#[macro_use]
extern crate lazy_static;
extern crate mustache;

#[macro_use]
extern crate iron;
extern crate iron_sessionstorage;

// #[macro_use]
// extern crate router;
extern crate urlencoded;

extern crate diesel;
// extern crate time;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// #[macro_use]
// extern crate hyper;
extern crate params;

// use router::Router;

use iron::modifiers::Redirect;

use iron_sessionstorage::backends::SignedCookieBackend;
use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;

use iron::Handler;

use iron::prelude::*;
use iron::status;
use mustache::MapBuilder;
use mustache::Template;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RspPage<S> {
    frontend_state: Option<S>,
    initial_state: Option<S>,
    default_state: S,
    event: Option<RspEvent>,
    data: mustache::MapBuilder,
    redirect_to: Option<String>,
    reset_state: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlText {
    pub id: String,
    pub value: String,
    pub labeltext: String,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlButton {
    pub id: String,
    pub value: String,
    pub labeltext: String,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlCheck {
    pub id: String,
    pub labeltext: String,
    pub checked: bool,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelectItem<T> {
    pub i: usize,
    pub user_label: String,
    pub value: T,
    pub selected: bool,
}

use std::fmt::Debug;
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HtmlSelect<T: PartialEq + Clone + Debug> {
    pub id: String,
    pub items: Vec<HtmlSelectItem<T>>,
    pub selected_value: T,
    pub highlight: bool,
    pub hidden: bool,
    pub disabled: bool,
}

impl<T> HtmlSelect<T>
where
    T: std::cmp::PartialEq + std::clone::Clone + Debug,
{
    pub fn item(self: &mut HtmlSelect<T>, user_label: &str, value: T) {
        let i = HtmlSelectItem::<T> {
            user_label: user_label.into(),
            value: value.into(),
            selected: false,
            i: self.items.len(),
        };
        self.items.push(i);
    }

    pub fn set_selected_value(self: &mut HtmlSelect<T>, selected_value: &mut T) {
        let mut found = false;
        // println!("Setting selected value: {:?}", &selected_value);
        for mut item in &mut self.items {
            if item.value == *selected_value {
                item.selected = true;
                found = true;
                self.selected_value = selected_value.clone();
            } else {
                item.selected = false;
            }
            // println!("Item: {:?}", &item);
        }
        if !found {
            if self.items.len() > 0 {
                self.items[0].selected = true;
                *selected_value = self.items[0].value.clone();
                self.selected_value = selected_value.clone();

            // println!("Setting selected value to {:?}", &selected_value);
            } else {
                // println!("Can not set default selected value");
            }
        } else {
            // println!("Found in items, selected value not reset");
        }
    }
}
impl HtmlSelect<String> {
    pub fn item1(self: &mut HtmlSelect<String>, user_label: &str) {
        let i = HtmlSelectItem::<String> {
            i: self.items.len(),
            user_label: user_label.into(),
            value: user_label.into(),
            selected: false,
        };
        self.items.push(i);
    }
}

macro_rules! rspten_page {
    ( $router: ident, $path: expr, $handler: path) => {
        $router.get($path, $handler, $path);
        $router.post($path, $handler, $path);
    };
}

macro_rules! html_select {
    ( $elt: ident, $from: expr , $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = $from;
        $elt.set_selected_value(&mut $state.$elt);
        $elt.highlight = $state.$elt != $default_state.$elt;
        $elt.id = format!("{}", stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

macro_rules! html_nested_select {
    ( $parent: ident, $idx: ident, $elt: ident, $from: expr , $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt = $from;
        $elt.set_selected_value(&mut $state.$parent[$idx].$elt);
        $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

macro_rules! html_text {
    ( $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlText = Default::default();
        $elt.highlight = $state.$elt != $default_state.$elt;
        $elt.value = $state.$elt.clone();
        $elt.id = format!("{}", stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

macro_rules! html_text_escape_backtick {
    ( $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlText = Default::default();
        $elt.highlight = $state.$elt != $default_state.$elt;
        $elt.value = $state.$elt.clone().replace("`", r#"\`"#);
        $elt.id = format!("{}", stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

macro_rules! html_nested_text {
    ( $parent: ident, $idx: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlText = Default::default();
        $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
        $elt.value = $state.$parent[$idx].$elt.clone();
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $modified = $modified || $elt.highlight;
    };
}

macro_rules! html_check {
    ( $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlCheck = Default::default();
        $elt.highlight = $state.$elt != $default_state.$elt;
        $modified = $modified || $elt.highlight;
        $elt.id = format!("{}", stringify!($elt));
        $elt.checked = $state.$elt;
    };
}

macro_rules! html_nested_check {
    ( $parent: ident, $idx: ident, $elt: ident, $state: ident, $default_state: ident, $modified: ident) => {
        let mut $elt: HtmlCheck = Default::default();
        $elt.highlight = $state.$parent[$idx].$elt != $default_state.$parent[$idx].$elt;
        $modified = $modified || $elt.highlight;
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $elt.checked = $state.$parent[$idx].$elt;
    };
}

macro_rules! html_button {
    ( $elt: ident, $label: expr) => {
        let mut $elt: HtmlButton = Default::default();
        $elt.id = format!("{}", stringify!($elt));
        $elt.value = $label.into();
    };
}

macro_rules! html_nested_button {
    ($parent: ident, $idx: ident,  $elt: ident, $label: expr) => {
        let mut $elt: HtmlButton = Default::default();
        $elt.id = format!("{}__{}__{}", stringify!($parent), $idx, stringify!($elt));
        $elt.value = $label.into();
    };
}

macro_rules! data_insert {
    ($data: ident, $elt: ident) => {
        $data = $data.insert(stringify!($elt), &$elt).unwrap();
    };
}

macro_rules! data_insert_state {
    ($data: ident, $state: ident, $initial_state: ident) => {
        $data = $data.insert("state", &$state).unwrap();
        $data = $data.insert("initial_state", &$initial_state).unwrap();
        $data = $data
            .insert("state_json", &serde_json::to_string(&$state).unwrap())
            .unwrap();

        $data = $data
            .insert(
                "initial_state_json",
                &serde_json::to_string(&$initial_state).unwrap(),
            )
            .unwrap();
    };
}

macro_rules! render_response {
    ($template: ident, $data: ident, $redirect_to: ident) => {
        if $redirect_to.is_empty() {
            let mut resp = build_response($template, $data);
            Ok(resp)
        } else {
            use iron::headers::Location;
            // let mut resp = Response::with((status::TemporaryRedirect, $redirect_to.clone()));
            let mut resp = Response::with((status::Found, $redirect_to.clone()));
            resp.headers.set(ContentType::html());
            resp.headers.set(Location($redirect_to));
            Ok(resp)
        }
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
            match (hashmap.get("debug")) {
                Some(a) => {
                    println!("IN DEBUG: {}", &a[0].clone().to_string());
                }
                _ => {}
            }
            match (hashmap.get("event")) {
                Some(a) => {
                    event = a[0].clone().into();
                }
                _ => {}
            }
            match (hashmap.get("event_target")) {
                Some(a) => {
                    target = a[0].clone().into();
                }
                _ => {}
            }
        }
        Err(ref e) => {
            println!("{:?}", e);
        }
    };
    return RspEvent {
        event: event,
        target: target,
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
pub fn match_nested_target<'a>(root: &str, unparsed_target: &'a str) -> Option<(&'a str, usize)> {
    use regex::Regex;
    lazy_static! {
        static ref RE_nested_target: Regex =
            Regex::new(r"^(?P<root>[^_]+)__(?P<index>[0-9]+)__(?P<target>[^_]+)").unwrap();
    }

    if let Some(cap) = RE_nested_target.captures(unparsed_target) {
        if &cap["root"] == root {
            let ct = cap.name("target").unwrap();
            let out_target = &unparsed_target[ct.start()..ct.end()];
            let out_index = cap["index"].parse::<usize>().unwrap();
            Some((out_target, out_index))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn req_get_state_string(req: &mut Request) -> String {
    req_get_post_argument(req, "state")
}

pub fn req_get_initial_state_string(req: &mut Request) -> String {
    req_get_post_argument(req, "initial_state")
}

pub fn req_get_post_argument(req: &mut Request, argname: &str) -> String {
    use urlencoded::UrlEncodedBody;
    let arg_state_string = match req.get_ref::<UrlEncodedBody>() {
        Ok(ref hashmap) => {
            // println!("Parsed POST request body:\n {:?}", &hashmap);
            match (hashmap.get(argname)) {
                Some(a) => format!("{}", a[0]),
                _ => String::new(),
            }
        }
        Err(ref e) => {
            println!("{:?}", e);
            String::new()
        }
    };
    return arg_state_string;
}

macro_rules! get_arg_i32 {
    ($src: ident, $field: expr, $target: ident) => {
        if let Some(arg_val) = $src.get($field) {
            if arg_val.len() >= 1 {
                if let Ok(val) = arg_val[0].parse::<i32>() {
                    $target = val;
                }
            }
        }
    };
}
