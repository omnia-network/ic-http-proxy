import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type CanisterCallbackMethodName = string;
export type CanisterId = Principal;
export interface CanisterOutputCertifiedMessages {
  'messages' : Array<CanisterOutputMessage>,
  'cert' : Uint8Array | number[],
  'tree' : Uint8Array | number[],
  'is_end_of_queue' : boolean,
}
export interface CanisterOutputMessage {
  'key' : string,
  'content' : Uint8Array | number[],
  'client_key' : ClientKey,
}
export interface CanisterRequest {
  'canister_id' : CanisterId,
  'state' : RequestState,
}
export interface CanisterWsCloseArguments { 'client_key' : ClientKey }
export type CanisterWsCloseResult = { 'Ok' : null } |
  { 'Err' : string };
export interface CanisterWsGetMessagesArguments { 'nonce' : bigint }
export type CanisterWsGetMessagesResult = {
    'Ok' : CanisterOutputCertifiedMessages
  } |
  { 'Err' : string };
export interface CanisterWsMessageArguments { 'msg' : WebsocketMessage }
export type CanisterWsMessageResult = { 'Ok' : null } |
  { 'Err' : string };
export interface CanisterWsOpenArguments {
  'gateway_principal' : GatewayPrincipal,
  'client_nonce' : bigint,
}
export type CanisterWsOpenResult = { 'Ok' : null } |
  { 'Err' : string };
export interface ClientKey {
  'client_principal' : ClientPrincipal,
  'client_nonce' : bigint,
}
export type ClientPrincipal = Principal;
export type GatewayPrincipal = Principal;
export type HttpFailureReason = { 'ProxyError' : string } |
  { 'RequestTimeout' : null };
export interface HttpHeader { 'value' : string, 'name' : string }
export type HttpMethod = { 'GET' : null } |
  { 'PUT' : null } |
  { 'DELETE' : null } |
  { 'HEAD' : null } |
  { 'POST' : null };
export type HttpOverWsError = { 'NotHttpOverWsType' : string } |
  { 'ProxyNotFound' : null } |
  { 'NotYetReceived' : null } |
  { 'ConnectionNotAssignedToProxy' : null } |
  { 'RequestIdNotFound' : null } |
  { 'NoProxiesConnected' : null } |
  { 'InvalidHttpMessage' : null } |
  { 'RequestFailed' : HttpFailureReason };
export type HttpOverWsMessage = { 'Error' : [[] | [HttpRequestId], string] } |
  { 'HttpRequest' : [HttpRequestId, HttpRequest] } |
  { 'SetupProxyClient' : null } |
  { 'HttpResponse' : [HttpRequestId, HttpResponse] };
export interface HttpRequest {
  'url' : string,
  'method' : HttpMethod,
  'body' : [] | [Uint8Array | number[]],
  'headers' : Array<HttpHeader>,
}
export interface HttpRequestEndpointArgs {
  'request' : HttpRequest,
  'timeout_ms' : [] | [HttpRequestTimeoutMs],
  'callback_method_name' : [] | [CanisterCallbackMethodName],
}
export type HttpRequestEndpointResult = { 'Ok' : HttpRequestId } |
  { 'Err' : ProxyCanisterError };
export type HttpRequestId = bigint;
export type HttpRequestTimeoutMs = bigint;
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export type InvalidRequest = { 'TooManyHeaders' : null } |
  { 'InvalidTimeout' : null } |
  { 'InvalidUrl' : string };
export type ProxyCanisterError = { 'HttpOverWs' : HttpOverWsError } |
  { 'InvalidRequest' : InvalidRequest };
export type RequestState = { 'Executing' : [] | [CanisterCallbackMethodName] } |
  { 'Executed' : null } |
  { 'CallbackFailed' : string };
export interface WebsocketMessage {
  'sequence_num' : bigint,
  'content' : Uint8Array | number[],
  'client_key' : ClientKey,
  'timestamp' : bigint,
  'is_service_message' : boolean,
}
export interface _SERVICE {
  'disconnect_all_proxies' : ActorMethod<[], undefined>,
  'get_logs' : ActorMethod<[], Array<[string, string]>>,
  'get_request_by_id' : ActorMethod<[HttpRequestId], [] | [CanisterRequest]>,
  'http_request' : ActorMethod<
    [HttpRequestEndpointArgs],
    HttpRequestEndpointResult
  >,
  'ws_close' : ActorMethod<[CanisterWsCloseArguments], CanisterWsCloseResult>,
  'ws_get_messages' : ActorMethod<
    [CanisterWsGetMessagesArguments],
    CanisterWsGetMessagesResult
  >,
  'ws_message' : ActorMethod<
    [CanisterWsMessageArguments, [] | [HttpOverWsMessage]],
    CanisterWsMessageResult
  >,
  'ws_open' : ActorMethod<[CanisterWsOpenArguments], CanisterWsOpenResult>,
}
