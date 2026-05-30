#![no_main]

use libfuzzer_sys::fuzz_target;
use petkit_protocol::{build_ble_frame, build_fountain_ble_command, encode_ble_data};
use petkit_types::FountainAction;

fuzz_target!(|data: &[u8]| {
    let counter = data.first().copied().unwrap_or_default();
    let command = data.get(1..).unwrap_or_default();
    let frame = build_ble_frame(command, counter);
    let _ = encode_ble_data(&frame);

    let action = match counter % 6 {
        0 => FountainAction::Pause,
        1 => FountainAction::Continue,
        2 => FountainAction::ResetFilter,
        3 => FountainAction::PowerOff,
        4 => FountainAction::PowerOn,
        _ => FountainAction::ModeSmart,
    };
    let _ = build_fountain_ble_command(action, counter);
});
