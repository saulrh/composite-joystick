#![feature(iter_intersperse)]

use anyhow::{Context, Result};
use clap::Parser;
use evdev_rs::enums::EventCode;
use evdev_rs::DeviceWrapper;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
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

static DEVICE_INDEX_SEQ: Mutex<u16> = Mutex::new(0);
fn make_device<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<(u16, evdev_rs::Device, HashMap<EventCode, InputAxis>)> {
    let mut idx = DEVICE_INDEX_SEQ.lock().unwrap();
    let dev = evdev_rs::Device::new_from_path(path).context("failed to open device")?;
    let axes = get_input_axes(&dev, *idx);
    *idx += 1;
    return Ok((*idx, dev, axes));
}

fn run() -> Result<()> {
    let (update_s, update_r) = crossbeam_channel::bounded::<joystick_mux::AxisUpdate>(5);
    let (output_s, output_r) = crossbeam_channel::bounded::<joystick_mux::OutputState>(5);

    let (js_idx, js_device, js_axes) =
        make_device("/dev/input/by-id/usb-Thrustmaster_T.16000M-event-joystick")
            .context("while opening joystick")?;

    let (sp_idx, sp_device, sp_axes) =
        make_device("/dev/input/by-id/usb-3Dconnexion_SpaceMouse_Pro-event-mouse")
            .context("while opening spacemouse")?;

    let (th_idx, th_device, th_axes) =
        make_device("/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick")
            .context("while opening throttle")?;

    let mut mux = joystick_mux::JoystickMux::new(Some(output_s));
    configuration::configure_mux(&mut mux, &js_axes, &th_axes, &sp_axes);

    let js_s = update_s.clone();
    thread::spawn(move || {
        handle_device(js_device, JoystickId(js_idx), js_s);
    });

    let sp_s = update_s.clone();
    thread::spawn(move || {
        handle_device(sp_device, JoystickId(sp_idx), sp_s);
    });

    let th_s = update_s.clone();
    thread::spawn(move || {
        handle_device(th_device, JoystickId(th_idx), th_s);
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

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::Init => gadget::init_gadget(),
        Command::Uninit => gadget::uninit_gadget(),
        Command::Run => run(),
    }
}
