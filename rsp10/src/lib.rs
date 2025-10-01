// RSP10 - Rust Server Pages
// Refactored to be framework-agnostic with pluggable HTTP adapters

extern crate chrono;
extern crate mustache;
extern crate rsp10_derive;

#[macro_use]
extern crate serde_derive;
extern crate req2struct;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::collections::HashMap;
use mustache::{MapBuilder, Template};
use std::fmt::Debug;
use crate::core::RspKey;

// New modular architecture - framework agnostic core
pub mod http_adapter;
pub mod core;

// Framework-specific adapters
#[cfg(feature = "iron")]
pub mod iron_adapter;

#[cfg(feature = "axum")]
pub mod axum_adapter;

// HTML types for form elements
mod html_types;
pub use html_types::*;

// Template data builder
pub mod foobuilder;
pub type RspDataBuilder = foobuilder::FooMapBuilder;

// Re-export derive macro
pub use rsp10_derive::RspState as DeriveRspState;

// Re-export core types for public API
pub use core::{
    RspEvent, RspAction, RspInfo, RspEventHandlerResult, RspFillDataResult,
    RspUserAuth, RspState, extract_event, extract_json_state, amend_json_value,
};

// Common auth types
pub mod common_auth;
pub use common_auth::{NoPageAuth, CookiePageAuth};

// Re-export HTTP abstraction
pub use http_adapter::{HttpRequest, HttpResponse, HttpResult, HttpError};

// Re-export Iron adapter if feature is enabled
#[cfg(feature = "iron")]
pub use iron_adapter::{make_iron_handler, IronRequestAdapter, IronResponseBuilder};

// Unified web handler that works with both Iron and Axum
pub struct WebHandler<S, T, TA> {
    _phantom: std::marker::PhantomData<(S, T, TA)>,
}

impl<S, T, TA> WebHandler<S, T, TA> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "iron")]
    pub fn to_iron(self) -> iron_adapter::RspIronHandler<S, T, TA>
    where
        S: RspState<T, TA> + Send + Sync + 'static,
        T: RspKey + serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned + Send + Sync + 'static,
        TA: RspUserAuth + serde::Serialize + Send + Sync + iron_sessionstorage::Value + Clone + iron::typemap::Key<Value = TA> + 'static,
    {
        make_iron_handler::<S, T, TA>()
    }

    #[cfg(feature = "axum")]
    pub fn to_axum(
        self
    ) -> impl Fn(
        axum::extract::State<std::sync::Arc<tokio::sync::Mutex<axum_adapter::SessionData>>>,
        axum::extract::Query<std::collections::HashMap<String, String>>,
        Option<axum::extract::Form<std::collections::HashMap<String, String>>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::response::Response> + Send>> + Clone
    where
        S: RspState<T, TA> + 'static,
        T: RspKey + 'static,
        TA: RspUserAuth + serde::Serialize + serde::de::DeserializeOwned + Default + 'static,
    {
        move |state, query, form| {
            Box::pin(axum_adapter::axum_handler_fn::<S, T, TA>((query, form, state)))
        }
    }
}


// Template utilities
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
        match $crate::maybe_compile_template($name) {
            Ok(t) => t,
            Err(e) => {
                return Err($crate::HttpError::InternalError(format!("Template error: {}", e)));
            }
        }
    };
}

// Macros for form elements - these work with the derive macro
#[cfg(feature = "iron")]
#[macro_export]
macro_rules! rsp10_page {
    ($router: ident, $url: expr, $name: ident, $file: expr) => {
        #[path=$file]
        mod $name;
        let handler = $crate::make_iron_handler::<$name::PageState, _, _>();
        $router.get($url, handler.clone(), format!("GET/{}", $url));
        $router.post($url, handler.clone(), format!("POST/{}", $url));
        $router.put($url, handler, format!("PUT/{}", $url));
    };
}

#[cfg(not(feature = "iron"))]
#[macro_export]
macro_rules! rsp10_page {
    ($router: ident, $url: expr, $name: ident, $file: expr) => {
        compile_error!("rsp10_page! macro requires iron feature to be enabled");
    };
}

#[macro_export]
macro_rules! rsp10_data {
    ( $elt: ident => $gd: ident) => {
        $gd.insert(stringify!($elt), &$elt);
    };
}

