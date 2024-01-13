use candid::CandidType;
use canister::{on_close, on_message, on_open};
use ic_cdk_macros::*;
use ic_websocket_cdk::{
    CanisterWsCloseArguments, CanisterWsCloseResult, CanisterWsGetMessagesArguments,
    CanisterWsGetMessagesResult, CanisterWsMessageArguments, CanisterWsMessageResult,
    CanisterWsOpenArguments, CanisterWsOpenResult, WsHandlers, WsInitParams,
};
use serde::{Deserialize, Serialize};

mod canister;

#[init]
fn init() {
    init_ws();
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct AppMessage {
    pub text: String,
}

pub fn init_ws() {
    let params = WsInitParams::new(WsHandlers {
        on_open: Some(on_open),
        on_message: Some(on_message),
        on_close: Some(on_close),
    });

    ic_websocket_cdk::init(params);
}

// method called by the WS Gateway after receiving FirstMessage from the proxy
#[update]
pub fn ws_open(args: CanisterWsOpenArguments) -> CanisterWsOpenResult {
    ic_websocket_cdk::ws_open(args)
}

// method called by the Ws Gateway when closing the IcWebSocket connection
#[update]
pub fn ws_close(args: CanisterWsCloseArguments) -> CanisterWsCloseResult {
    ic_websocket_cdk::ws_close(args)
}

// method called by the WS Gateway to send a message of type GatewayMessage to the canister
#[update]
pub fn ws_message(
    args: CanisterWsMessageArguments,
    msg_type: Option<AppMessage>,
) -> CanisterWsMessageResult {
    ic_websocket_cdk::ws_message(args, msg_type)
}

// method called by the WS Gateway to get messages for all the proxies it serves
#[query]
pub fn ws_get_messages(args: CanisterWsGetMessagesArguments) -> CanisterWsGetMessagesResult {
    ic_websocket_cdk::ws_get_messages(args)
}
