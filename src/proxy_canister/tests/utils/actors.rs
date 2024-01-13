use std::collections::HashMap;

use candid::Principal;
use http_over_ws::{HttpRequestId, HttpResult};
use pocket_ic::UserError;
use proxy_canister_types::{CanisterRequest, HttpRequestEndpointArgs, HttpRequestEndpointResult};
use test_utils::{ic_env::TestEnv, identity::generate_random_principal};

pub struct TestUserCanisterActor<'a> {
    test_env: &'a TestEnv,
    principal: Principal,
    canister_id: Principal,
}

impl<'a> TestUserCanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv, canister_id: Principal) -> Self {
        Self {
            test_env,
            principal: generate_random_principal(),
            canister_id,
        }
    }

    pub fn call_http_request_via_proxy(
        &self,
        args: HttpRequestEndpointArgs,
    ) -> HttpRequestEndpointResult {
        self.test_env.call_canister_method_with_panic(
            self.canister_id,
            self.principal,
            "http_request_via_proxy",
            (args,),
        )
    }

    pub fn query_get_callback_results(&self) -> HashMap<HttpRequestId, HttpResult> {
        self.test_env.query_canister_method_with_panic(
            self.canister_id,
            self.principal,
            "get_callback_results",
            (),
        )
    }
}

pub struct ProxyCanisterActor<'a> {
    test_env: &'a TestEnv,
    canister_id: Principal,
}

impl<'a> ProxyCanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv, canister_id: Principal) -> Self {
        Self {
            test_env,
            canister_id,
        }
    }

    pub fn call_http_request(
        &self,
        caller: Principal,
        args: HttpRequestEndpointArgs,
    ) -> Result<HttpRequestEndpointResult, UserError> {
        self.test_env
            .call_canister_method(self.canister_id, caller, "http_request", (args,))
    }

    pub fn query_get_request_by_id(
        &self,
        caller: Principal,
        request_id: HttpRequestId,
    ) -> Result<Option<CanisterRequest>, UserError> {
        self.test_env.query_canister_method(
            self.canister_id,
            caller,
            "get_request_by_id",
            (request_id,),
        )
    }

    pub fn query_get_request_by_id_with_panic(
        &self,
        caller: Principal,
        request_id: HttpRequestId,
    ) -> Option<CanisterRequest> {
        self.query_get_request_by_id(caller, request_id)
            .expect("query_get_request_by_id should succeed")
    }

    pub fn query_get_logs(&self, caller: Principal) -> Result<Vec<(String, String)>, UserError> {
        self.test_env
            .query_canister_method(self.canister_id, caller, "get_logs", ())
    }
}
