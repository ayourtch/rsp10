#![allow(non_snake_case)]
use super::imports::*;
// use iron::Plugin;
use iron::Plugin;
use persistent::State;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {
    message: Option<String>,
}

pub type MyPageAuth = NoPageAuth;
// Type alias removed - RspInfo now has only one lifetime

impl RspState<String, MyPageAuth> for PageState {
    fn get_key(
        auth: &MyPageAuth,
        args: &HashMap<String, Vec<String>>,
        maybe_state: &Option<PageState>,
    ) -> Option<String> {
        if let Some(st) = maybe_state {
            None
        } else {
            None
        }
    }
    fn get_state(auth: &MyPageAuth, key: String) -> PageState {
        PageState {
            message: Some(key),
        }
    }
    fn fill_data(ri: RspInfo<Self, String, MyPageAuth>) -> RspFillDataResult<Self> {
        use std::{thread, time};

        let num_sec = 60;
        // println!("Sleeping for {} seconds...", num_sec);
        //
        // TODO: request_stop needs Iron-specific access
        // rsp10::request_stop(ri.req);

        let mut modified = false;
        let mut gd = RspDataBuilder::new();

        Self::fill_data_result(ri, gd)
    }

    fn event_handler(ri: RspInfo<Self, String, MyPageAuth>) -> RspEventHandlerResult<Self, String> {
        let mut action = rsp10::RspAction::Render;
        let mut initial_state = ri.initial_state;
        let mut state = ri.state;

        RspEventHandlerResult {
            initial_state,
            state,
            action,
            new_auth: None,
        }
    }
}
