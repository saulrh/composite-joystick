use evdev_rs::enums::EventCode;
use evdev_rs::InputEvent;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

const OUTPUT_UPPER_BOUND: i64 = 32767;
const OUTPUT_LOWER_BOUND: i64 = -32767;

#[derive(Error, Debug)]
pub enum JoystickMuxError {}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct JoystickId(pub u16);

#[derive(Debug, Clone)]
pub enum ButtonMode {
    NonZero,
    Positive,
    Negative,
}

#[derive(Debug, Clone)]
pub enum AxisCombineFn {
    LargestMagnitude {
        inputs: Vec<InputAxis>,
    },
    Button {
        mode: ButtonMode,
        inputs: Vec<InputAxis>,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct InputAxisId {
    pub joystick: JoystickId,
    pub axis: EventCode,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct InputAxis {
    pub id: InputAxisId,
    pub lower_bound: i64,
    pub upper_bound: i64,
}

impl std::ops::Neg for InputAxis {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            id: self.id,
            lower_bound: self.upper_bound,
            upper_bound: self.lower_bound,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct OutputAxisId(pub EventCode);

impl Ord for OutputAxisId {
    fn cmp(&self, other: &Self) -> Ordering {
        let OutputAxisId(self_code) = &self;
        let OutputAxisId(other_code) = &other;
        evdev_rs::util::event_code_to_int(self_code)
            .cmp(&evdev_rs::util::event_code_to_int(other_code))
    }
}

impl PartialOrd for OutputAxisId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct JoystickMux {
    axis_states: HashMap<InputAxisId, InputEvent>,
    axes: HashMap<OutputAxisId, AxisCombineFn>,
    output_s: Option<crossbeam_channel::Sender<OutputState>>,
}

#[derive(Debug)]
pub struct AxisUpdate {
    pub joystick: JoystickId,
    pub event: InputEvent,
}

#[derive(Debug, PartialEq)]
pub struct OutputState {
    pub axes: Vec<(OutputAxisId, i64)>,
}

impl OutputState {
    pub fn new(axes: impl Iterator<Item = (OutputAxisId, i64)>) -> Self {
        let mut result = OutputState {
            axes: axes.into_iter().collect(),
        };
        result.sort();
        result
    }

    pub fn sort(&mut self) {
        self.axes.sort();
    }
}

impl fmt::Display for OutputState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (axis, value) in self.axes.iter() {
            let OutputAxisId(code) = axis;
            write!(f, "{code}: {value}\t")?;
        }
        Ok(())
    }
}

impl JoystickMux {
    pub fn new(output_s: Option<crossbeam_channel::Sender<OutputState>>) -> Self {
        Self {
            axis_states: HashMap::new(),
            axes: HashMap::new(),
            output_s,
        }
    }

    pub fn configure_axis(&mut self, output_axis: OutputAxisId, combine_fn: AxisCombineFn) {
        self.axes.insert(output_axis, combine_fn);
    }

    pub fn update(&mut self, update: AxisUpdate) {
        match update.event.event_code {
            EventCode::EV_SYN(_) => self.send_output(),
            code => {
                self.axis_states.insert(
                    InputAxisId {
                        joystick: update.joystick,
                        axis: code,
                    },
                    update.event,
                );
            }
        }
    }

    pub fn output_axis(&self, axis_id: &OutputAxisId) -> Option<i64> {
        match self.axes.get(axis_id) {
            Some(combine_fn) => match combine_fn {
                AxisCombineFn::Button { inputs, mode } => {
                    let pressed = inputs
                        .iter()
                        .map(|input| match self.axis_states.get(&input.id) {
                            Some(event) => match mode {
                                ButtonMode::NonZero => event.value != 0,
                                ButtonMode::Positive => event.value > 0,
                                ButtonMode::Negative => event.value < 0,
                            },
                            None => false,
                        })
                        .any(|value| value);
                    if pressed {
                        Some(1)
                    } else {
                        Some(0)
                    }
                }
                AxisCombineFn::LargestMagnitude { inputs } => inputs
                    .iter()
                    .map(|input| match self.axis_states.get(&input.id) {
                        Some(event) => {
                            OUTPUT_LOWER_BOUND
                                + ((i64::from(event.value) - input.lower_bound)
                                    * (OUTPUT_UPPER_BOUND - OUTPUT_LOWER_BOUND)
                                    / (input.upper_bound - input.lower_bound))
                        }
                        None => 0,
                    })
                    .max_by_key(|value| value.abs()),
            },
            None => None,
        }
    }

    pub fn output(&self) -> OutputState {
        OutputState::new(
            self.axes
                .keys()
                .map(|output_id| (*output_id, self.output_axis(output_id).unwrap_or(0))),
        )
    }

    pub fn send_output(&mut self) {
        if let Some(sender) = &self.output_s {
            sender.send(self.output()).expect("Failed to send state");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev_rs::enums::EV_ABS;

    const ZERO_TIME: evdev_rs::TimeVal = evdev_rs::TimeVal {
        tv_sec: 0,
        tv_usec: 0,
    };

    #[test]
    fn test_inputless_axis() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude { inputs: vec![] },
        );
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 0)],
            }
        );
    }

    #[test]
    fn test_axis_with_no_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: -32767,
                    upper_bound: 32767,
                }],
            },
        );
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 0)],
            }
        );
    }

    #[test]
    fn test_axis_with_some_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: -32767,
                    upper_bound: 32767,
                }],
            },
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 5)],
            }
        );
    }

    #[test]
    fn test_largest_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                        },
                        lower_bound: -32767,
                        upper_bound: 32767,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_Y),
                        },
                        lower_bound: -32767,
                        upper_bound: 32767,
                    },
                ],
            },
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Y),
                value: 12,
            },
        });

        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 12)],
            }
        );
    }

    #[test]
    fn test_negative_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                        },
                        lower_bound: -32767,
                        upper_bound: 32767,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_Y),
                        },
                        lower_bound: -32767,
                        upper_bound: 32767,
                    },
                ],
            },
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Y),
                value: -12,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), -12)],
            }
        );
    }

    #[test]
    fn test_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: -5,
                    upper_bound: 5,
                }],
            },
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 0,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 0)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), -32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 1,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 6553)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -1,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), -6554)],
            }
        );
    }

    #[test]
    fn test_inverted_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: 5,
                    upper_bound: -5,
                }],
            },
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), -32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 32767)],
            }
        );
    }

    #[test]
    fn test_button_mode_nonzero() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER)),
            AxisCombineFn::Button {
                mode: ButtonMode::NonZero,
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                    },
                    lower_bound: 0,
                    upper_bound: 1,
                }],
            },
        );

        // Test zero value
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 0,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Test positive value
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Test negative value (should also trigger for NonZero)
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: -1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));
    }

    #[test]
    fn test_button_mode_positive() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER)),
            AxisCombineFn::Button {
                mode: ButtonMode::Positive,
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                    },
                    lower_bound: -1,
                    upper_bound: 1,
                }],
            },
        );

        // Test zero value - should not trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                value: 0,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Test positive value - should trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Test negative value - should not trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                value: -1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));
    }

    #[test]
    fn test_button_mode_negative() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER)),
            AxisCombineFn::Button {
                mode: ButtonMode::Negative,
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                    },
                    lower_bound: -1,
                    upper_bound: 1,
                }],
            },
        );

        // Test zero value - should not trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                value: 0,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Test negative value - should trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                value: -1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Test positive value - should not trigger
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));
    }

    #[test]
    fn test_button_multiple_inputs_any() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER)),
            AxisCombineFn::Button {
                mode: ButtonMode::NonZero,
                inputs: vec![
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                        },
                        lower_bound: 0,
                        upper_bound: 1,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_1),
                        },
                        lower_bound: 0,
                        upper_bound: 1,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(1),
                            axis: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_2),
                        },
                        lower_bound: 0,
                        upper_bound: 1,
                    },
                ],
            },
        );

        // No buttons pressed
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Press first button
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Release first, press second
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 0,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_1),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Press third button from different joystick
        m.update(AxisUpdate {
            joystick: JoystickId(1),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_2),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Release all
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_1),
                value: 0,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(1),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_2),
                value: 0,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));
    }

    #[test]
    fn test_ev_syn_triggers_send() {
        use evdev_rs::enums::EV_SYN;

        let (sender, receiver) = crossbeam_channel::bounded::<OutputState>(5);
        let mut m = JoystickMux::new(Some(sender));

        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: -32767,
                    upper_bound: 32767,
                }],
            },
        );

        // Update axis value
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 100,
            },
        });

        // No output should be sent yet
        assert!(receiver.try_recv().is_err());

        // Send EV_SYN to trigger output
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                value: 0,
            },
        });

        // Now output should be available
        let output = receiver.try_recv().expect("Expected output after EV_SYN");
        assert_eq!(
            output,
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 100)],
            }
        );
    }

    #[test]
    fn test_multiple_joysticks() {
        let mut m = JoystickMux::new(None);

        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(0),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                        },
                        lower_bound: -100,
                        upper_bound: 100,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(1),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_Y),
                        },
                        lower_bound: -100,
                        upper_bound: 100,
                    },
                    InputAxis {
                        id: InputAxisId {
                            joystick: JoystickId(2),
                            axis: EventCode::EV_ABS(EV_ABS::ABS_Z),
                        },
                        lower_bound: -100,
                        upper_bound: 100,
                    },
                ],
            },
        );

        // Update from joystick 0
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 50,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X))), Some(16383));

        // Update from joystick 1 with larger magnitude
        m.update(AxisUpdate {
            joystick: JoystickId(1),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Y),
                value: -75,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X))), Some(-24576));

        // Update from joystick 2 with smaller magnitude (should not change output)
        m.update(AxisUpdate {
            joystick: JoystickId(2),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Z),
                value: 25,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X))), Some(-24576));

        // Update joystick 2 with largest magnitude
        m.update(AxisUpdate {
            joystick: JoystickId(2),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Z),
                value: 100,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X))), Some(32767));
    }

    #[test]
    fn test_send_output_with_channel() {
        let (sender, receiver) = crossbeam_channel::bounded::<OutputState>(5);
        let mut m = JoystickMux::new(Some(sender));

        m.configure_axis(
            OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)),
            AxisCombineFn::LargestMagnitude {
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                    },
                    lower_bound: -32767,
                    upper_bound: 32767,
                }],
            },
        );

        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 1234,
            },
        });

        // Manually call send_output
        m.send_output();

        let output = receiver.try_recv().expect("Expected output");
        assert_eq!(
            output,
            OutputState {
                axes: vec![(OutputAxisId(EventCode::EV_ABS(EV_ABS::ABS_X)), 1234)],
            }
        );
    }

    #[test]
    fn test_button_state_transitions() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER)),
            AxisCombineFn::Button {
                mode: ButtonMode::NonZero,
                inputs: vec![InputAxis {
                    id: InputAxisId {
                        joystick: JoystickId(0),
                        axis: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                    },
                    lower_bound: 0,
                    upper_bound: 1,
                }],
            },
        );

        // Initial state - no data
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Press
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Hold (same value)
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 1,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(1));

        // Release
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: ZERO_TIME,
                event_code: EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_0),
                value: 0,
            },
        });
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));

        // Stay released
        assert_eq!(m.output_axis(&OutputAxisId(EventCode::EV_KEY(evdev_rs::enums::EV_KEY::BTN_TRIGGER))), Some(0));
    }
}
