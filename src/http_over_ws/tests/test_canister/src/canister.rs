use std::cell::RefCell;

use http_over_ws::{
    ExecuteHttpRequestResult, GetHttpResponseResult, HttpOverWsError, HttpRequest, HttpRequestId,
    HttpRequestTimeoutMs, HttpResult,
};
use ic_cdk_macros::{query, update};
use ic_websocket_cdk::{OnCloseCallbackArgs, OnMessageCallbackArgs, OnOpenCallbackArgs};
use logger::log;

thread_local! {
    /* flexible */ static CALLBACK_RESPONSES: RefCell<Vec<HttpResult>> = RefCell::new(Vec::new());
}

pub fn on_open(args: OnOpenCallbackArgs) {
    log!("WS proxy: {} connected", args.client_principal);
}

pub fn on_message(args: OnMessageCallbackArgs) {
    if let Err(HttpOverWsError::NotHttpOverWsType(_)) =
        http_over_ws::try_handle_http_over_ws_message(args.client_principal, args.message.clone())
    {
        log!(
            "Received WS proxy message: {:?} from {}",
            args.message,
            args.client_principal
        );
    }
}

pub fn on_close(args: OnCloseCallbackArgs) {
    if let Err(_) = http_over_ws::try_disconnect_http_proxy(args.client_principal) {
        log!("WS proxy {} disconnected", args.client_principal);
    } else {
        log!("Proxy client {} disconnected", args.client_principal);
    }
}

#[update]
fn execute_http_request(
    req: HttpRequest,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    with_callback: bool,
) -> ExecuteHttpRequestResult {
    http_over_ws::execute_http_request(
        req,
        with_callback.then_some(|_, res| Box::pin(callback(res))),
        timeout_ms,
        ic_websocket_cdk::send,
    )
}

async fn callback(http_result: HttpResult) {
    CALLBACK_RESPONSES.with(|http_results| http_results.borrow_mut().push(http_result));
}

#[update]
fn disconnect_all_proxies() {
    http_over_ws::disconnect_all_connected_proxies(ic_websocket_cdk::close);
}

#[query]
fn get_http_response(id: HttpRequestId) -> GetHttpResponseResult {
    http_over_ws::get_http_response(id)
}

#[query]
fn get_callback_results() -> Vec<HttpResult> {
    CALLBACK_RESPONSES.with(|http_results| http_results.borrow().clone())
}
