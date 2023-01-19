#![feature(iter_intersperse)]

use anyhow::{Context, Result};
use clap::Parser;
use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY, EV_REL};
use evdev_rs::DeviceWrapper;
use std::collections::HashMap;
use std::thread;

mod gadget;
mod joystick_mux;
mod report;

use joystick_mux::{AxisCombineFn, AxisUpdate, InputAxis, InputAxisId, JoystickId, OutputAxisId};

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Init,
    Uninit,
    Run,
}

fn lower_bound_for(code: EventCode) -> i64 {
    match code {
        EventCode::EV_ABS(_) => -350,
        EventCode::EV_REL(_) => -350,
        EventCode::EV_KEY(_) => 0,
        _ => -350,
    }
}

fn upper_bound_for(code: EventCode) -> i64 {
    match code {
        EventCode::EV_ABS(_) => 350,
        EventCode::EV_REL(_) => 350,
        EventCode::EV_KEY(_) => 1,
        _ => 350,
    }
}

fn get_input_axes(device: &evdev_rs::Device, id: u16) -> HashMap<EventCode, InputAxis> {
    let mut result = HashMap::new();
    let iterator = evdev_rs::EventCodeIterator::new(&evdev_rs::enums::EventType::EV_ABS)
        .chain(evdev_rs::EventCodeIterator::new(
            &evdev_rs::enums::EventType::EV_REL,
        ))
        .chain(evdev_rs::EventCodeIterator::new(
            &evdev_rs::enums::EventType::EV_KEY,
        ));
    for code in iterator {
        let id = InputAxisId {
            joystick: joystick_mux::JoystickId(id),
            axis: code,
        };
        if let Some(ai) = device.abs_info(&code) {
            result.insert(
                code,
                InputAxis {
                    id,
                    lower_bound: ai.minimum.into(),
                    upper_bound: ai.maximum.into(),
                },
            );
        } else if device.has(code) {
            result.insert(
                code,
                InputAxis {
                    id,
                    lower_bound: lower_bound_for(code),
                    upper_bound: upper_bound_for(code),
                },
            );
        }
    }
    result
}

fn handle_device(
    device: evdev_rs::Device,
    id: JoystickId,
    updates: crossbeam_channel::Sender<joystick_mux::AxisUpdate>,
) -> ! {
    loop {
        if let Ok(ev) = device
            .next_event(evdev_rs::ReadFlag::NORMAL)
            .map(|val| val.1)
        {
            updates
                .send(AxisUpdate {
                    joystick: id,
                    event: ev,
                })
                .expect("Failed to send");
        }
    }
}

fn run() -> Result<()> {
    let (update_s, update_r) = crossbeam_channel::bounded::<joystick_mux::AxisUpdate>(5);
    let (output_s, output_r) = crossbeam_channel::bounded::<joystick_mux::OutputState>(5);

    let js_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-Thrustmaster_T.16000M-event-joystick",
    )
    .context("Failed to open joystick")?;
    let js_axes = get_input_axes(&js_device, 0);

    let sp_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-3Dconnexion_SpaceMouse_Pro-event-mouse",
    )
    .context("Failed to open spacemouse")?;
    let sp_axes = get_input_axes(&sp_device, 1);

    let th_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick",
    )
    .context("Failed to open throttle")?;
    let th_axes = get_input_axes(&th_device, 2);

    let mut mux = joystick_mux::JoystickMux::new(Some(output_s));
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
                sp_axes[&EventCode::EV_REL(EV_REL::REL_Y)],
                th_axes[&EventCode::EV_ABS(EV_ABS::ABS_Z)],
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
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TRIGGER)),
        AxisCombineFn::Button {
            // JS trigger
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_TRIGGER)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_THUMB)),
        AxisCombineFn::Button {
            // JS thumb
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_THUMB2)),
        AxisCombineFn::Button {
            // JS thumb left
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_THUMB2)]],
        },
    );
    mux.configure_axis(
        OutputAxisId(EventCode::EV_KEY(EV_KEY::BTN_TOP)),
        AxisCombineFn::Button {
            // JS thumb right
            inputs: vec![js_axes[&EventCode::EV_KEY(EV_KEY::BTN_TOP)]],
        },
    );

    let js_s = update_s.clone();
    thread::spawn(move || {
        handle_device(js_device, JoystickId(0), js_s);
    });

    let sp_s = update_s.clone();
    thread::spawn(move || {
        handle_device(sp_device, JoystickId(1), sp_s);
    });

    let th_s = update_s.clone();
    thread::spawn(move || {
        handle_device(th_device, JoystickId(2), th_s);
    });

    thread::spawn(move || loop {
        if let Ok(update) = update_r.recv() {
            mux.update(update);
        }
    });

    let mut device = gadget::get_gadget_device().context("Failed to open gadget device")?;
    loop {
        if let Ok(output) = output_r.recv() {
            println!("{}", output);
            println!(
                "{}",
                hex::encode(report::make_report(
                    output
                        .axes
                        .into_iter()
                        .map(|(OutputAxisId(axis_id), value)| (axis_id, value))
                ))
            );
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::Init => gadget::init_gadget(),
        Command::Uninit => gadget::uninit_gadget(),
        Command::Run => run(),
    }
}
