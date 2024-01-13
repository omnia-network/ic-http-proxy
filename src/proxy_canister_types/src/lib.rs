use candid::{CandidType, Deserialize, Principal};
#[allow(unused_imports)] // needed for comments
use http_over_ws::HttpResult;
use http_over_ws::{HttpOverWsError, HttpRequest, HttpRequestId, HttpRequestTimeoutMs};

pub type CanisterId = Principal;
pub type CanisterCallbackMethodName = String;

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub struct HttpRequestEndpointArgs {
    pub request: HttpRequest,
    pub timeout_ms: Option<HttpRequestTimeoutMs>,
    pub callback_method_name: Option<CanisterCallbackMethodName>,
}

pub type HttpRequestEndpointResult = Result<HttpRequestId, ProxyCanisterError>;

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum ProxyCanisterError {
    InvalidRequest(InvalidRequest),
    HttpOverWs(HttpOverWsError),
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum InvalidRequest {
    InvalidUrl(String),
    TooManyHeaders,
    InvalidTimeout,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RequestState {
    /// Used to indicate that the request is being executed.
    Executing(Option<CanisterCallbackMethodName>),
    /// Used to indicate that the request has been executed.
    ///
    /// Note: a request that resulted in a [HttpResult::Failure] error will still be in this state,
    /// because from the proxy canister perspective, the request has been executed.
    Executed,
    /// Used to indicate that the proxy canister failed to call the callback method
    /// on the user canister.
    CallbackFailed(String),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CanisterRequest {
    pub canister_id: CanisterId,
    pub state: RequestState,
}

impl CanisterRequest {
    pub fn new(
        canister_id: CanisterId,
        callback_method_name: Option<CanisterCallbackMethodName>,
    ) -> Self {
        Self {
            canister_id,
            state: RequestState::Executing(callback_method_name),
        }
    }

    pub fn set_executed(&mut self) {
        self.state = RequestState::Executed;
    }

    pub fn set_failed(&mut self, reason: String) {
        self.state = RequestState::CallbackFailed(reason);
    }
}
