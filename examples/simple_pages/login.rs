#![allow(non_snake_case)]
use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {
    txtUsername: String,
    txtPassword: String,
    message: Option<String>,
    return_url: String,
}

type MyPageAuth = NoPageAuth;
type MyPageInfo<'a, 'b, 'c> = RspInfo<'a, 'b, 'c, RspState<String, MyPageAuth>, String, MyPageAuth>;

impl RspState<String, MyPageAuth> for PageState {
    fn get_key(
        auth: &MyPageAuth,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<PageState>,
    ) -> Option<String> {
        if let Some(st) = maybe_state {
            Some(st.return_url.clone())
        } else {
            let root = vec!["/".to_string()];
            Some(args.get("ReturnUrl").unwrap_or(&root)[0].clone())
        }
    }
    fn get_state(req: &mut Request, auth: &MyPageAuth, key: String) -> PageState {
        PageState {
            txtUsername: "".to_string(),
            txtPassword: "".to_string(),
            message: None,
            return_url: key,
        }
    }
    fn fill_data(ri: RspInfo<Self, String, MyPageAuth>) -> RspFillDataResult<Self> {
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

    fn event_handler(ri: RspInfo<Self, String, MyPageAuth>) -> RspEventHandlerResult<Self, String> {
        let mut action = rsp10::RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;
        if ri.event.event == "submit" {
            println!("Submit on login page");
            /* replace this "validation" with something more meaningful */
            let env_username = std::env::var("TEST_USERNAME").ok();
            let env_password = std::env::var("TEST_PASSWORD").ok();

            if Some(state.txtUsername.clone()) == env_username
                && Some(state.txtPassword.clone()) == env_password
            {
                let mut groups: HashMap<String, bool> = HashMap::new();
                let username = state.txtUsername.clone();
                println!("Success!");
                let res = ri
                    .req
                    .session()
                    .set(CookiePageAuth::new(&username, Some(groups)));
                match res {
                    Ok(x) => {
                        println!("OK: {:?}", &x);
                        action = rsp10::RspAction::RedirectTo(state.return_url.clone());
                    }
                    Err(e) => {
                        state.message = Some(format!("Error: {:?}", &e));
                    }
                }
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
        }
    }
}
