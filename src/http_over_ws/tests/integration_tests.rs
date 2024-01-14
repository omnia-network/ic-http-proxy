mod utils;

use std::{path::PathBuf, sync::Once};

use candid::{Nat, Principal};
use http_over_ws::{
    HttpFailureReason, HttpMethod, HttpOverWsError, HttpOverWsMessage, HttpRequest, HttpResponse,
    HttpResult,
};
use ic_websocket_cdk::types::{
    CanisterCloseMessageContent, CloseMessageReason, WebsocketServiceMessageContent,
};
use lazy_static::lazy_static;
use test_utils::{
    ic_env::{get_test_env, load_canister_wasm_from_path, CanisterData, TestEnv},
    proxy_client::ProxyClient,
};

use crate::utils::{
    actor::CanisterActor,
    constants::TEST_HTTP_REQUEST_HEADER,
    constants::{TEST_HTTP_RESPONSE_HEADER, TEST_URL},
};

lazy_static! {
    pub static ref TEST_CANISTER_WASM_MODULE: Vec<u8> =
        load_canister_wasm_from_path(&PathBuf::from(
            std::env::var("TEST_CANISTER_WASM_PATH").expect("TEST_CANISTER_WASM_PATH must be set")
        ));
}

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        get_test_env().add_canister(CanisterData {
            wasm_module: TEST_CANISTER_WASM_MODULE.clone(),
            args: vec![],
            controller: None,
        });
    });
}

fn reset_canister() {
    let test_env = get_test_env();
    let canister_id = test_env.get_canisters().into_keys().next().unwrap();
    test_env.reset_canister(&canister_id);
}

fn get_test_canister_id(test_env: &TestEnv) -> Principal {
    test_env.get_canisters().into_keys().next().unwrap()
}

#[test]
fn test_execute_http_request_no_clients_connected() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let canister_actor = CanisterActor::new(&test_env);

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let res = canister_actor.call_execute_http_request(request.clone(), None, false);

    assert_eq!(res, Err(HttpOverWsError::NoProxiesConnected));
}

#[test]
fn test_execute_http_request_after_client_disconnected() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();
    proxy_client.close_ws_connection();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let res = canister_actor.call_execute_http_request(request, None, false);

    assert_eq!(res, Err(HttpOverWsError::NoProxiesConnected));
}

#[test]
fn test_execute_http_request_without_response() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let http_response = canister_actor.query_get_http_response(request_id);
    assert_eq!(http_response, Err(HttpOverWsError::NotYetReceived));
}

#[test]
fn test_execute_http_request() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response)));

    let callback_res = canister_actor.query_get_callback_results();
    assert_eq!(callback_res.len(), 0);
}

#[test]
fn test_execute_http_request_with_body() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        Some(vec![1, 2, 3]),
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response)));
}

#[test]
fn test_execute_http_request_with_proxy_error() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let error_message = String::from("proxy error");

    proxy_client.send_http_over_ws_message(HttpOverWsMessage::Error(
        Some(request_id),
        error_message.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(
        res,
        Err(HttpOverWsError::RequestFailed(
            HttpFailureReason::ProxyError(error_message)
        ))
    );
}

#[test]
fn test_execute_http_request_only_assigned_proxy() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client1 = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let mut proxy_client2 = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client1.setup_proxy();
    proxy_client2.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    // discover to which proxy the request was assigned
    let proxy1_messages = proxy_client1.get_http_over_ws_messages();
    let proxy2_messages = proxy_client2.get_http_over_ws_messages();
    assert!(proxy1_messages.len() != proxy2_messages.len());

    let (mut assigned_proxy, mut idle_proxy) = if proxy1_messages.len() > 0 {
        (proxy_client1, proxy_client2)
    } else {
        (proxy_client2, proxy_client1)
    };

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };

    // test that the canister doesn't trap or break the state
    // if the response comes from an unassigned proxy
    idle_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));
    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Err(HttpOverWsError::NotYetReceived));

    assigned_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response)));
}

