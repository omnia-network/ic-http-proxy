export const idlFactory = ({ IDL }) => {
  const HttpRequestId = IDL.Nat64;
  const CanisterId = IDL.Principal;
  const CanisterCallbackMethodName = IDL.Text;
  const RequestState = IDL.Variant({
    'Executing' : IDL.Opt(CanisterCallbackMethodName),
    'Executed' : IDL.Null,
    'CallbackFailed' : IDL.Text,
  });
  const CanisterRequest = IDL.Record({
    'canister_id' : CanisterId,
    'state' : RequestState,
  });
  const HttpMethod = IDL.Variant({
    'GET' : IDL.Null,
    'PUT' : IDL.Null,
    'DELETE' : IDL.Null,
    'HEAD' : IDL.Null,
    'POST' : IDL.Null,
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : HttpMethod,
    'body' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'headers' : IDL.Vec(HttpHeader),
  });
  const HttpRequestTimeoutMs = IDL.Nat64;
  const HttpRequestEndpointArgs = IDL.Record({
    'request' : HttpRequest,
    'timeout_ms' : IDL.Opt(HttpRequestTimeoutMs),
    'callback_method_name' : IDL.Opt(CanisterCallbackMethodName),
  });
  const HttpFailureReason = IDL.Variant({
    'ProxyError' : IDL.Text,
    'RequestTimeout' : IDL.Null,
  });
  const HttpOverWsError = IDL.Variant({
    'NotHttpOverWsType' : IDL.Text,
    'ProxyNotFound' : IDL.Null,
    'NotYetReceived' : IDL.Null,
    'ConnectionNotAssignedToProxy' : IDL.Null,
    'RequestIdNotFound' : IDL.Null,
    'NoProxiesConnected' : IDL.Null,
    'InvalidHttpMessage' : IDL.Null,
    'RequestFailed' : HttpFailureReason,
  });
  const InvalidRequest = IDL.Variant({
    'TooManyHeaders' : IDL.Null,
    'InvalidTimeout' : IDL.Null,
    'InvalidUrl' : IDL.Text,
  });
  const ProxyCanisterError = IDL.Variant({
    'HttpOverWs' : HttpOverWsError,
    'InvalidRequest' : InvalidRequest,
  });
  const HttpRequestEndpointResult = IDL.Variant({
    'Ok' : HttpRequestId,
    'Err' : ProxyCanisterError,
  });
  const ClientPrincipal = IDL.Principal;
  const ClientKey = IDL.Record({
    'client_principal' : ClientPrincipal,
    'client_nonce' : IDL.Nat64,
  });
  const CanisterWsCloseArguments = IDL.Record({ 'client_key' : ClientKey });
  const CanisterWsCloseResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  const CanisterWsGetMessagesArguments = IDL.Record({ 'nonce' : IDL.Nat64 });
  const CanisterOutputMessage = IDL.Record({
    'key' : IDL.Text,
    'content' : IDL.Vec(IDL.Nat8),
    'client_key' : ClientKey,
  });
  const CanisterOutputCertifiedMessages = IDL.Record({
    'messages' : IDL.Vec(CanisterOutputMessage),
    'cert' : IDL.Vec(IDL.Nat8),
    'tree' : IDL.Vec(IDL.Nat8),
    'is_end_of_queue' : IDL.Bool,
  });
  const CanisterWsGetMessagesResult = IDL.Variant({
    'Ok' : CanisterOutputCertifiedMessages,
    'Err' : IDL.Text,
  });
  const WebsocketMessage = IDL.Record({
    'sequence_num' : IDL.Nat64,
    'content' : IDL.Vec(IDL.Nat8),
    'client_key' : ClientKey,
    'timestamp' : IDL.Nat64,
    'is_service_message' : IDL.Bool,
  });
  const CanisterWsMessageArguments = IDL.Record({ 'msg' : WebsocketMessage });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const HttpOverWsMessage = IDL.Variant({
    'Error' : IDL.Tuple(IDL.Opt(HttpRequestId), IDL.Text),
    'HttpRequest' : IDL.Tuple(HttpRequestId, HttpRequest),
    'SetupProxyClient' : IDL.Null,
    'HttpResponse' : IDL.Tuple(HttpRequestId, HttpResponse),
  });
  const CanisterWsMessageResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  const GatewayPrincipal = IDL.Principal;
  const CanisterWsOpenArguments = IDL.Record({
    'gateway_principal' : GatewayPrincipal,
    'client_nonce' : IDL.Nat64,
  });
  const CanisterWsOpenResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  return IDL.Service({
    'get_logs' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        ['query'],
      ),
    'get_request_by_id' : IDL.Func(
        [HttpRequestId],
        [IDL.Opt(CanisterRequest)],
        ['query'],
      ),
    'http_request' : IDL.Func(
        [HttpRequestEndpointArgs],
        [HttpRequestEndpointResult],
        [],
      ),
    'ws_close' : IDL.Func(
        [CanisterWsCloseArguments],
        [CanisterWsCloseResult],
        [],
      ),
    'ws_get_messages' : IDL.Func(
        [CanisterWsGetMessagesArguments],
        [CanisterWsGetMessagesResult],
        ['query'],
      ),
    'ws_message' : IDL.Func(
        [CanisterWsMessageArguments, IDL.Opt(HttpOverWsMessage)],
        [CanisterWsMessageResult],
        [],
      ),
    'ws_open' : IDL.Func([CanisterWsOpenArguments], [CanisterWsOpenResult], []),
  });
};
export const init = ({ IDL }) => { return []; };