#[macro_export]
macro_rules! rsp10_gd {
    ( $gd: ident, $elt: ident) => {
        $gd.item(stringify!($elt), &$elt);
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
macro_rules! rsp10_text {
    ($elt: ident, $rinfo: ident => $gd: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<$crate::HtmlText>> =
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
macro_rules! rsp10_check {
    ( $elt: ident, $rinfo: ident => $gd: ident, $modified: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<$crate::HtmlCheck>> =
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
macro_rules! rsp10_button {
    ($elt: ident, $label: expr => $gd: ident) => {
        let mut $elt: std::rc::Rc<std::cell::RefCell<$crate::HtmlButton>> =
            std::rc::Rc::new(std::cell::RefCell::new(Default::default()));
        {
            let mut $elt = $elt.borrow_mut();
            $elt.id = format!("{}", stringify!($elt));
            $elt.value = $label.into();
        }
        rsp10_gd!($gd, $elt);
    };
}

// ========================================
// Iron-specific code (feature-gated)
// ========================================

#[cfg(feature = "iron")]
mod iron_support {
    use super::*;

    use iron::prelude::*;
    use iron::{Handler, status};
    use iron_sessionstorage::backends::SignedCookieBackend;
    use iron_sessionstorage::SessionStorage;
    use persistent::State;
    use iron::typemap::Key;
    use std::sync::{Arc, RwLock};
    use std::env;

    #[derive(Clone, Debug)]
    pub struct Rsp10GlobalData {
        stop_requested: Arc<RwLock<bool>>,
        test: Arc<RwLock<Option<String>>>,
    }

    impl Rsp10GlobalData {
        fn new() -> Self {
            Rsp10GlobalData {
                stop_requested: Arc::new(RwLock::new(false)),
                test: Arc::new(RwLock::new(None)),
            }
        }

        pub fn stop_requested(&self) -> bool {
            if let Ok(lock) = self.stop_requested.read() {
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
            if let Ok(lock) = self.test.read() {
                lock.clone()
            } else {
                None
            }
        }
    }

    impl Key for Rsp10GlobalData {
        type Value = Rsp10GlobalData;
    }

    pub fn request_stop(req: &mut Request) {
        let glob = req.get::<State<Rsp10GlobalData>>().unwrap();
        if let Ok(globals) = (*glob).write() {
            globals.request_stop();
        };
    }

    #[derive(Debug)]
    pub struct RspServer {
        default_secret: Option<Vec<u8>>,
    }

    impl RspServer {
        pub fn read_default_secret() -> Result<Vec<u8>, std::io::Error> {
            use std::fs::File;
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

            // Enable session storage with signed cookies
            ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));
            ch.link(State::<Rsp10GlobalData>::both(globals.clone()));

            let reuse_s = env::var("IRON_PORT_REUSE").unwrap_or_else(|_| "false".to_string());
            let reuse = reuse_s.parse::<bool>().unwrap_or(false);

            let listening = if reuse {
                run_http_server_with_reuse(service_name, port, ch)
            } else {
                run_http_server(service_name, port, ch)
            };

            globals.set_test("testing".to_string());
            (globals, listening)
        }
    }

    fn run_http_server_with_reuse<H: Handler>(
        service_name: &str,
        port: u16,
        handler: H,
    ) -> iron::Listening {
        // For now, fall back to non-reuse version due to hyper compatibility issues
        run_http_server(service_name, port, handler)
    }

    fn run_http_server<H: Handler>(service_name: &str, port: u16, handler: H) -> iron::Listening {
        use iron::Timeouts;
        use std::time::Duration;

        env_logger::init();

        let bind_ip = env::var("BIND_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
        let bind_port_s = env::var("BIND_PORT").unwrap_or_else(|_| port.to_string());
        let bind_port = bind_port_s.parse::<u16>().unwrap_or(port);

        let mut iron = Iron::new(handler);
        let threads_s = env::var("IRON_HTTP_THREADS").unwrap_or_else(|_| "1".to_string());
        let threads = threads_s.parse::<usize>().unwrap_or(1);
        iron.threads = threads;
        iron.timeouts = Timeouts {
            keep_alive: Some(Duration::from_millis(10)),
            read: Some(Duration::from_secs(10)),
            write: Some(Duration::from_secs(10)),
        };

        println!(
            "HTTP server for {} starting on {}:{}",
            service_name, &bind_ip, bind_port
        );
        iron.http(&format!("{}:{}", &bind_ip, bind_port)).unwrap()
    }
}

// Re-export Iron support at crate root
#[cfg(feature = "iron")]
pub use iron_support::*;
