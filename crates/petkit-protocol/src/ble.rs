use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use percent_encoding::{percent_encode, AsciiSet, CONTROLS};
use petkit_types::{FountainAction, PetkitError};

pub const BLE_START_FRAME: [u8; 3] = [0xFA, 0xFC, 0xFD];
pub const BLE_END_FRAME: [u8; 1] = [0xFB];

const BLE_DATA_ENCODE_SET: &AsciiSet = &CONTROLS.add(b'+').add(b'/').add(b'=');

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BleEncodedCommand {
    pub cmd: u8,
    pub data: String,
}

pub fn build_ble_frame(command: &[u8], counter: u8) -> Vec<u8> {
    let mut frame =
        Vec::with_capacity(BLE_START_FRAME.len() + command.len() + BLE_END_FRAME.len() + 1);
    frame.extend_from_slice(&BLE_START_FRAME);
    if command.len() >= 2 {
        frame.extend_from_slice(&command[..2]);
        frame.push(counter);
        frame.extend_from_slice(&command[2..]);
    } else {
        frame.extend_from_slice(command);
        frame.push(counter);
    }
    frame.extend_from_slice(&BLE_END_FRAME);
    frame
}

pub fn encode_ble_data(frame: &[u8]) -> String {
    let encoded = STANDARD.encode(frame);
    percent_encode(encoded.as_bytes(), BLE_DATA_ENCODE_SET).to_string()
}

pub fn build_fountain_ble_command(
    action: FountainAction,
    counter: u8,
) -> Result<BleEncodedCommand, PetkitError> {
    let command = match action {
        FountainAction::Pause => &[220, 1, 3, 0, 1, 0, 2][..],
        FountainAction::Continue => &[220, 1, 3, 0, 1, 1, 2][..],
        FountainAction::ResetFilter => &[222, 1, 0, 0][..],
        FountainAction::PowerOff => &[220, 1, 3, 0, 0, 1, 1][..],
        FountainAction::PowerOn | FountainAction::ModeNormal | FountainAction::ModeStandard => {
            &[220, 1, 3, 0, 1, 1, 1][..]
        }
        FountainAction::ModeSmart | FountainAction::ModeIntermittent => {
            &[220, 1, 3, 0, 1, 2, 1][..]
        }
        unsupported => {
            return Err(PetkitError::InvalidArgument(format!(
                "unsupported fountain BLE action `{unsupported}`"
            )));
        }
    };

    let frame = build_ble_frame(command, counter);
    Ok(BleEncodedCommand {
        cmd: command[0],
        data: encode_ble_data(&frame),
    })
}
