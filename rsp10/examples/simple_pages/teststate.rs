#![allow(non_snake_case)]

use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, rsp10_derive::RspKey)]
pub struct KeyI32 {
    pub id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, rsp10::DeriveRspState)]
#[rsp_key(KeyI32)]
#[rsp_auth(CookiePageAuth)]
pub struct PageState {
    message: String,
    dd_testing: i32,
    txt_text_message: String,
    cbTestCheck: bool,
    ddMyDropdown: i32,
    #[serde(skip)]
    btnTest: (), // Button marker - not serialized
}

// Convention-based dropdown source for dd_testing
pub fn get_dd_testing(value: i32) -> HtmlSelect<i32> {
    dbh_get_dropdown(value)
}

// Source function for ddMyDropdown dropdown
pub fn get_ddMyDropdown(value: i32) -> HtmlSelect<i32> {
    dbh_get_dropdown(value)
}

pub type MyPageAuth = CookiePageAuth;

pub fn dbh_get_dropdown(_switchtype: i32) -> HtmlSelect<i32> {
    let mut dd: HtmlSelect<i32> = Default::default();
    dd.item(" --- ".into(), -1);
    for i in 1..23 {
        dd.item(&format!("item {}", i), i);
    }
    dd
}

pub fn dbh_get_testing_dropdown(_switchtype: i32) -> HtmlSelect<i32> {
    let mut dd: HtmlSelect<i32> = Default::default();
    dd.item(" --- ".into(), -1);
    for i in 1..23 {
        dd.item(&format!("testing item {}", i), i);
    }
    dd
}

impl RspState<KeyI32, MyPageAuth> for PageState {
    fn get_template_name() -> String {
        "teststate".to_string()
    }
    fn get_state(auth: &MyPageAuth, key: KeyI32) -> PageState {
        println!("default state for PageState with key: {:?}", &key);
        PageState {
            dd_testing: -1,
            txt_text_message: "test".to_string(),
            ddMyDropdown: key.id.unwrap_or(-1),
            cbTestCheck: true,
            ..Default::default()
        }
    }

    fn fill_data<'a>(ri: RspInfo<'a, Self, KeyI32, MyPageAuth>) -> RspFillDataResult<Self> {
        Self::derive_auto_fill_data_impl(ri)
    }

    fn event_handler<'a>(ri: RspInfo<'a, Self, KeyI32, MyPageAuth>) -> RspEventHandlerResult<Self, KeyI32> {
        let mut action = rsp10::RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;

        if ri.event.event == "submit" {
            state.message = "".to_string();
            if !ri.state_none {
                let ev = ri.event;
                let tgt = &ev.target[..];
                match tgt {
                    "_eq" => {
                        state.txt_text_message =
                            format!("Pressed eq when state is {}", state.dd_testing);
                    }
                    "_lt" => {
                        state.dd_testing = state.dd_testing - 1;
                    }
                    "_gt" => {
                        if state.dd_testing == -1 {
                            state.message = format!("Select a value from the right dropdown first");
                        } else {
                            state.dd_testing = state.dd_testing + 1;
                        }
                    }
                    _ => {}
                }
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
