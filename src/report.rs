use anyhow::{Context, Result};
use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY};
use packed_struct::prelude::*;

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0", endian = "msb")]
pub struct CompositeJoystickReport {
    #[packed_field(bits = "0..16")]
    pub x: i16,
    #[packed_field(bits = "16..32")]
    pub y: i16,
    #[packed_field(bits = "32..48")]
    pub z: i16,
    #[packed_field(bits = "48..64")]
    pub rx: i16,
    #[packed_field(bits = "64..80")]
    pub ry: i16,
    #[packed_field(bits = "80..96")]
    pub rz: i16,
    #[packed_field(bits = "96..112")]
    pub slider: i16,
    #[packed_field(bits = "112..128")]
    pub dial: i16,
    #[packed_field(bits = "128..152", element_size_bits = "1")]
    pub buttons: [bool; 24],
}

pub fn make_report(state: impl Iterator<Item = (EventCode, i64)>) -> [u8; 19] {
    let mut result = CompositeJoystickReport {
        x: 0,
        y: 0,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        slider: 0,
        dial: 0,
        buttons: [false; 24],
    };
    for (code, value) in state.into_iter() {
        match code {
            EventCode::EV_ABS(EV_ABS::ABS_X) => result.x = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_Y) => result.y = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_Z) => result.z = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RX) => result.rx = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RY) => result.ry = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RZ) => result.rz = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_THROTTLE) => result.slider = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RUDDER) => result.dial = value as i16,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER) => result.buttons[0] = value != 0,
            _ => {}
        }
    }
    result.pack().expect("failed to pack report into bits")
}
