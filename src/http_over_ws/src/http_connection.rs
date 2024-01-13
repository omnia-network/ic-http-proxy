use candid::{decode_one, encode_one, CandidType, Deserialize};
use ic_cdk::api::management_canister::http_request::{
    HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
};
use ic_cdk_timers::TimerId;
use logger::log;
use std::{future::Future, pin::Pin};

pub type HttpRequestId = u64;

pub type ExecuteHttpRequestResult = Result<HttpRequestId, HttpOverWsError>;
pub type GetHttpResponseResult = Result<HttpResult, HttpOverWsError>;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
}

pub type HttpHeader = ApiHttpHeader;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(
        url: &str,
        method: HttpMethod,
        headers: Vec<HttpHeader>,
        body: Option<Vec<u8>>,
    ) -> Self {
        HttpRequest {
            url: url.to_string(),
            method,
            headers,
            body,
        }
    }
}

pub type HttpResponse = ApiHttpResponse;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpResult {
    Success(HttpResponse),
    Failure(HttpFailureReason),
}

pub(crate) type HttpCallbackWithResult = (HttpCallback, HttpResult);
pub(crate) type HttpCallback = fn(HttpRequestId, HttpResult) -> Pin<Box<dyn Future<Output = ()>>>;

pub type HttpRequestTimeoutMs = u64;

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsMessage {
    SetupProxyClient,
    HttpRequest(HttpRequestId, HttpRequest),
    HttpResponse(HttpRequestId, HttpResponse),
    Error(Option<HttpRequestId>, String),
}

impl HttpOverWsMessage {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        encode_one(self).unwrap()
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_one(bytes).map_err(|e| e.to_string())
    }
}

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsError {
    /// The message is not an HttpOverWsMessage, therefore the on_message callback given to the IC WS cdk
    /// should try to parse it as its own message type.
    NotHttpOverWsType(String),
    /// The message is an HttpOverWsMessage, however it is not what it is expected to be.
    InvalidHttpMessage,
    ProxyNotFound,
    RequestIdNotFound,
    NotYetReceived,
    NoProxiesConnected,
    ConnectionNotAssignedToProxy,
    RequestFailed(HttpFailureReason),
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpFailureReason {
    RequestTimeout,
    ProxyError(String),
}

pub(crate) struct HttpConnection {
    id: HttpRequestId,
    request: HttpRequest,
    state: HttpConnectionState,
}

impl HttpConnection {
    pub(crate) fn new(
        id: HttpRequestId,
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timer_id: Option<TimerId>,
    ) -> Self {
        HttpConnection {
            id,
            request,
            state: HttpConnectionState::new(timer_id, callback),
        }
    }

    pub(crate) fn get_request(&self) -> HttpRequest {
        self.request.clone()
    }

    pub(crate) fn get_response(&self) -> GetHttpResponseResult {
        match self.state {
            HttpConnectionState::WaitingForResponse(_) => Err(HttpOverWsError::NotYetReceived),
            HttpConnectionState::Failed(ref reason) => {
                Err(HttpOverWsError::RequestFailed(reason.clone()))
            }
            HttpConnectionState::Success(ref response) => Ok(HttpResult::Success(response.clone())),
        }
    }

    pub(crate) fn update_state(
        &mut self,
        http_result: HttpResult,
    ) -> Option<HttpCallbackWithResult> {
        match &mut self.state {
            HttpConnectionState::WaitingForResponse((timer_id, callback)) => {
                match http_result {
                    HttpResult::Success(response) => {
                        // response has been received, clear the timer if it was set
                        if let Some(timer_id) = timer_id.take() {
                            ic_cdk_timers::clear_timer(timer_id);
                        }

                        log!(
                            "http_over_ws: HTTP connection with id {} received response",
                            self.id
                        );

                        let mut res = None;
                        if let Some(callback) = callback.take() {
                            res = Some((callback, HttpResult::Success(response.clone())));
                        }

                        self.state = HttpConnectionState::Success(response);

                        return res;
                    }
                    HttpResult::Failure(reason) => {
                        log!(
                            "http_over_ws: HTTP connection with id {} failed with reason {:?}",
                            self.id,
                            reason
                        );

                        let mut res = None;
                        if let Some(callback) = callback.take() {
                            res = Some((callback, HttpResult::Failure(reason.clone())));
                        }

                        self.state = HttpConnectionState::Failed(reason);

                        return res;
                    }
                }
            }
            HttpConnectionState::Failed(_) => {
                log!(
                    "http_over_ws: HTTP connection with id {} has already failed",
                    self.id
                );
            }
            HttpConnectionState::Success(_) => {
                log!(
                    "http_over_ws: HTTP connection with id {} has already succeeded",
                    self.id
                );
            }
        }
        None
    }
}

#[derive(Clone)]
pub(crate) enum HttpConnectionState {
    WaitingForResponse((Option<TimerId>, Option<HttpCallback>)),
    Failed(HttpFailureReason),
    Success(HttpResponse),
}

impl HttpConnectionState {
    pub(crate) fn new(timer_id: Option<TimerId>, callback: Option<HttpCallback>) -> Self {
        HttpConnectionState::WaitingForResponse((timer_id, callback))
    }
}
