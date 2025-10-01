use rsp10::*;

#[derive(Debug, Clone, Default, rsp10::DeriveRspState)]
pub struct TestState {
    message: String,
    dd_testing: i32,
    txt_text_message: String,
    cbTestCheck: bool,
    #[rsp_source(get_custom_dropdown)]
    ddMyDropdown: i32,
}

pub fn get_dd_testing(value: i32) -> HtmlSelect<i32> {
    let mut dd: HtmlSelect<i32> = Default::default();
    dd.item(" --- ".into(), -1);
    dd
}

pub fn get_custom_dropdown(value: i32) -> HtmlSelect<i32> {
    let mut dd: HtmlSelect<i32> = Default::default();
    dd.item("Custom".into(), value);
    dd
}

fn main() {
    println!("Derive macro test");
}
