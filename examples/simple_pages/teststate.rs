#![allow(non_snake_case)]

use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyI32 {
    id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageState {
    message: String,
    dd_testing: i32,
    txt_text_message: String,
    cbTestCheck: bool,
    ddMyDropdown: i32,
}

type MyPageAuth = CookiePageAuth;

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
    fn get_key(
        auth: &MyPageAuth,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<PageState>,
    ) -> Option<KeyI32> {
        Some(KeyI32 {
            id: args.get("id").map_or(None, |x| x[0].parse::<i32>().ok()),
        })
    }
    fn get_state(req: &mut Request, auth: &MyPageAuth, key: KeyI32) -> PageState {
        println!("default state for PageState with key: {:?}", &key);
        PageState {
            dd_testing: -1,
            txt_text_message: "test".to_string(),
            ddMyDropdown: key.id.unwrap_or(-1),
            cbTestCheck: true,
            ..Default::default()
        }
    }
    fn fill_data(ri: RspInfo<Self, KeyI32, MyPageAuth>) -> RspFillDataResult<Self> {
        let mut modified = false;
        let mut ri = ri;
        let mut gd = RspDataBuilder::new();
        let real_key = ri.key.id.unwrap_or(-1);
        println!("{:?}", &ri.state);

        rsp10_button!(btnTest, "Test button" => gd);
        rsp10_select!(dd_testing, dbh_get_dropdown(ri.state.dd_testing), ri => gd, modified);
        rsp10_select!(ddMyDropdown, dbh_get_dropdown(real_key), ri => gd, modified);
        rsp10_text!(txt_text_message, ri => gd, modified);
        rsp10_check!(cbTestCheck, ri => gd, modified);
        rsp10_data!(modified => gd);
        gd.insert_fn2("FooFunction", |x, render| {
            eprintln!("FooFunction before render: {:?}", &x);
            let res = render(x);
            eprintln!("FooFunction after render: {:?}", &res);
            res
        });

        Self::fill_data_result(ri, gd)
    }

    fn event_handler(ri: RspInfo<Self, KeyI32, MyPageAuth>) -> RspEventHandlerResult<Self, KeyI32> {
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
        }
    }
}
