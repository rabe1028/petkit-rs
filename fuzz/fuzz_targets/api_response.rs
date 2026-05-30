#![no_main]

use libfuzzer_sys::fuzz_target;
use nojson::{JsonParseError, RawJsonValue};
use petkit_protocol::{parse_api_response, parse_text_response, ResponseParts};

/// Discarding wrapper that satisfies the `TryFrom<RawJsonValue, …>` bound on
/// `parse_api_response` without restricting which JSON shapes we explore.
struct Any;

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Any {
    type Error = JsonParseError;

    fn try_from(_value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

fuzz_target!(|data: &[u8]| {
    let response = ResponseParts::new(200, vec![], data.to_vec());
    let _ = parse_api_response::<Any>(&response);
    let _ = parse_text_response(response);
});
