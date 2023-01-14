use evdev_rs::enums::EventCode;
use evdev_rs::enums::EV_ABS;
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

impl InputAxisId {
    pub fn new(js_id: u16, axis: EventCode) -> Self {
        Self {
            axis: axis,
            joystick: JoystickId(js_id),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct InputAxis {
    pub id: InputAxisId,
    pub lower_bound: i64,
    pub upper_bound: i64,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct OutputAxisId {
    axis_number: u16,
}

impl OutputAxisId {
    pub fn new(axis_number: u16) -> Self {
        Self { axis_number }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxisState(pub Option<i64>);

#[derive(Debug)]
struct OutputAxis {
    output: OutputAxisId,
    inputs: Vec<InputAxis>,
    combine_fn: AxisCombineFn,
}

#[derive(Debug, Copy, Clone)]
pub struct AxisUpdate {
    pub axis: InputAxisId,
    pub state: AxisState,
}

#[derive(Debug)]
pub struct JoystickMux {
    axis_states: HashMap<InputAxisId, AxisState>,
    axes: HashMap<OutputAxisId, OutputAxis>,
    update: Option<crossbeam_channel::Sender<AxisState>>,
}

impl JoystickMux {
    pub fn new(update: Option<crossbeam_channel::Sender<()>>) -> Self {
        Self {
            axis_states: HashMap::new(),
            axes: HashMap::new(),
            update: update,
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
        self.axis_states.insert(update.axis, update.state);
    }

    pub fn output(&mut self, axis_id: OutputAxisId) -> AxisState {
        match self.axes.get(&axis_id) {
            Some(output_axis) => match output_axis.combine_fn {
                AxisCombineFn::LargestMagnitude => output_axis
                    .inputs
                    .iter()
                    .filter_map(|input| match self.axis_states.get(&input.id) {
                        Some(ax) => Some((input, ax)),
                        None => None,
                    })
                    .map(|(input, state)| match state {
                        AxisState(Some(v)) => {
                            OUTPUT_LOWER_BOUND
                                + ((v - input.lower_bound)
                                    * (OUTPUT_UPPER_BOUND - OUTPUT_LOWER_BOUND)
                                    / (input.upper_bound - input.lower_bound))
                        }
                        AxisState(None) => 0,
                    })
                    .max_by_key(|value| value.abs())
                    .map_or(AxisState(None), |state| AxisState(Some(state))),
                AxisCombineFn::Hat { x: _, y: _ } => AxisState(None),
            },
            None => AxisState(None),
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
            OutputAxisId::new(1),
            [].into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(None));
    }

    #[test]
    fn test_axis_with_no_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [InputAxis {
                id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                lower_bound: -32767,
                upper_bound: 32767,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(None));
    }

    #[test]
    fn test_axis_with_some_data() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [InputAxis {
                id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                lower_bound: -32767,
                upper_bound: 32767,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(5)));
    }

    #[test]
    fn test_largest_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
            ]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
            state: AxisState(Some(12)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(12)));
    }

    #[test]
    fn test_negative_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
            ]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
            state: AxisState(Some(-12)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(-12)));
    }

    #[test]
    fn test_none_magnitude() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
                InputAxis {
                    id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
                    lower_bound: -32767,
                    upper_bound: 32767,
                },
            ]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_Y)),
            state: AxisState(None),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(5)));
    }

    #[test]
    fn test_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [InputAxis {
                id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                lower_bound: -5,
                upper_bound: 5,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(0)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(0)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(32767)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(-5)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(-32767)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(1)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(6553)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(-1)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(-6554)));
    }
    #[test]
    fn test_inverted_input_range() {
        let mut m = JoystickMux::new(None);
        m.configure_axis(
            OutputAxisId::new(1),
            [InputAxis {
                id: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
                lower_bound: 5,
                upper_bound: -5,
            }]
            .into_iter(),
            AxisCombineFn::LargestMagnitude,
        );
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(0)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(0)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(5)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(-32767)));
        m.update(AxisUpdate {
            axis: InputAxisId::new(0, EventCode::EV_ABS(EV_ABS::ABS_X)),
            state: AxisState(Some(-5)),
        });
        assert_eq!(m.output(OutputAxisId::new(1)), AxisState(Some(32767)));
    }
}
