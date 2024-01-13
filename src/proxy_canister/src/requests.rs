use proxy_canister_types::{HttpRequestEndpointArgs, InvalidRequest};
use url::Url;

use crate::constants::{
    MAX_HTTP_HEADERS_COUNT, MAX_HTTP_REQUEST_TIMEOUT_MS, MIN_HTTP_REQUEST_TIMEOUT_MS,
};

pub fn validate_incoming_request(args: &HttpRequestEndpointArgs) -> Result<(), InvalidRequest> {
    Url::parse(&args.request.url).map_err(|e| InvalidRequest::InvalidUrl(e.to_string()))?;

    if args.request.headers.len() > MAX_HTTP_HEADERS_COUNT {
        return Err(InvalidRequest::TooManyHeaders);
    }

    if args.timeout_ms.is_some_and(|timeout_ms| {
        timeout_ms > MAX_HTTP_REQUEST_TIMEOUT_MS || timeout_ms < MIN_HTTP_REQUEST_TIMEOUT_MS
    }) {
        return Err(InvalidRequest::InvalidTimeout);
    }

    Ok(())
}
