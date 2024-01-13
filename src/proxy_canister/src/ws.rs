use http_over_ws::{HttpOverWsError, HttpOverWsMessage};
use ic_cdk_macros::{query, update};
use ic_websocket_cdk::{
    CanisterWsCloseArguments, CanisterWsCloseResult, CanisterWsGetMessagesArguments,
    CanisterWsGetMessagesResult, CanisterWsMessageArguments, CanisterWsMessageResult,
    CanisterWsOpenArguments, CanisterWsOpenResult, OnCloseCallbackArgs, OnMessageCallbackArgs,
    OnOpenCallbackArgs, WsHandlers, WsInitParams,
};

pub use ic_websocket_cdk::send;
use logger::log;

pub fn init_ws() {
    let params = WsInitParams::new(WsHandlers {
        on_open: Some(on_open),
        on_message: Some(on_message),
        on_close: Some(on_close),
    });

    ic_websocket_cdk::init(params);
}

pub fn on_open(args: OnOpenCallbackArgs) {
    log!("[ws]: WS client: {} connected", args.client_principal);
}

pub fn on_message(args: OnMessageCallbackArgs) {
    if let Err(HttpOverWsError::NotHttpOverWsType(_)) =
        http_over_ws::try_handle_http_over_ws_message(args.client_principal, args.message.clone())
    {
        log!(
            "[ws]: Received WS client message: {:?} from {}",
            args.message,
            args.client_principal
        );
    }
}

pub fn on_close(args: OnCloseCallbackArgs) {
    if let Err(_) = http_over_ws::try_disconnect_http_proxy(args.client_principal) {
        log!("[ws]: WS client {} disconnected", args.client_principal);
    } else {
        log!("[ws]: Proxy client {} disconnected", args.client_principal);
    }
}

// method called by the WS Gateway after receiving FirstMessage from the client
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
    msg_type: Option<HttpOverWsMessage>,
) -> CanisterWsMessageResult {
    ic_websocket_cdk::ws_message(args, msg_type)
}

// method called by the WS Gateway to get messages for all the clients it serves
#[query]
pub fn ws_get_messages(args: CanisterWsGetMessagesArguments) -> CanisterWsGetMessagesResult {
    ic_websocket_cdk::ws_get_messages(args)
}
