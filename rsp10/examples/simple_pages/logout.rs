#![allow(unused_variables)]

use super::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, RspStateDerive)]
#[rsp_key(())]
#[rsp_auth(NoPageAuth)]
pub struct PageState {}

impl RspState<(), MyPageAuth> for PageState {
    fn get_state(auth: &MyPageAuth, key: ()) -> PageState {
        PageState {}
    }

    fn event_handler(ri: RspInfo<Self, (), MyPageAuth>) -> RspEventHandlerResult<Self, ()> {
        // TODO: Clear session once SessionStorage is fixed
        // For now, just redirect
        RspEventHandlerResult {
            initial_state: ri.initial_state,
            state: ri.state,
            action: rsp10::RspAction::RedirectTo("/".to_string()),
            new_auth: None,  // TODO: Clear session by setting empty/default auth
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
