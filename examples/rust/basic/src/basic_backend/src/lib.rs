use std::{cell::RefCell, collections::HashMap};

use candid::Principal;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{init, post_upgrade, query, update};
use proxy_canister_types::{
    HttpRequest, HttpRequestEndpointArgs, HttpRequestEndpointResult, HttpRequestId,
    HttpRequestTimeoutMs, HttpResult,
};

thread_local! {
    /* flexible */ static PROXY_CANISTER_ID: RefCell<Principal> = RefCell::new(Principal::anonymous());
    /* flexible */ static CALLBACK_RESPONSES: RefCell<HashMap<HttpRequestId, HttpResult>> = RefCell::new(HashMap::new());
}

#[init]
fn init(proxy_canister_id: Principal) {
    PROXY_CANISTER_ID.with(|id| {
        id.replace(proxy_canister_id);
    });
}

#[post_upgrade]
fn post_upgrade(proxy_canister_id: Principal) {
    init(proxy_canister_id);
}

#[update]
async fn http_request_via_proxy(
    req: HttpRequest,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    with_callback: bool,
) -> HttpRequestEndpointResult {
    let proxy_canister_id = PROXY_CANISTER_ID.with(|id| id.borrow().clone());
    let res: Result<(HttpRequestEndpointResult,), _> = ic_cdk::call(
        proxy_canister_id,
        "http_request",
        (HttpRequestEndpointArgs {
            request: req,
            timeout_ms,
            callback_method_name: with_callback.then_some("http_response_callback".to_string()),
        },),
    )
    .await;

    match res {
        Ok(http_res) => http_res.0,
        Err(e) => {
            trap(format!("{:?}", e).as_str());
        }
    }
}

#[update]
fn http_response_callback(request_id: HttpRequestId, res: HttpResult) {
    if caller() != PROXY_CANISTER_ID.with(|id| id.borrow().clone()) {
        trap("Caller is not the proxy canister");
    }

    CALLBACK_RESPONSES.with(|callbacks| {
        let mut callbacks = callbacks.borrow_mut();
        callbacks.insert(request_id, res);
    });
}

#[query]
fn get_http_results() -> HashMap<HttpRequestId, HttpResult> {
    CALLBACK_RESPONSES.with(|responses| responses.borrow().clone())
}

#[query]
fn get_http_result_by_id(request_id: HttpRequestId) -> Option<HttpResult> {
    CALLBACK_RESPONSES.with(|responses| responses.borrow().get(&request_id).cloned())
}
