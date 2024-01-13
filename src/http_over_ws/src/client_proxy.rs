use crate::{
    http_connection::{HttpConnection, HttpRequestId},
    HttpOverWsError,
};
use std::collections::BTreeMap;

pub(crate) struct ClientProxy {
    connections: BTreeMap<HttpRequestId, HttpConnection>,
}

impl ClientProxy {
    pub(crate) fn new() -> Self {
        ClientProxy {
            connections: BTreeMap::new(),
        }
    }

    pub(crate) fn assign_connection(
        &mut self,
        request_id: HttpRequestId,
        connection: HttpConnection,
    ) {
        self.connections.insert(request_id, connection);
    }

    pub(crate) fn get_connection_mut(
        &mut self,
        request_id: HttpRequestId,
    ) -> Result<&mut HttpConnection, HttpOverWsError> {
        self.connections
            .get_mut(&request_id)
            .ok_or(HttpOverWsError::RequestIdNotFound)
    }

    pub(crate) fn get_connections(&self) -> &BTreeMap<HttpRequestId, HttpConnection> {
        &self.connections
    }

    pub(crate) fn remove_connection(
        &mut self,
        request_id: HttpRequestId,
    ) -> Result<HttpConnection, HttpOverWsError> {
        self.connections
            .remove(&request_id)
            .ok_or(HttpOverWsError::ConnectionNotAssignedToProxy)
    }
}
