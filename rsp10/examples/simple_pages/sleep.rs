#![allow(non_snake_case)]
use super::imports::*;
// use iron::Plugin;
use iron::Plugin;
use persistent::State;

#[derive(Debug, Clone, Serialize, Deserialize, Default, RspKey)]
pub struct SleepKey {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {
    message: Option<String>,
}

pub type MyPageAuth = NoPageAuth;
// Type alias removed - RspInfo now has only one lifetime

impl RspState<SleepKey, MyPageAuth> for PageState {
    fn get_state(auth: &MyPageAuth, key: SleepKey) -> PageState {
        PageState {
            message: Some(key.message),
        }
    }
    fn fill_data(ri: RspInfo<Self, SleepKey, MyPageAuth>) -> RspFillDataResult<Self> {
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

    fn event_handler(ri: RspInfo<Self, SleepKey, MyPageAuth>) -> RspEventHandlerResult<Self, SleepKey> {
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
