#![allow(non_snake_case)]
use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, RspKey)]
pub struct LoginKey {
    pub return_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, RspStateDerive)]
#[rsp_key(LoginKey)]
#[rsp_auth(NoPageAuth)]
pub struct PageState {
    txtUsername: String,
    txtPassword: String,
    message: Option<String>,
    return_url: String,
}

// Type alias removed - RspInfo now has only one lifetime

impl RspState<LoginKey, MyPageAuth> for PageState {
    fn get_state(auth: &MyPageAuth, key: LoginKey) -> PageState {
        PageState {
            txtUsername: "".to_string(),
            txtPassword: "".to_string(),
            message: None,
            return_url: if key.return_url.len() == 0 { "/".to_string() } else { key.return_url },
        }
    }

    fn fill_data<'a>(ri: RspInfo<'a, Self, LoginKey, MyPageAuth>) -> RspFillDataResult<Self> {
        let mut modified = false;
        let mut gd = RspDataBuilder::new();
        let env_username = std::env::var("TEST_USERNAME").ok();
        let env_password = std::env::var("TEST_PASSWORD").ok();
        rsp10_text!(txtUsername, ri => gd, modified);
        rsp10_text!(txtPassword, ri => gd, modified);
        rsp10_data!(env_username => gd);
        rsp10_data!(env_password => gd);
        Self::fill_data_result(ri, gd)
    }

    fn event_handler<'a>(ri: RspInfo<'a, Self, LoginKey, MyPageAuth>) -> RspEventHandlerResult<Self, LoginKey> {
        let mut action = rsp10::RspAction::Render;
        let mut initial_state = ri.initial_state.clone();
        let mut state = ri.state.clone();

        println!("Debug - event_handler called");
        println!("Debug - state_none: {}", ri.state_none);
        println!("Debug - initial_state: {:?}", initial_state);
        println!("Debug - state: {:?}", state);

        if ri.event.event == "submit" {
            println!("Submit on login page");
            println!("Debug - received username: '{}'", state.txtUsername);
            println!("Debug - received password: '{}'", state.txtPassword);
            /* replace this "validation" with something more meaningful */
            let env_username = std::env::var("TEST_USERNAME").ok();
            let env_password = std::env::var("TEST_PASSWORD").ok();
            println!("Debug - env username: {:?}", env_username);
            println!("Debug - env password: {:?}", env_password);

            if Some(state.txtUsername.clone()) == env_username
                && Some(state.txtPassword.clone()) == env_password
            {
                println!("Success! Login for: {}", &state.txtUsername);
                // Create authenticated user and store in session
                let auth = CookiePageAuth::new(&state.txtUsername, None);
                action = rsp10::RspAction::RedirectTo(state.return_url.clone());

                return RspEventHandlerResult {
                    initial_state,
                    state,
                    action,
                    new_auth: Some(Box::new(auth)),
                };
            } else {
                println!("Login failure");
                state.message = Some(format!("Login {} invalid", &state.txtUsername));
                state.txtUsername = format!("");
                state.txtPassword = format!("");
            }
        }
        RspEventHandlerResult {
            initial_state,
            state,
            action,
            new_auth: None,
        }
    }
}

// Public bridge function for Axum - delegates to auto-generated handler
#[cfg(feature = "axum")]
pub async fn axum_bridge(
    state: axum::extract::State<std::sync::Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
    form: Option<axum::extract::Form<std::collections::HashMap<String, String>>>,
) -> axum::response::Response {
    axum_handler(query, form, state).await
}
