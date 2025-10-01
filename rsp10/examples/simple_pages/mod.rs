mod imports;

#[path = "login.rs"]
mod login;
#[path = "logout.rs"]
mod logout;
#[path = "teststate.rs"]
mod teststate;
#[path = "sleep.rs"]
mod sleep;

pub fn get_router() -> router::Router {
    use router::Router;

    let mut r = Router::new();

    // Create handlers using the new Iron adapter
    let login_handler = rsp10::make_iron_handler::<login::PageState, String, login::MyPageAuth>();
    let logout_handler = rsp10::make_iron_handler::<logout::PageState, (), logout::MyPageAuth>();
    let teststate_handler = rsp10::make_iron_handler::<teststate::PageState, teststate::KeyI32, teststate::MyPageAuth>();
    let sleep_handler = rsp10::make_iron_handler::<sleep::PageState, String, sleep::MyPageAuth>();

    // Register routes
    r.get("/login", login_handler.clone(), "GET/login".to_string());
    r.post("/login", login_handler, "POST/login".to_string());

    r.get("/logout", logout_handler.clone(), "GET/logout".to_string());
    r.post("/logout", logout_handler, "POST/logout".to_string());

    r.get("/teststate", teststate_handler.clone(), "GET/teststate".to_string());
    r.post("/teststate", teststate_handler.clone(), "POST/teststate".to_string());

    r.get("/sleep", sleep_handler.clone(), "GET/sleep".to_string());
    r.post("/sleep", sleep_handler, "POST/sleep".to_string());

    // Mount teststate on root as well
    let root_handler = rsp10::make_iron_handler::<teststate::PageState, teststate::KeyI32, teststate::MyPageAuth>();
    r.get("/", root_handler.clone(), "GET/".to_string());
    r.post("/", root_handler, "POST/".to_string());

    r
}
