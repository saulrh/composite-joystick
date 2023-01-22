use crate::joystick_mux::{AxisCombineFn, ButtonMode, InputAxis, JoystickMux, OutputAxisId};
use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY, EV_REL};
use std::collections::HashMap;

pub fn configure_mux(
    mux: &mut JoystickMux,
    js_axes: &HashMap<EventCode, InputAxis>,
    th_axes: &HashMap<EventCode, InputAxis>,
    sp_axes: &HashMap<EventCode, InputAxis>,
) {
    mux.configure_axis(
        // Yaw
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_RZ)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                js_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
                sp_axes[&EventCode::EV_REL(EV_REL::REL_RZ)],
            ],
        },
    );
    mux.configure_axis(
        // Pitch
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_RX)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                js_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
                sp_axes[&EventCode::EV_REL(EV_REL::REL_RX)],
            ],
        },
    );
    mux.configure_axis(
        // Roll
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_RY)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                js_axes[&EventCode::EV_ABS(EV_ABS::ABS_RZ)],
                -sp_axes[&EventCode::EV_REL(EV_REL::REL_RY)],
            ],
        },
    );
    mux.configure_axis(
        // Throttle/translate f/b
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_Y)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                -sp_axes[&EventCode::EV_REL(EV_REL::REL_Y)],
                -th_axes[&EventCode::EV_ABS(EV_ABS::ABS_Z)],
            ],
        },
    );
    mux.configure_axis(
        // translate l/r
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                sp_axes[&EventCode::EV_REL(EV_REL::REL_X)],
                th_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
            ],
        },
    );
    mux.configure_axis(
        // translate u/d
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_Z)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![
                sp_axes[&EventCode::EV_REL(EV_REL::REL_Z)],
                th_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
            ],
        },
    );
    mux.configure_axis(
        // dial
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_RUDDER)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_RUDDER)]],
        },
    );
    mux.configure_axis(
        // slider
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_THROTTLE)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_RZ)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_HAT0X)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![js_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0X)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_HAT0Y)),
        AxisCombineFn::LargestMagnitude {
            inputs: vec![js_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0Y)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // JS trigger
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_TRIGGER)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_THUMB)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // JS thumb
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_THUMB2)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // JS thumb left
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TOP)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // JS thumb right
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_TOP)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TOP2)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle pinkie
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_PINKIE)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle ring
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle switch up
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_TOP)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE2)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle switch down
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_TOP2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE3)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle click stick
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_PINKIE)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE4)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle thumb orange
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_TRIGGER)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE5)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle middle hat up
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_BASE6)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle middle hat forward
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY1)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle middle hat down
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE3)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY2)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle middle hat back
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE4)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY3)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle bottom hat up
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE5)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY4)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle bottom hat forward
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE6)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY5)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle bottom hat down
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_300)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY6)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // throttle bottom hat back
            inputs: vec![th_axes[&EventCode::EV_KEY(EV_KEY::BTN_301)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY7)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse macro 1
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_268)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY8)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse macro 2
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_269)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY9)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse macro 3
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_270)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY10)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse macro 4
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_271)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY11)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse esc
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_BACK)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY12)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse shift
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_280)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY13)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse ctrl
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_281)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY14)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse alt
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_TASK)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY15)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse rotate
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_8)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY16)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse T
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY17)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse middle
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_282)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY18)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse F
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_5)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY19)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse R
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_4)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY20)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse fit
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_1)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY21)),
        AxisCombineFn::Button {
            mode: ButtonMode::Negative,
            // throttle top hat up
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0Y)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY22)),
        AxisCombineFn::Button {
            mode: ButtonMode::Positive,
            // throttle top hat forward
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0X)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY23)),
        AxisCombineFn::Button {
            mode: ButtonMode::Positive,
            // throttle top hat down
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0Y)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY24)),
        AxisCombineFn::Button {
            mode: ButtonMode::Negative,
            // throttle top hat back
            inputs: vec![th_axes[&EventCode::EV_ABS(EV_ABS::ABS_HAT0X)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY25)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left top-left
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_TOP2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY26)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left top-mid
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_PINKIE)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY27)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left top-right
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY28)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left bottom-left
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE4)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY29)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left bottom-middle
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE3)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY30)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // joystick base-left bottom-right
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_BASE2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY31)),
        AxisCombineFn::Button {
            mode: ButtonMode::NonZero,
            // spacemouse menu
            inputs: vec![sp_axes[&EventCode::EV_KEY(EV_KEY::BTN_0)]],
        },
    );
}
