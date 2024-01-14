use candid::{decode_one, encode_one, Principal};
use http_over_ws::HttpOverWsMessage;

use crate::ic_env::TestEnv;

use super::{identity::generate_random_principal, rand::generate_random_nonce};
use ic_websocket_cdk::types::{CanisterOutputCertifiedMessages, ClientKey, WebsocketMessage};
use ic_websocket_cdk::{
    CanisterWsCloseArguments, CanisterWsCloseResult, CanisterWsGetMessagesArguments,
    CanisterWsGetMessagesResult, CanisterWsMessageArguments, CanisterWsMessageResult,
    CanisterWsOpenArguments, CanisterWsOpenResult,
};

pub struct ProxyClient<'a> {
    test_env: &'a TestEnv,
    canister_id: Principal,
    client_key: ClientKey,
    gateway_principal: Principal,
    outgoing_messages_sequence_num: u64,
    polling_nonce: u64,
}

impl<'a> ProxyClient<'a> {
    pub fn new(test_env: &'a TestEnv, canister_id: Principal) -> Self {
        Self {
            test_env,
            canister_id,
            client_key: generate_random_client_key(),
            gateway_principal: generate_random_principal(),
            outgoing_messages_sequence_num: 0,
            polling_nonce: 0,
        }
    }

    pub fn open_ws_connection(&self) {
        let res: CanisterWsOpenResult = self.test_env.call_canister_method_with_panic(
            self.canister_id,
            self.client_key.client_principal,
            "ws_open",
            (CanisterWsOpenArguments::new(
                self.client_key.client_nonce,
                self.gateway_principal,
            ),),
        );

        assert!(res.is_ok());
    }

    pub fn setup_proxy(&mut self) {
        self.open_ws_connection();

        self.send_http_over_ws_message(HttpOverWsMessage::SetupProxyClient);
    }

    pub fn send_ws_message(&mut self, message: Vec<u8>) {
        self.outgoing_messages_sequence_num += 1;

        let res: CanisterWsMessageResult = self.test_env.call_canister_method_with_panic(
            self.canister_id,
            self.client_key.client_principal,
            "ws_message",
            (CanisterWsMessageArguments::new(WebsocketMessage::new(
                self.client_key.clone(),
                self.outgoing_messages_sequence_num,
                0, // we can ignore the timestamp here
                false,
                message,
            )),),
        );

        assert!(res.is_ok());
    }

    pub fn send_http_over_ws_message(&mut self, message: HttpOverWsMessage) {
        self.send_ws_message(encode_one(message).unwrap());
    }

    pub fn get_ws_messages(&mut self) -> Vec<WebsocketMessage> {
        let res: CanisterWsGetMessagesResult = self.test_env.query_canister_method_with_panic(
            self.canister_id,
            self.gateway_principal,
            "ws_get_messages",
            (CanisterWsGetMessagesArguments::new(self.polling_nonce),),
        );

        match res {
            CanisterWsGetMessagesResult::Ok(CanisterOutputCertifiedMessages {
                messages, ..
            }) => {
                self.polling_nonce += messages.len() as u64;

                messages
                    .iter()
                    .filter_map(|m| {
                        let msg: WebsocketMessage = serde_cbor::from_slice(&m.content).unwrap();

                        msg.client_key.eq(&self.client_key).then_some(msg)
                    })
                    .collect()
            }
            CanisterWsGetMessagesResult::Err(_) => panic!("Failed to get messages"),
        }
    }

    pub fn get_http_over_ws_messages(&mut self) -> Vec<HttpOverWsMessage> {
        self.get_ws_messages()
            .iter()
            .filter_map(|msg| (!msg.is_service_message).then(|| decode_one(&msg.content).unwrap()))
            .collect()
    }

    pub fn close_ws_connection(&self) {
        let res: CanisterWsCloseResult = self.test_env.call_canister_method_with_panic(
            self.canister_id,
            self.gateway_principal,
            "ws_close",
            (CanisterWsCloseArguments::new(self.client_key.clone()),),
        );

        assert!(res.is_ok());
    }

    pub fn expect_received_http_requests_count(&mut self, count: usize) {
        let messages = self.get_http_over_ws_messages();

        assert_eq!(messages.len(), count);

        for msg in messages {
            assert!(matches!(msg, HttpOverWsMessage::HttpRequest(..)));
        }
    }
}

fn generate_random_client_key() -> ClientKey {
    ClientKey::new(generate_random_principal(), generate_random_nonce())
}
