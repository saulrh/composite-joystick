use evdev_rs::enums::{EventCode, EV_ABS, EV_SYN};
use evdev_rs::InputEvent;
use std::collections::HashMap;

use thiserror::Error;

const OUTPUT_UPPER_BOUND: i64 = 32767;
const OUTPUT_LOWER_BOUND: i64 = -32767;

#[derive(Error, Debug)]
pub enum JoystickMuxError {}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct JoystickId(pub u16);

#[derive(Debug, Copy, Clone)]
pub enum AxisCombineFn {
    LargestMagnitude,
    Hat { x: InputAxisId, y: InputAxisId },
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

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct OutputAxisId(pub u16);

#[derive(Debug)]
struct OutputAxis {
    output: OutputAxisId,
    inputs: Vec<InputAxis>,
    combine_fn: AxisCombineFn,
}

#[derive(Debug)]
pub struct JoystickMux {
    axis_states: HashMap<InputAxisId, InputEvent>,
    axes: HashMap<OutputAxisId, OutputAxis>,
    output_s: Option<crossbeam_channel::Sender<OutputState>>,
}

#[derive(Debug)]
pub struct AxisUpdate {
    pub joystick: JoystickId,
    pub event: InputEvent,
}

#[derive(Debug)]
pub struct OutputState {
    axes: Vec<(OutputAxisId, i64)>,
}

impl PartialEq for OutputState {
    fn eq(&self, other: &Self) -> bool {
        let mut self_axes = self.axes.clone();
        let mut other_axes = other.axes.clone();
        self_axes.sort();
        other_axes.sort();
        self_axes == other_axes
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

    pub fn configure_axis(
        &mut self,
        output_axis: OutputAxisId,
        input_axes: impl Iterator<Item = InputAxis>,
        combine_fn: AxisCombineFn,
    ) {
        self.axes.insert(
            output_axis,
            OutputAxis {
                inputs: input_axes.into_iter().collect(),
                output: output_axis,
                combine_fn: combine_fn,
            },
        );
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

    pub fn output_axis(&self, axis_id: OutputAxisId) -> Option<i64> {
        match self.axes.get(&axis_id) {
            Some(output_axis) => match output_axis.combine_fn {
                AxisCombineFn::LargestMagnitude => output_axis
                    .inputs
                    .iter()
                    .map(|input| match self.axis_states.get(&input.id) {
                        Some(code) => {
                            OUTPUT_LOWER_BOUND
                                + ((i64::from(code.value) - input.lower_bound)
                                    * (OUTPUT_UPPER_BOUND - OUTPUT_LOWER_BOUND)
                                    / (input.upper_bound - input.lower_bound))
                        }
                        None => 0,
                    })
                    .max_by_key(|value| value.abs())
                    .map_or(None, |state| Some(state)),
                // TODO: implement hats
                AxisCombineFn::Hat { x: _, y: _ } => None,
            },
            None => None,
        }
    }

    pub fn output(&self) -> OutputState {
        OutputState {
            axes: self
                .axes
                .iter()
                .map(|(_, axis)| {
                    (
                        axis.output,
                        match self.output_axis(axis.output) {
                            Some(s) => s,
                            None => 0,
                        },
                    )
                })
                .collect(),
        }
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

    #[test]
    fn test_inputless_axis() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [].into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 0)],
            }
        );
    }

    #[test]
    fn test_axis_with_no_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [InputAxis {
                id: InputAxisId {
                    joystick: JoystickId(0),
                    axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                },
                lower_bound: -32767,
                upper_bound: 32767,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 0)],
            }
        );
    }

    #[test]
    fn test_axis_with_some_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [InputAxis {
                id: InputAxisId {
                    joystick: JoystickId(0),
                    axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                },
                lower_bound: -32767,
                upper_bound: 32767,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 5)],
            }
        );
    }

    #[test]
    fn test_largest_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [
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
            ]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Y),
                value: 12,
            },
        });

        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 12)],
            }
        );
    }

    #[test]
    fn test_negative_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [
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
            ]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_Y),
                value: -12,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), -12)],
            }
        );
    }

    #[test]
    fn test_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [InputAxis {
                id: InputAxisId {
                    joystick: JoystickId(0),
                    axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                },
                lower_bound: -5,
                upper_bound: 5,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 0,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 0)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), -32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 1,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 6553)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -1,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), -6554)],
            }
        );
    }

    #[test]
    fn test_inverted_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId(1),
            [InputAxis {
                id: InputAxisId {
                    joystick: JoystickId(0),
                    axis: EventCode::EV_ABS(EV_ABS::ABS_X),
                },
                lower_bound: 5,
                upper_bound: -5,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: 5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), -32767)],
            }
        );
        m.update(AxisUpdate {
            joystick: JoystickId(0),
            event: InputEvent {
                time: evdev_rs::TimeVal::new(0, 0),
                event_code: EventCode::EV_ABS(EV_ABS::ABS_X),
                value: -5,
            },
        });
        assert_eq!(
            m.output(),
            OutputState {
                axes: vec![(OutputAxisId(1), 32767)],
            }
        );
    }
}
