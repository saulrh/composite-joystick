use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY};
use packed_struct::prelude::*;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct CompositeJoystickReport {
    #[packed_field(bits = "0..16", endian = "lsb")]
    pub x: i16,
    #[packed_field(bits = "16..32", endian = "lsb")]
    pub y: i16,
    #[packed_field(bits = "32..48", endian = "lsb")]
    pub z: i16,
    #[packed_field(bits = "48..64", endian = "lsb")]
    pub rx: i16,
    #[packed_field(bits = "64..80", endian = "lsb")]
    pub ry: i16,
    #[packed_field(bits = "80..96", endian = "lsb")]
    pub rz: i16,
    #[packed_field(bits = "96..112", endian = "lsb")]
    pub slider: i16,
    #[packed_field(bits = "112..128", endian = "lsb")]
    pub dial: i16,
    #[packed_field(bits = "128..132", endian = "lsb")]
    pub hat: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "132..176", element_size_bits = "1")]
    pub buttons: [bool; 44],
}

pub fn make_report(state: impl Iterator<Item = (EventCode, i64)>) -> [u8; 22] {
    let mut result = CompositeJoystickReport {
        x: 0,
        y: 0,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        slider: 0,
        dial: 0,
        hat: (15).into(),
        buttons: [false; 44],
    };
    let mut hatx: i64 = 0;
    let mut haty: i64 = 0;
    for (code, value) in state {
        match code {
            EventCode::EV_ABS(EV_ABS::ABS_X) => result.x = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_Y) => result.y = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_Z) => result.z = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RX) => result.rx = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RY) => result.ry = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RZ) => result.rz = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_THROTTLE) => result.slider = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_RUDDER) => result.dial = value as i16,
            EventCode::EV_ABS(EV_ABS::ABS_HAT0X) => hatx = value.signum(),
            EventCode::EV_ABS(EV_ABS::ABS_HAT0Y) => haty = value.signum(),
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER) => result.buttons[0] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_THUMB) => result.buttons[1] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_THUMB2) => result.buttons[2] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TOP) => result.buttons[3] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TOP2) => result.buttons[4] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_PINKIE) => result.buttons[5] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE) => result.buttons[6] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE2) => result.buttons[7] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE3) => result.buttons[8] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE4) => result.buttons[9] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE5) => result.buttons[10] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_BASE6) => result.buttons[11] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY1) => result.buttons[12] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY2) => result.buttons[13] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY3) => result.buttons[14] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY4) => result.buttons[15] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY5) => result.buttons[16] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY6) => result.buttons[17] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY7) => result.buttons[18] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY8) => result.buttons[19] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY9) => result.buttons[20] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY10) => result.buttons[21] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY11) => result.buttons[22] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY12) => result.buttons[23] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY13) => result.buttons[24] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY14) => result.buttons[25] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY15) => result.buttons[26] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY16) => result.buttons[27] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY17) => result.buttons[28] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY18) => result.buttons[29] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY19) => result.buttons[30] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY20) => result.buttons[31] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY21) => result.buttons[32] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY22) => result.buttons[33] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY23) => result.buttons[34] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY24) => result.buttons[35] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY25) => result.buttons[36] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY26) => result.buttons[37] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY27) => result.buttons[38] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY28) => result.buttons[39] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY29) => result.buttons[40] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY30) => result.buttons[41] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY31) => result.buttons[42] = value != 0,
            EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY32) => result.buttons[43] = value != 0,
            _ => {}
        }
    }

    // Have to flip the hat bits ourselves (0..3 -> 3..0) because
    // packed_bits only handles *byte* ordering, not *bit* ordering
    result.hat = flip_hat_bits(hatxy_to_angle(hatx, haty)).into();
    let mut bytes = result.pack().expect("Failed to pack report to bytes");
    // packed_struct can't handle endianness on bit arrays, so we have
    // to do this ourselves
    bytes[21] = bytes[21].reverse_bits();
    bytes[20] = bytes[20].reverse_bits();
    bytes[19] = bytes[19].reverse_bits();
    bytes[18] = bytes[18].reverse_bits();
    bytes[17] = bytes[17].reverse_bits();
    bytes[16] = bytes[16].reverse_bits();
    bytes
}

fn flip_hat_bits(hat: u8) -> u8 {
    hat.reverse_bits() >> 4
}

fn hatxy_to_angle(hatx: i64, haty: i64) -> u8 {
    // zero to seven, starting at the top and going clockwise. out of
    // range is null.
    //
    // +Y is down.
    // +X is right.
    match (hatx, haty) {
        (0, 0) => 15,
        (0, -1) => 0,
        (1, -1) => 1,
        (1, 0) => 2,
        (1, 1) => 3,
        (0, 1) => 4,
        (-1, 1) => 5,
        (-1, 0) => 6,
        (-1, -1) => 7,
        _ => unreachable!("hat_x and hat_y should be in [-1 .. 1]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hat_zero() {
        assert_eq!(
            make_report(vec! {}.into_iter()),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_hat_plus_x() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_HAT0X), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_hat_plus_y() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_HAT0Y), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_hat_combines() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_HAT0Y), 1),
                    (EventCode::EV_ABS(EV_ABS::ABS_HAT0X), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_button_firstbyte() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_KEY(EV_KEY::BTN_TRIGGER), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_button_nextbyte() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_KEY(EV_KEY::BTN_TOP2), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x0f, 0x01, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_button_combo_bytes() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_KEY(EV_KEY::BTN_TOP2), 1),
                    (EventCode::EV_KEY(EV_KEY::BTN_PINKIE), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x0f, 0x03, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_button_lastbyte() {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY32), 1),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x80
            ]
        );
    }

    fn assert_x(x: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_X), x),
                }
                .into_iter()
            ),
            [
                bytea, byteb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_x() {
        assert_x(1, 0x01, 0x00);
        assert_x(15, 0x0f, 0x00);
        assert_x(16, 0x10, 0x00);
        assert_x(17, 0x11, 0x00);
        assert_x(123, 0x7b, 0x00);
        assert_x(1001, 0xe9, 0x03);
        assert_x(-1, 0xff, 0xff);
        assert_x(-1001, 0x17, 0xfc);
    }
}
