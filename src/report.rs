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

    fn assert_y(y: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_Y), y),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, bytea, byteb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_y() {
        assert_y(0, 0x00, 0x00);
        assert_y(100, 0x64, 0x00);
        assert_y(-100, 0x9c, 0xff);
        assert_y(32767, 0xff, 0x7f);
        assert_y(-32767, 0x01, 0x80);
    }

    fn assert_z(z: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_Z), z),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, bytea, byteb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_z() {
        assert_z(0, 0x00, 0x00);
        assert_z(256, 0x00, 0x01);
        assert_z(-256, 0x00, 0xff);
        assert_z(32767, 0xff, 0x7f);
        assert_z(-32768, 0x00, 0x80);
    }

    fn assert_rx(rx: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_RX), rx),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, bytea, byteb, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_rx() {
        assert_rx(0, 0x00, 0x00);
        assert_rx(1000, 0xe8, 0x03);
        assert_rx(-1000, 0x18, 0xfc);
        assert_rx(32767, 0xff, 0x7f);
        assert_rx(-32767, 0x01, 0x80);
    }

    fn assert_ry(ry: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_RY), ry),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, bytea, byteb, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_ry() {
        assert_ry(0, 0x00, 0x00);
        assert_ry(5000, 0x88, 0x13);
        assert_ry(-5000, 0x78, 0xec);
        assert_ry(32767, 0xff, 0x7f);
        assert_ry(-32768, 0x00, 0x80);
    }

    fn assert_rz(rz: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_RZ), rz),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, bytea, byteb, 0x00,
                0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_rz() {
        assert_rz(0, 0x00, 0x00);
        assert_rz(10000, 0x10, 0x27);
        assert_rz(-10000, 0xf0, 0xd8);
        assert_rz(32767, 0xff, 0x7f);
        assert_rz(-32767, 0x01, 0x80);
    }

    fn assert_slider(slider: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_THROTTLE), slider),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, bytea,
                byteb, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_slider() {
        assert_slider(0, 0x00, 0x00);
        assert_slider(255, 0xff, 0x00);
        assert_slider(-255, 0x01, 0xff);
        assert_slider(32767, 0xff, 0x7f);
        assert_slider(-32768, 0x00, 0x80);
    }

    fn assert_dial(dial: i64, bytea: u8, byteb: u8) {
        assert_eq!(
            make_report(
                vec! {
                    (EventCode::EV_ABS(EV_ABS::ABS_RUDDER), dial),
                }
                .into_iter()
            ),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, bytea, byteb, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_axes_dial() {
        assert_dial(0, 0x00, 0x00);
        assert_dial(1234, 0xd2, 0x04);
        assert_dial(-1234, 0x2e, 0xfb);
        assert_dial(32767, 0xff, 0x7f);
        assert_dial(-32767, 0x01, 0x80);
    }

    #[test]
    fn test_all_hat_positions() {
        // Test all 8 directions + neutral
        let test_cases = vec![
            ((0, 0), 0x0f),   // neutral
            ((0, -1), 0x00),  // up
            ((1, -1), 0x01),  // up-right
            ((1, 0), 0x02),   // right
            ((1, 1), 0x03),   // down-right
            ((0, 1), 0x04),   // down
            ((-1, 1), 0x05),  // down-left
            ((-1, 0), 0x06),  // left
            ((-1, -1), 0x07), // up-left
        ];

        for ((x, y), expected_hat) in test_cases {
            let mut inputs = vec![];
            if x != 0 {
                inputs.push((EventCode::EV_ABS(EV_ABS::ABS_HAT0X), x));
            }
            if y != 0 {
                inputs.push((EventCode::EV_ABS(EV_ABS::ABS_HAT0Y), y));
            }

            let report = make_report(inputs.into_iter());
            assert_eq!(
                report[16], expected_hat,
                "Failed for HAT position ({}, {}): expected {:#04x}, got {:#04x}",
                x, y, expected_hat, report[16]
            );
        }
    }

    #[test]
    fn test_all_buttons() {
        // Test each button individually
        // Note: Byte 16 has HAT in lower nibble (0x0f), buttons start in upper nibble
        let button_map = vec![
            (EV_KEY::BTN_TRIGGER, 16, 0x10),    // buttons[0]
            (EV_KEY::BTN_THUMB, 16, 0x20),      // buttons[1]
            (EV_KEY::BTN_THUMB2, 16, 0x40),     // buttons[2]
            (EV_KEY::BTN_TOP, 16, 0x80),        // buttons[3]
            (EV_KEY::BTN_TOP2, 17, 0x01),       // buttons[4]
            (EV_KEY::BTN_PINKIE, 17, 0x02),     // buttons[5]
            (EV_KEY::BTN_BASE, 17, 0x04),       // buttons[6]
            (EV_KEY::BTN_BASE2, 17, 0x08),      // buttons[7]
            (EV_KEY::BTN_BASE3, 17, 0x10),      // buttons[8]
            (EV_KEY::BTN_BASE4, 17, 0x20),      // buttons[9]
            (EV_KEY::BTN_BASE5, 17, 0x40),      // buttons[10]
            (EV_KEY::BTN_BASE6, 17, 0x80),      // buttons[11]
            (EV_KEY::BTN_TRIGGER_HAPPY1, 18, 0x01),   // buttons[12]
            (EV_KEY::BTN_TRIGGER_HAPPY2, 18, 0x02),   // buttons[13]
            (EV_KEY::BTN_TRIGGER_HAPPY3, 18, 0x04),   // buttons[14]
            (EV_KEY::BTN_TRIGGER_HAPPY4, 18, 0x08),   // buttons[15]
            (EV_KEY::BTN_TRIGGER_HAPPY5, 18, 0x10),   // buttons[16]
            (EV_KEY::BTN_TRIGGER_HAPPY6, 18, 0x20),   // buttons[17]
            (EV_KEY::BTN_TRIGGER_HAPPY7, 18, 0x40),   // buttons[18]
            (EV_KEY::BTN_TRIGGER_HAPPY8, 18, 0x80),   // buttons[19]
            (EV_KEY::BTN_TRIGGER_HAPPY9, 19, 0x01),   // buttons[20]
            (EV_KEY::BTN_TRIGGER_HAPPY10, 19, 0x02),  // buttons[21]
            (EV_KEY::BTN_TRIGGER_HAPPY11, 19, 0x04),  // buttons[22]
            (EV_KEY::BTN_TRIGGER_HAPPY12, 19, 0x08),  // buttons[23]
            (EV_KEY::BTN_TRIGGER_HAPPY13, 19, 0x10),  // buttons[24]
            (EV_KEY::BTN_TRIGGER_HAPPY14, 19, 0x20),  // buttons[25]
            (EV_KEY::BTN_TRIGGER_HAPPY15, 19, 0x40),  // buttons[26]
            (EV_KEY::BTN_TRIGGER_HAPPY16, 19, 0x80),  // buttons[27]
            (EV_KEY::BTN_TRIGGER_HAPPY17, 20, 0x01),  // buttons[28]
            (EV_KEY::BTN_TRIGGER_HAPPY18, 20, 0x02),  // buttons[29]
            (EV_KEY::BTN_TRIGGER_HAPPY19, 20, 0x04),  // buttons[30]
            (EV_KEY::BTN_TRIGGER_HAPPY20, 20, 0x08),  // buttons[31]
            (EV_KEY::BTN_TRIGGER_HAPPY21, 20, 0x10),  // buttons[32]
            (EV_KEY::BTN_TRIGGER_HAPPY22, 20, 0x20),  // buttons[33]
            (EV_KEY::BTN_TRIGGER_HAPPY23, 20, 0x40),  // buttons[34]
            (EV_KEY::BTN_TRIGGER_HAPPY24, 20, 0x80),  // buttons[35]
            (EV_KEY::BTN_TRIGGER_HAPPY25, 21, 0x01),  // buttons[36]
            (EV_KEY::BTN_TRIGGER_HAPPY26, 21, 0x02),  // buttons[37]
            (EV_KEY::BTN_TRIGGER_HAPPY27, 21, 0x04),  // buttons[38]
            (EV_KEY::BTN_TRIGGER_HAPPY28, 21, 0x08),  // buttons[39]
            (EV_KEY::BTN_TRIGGER_HAPPY29, 21, 0x10),  // buttons[40]
            (EV_KEY::BTN_TRIGGER_HAPPY30, 21, 0x20),  // buttons[41]
            (EV_KEY::BTN_TRIGGER_HAPPY31, 21, 0x40),  // buttons[42]
            (EV_KEY::BTN_TRIGGER_HAPPY32, 21, 0x80),  // buttons[43]
        ];

        for (button, byte_idx, expected_mask) in button_map {
            let report = make_report(vec![(EventCode::EV_KEY(button), 1)].into_iter());
            // For byte 16, need to account for HAT neutral (0x0f)
            let expected_value = if byte_idx == 16 {
                expected_mask | 0x0f
            } else {
                expected_mask
            };
            assert_eq!(
                report[byte_idx], expected_value,
                "Failed for button {:?}: expected byte[{}] = {:#04x}, got {:#04x}",
                button, byte_idx, expected_value, report[byte_idx]
            );
        }
    }

    #[test]
    fn test_multiple_buttons_same_byte() {
        let report = make_report(
            vec![
                (EventCode::EV_KEY(EV_KEY::BTN_TRIGGER), 1),
                (EventCode::EV_KEY(EV_KEY::BTN_THUMB), 1),
                (EventCode::EV_KEY(EV_KEY::BTN_THUMB2), 1),
            ]
            .into_iter(),
        );
        // All three buttons are in byte 16 (upper nibble)
        // BTN_TRIGGER=0x10, BTN_THUMB=0x20, BTN_THUMB2=0x40
        // Combined: 0x10 | 0x20 | 0x40 = 0x70, plus HAT neutral (0x0f) = 0x7f
        assert_eq!(report[16], 0x7f); // HAT neutral (0x0f) + three buttons (0x70)
    }

    #[test]
    fn test_combined_state() {
        // Test multiple axes and buttons at once
        let report = make_report(
            vec![
                (EventCode::EV_ABS(EV_ABS::ABS_X), 1000),
                (EventCode::EV_ABS(EV_ABS::ABS_Y), -500),
                (EventCode::EV_ABS(EV_ABS::ABS_RZ), 2000),
                (EventCode::EV_ABS(EV_ABS::ABS_HAT0X), 1),
                (EventCode::EV_ABS(EV_ABS::ABS_HAT0Y), -1),
                (EventCode::EV_KEY(EV_KEY::BTN_TRIGGER), 1),
                (EventCode::EV_KEY(EV_KEY::BTN_BASE5), 1),
            ]
            .into_iter(),
        );

        // Check X axis
        assert_eq!(report[0], 0xe8);
        assert_eq!(report[1], 0x03);

        // Check Y axis
        assert_eq!(report[2], 0x0c);
        assert_eq!(report[3], 0xfe);

        // Check RZ axis
        assert_eq!(report[10], 0xd0);
        assert_eq!(report[11], 0x07);

        // Check HAT (1, -1) = up-right = 1, plus BTN_TRIGGER (0x10)
        assert_eq!(report[16], 0x11); // HAT=0x01 | BTN_TRIGGER=0x10

        // Check BTN_BASE5 in byte 17
        assert_eq!(report[17], 0x40); // BTN_BASE5
    }

    #[test]
    fn test_boundary_values() {
        // Test i16 min/max values don't cause issues
        let report = make_report(
            vec![
                (EventCode::EV_ABS(EV_ABS::ABS_X), i16::MAX as i64),
                (EventCode::EV_ABS(EV_ABS::ABS_Y), i16::MIN as i64),
            ]
            .into_iter(),
        );

        // X should be max positive
        assert_eq!(report[0], 0xff);
        assert_eq!(report[1], 0x7f);

        // Y should be min negative
        assert_eq!(report[2], 0x00);
        assert_eq!(report[3], 0x80);
    }
}
