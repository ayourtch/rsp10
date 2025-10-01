mod imports;

#[path = "login.rs"]
mod login;
#[path = "logout.rs"]
mod logout;
#[path = "teststate.rs"]
mod teststate;
// #[path = "sleep.rs"]
// mod sleep; // Temporarily disabled due to serde issues

// Define all routes in one place - generates both Iron and Axum routers
rsp10::rsp_routes! {
    "/login" => login,
    "/logout" => logout,
    "/teststate" => teststate,
    "/" => teststate,
}
