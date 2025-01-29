#![allow(unused_variables)]

use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {}

type MyPageAuth = NoPageAuth;

impl RspState<(), MyPageAuth> for PageState {
    fn get_state(req: &mut Request, auth: &MyPageAuth, key: ()) -> PageState {
        PageState {}
    }

    fn event_handler(ri: RspInfo<Self, (), MyPageAuth>) -> RspEventHandlerResult<Self, ()> {
        ri.req.session().clear().unwrap();
        let mut res = Self::default_event_handler_result(ri);
        res.action = rsp10::RspAction::RedirectTo(format!("/"));
        res
    }
}
