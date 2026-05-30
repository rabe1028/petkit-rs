#![no_main]

use libfuzzer_sys::fuzz_target;
use petkit_protocol::BaseUrl;

fuzz_target!(|data: &[u8]| {
    let split_at = data.iter().position(|byte| *byte == 0).unwrap_or(data.len());
    let (base_bytes, path_bytes) = data.split_at(split_at);
    let path_bytes = path_bytes.get(1..).unwrap_or_default();
    let base_text = String::from_utf8_lossy(base_bytes);
    let path_text = String::from_utf8_lossy(path_bytes);

    let base_url = match data.first().copied().unwrap_or_default() % 4 {
        0 => BaseUrl::Passport,
        1 => BaseUrl::China,
        2 => BaseUrl::Regional(base_text.as_ref().into()),
        _ => BaseUrl::Absolute(base_text.as_ref().into()),
    };

    let _ = base_url.join(path_text.as_ref());
});
