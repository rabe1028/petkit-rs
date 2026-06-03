use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use nojson::{JsonValueKind, RawJsonValue};
use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use petkit_types::{DeviceDetailResponse, FountainAction, PetkitError};

pub const BLE_START_FRAME: [u8; 3] = [0xFA, 0xFC, 0xFD];
pub const BLE_END_FRAME: [u8; 1] = [0xFB];

const BLE_DATA_ENCODE_SET: &AsciiSet = &CONTROLS.add(b'+').add(b'/').add(b'=');

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BleEncodedCommand {
    pub cmd: u8,
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BleFrameCommand {
    pub cmd: u8,
    pub frame: Vec<u8>,
    pub data: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FountainBleSettings {
    pub smart_working_time: u8,
    pub smart_sleep_time: u8,
    pub lamp_ring_switch: bool,
    pub lamp_ring_brightness: u8,
    pub lamp_ring_light_up_time: u16,
    pub lamp_ring_go_out_time: u16,
    pub no_disturbing_switch: bool,
    pub no_disturbing_start_time: u16,
    pub no_disturbing_end_time: u16,
}

impl FountainBleSettings {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        smart_working_time: u8,
        smart_sleep_time: u8,
        lamp_ring_switch: bool,
        lamp_ring_brightness: u8,
        lamp_ring_light_up_time: u16,
        lamp_ring_go_out_time: u16,
        no_disturbing_switch: bool,
        no_disturbing_start_time: u16,
        no_disturbing_end_time: u16,
    ) -> Result<Self, PetkitError> {
        validate_max("lamp_ring_brightness", lamp_ring_brightness.into(), 3)?;
        validate_max("lamp_ring_light_up_time", lamp_ring_light_up_time, 1439)?;
        validate_max("lamp_ring_go_out_time", lamp_ring_go_out_time, 1439)?;
        validate_max("no_disturbing_start_time", no_disturbing_start_time, 1439)?;
        validate_max("no_disturbing_end_time", no_disturbing_end_time, 1439)?;

        Ok(Self {
            smart_working_time,
            smart_sleep_time,
            lamp_ring_switch,
            lamp_ring_brightness,
            lamp_ring_light_up_time,
            lamp_ring_go_out_time,
            no_disturbing_switch,
            no_disturbing_start_time,
            no_disturbing_end_time,
        })
    }

    pub fn from_device_detail(detail: &DeviceDetailResponse) -> Result<Self, PetkitError> {
        let settings = detail
            .settings
            .as_ref()
            .ok_or(PetkitError::InvalidResponse("missing fountain settings"))?
            .value();
        Self::new(
            required_u8(settings, "smartWorkingTime")?,
            required_u8(settings, "smartSleepTime")?,
            required_boolish(settings, "lampRingSwitch")?,
            required_u8(settings, "lampRingBrightness")?,
            required_u16(settings, "lampRingLightUpTime")?,
            required_u16(settings, "lampRingGoOutTime")?,
            required_boolish(settings, "noDisturbingSwitch")?,
            required_u16(settings, "noDisturbingStartTime")?,
            required_u16(settings, "noDisturbingEndTime")?,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FountainBleClient {
    device_type: petkit_types::FountainDeviceType,
}

impl FountainBleClient {
    pub const fn new(device_type: petkit_types::FountainDeviceType) -> Self {
        Self { device_type }
    }

    pub const fn device_type(self) -> petkit_types::FountainDeviceType {
        self.device_type
    }

    pub fn command(
        self,
        action: FountainAction,
        counter: u8,
    ) -> Result<BleFrameCommand, PetkitError> {
        build_fountain_ble_frame_command(action, counter)
    }

    pub fn command_with_settings(
        self,
        action: FountainAction,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, PetkitError> {
        build_fountain_ble_frame_command_with_settings(action, counter, settings)
    }

    pub fn execute<W>(
        self,
        writer: &mut W,
        action: FountainAction,
        counter: u8,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        write_fountain_ble_frame(writer, action, counter)
    }

    pub fn execute_with_settings<W>(
        self,
        writer: &mut W,
        action: FountainAction,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        write_fountain_ble_frame_with_settings(writer, action, counter, settings)
    }

    pub fn power<W>(
        self,
        writer: &mut W,
        on: bool,
        counter: u8,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        let action = if on {
            FountainAction::PowerOn
        } else {
            FountainAction::PowerOff
        };
        self.execute(writer, action, counter)
    }

    pub fn pause<W>(
        self,
        writer: &mut W,
        counter: u8,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute(writer, FountainAction::Pause, counter)
    }

    pub fn resume<W>(
        self,
        writer: &mut W,
        counter: u8,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute(writer, FountainAction::Continue, counter)
    }

    pub fn reset_filter<W>(
        self,
        writer: &mut W,
        counter: u8,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute(writer, FountainAction::ResetFilter, counter)
    }

    pub fn do_not_disturb<W>(
        self,
        writer: &mut W,
        enabled: bool,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        let action = if enabled {
            FountainAction::DoNotDisturb
        } else {
            FountainAction::DoNotDisturbOff
        };
        self.execute_with_settings(writer, action, counter, settings)
    }

    pub fn light<W>(
        self,
        writer: &mut W,
        enabled: bool,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        let action = if enabled {
            FountainAction::LightOn
        } else {
            FountainAction::LightOff
        };
        self.execute_with_settings(writer, action, counter, settings)
    }

    pub fn light_low<W>(
        self,
        writer: &mut W,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute_with_settings(writer, FountainAction::LightLow, counter, settings)
    }

    pub fn light_medium<W>(
        self,
        writer: &mut W,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute_with_settings(writer, FountainAction::LightMedium, counter, settings)
    }

    pub fn light_high<W>(
        self,
        writer: &mut W,
        counter: u8,
        settings: &FountainBleSettings,
    ) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
    where
        W: BleGattWriter,
    {
        self.execute_with_settings(writer, FountainAction::LightHigh, counter, settings)
    }
}

pub trait BleGattWriter {
    type Error;

    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BleGattWriteError<E> {
    #[error("failed to build BLE frame: {0}")]
    Build(PetkitError),
    #[error("failed to write BLE frame: {0}")]
    Transport(E),
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
    let command = build_fountain_ble_frame_command(action, counter)?;
    Ok(BleEncodedCommand {
        cmd: command.cmd,
        data: command.data,
    })
}

pub fn build_fountain_ble_frame_command(
    action: FountainAction,
    counter: u8,
) -> Result<BleFrameCommand, PetkitError> {
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
        FountainAction::DoNotDisturb
        | FountainAction::DoNotDisturbOff
        | FountainAction::LightLow
        | FountainAction::LightMedium
        | FountainAction::LightHigh
        | FountainAction::LightOn
        | FountainAction::LightOff => {
            return Err(PetkitError::InvalidArgument(format!(
                "fountain BLE action `{action}` requires settings"
            )));
        }
    };

    Ok(build_fountain_ble_command_from_payload(command, counter))
}

pub fn build_fountain_ble_frame_command_with_settings(
    action: FountainAction,
    counter: u8,
    settings: &FountainBleSettings,
) -> Result<BleFrameCommand, PetkitError> {
    match action {
        FountainAction::DoNotDisturb
        | FountainAction::DoNotDisturbOff
        | FountainAction::LightLow
        | FountainAction::LightMedium
        | FountainAction::LightHigh
        | FountainAction::LightOn
        | FountainAction::LightOff => {
            let data = fountain_settings_payload(action, settings);
            let payload = build_fountain_settings_payload(&data);
            Ok(build_fountain_ble_command_from_payload(&payload, counter))
        }
        _ => build_fountain_ble_frame_command(action, counter),
    }
}

pub fn write_fountain_ble_frame<W>(
    writer: &mut W,
    action: FountainAction,
    counter: u8,
) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
where
    W: BleGattWriter,
{
    let command =
        build_fountain_ble_frame_command(action, counter).map_err(BleGattWriteError::Build)?;
    writer
        .write_frame(&command.frame)
        .map_err(BleGattWriteError::Transport)?;
    Ok(command)
}

pub fn write_fountain_ble_frame_with_settings<W>(
    writer: &mut W,
    action: FountainAction,
    counter: u8,
    settings: &FountainBleSettings,
) -> Result<BleFrameCommand, BleGattWriteError<W::Error>>
where
    W: BleGattWriter,
{
    let command = build_fountain_ble_frame_command_with_settings(action, counter, settings)
        .map_err(BleGattWriteError::Build)?;
    writer
        .write_frame(&command.frame)
        .map_err(BleGattWriteError::Transport)?;
    Ok(command)
}

fn build_fountain_ble_command_from_payload(command: &[u8], counter: u8) -> BleFrameCommand {
    let frame = build_ble_frame(command, counter);
    let data = encode_ble_data(&frame);
    BleFrameCommand {
        cmd: command[0],
        frame,
        data,
    }
}

fn build_fountain_settings_payload(data: &[u8]) -> Vec<u8> {
    let len = data.len() as u16;
    let mut payload = Vec::with_capacity(4 + data.len());
    payload.extend_from_slice(&[221, 1]);
    payload.extend_from_slice(&len.to_le_bytes());
    payload.extend_from_slice(data);
    payload
}

fn fountain_settings_payload(action: FountainAction, settings: &FountainBleSettings) -> Vec<u8> {
    let mut data = Vec::with_capacity(13);
    data.push(settings.smart_working_time);
    data.push(settings.smart_sleep_time);
    match action {
        FountainAction::LightOff => {
            data.push(0);
            data.push(settings.lamp_ring_brightness);
        }
        FountainAction::LightOn => {
            data.push(1);
            data.push(settings.lamp_ring_brightness);
        }
        FountainAction::LightLow => {
            data.push(u8::from(settings.lamp_ring_switch));
            data.push(1);
        }
        FountainAction::LightMedium => {
            data.push(u8::from(settings.lamp_ring_switch));
            data.push(2);
        }
        FountainAction::LightHigh => {
            data.push(u8::from(settings.lamp_ring_switch));
            data.push(3);
        }
        FountainAction::DoNotDisturb => {
            data.push(u8::from(settings.lamp_ring_switch));
            data.push(settings.lamp_ring_brightness);
        }
        FountainAction::DoNotDisturbOff => {
            data.push(u8::from(settings.lamp_ring_switch));
            data.push(settings.lamp_ring_brightness);
        }
        _ => {}
    }
    data.extend_from_slice(&settings.lamp_ring_light_up_time.to_be_bytes());
    data.extend_from_slice(&settings.lamp_ring_go_out_time.to_be_bytes());
    match action {
        FountainAction::DoNotDisturb => data.push(1),
        FountainAction::DoNotDisturbOff => data.push(0),
        _ => data.push(u8::from(settings.no_disturbing_switch)),
    }
    data.extend_from_slice(&settings.no_disturbing_start_time.to_be_bytes());
    data.extend_from_slice(&settings.no_disturbing_end_time.to_be_bytes());
    data
}

fn required_u8(value: RawJsonValue<'_, '_>, key: &'static str) -> Result<u8, PetkitError> {
    let value = required_member(value, key)?;
    let parsed = u32_value(value, key)?;
    u8::try_from(parsed)
        .map_err(|_| PetkitError::InvalidArgument(format!("fountain setting `{key}` is too large")))
}

fn required_u16(value: RawJsonValue<'_, '_>, key: &'static str) -> Result<u16, PetkitError> {
    let value = required_member(value, key)?;
    let parsed = u32_value(value, key)?;
    u16::try_from(parsed)
        .map_err(|_| PetkitError::InvalidArgument(format!("fountain setting `{key}` is too large")))
}

fn required_boolish(value: RawJsonValue<'_, '_>, key: &'static str) -> Result<bool, PetkitError> {
    let value = required_member(value, key)?;
    match value.kind() {
        JsonValueKind::Boolean => bool::try_from(value).map_err(PetkitError::from),
        JsonValueKind::String => {
            let raw = String::try_from(value).map_err(PetkitError::from)?;
            match raw.as_str() {
                "1" | "true" | "on" => Ok(true),
                "0" | "false" | "off" => Ok(false),
                _ => Err(PetkitError::InvalidArgument(format!(
                    "fountain setting `{key}` is not boolean-like"
                ))),
            }
        }
        JsonValueKind::Integer | JsonValueKind::Float => {
            Ok(u32::try_from(value).map_err(PetkitError::from)? != 0)
        }
        JsonValueKind::Null | JsonValueKind::Array | JsonValueKind::Object => Err(
            PetkitError::InvalidArgument(format!("fountain setting `{key}` is not boolean-like")),
        ),
    }
}

fn required_member<'text, 'raw>(
    value: RawJsonValue<'text, 'raw>,
    key: &'static str,
) -> Result<RawJsonValue<'text, 'raw>, PetkitError> {
    value
        .to_member(key)?
        .required()
        .map_err(|_| PetkitError::InvalidResponse("missing fountain setting"))
}

fn u32_value(value: RawJsonValue<'_, '_>, key: &'static str) -> Result<u32, PetkitError> {
    match value.kind() {
        JsonValueKind::String => {
            let raw = String::try_from(value).map_err(PetkitError::from)?;
            raw.parse::<u32>().map_err(|error| {
                PetkitError::InvalidArgument(format!(
                    "fountain setting `{key}` is not an integer: {error}"
                ))
            })
        }
        JsonValueKind::Integer | JsonValueKind::Float => {
            u32::try_from(value).map_err(PetkitError::from)
        }
        JsonValueKind::Null
        | JsonValueKind::Boolean
        | JsonValueKind::Array
        | JsonValueKind::Object => Err(PetkitError::InvalidArgument(format!(
            "fountain setting `{key}` is not an integer"
        ))),
    }
}

fn validate_max(key: &'static str, value: u16, max: u16) -> Result<(), PetkitError> {
    if value <= max {
        Ok(())
    } else {
        Err(PetkitError::InvalidArgument(format!(
            "fountain setting `{key}` must be <= {max}, got `{value}`"
        )))
    }
}
