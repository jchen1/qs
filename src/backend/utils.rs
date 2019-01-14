use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub fn urlencode(to_encode: &str) -> String {
    utf8_percent_encode(to_encode, DEFAULT_ENCODE_SET).to_string()
}
