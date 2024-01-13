use crate::{
    client_proxy::ClientProxy,
    http_connection::{
        GetHttpResponseResult, HttpCallback, HttpConnection, HttpFailureReason, HttpRequest,
        HttpRequestId, HttpRequestTimeoutMs,
    },
    trigger_callback_with_result, HttpCallbackWithResult, HttpOverWsError, HttpResult,
};
use candid::Principal;
use std::{cell::RefCell, collections::HashMap, time::Duration};

// local state
thread_local! {
    /* flexible */ pub static STATE: RefCell<State> = RefCell::new(State::new());
}

pub(crate) struct State {
    connected_proxies: ConnectedProxies,
    next_request_id: HttpRequestId,
}

impl State {
    pub(crate) fn new() -> Self {
        State {
            connected_proxies: ConnectedProxies::new(),
            next_request_id: 0,
        }
    }

    pub(crate) fn add_proxy(&mut self, proxy_principal: Principal) {
        self.connected_proxies.add_proxy(proxy_principal);
    }

    pub(crate) fn remove_proxy(
        &mut self,
        proxy_principal: &Principal,
    ) -> Result<(), HttpOverWsError> {
        self.connected_proxies.remove_proxy(proxy_principal)
    }

    pub(crate) fn assign_connection(
        &mut self,
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timeout_ms: Option<HttpRequestTimeoutMs>,
    ) -> Result<(Principal, HttpRequestId), HttpOverWsError> {
        let request_id = self.next_request_id();

        let proxy_principal = self
            .get_proxy_for_connection(request_id)
            .ok_or(HttpOverWsError::NoProxiesConnected)?
            .clone();

        let timer_id = timeout_ms.and_then(|millis| {
            Some(ic_cdk_timers::set_timer(
                Duration::from_millis(millis),
                move || {
                    let callback_with_result = http_connection_timeout(proxy_principal, request_id);
                    trigger_callback_with_result(request_id, callback_with_result);
                },
            ))
        });

        let connection = HttpConnection::new(request_id, request, callback, timer_id);

        self.connected_proxies.assign_connection_to_proxy(
            &proxy_principal,
            request_id,
            connection,
        )?;
        Ok((proxy_principal, request_id))
    }

    fn next_request_id(&mut self) -> HttpRequestId {
        self.next_request_id += 1;
        self.next_request_id
    }

    fn get_proxy_for_connection(&self, request_id: HttpRequestId) -> Option<Principal> {
        let connected_proxies_count = self.connected_proxies.0.len();
        if connected_proxies_count == 0 {
            return None;
        }
        let chosen_proxy_index = request_id as usize % connected_proxies_count;
        // chosen_proxy_index is in [0, connected_proxies_count)
        // where connected_proxies_count is the number of proxies currently connected.
        // as no proxy is removed while executing this method,
        // the entry at 'chosen_proxy_index' is guaranteed to exist
        Some(
            self.connected_proxies
                .0
                .iter()
                .nth(chosen_proxy_index)
                .expect("proxy is not connected")
                .0
                .clone(),
        )
    }

    pub(crate) fn update_connection_state(
        &mut self,
        proxy_principal: Principal,
        request_id: HttpRequestId,
        http_result: HttpResult,
    ) -> Result<Option<HttpCallbackWithResult>, HttpOverWsError> {
        let proxy = self
            .connected_proxies
            .0
            .get_mut(&proxy_principal)
            .ok_or(HttpOverWsError::ProxyNotFound)?;
        let connection = proxy.get_connection_mut(request_id)?;

        let callback_with_result = connection.update_state(http_result);

        Ok(callback_with_result)
    }

    pub(crate) fn get_http_connection(&self, request_id: HttpRequestId) -> Option<HttpRequest> {
        for (_, proxy) in self.connected_proxies.0.iter() {
            for (id, connection) in proxy.get_connections() {
                if id.to_owned() == request_id {
                    return Some(connection.get_request());
                }
            }
        }
        None
    }

    pub(crate) fn get_http_response(&self, request_id: HttpRequestId) -> GetHttpResponseResult {
        for (_, proxy) in self.connected_proxies.0.iter() {
            for (id, connection) in proxy.get_connections() {
                if id.to_owned() == request_id {
                    return connection.get_response();
                }
            }
        }
        Err(HttpOverWsError::RequestIdNotFound)
    }
}

fn http_connection_timeout(
    proxy_principal: Principal,
    request_id: HttpRequestId,
) -> Option<HttpCallbackWithResult> {
    STATE.with(|state| {
        state
            .borrow_mut()
            .connected_proxies
            .0
            .get_mut(&proxy_principal)
            .and_then(|proxy| proxy.get_connection_mut(request_id).ok())
            .and_then(|connection| {
                connection.update_state(HttpResult::Failure(HttpFailureReason::RequestTimeout))
            })
    })
}

pub(crate) struct ConnectedProxies(HashMap<Principal, ClientProxy>);

impl ConnectedProxies {
    fn new() -> Self {
        ConnectedProxies(HashMap::new())
    }

    fn add_proxy(&mut self, proxy_principal: Principal) {
        self.0.insert(proxy_principal, ClientProxy::new());
    }

    fn assign_connection_to_proxy(
        &mut self,
        proxy_principal: &Principal,
        request_id: HttpRequestId,
        connection: HttpConnection,
    ) -> Result<(), HttpOverWsError> {
        let proxy: &mut ClientProxy = self
            .0
            .get_mut(proxy_principal)
            .ok_or(HttpOverWsError::ProxyNotFound)?;
        proxy.assign_connection(request_id, connection);
        Ok(())
    }

    #[allow(dead_code)]
    /// Removes the connection from the proxy.
    //  Not used for now but will be needed when we want to remove old connections
    fn complete_connection_for_proxy(
        &mut self,
        proxy_principal: &Principal,
        request_id: HttpRequestId,
    ) -> Result<(), HttpOverWsError> {
        let proxy = self
            .0
            .get_mut(proxy_principal)
            .ok_or(HttpOverWsError::ProxyNotFound)?;
        proxy.remove_connection(request_id)?;
        Ok(())
    }

    fn remove_proxy(&mut self, proxy_principal: &Principal) -> Result<(), HttpOverWsError> {
        self.0
            .remove(proxy_principal)
            .ok_or(HttpOverWsError::ProxyNotFound)?;
        Ok(())
    }
}