#[test]
fn test_execute_http_request_multiple() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request1 = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );
    let request2 = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id1 = canister_actor
        .call_execute_http_request(request1.clone(), None, false)
        .unwrap();
    let request_id2 = canister_actor
        .call_execute_http_request(request2.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_client.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(request_id1, request1),
    );
    assert_eq!(
        proxy_messages[1],
        HttpOverWsMessage::HttpRequest(request_id2, request2),
    );

    let http_response1 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    let http_response2 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };

    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id1,
        http_response1.clone(),
    ));
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id2,
        http_response2.clone(),
    ));

    let res1 = canister_actor.query_get_http_response(request_id1);
    assert_eq!(res1, Ok(HttpResult::Success(http_response1)));
    let res2 = canister_actor.query_get_http_response(request_id2);
    assert_eq!(res2, Ok(HttpResult::Success(http_response2)));
}

#[test]
fn test_execute_http_request_before_timeout() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), Some(10_000), false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    // make some time pass, but not enough to trigger the timeout
    test_env.advance_canister_time_ms(8_000);

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response)));
}

#[test]
fn test_execute_http_request_timeout_expired() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), Some(10_000), false)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    // advance time so that the timeout expires
    test_env.advance_canister_time_ms(10_000);

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(
        res,
        Err(HttpOverWsError::RequestFailed(
            HttpFailureReason::RequestTimeout
        ))
    );

    // even after sending the response,
    // the request shouldn't change its state
    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(
        res,
        Err(HttpOverWsError::RequestFailed(
            HttpFailureReason::RequestTimeout
        ))
    );
}

#[test]
fn test_execute_http_request_with_callback() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, true)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response.clone())));

    let callback_res = canister_actor.query_get_callback_results();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], HttpResult::Success(http_response));
}

#[test]
fn test_execute_http_request_duplicate_response() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let request_id = canister_actor
        .call_execute_http_request(request.clone(), None, true)
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    let http_response1 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response1.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response1.clone())));
    let callback_res = canister_actor.query_get_callback_results();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], HttpResult::Success(http_response1.clone()));

    // sending another response again should not change the state
    // and hence not invoke the callback again
    let http_response2 = HttpResponse {
        status: Nat::from(400),
        headers: vec![],
        body: vec![4, 5, 6],
    };
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id,
        http_response2.clone(),
    ));

    let res = canister_actor.query_get_http_response(request_id);
    assert_eq!(res, Ok(HttpResult::Success(http_response1.clone())));
    let callback_res = canister_actor.query_get_callback_results();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], HttpResult::Success(http_response1));
}

#[test]
fn test_get_http_response_not_found() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let canister_actor = CanisterActor::new(&test_env);

    let res = canister_actor.query_get_http_response(0);
    assert_eq!(res, Err(HttpOverWsError::RequestIdNotFound));
}

#[test]
fn test_disconnect_all_proxies() {
    setup();
    reset_canister();
    let test_env = get_test_env();
    let mut proxy_client1 = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let mut proxy_client2 = ProxyClient::new(&test_env, get_test_canister_id(&test_env));
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client1.setup_proxy();
    proxy_client2.setup_proxy();

    canister_actor.call_disconnect_all_proxies();

    let close_msg1 = proxy_client1
        .get_ws_messages()
        .last()
        .and_then(|m| WebsocketServiceMessageContent::from_candid_bytes(&m.content).ok())
        .unwrap();
    let close_msg2 = proxy_client2
        .get_ws_messages()
        .last()
        .and_then(|m| WebsocketServiceMessageContent::from_candid_bytes(&m.content).ok())
        .unwrap();

    assert_eq!(
        close_msg1,
        WebsocketServiceMessageContent::CloseMessage(CanisterCloseMessageContent {
            reason: CloseMessageReason::ClosedByApplication
        }),
    );
    assert_eq!(
        close_msg2,
        WebsocketServiceMessageContent::CloseMessage(CanisterCloseMessageContent {
            reason: CloseMessageReason::ClosedByApplication
        }),
    );

    let res = canister_actor.call_execute_http_request(
        HttpRequest::new(
            TEST_URL,
            HttpMethod::GET,
            vec![TEST_HTTP_REQUEST_HEADER.clone()],
            None,
        ),
        None,
        false,
    );
    assert_eq!(res, Err(HttpOverWsError::NoProxiesConnected));
}
