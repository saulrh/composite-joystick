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
}
