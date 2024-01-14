use std::{cell::RefCell, collections::HashMap};

use candid::Principal;
use ic_cdk::{print, trap};
use ic_cdk_macros::{init, query, update};
use proxy_canister_types::{
    HttpRequestEndpointArgs, HttpRequestEndpointResult, HttpRequestId, HttpResult,
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

#[update]
async fn http_request_via_proxy(args: HttpRequestEndpointArgs) -> HttpRequestEndpointResult {
    let proxy_canister_id = PROXY_CANISTER_ID.with(|id| id.borrow().clone());
    let res: Result<(HttpRequestEndpointResult,), _> =
        ic_cdk::call(proxy_canister_id, "http_request", (args,)).await;

    match res {
        Ok(http_res) => http_res.0,
        Err(e) => {
            trap(format!("{:?}", e).as_str());
        }
    }
}

#[update]
fn http_response_callback(request_id: HttpRequestId, res: HttpResult) {
    CALLBACK_RESPONSES.with(|callbacks| {
        let mut callbacks = callbacks.borrow_mut();
        callbacks.insert(request_id, res);
    });
}

#[update]
fn http_response_callback_traps(request_id: HttpRequestId) {
    trap(format!("request_id: {}", request_id).as_str());
}

#[update]
fn http_response_callback_wrong_args(s: String) {
    print(s);
}

#[query]
fn get_callback_results() -> HashMap<HttpRequestId, HttpResult> {
    CALLBACK_RESPONSES.with(|responses| responses.borrow().clone())
}
