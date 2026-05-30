use alloc::string::{String, ToString};

use core::str;

use nojson::{JsonParseError, JsonValueKind, RawJson, RawJsonValue};
use petkit_types::PetkitError;

use crate::ResponseParts;

pub fn parse_api_response<T>(response: &ResponseParts) -> Result<T, PetkitError>
where
    T: for<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>, Error = JsonParseError>,
{
    if !(200..=299).contains(&response.status) {
        return Err(PetkitError::HttpStatus {
            status: response.status,
        });
    }

    let body =
        str::from_utf8(&response.body).map_err(|error| PetkitError::Decode(error.to_string()))?;
    let raw = RawJson::parse(body).map_err(|error| PetkitError::Decode(error.to_string()))?;
    let envelope = raw.value();

    if let Some(error_member) = envelope.to_member("error")?.optional() {
        if error_member.kind() != JsonValueKind::Null {
            let code = error_member
                .to_member("code")
                .and_then(nojson::RawJsonMember::required)
                .and_then(i32::try_from)
                .map_err(|_| {
                    PetkitError::InvalidResponse("malformed error envelope: bad `code`")
                })?;
            let msg = error_member
                .to_member("msg")
                .and_then(nojson::RawJsonMember::required)
                .and_then(String::try_from)
                .map_err(|_| PetkitError::InvalidResponse("malformed error envelope: bad `msg`"))?;
            return Err(PetkitError::api(code, msg));
        }
    }

    let result = envelope
        .to_member("result")?
        .required()
        .map_err(|_| PetkitError::InvalidResponse("missing `result` field"))?;

    T::try_from(result).map_err(|error| PetkitError::Decode(error.to_string()))
}

pub fn parse_text_response(response: ResponseParts) -> Result<String, PetkitError> {
    if !(200..=299).contains(&response.status) {
        return Err(PetkitError::HttpStatus {
            status: response.status,
        });
    }

    String::from_utf8(response.body).map_err(|error| PetkitError::Decode(error.to_string()))
}
