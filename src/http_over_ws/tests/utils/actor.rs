use candid::Principal;
use http_over_ws::{
    ExecuteHttpRequestResult, GetHttpResponseResult, HttpRequest, HttpRequestId,
    HttpRequestTimeoutMs, HttpResult,
};
use test_utils::{ic_env::TestEnv, identity::generate_random_principal};

pub struct CanisterActor<'a> {
    test_env: &'a TestEnv,
    principal: Principal,
    test_canister_id: Principal,
}

impl<'a> CanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv) -> Self {
        Self {
            test_env,
            principal: generate_random_principal(),
            test_canister_id: test_env.get_canisters().into_keys().next().unwrap(),
        }
    }

    pub fn call_execute_http_request(
        &self,
        req: HttpRequest,
        timeout_ms: Option<HttpRequestTimeoutMs>,
        with_callback: bool,
    ) -> ExecuteHttpRequestResult {
        self.test_env.call_canister_method_with_panic(
            self.test_canister_id,
            self.principal,
            "execute_http_request",
            (req, timeout_ms, with_callback),
        )
    }

    pub fn call_disconnect_all_proxies(&self) {
        self.test_env.call_canister_method_with_panic(
            self.test_canister_id,
            self.principal,
            "disconnect_all_proxies",
            (),
        )
    }

    pub fn query_get_http_response(&self, request_id: HttpRequestId) -> GetHttpResponseResult {
        self.test_env.query_canister_method_with_panic(
            self.test_canister_id,
            self.principal,
            "get_http_response",
            (request_id,),
        )
    }

    pub fn query_get_callback_results(&self) -> Vec<HttpResult> {
        self.test_env.query_canister_method_with_panic(
            self.test_canister_id,
            self.principal,
            "get_callback_results",
            (),
        )
    }
}
