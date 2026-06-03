#![allow(clippy::print_stdout)]

use std::convert::Infallible;

use petkit_client::{FountainBleClient, FountainBleSettings};
use petkit_protocol::BleGattWriter;
use petkit_types::FountainDeviceType;

#[derive(Default)]
struct HostBleWriter {
    frames: Vec<Vec<u8>>,
}

impl BleGattWriter for HostBleWriter {
    type Error = Infallible;

    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        // Replace this with a platform GATT write, e.g. write-with-response to
        // the fountain characteristic selected by the embedding application.
        self.frames.push(frame.to_vec());
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fountain = FountainBleClient::new(FountainDeviceType::W5);
    let mut writer = HostBleWriter::default();
    let settings = FountainBleSettings::new(5, 40, true, 2, 300, 600, false, 1320, 360)?;

    let power_on = fountain.power(&mut writer, true, 1)?;
    let pause = fountain.pause(&mut writer, 2)?;
    let resume = fountain.resume(&mut writer, 3)?;
    let reset_filter = fountain.reset_filter(&mut writer, 4)?;
    let light_high = fountain.light_high(&mut writer, 5, &settings)?;

    println!(
        "wrote {} fountain frames: {}, {}, {}, {}, {}",
        writer.frames.len(),
        power_on.cmd,
        pause.cmd,
        resume.cmd,
        reset_filter.cmd,
        light_high.cmd
    );
    Ok(())
}
