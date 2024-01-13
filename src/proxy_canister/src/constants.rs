use http_over_ws::HttpRequestTimeoutMs;

/// The minimum timeout a request can take before timing out.
pub const MIN_HTTP_REQUEST_TIMEOUT_MS: HttpRequestTimeoutMs = 5_000;

/// The maximum amount of time a request can take before timing out.
pub const MAX_HTTP_REQUEST_TIMEOUT_MS: HttpRequestTimeoutMs = 60_000;

/// The maximum amount of headers a request can have.
pub const MAX_HTTP_HEADERS_COUNT: usize = 50;
