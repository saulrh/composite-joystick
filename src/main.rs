#![feature(iter_intersperse)]

use anyhow::{Context, Result};
use clap::Parser;
use crossbeam_channel::select;
use evdev_rs::enums::{EventCode, EV_ABS, EV_REL, EV_SYN};
use evdev_rs::DeviceWrapper;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::thread;

mod gadget;
mod joystick_mux;

use joystick_mux::{AxisCombineFn, AxisUpdate, InputAxisId, JoystickId, OutputAxisId};

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

fn get_input_axes(
    device: &evdev_rs::Device,
    id: u16,
) -> HashMap<EventCode, joystick_mux::InputAxis> {
    let mut result = HashMap::new();
    let iterator = itertools::chain(
        evdev_rs::EventCodeIterator::new(&evdev_rs::enums::EventType::EV_ABS),
        evdev_rs::EventCodeIterator::new(&evdev_rs::enums::EventType::EV_REL),
    );
    for code in iterator {
        if let Some(ai) = device.abs_info(&code) {
            result.insert(
                code,
                joystick_mux::InputAxis {
                    id: joystick_mux::InputAxisId {
                        joystick: joystick_mux::JoystickId(id),
                        axis: code,
                    },
                    lower_bound: ai.minimum.into(),
                    upper_bound: ai.maximum.into(),
                },
            );
        } else if device.has(code) {
            result.insert(
                code,
                joystick_mux::InputAxis {
                    id: joystick_mux::InputAxisId {
                        joystick: joystick_mux::JoystickId(id),
                        axis: code,
                    },
                    lower_bound: -1000,
                    upper_bound: 1000,
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
        OutputAxisId(0),
        [
            js_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
            sp_axes[&EventCode::EV_REL(EV_REL::REL_RY)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // Pitch
        OutputAxisId(1),
        [
            js_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
            sp_axes[&EventCode::EV_REL(EV_REL::REL_RX)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // Roll
        OutputAxisId(2),
        // stick RZ, spacemouse RY, throttle n/a
        [
            js_axes[&EventCode::EV_ABS(EV_ABS::ABS_RZ)],
            sp_axes[&EventCode::EV_REL(EV_REL::REL_RY)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // Throttle/translate f/b
        OutputAxisId(3),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_Y)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_THROTTLE)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // translate l/r
        OutputAxisId(4),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_X)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // translate u/d
        OutputAxisId(5),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_Z)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
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

    loop {
        if let Ok(output) = output_r.recv() {
            dbg!(output);
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
