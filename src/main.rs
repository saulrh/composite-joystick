#![feature(iter_intersperse)]

use anyhow::{Context, Result};
use clap::Parser;
use evdev_rs::enums::{EventCode, EV_ABS, EV_KEY};
use evdev_rs::DeviceWrapper;
use std::collections::HashMap;
use std::io::Write;
use std::thread;

mod configuration;
mod gadget;
mod joystick_mux;
mod report;

use joystick_mux::{AxisUpdate, InputAxis, InputAxisId, JoystickId, OutputAxisId};

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
    Report { param: String, value: i64 },
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
    configuration::configure_mux(&mut mux, &js_axes, &th_axes, &sp_axes);

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
            let report = report::make_report(
                output
                    .axes
                    .into_iter()
                    .map(|(OutputAxisId(axis_id), value)| (axis_id, value)),
            );
            device
                .write(&report)
                .context("Failed to write to gadget device")?;
        }
    }
}

fn report(param: &str, value: &i64) -> Result<()> {
    let mut device = gadget::get_gadget_device().context("Failed to open gadget device")?;
    let mut state = HashMap::new();
    let param = match param {
        "x" => EventCode::EV_ABS(EV_ABS::ABS_X),
        "y" => EventCode::EV_ABS(EV_ABS::ABS_Y),
        "z" => EventCode::EV_ABS(EV_ABS::ABS_Z),
        "rx" => EventCode::EV_ABS(EV_ABS::ABS_RX),
        "ry" => EventCode::EV_ABS(EV_ABS::ABS_RY),
        "rz" => EventCode::EV_ABS(EV_ABS::ABS_RZ),
        "slider" => EventCode::EV_ABS(EV_ABS::ABS_THROTTLE),
        "dial" => EventCode::EV_ABS(EV_ABS::ABS_RUDDER),
        "hatx" => EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
        "haty" => EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
        "trigger" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER),
        "thumb" => EventCode::EV_KEY(EV_KEY::BTN_THUMB),
        "thumb2" => EventCode::EV_KEY(EV_KEY::BTN_THUMB2),
        "top" => EventCode::EV_KEY(EV_KEY::BTN_TOP),
        "top2" => EventCode::EV_KEY(EV_KEY::BTN_TOP2),
        "pinkie" => EventCode::EV_KEY(EV_KEY::BTN_PINKIE),
        "base" => EventCode::EV_KEY(EV_KEY::BTN_BASE),
        "base2" => EventCode::EV_KEY(EV_KEY::BTN_BASE2),
        "base3" => EventCode::EV_KEY(EV_KEY::BTN_BASE3),
        "base4" => EventCode::EV_KEY(EV_KEY::BTN_BASE4),
        "base5" => EventCode::EV_KEY(EV_KEY::BTN_BASE5),
        "base6" => EventCode::EV_KEY(EV_KEY::BTN_BASE6),
        "triggerhappy1" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY1),
        "triggerhappy2" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY2),
        "triggerhappy3" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY3),
        "triggerhappy4" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY4),
        "triggerhappy5" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY5),
        "triggerhappy6" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY6),
        "triggerhappy7" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY7),
        "triggerhappy8" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY8),
        "triggerhappy9" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY9),
        "triggerhappy10" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY10),
        "triggerhappy11" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY11),
        "triggerhappy12" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY12),
        "triggerhappy13" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY13),
        "triggerhappy14" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY14),
        "triggerhappy15" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY15),
        "triggerhappy16" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY16),
        "triggerhappy17" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY17),
        "triggerhappy18" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY18),
        "triggerhappy19" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY19),
        "triggerhappy20" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY20),
        "triggerhappy21" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY21),
        "triggerhappy22" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY22),
        "triggerhappy23" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY23),
        "triggerhappy24" => EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY24),
        _ => panic!("unknown axis"),
    };
    state.insert(param, *value);
    device
        .write(&report::make_report(state.into_iter()))
        .context("Failed to write to gadget device")?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::Init => gadget::init_gadget(),
        Command::Uninit => gadget::uninit_gadget(),
        Command::Run => run(),
        Command::Report { param, value } => report(param, value),
    }
}
