use http_over_ws::HttpHeader;
use lazy_static::lazy_static;

pub const TEST_URL: &str = "https://example.com/";

lazy_static! {
    pub static ref TEST_HTTP_REQUEST_HEADER: HttpHeader = HttpHeader {
        name: String::from("Accept"),
        value: String::from("text/plain"),
    };
    pub static ref TEST_HTTP_RESPONSE_HEADER: HttpHeader = HttpHeader {
        name: String::from("Content-Type"),
        value: String::from("text/plain"),
    };
}
