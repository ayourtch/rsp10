mod imports;

pub fn get_router() -> router::Router {
    use crate::rsp10::RspState;
    use router::Router;

    let mut r = Router::new();
    rsp10_page!(r, "/login", login, "login.rs");
    rsp10_page!(r, "/logout", logout, "logout.rs");
    rsp10_page!(r, "/teststate", teststate, "teststate.rs");
    rsp10_page!(r, "/sleep", sleep, "sleep.rs");
    // Mount the teststate as well on the root URL
    rsp10_page!(r, "/", teststate_root, "teststate.rs");
    return r;
}
