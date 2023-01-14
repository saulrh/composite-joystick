#![feature(iter_intersperse)]

use anyhow::{Context, Result};
use clap::Parser;
use crossbeam_channel::select;
use evdev_rs::enums::{EventCode, EV_ABS, EV_REL};
use evdev_rs::DeviceWrapper;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::thread;

mod joystick_mux;

use joystick_mux::{AxisCombineFn, AxisState, AxisUpdate, InputAxisId, JoystickId, OutputAxisId};

static GADGET_DIR: &'static str = "/sys/kernel/config/usb_gadget/composite_joystick";

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

fn init_gadget() -> Result<()> {
    // Make gadget dir
    fs::create_dir_all(PathBuf::from(GADGET_DIR)).context("Failed to create gadget dir")?;

    // Set IDs
    fs::write(PathBuf::from(GADGET_DIR).join("idVendor"), "0x1d6b")
        .context("Failed to write idVendor")?;
    fs::write(PathBuf::from(GADGET_DIR).join("idProduct"), "0x0104")
        .context("Failed to write idProduct")?;
    fs::write(PathBuf::from(GADGET_DIR).join("bcdDevice"), "0x0100")
        .context("Failed to write bcdDeviced")?;
    fs::write(PathBuf::from(GADGET_DIR).join("bcdUSB"), "0x0200")
        .context("Failed to write bcdUSB")?;

    // Set strings
    fs::create_dir_all(PathBuf::from(GADGET_DIR).join("strings").join("0x409"))
        .context("Failed to create strings dir")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("strings")
            .join("0x409")
            .join("manufacturer"),
        "Saul Reynolds-Haertle",
    )
    .context("Failed to write manufacturer")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("strings")
            .join("0x409")
            .join("product"),
        "Composite Joystick",
    )
    .context("Failed to write product")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("strings")
            .join("0x409")
            .join("serialnumber"),
        "ecd62a5ecbc8b29e",
    )
    .context("Failed to write serial")?;

    // Create function
    fs::create_dir_all(PathBuf::from(GADGET_DIR).join("functions").join("hid.usb0"))
        .context("Failed to create function")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("protocol"),
        "1",
    )
    .context("Failed to set protocol")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("subclass"),
        "1",
    )
    .context("Failed to set subclass")?;
    // TODO: what's our report length?
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("report_length"),
        "1",
    )
    .context("Failed to set report length")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("report_desc"),
        hex::decode(include_str!("descriptor.hex"))?,
    )
    .context("Failed to set report descriptor")?;

    // Create config
    fs::create_dir_all(PathBuf::from(GADGET_DIR).join("configs").join("c.1"))
        .context("Failed to create config dir")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("MaxPower"),
        "250",
    )
    .context("Failed to write MaxPower")?;
    fs::create_dir_all(
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("strings")
            .join("0x409"),
    )
    .context("Failed to create config strings dir")?;
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("strings")
            .join("0x409")
            .join("configuration"),
        "Config 1: Joystick",
    )
    .context("Failed to write config string")?;

    std::os::unix::fs::symlink(
        PathBuf::from(GADGET_DIR).join("functions").join("hid.usb0"),
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("hid.usb0"),
    )
    .context("Failed to symlink function into config dir")?;

    // write UDC
    let udcs: std::ffi::OsString = PathBuf::from("/sys/class/udc")
        .read_dir()
        .context("Failed to read /sys/class/udc")?
        .collect::<Result<Vec<_>, io::Error>>()
        .context("Failed to read child of /sys/class/udc")?
        .into_iter()
        .map(|de| -> std::ffi::OsString { de.file_name() })
        .intersperse(" ".into())
        .collect();
    fs::write(
        PathBuf::from(GADGET_DIR).join("UDC"),
        udcs.as_os_str().as_bytes(),
    )
    .context("Failed to write UDC")?;

    Ok(())
}

fn uninit_gadget() -> Result<()> {
    if !PathBuf::from(GADGET_DIR).exists() {
        // Already uninited
        return Ok(());
    }

    fs::write(PathBuf::from(GADGET_DIR).join("UDC"), "").context("Failed to clear UDC")?;
    fs::remove_file(
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("hid.usb0"),
    )
    .context("Failed to remove gadget symlink")?;
    fs::remove_dir(
        PathBuf::from(GADGET_DIR)
            .join("configs")
            .join("c.1")
            .join("strings")
            .join("0x409"),
    )
    .context("Failed to remove config strings dir")?;
    fs::remove_dir(PathBuf::from(GADGET_DIR).join("configs").join("c.1"))
        .context("Failed to remove config")?;
    fs::remove_dir(PathBuf::from(GADGET_DIR).join("functions").join("hid.usb0"))
        .context("Failed to remove function")?;
    fs::remove_dir(PathBuf::from(GADGET_DIR).join("strings").join("0x409"))
        .context("Failed to remove strings dir")?;
    fs::remove_dir(PathBuf::from(GADGET_DIR)).context("Failed to remove gadget")?;

    Ok(())
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
                    axis: InputAxisId {
                        joystick: id,
                        axis: ev.event_code,
                    },
                    state: AxisState(Some(ev.value.into())),
                })
                .expect("Failed to send");
        }
    }
}

fn run_gadget() -> Result<()> {
    let js_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-Thrustmaster_T.16000M-event-joystick",
    )
    .context("Failed to open joystick")?;
    let (js_s, js_r) = crossbeam_channel::bounded::<joystick_mux::AxisUpdate>(5);
    let js_axes = get_input_axes(&js_device, 0);

    let sp_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-3Dconnexion_SpaceMouse_Pro-event-mouse",
    )
    .context("Failed to open spacemouse")?;
    let (sp_s, sp_r) = crossbeam_channel::bounded::<joystick_mux::AxisUpdate>(5);
    let sp_axes = get_input_axes(&sp_device, 1);

    let th_device = evdev_rs::Device::new_from_path(
        "/dev/input/by-id/usb-Thrustmaster_TWCS_Throttle-event-joystick",
    )
    .context("Failed to open throttle")?;
    let (th_s, th_r) = crossbeam_channel::bounded::<joystick_mux::AxisUpdate>(5);
    let th_axes = get_input_axes(&th_device, 2);

    let (update_s, update_r) = crossbeam_channel::bounded::<()>(5);
    let mut mux = joystick_mux::JoystickMux::new(Some(update_s));
    mux.configure_axis(
        // Yaw
        OutputAxisId::new(0),
        [
            js_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
            sp_axes[&EventCode::EV_REL(EV_REL::REL_RY)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // Pitch
        OutputAxisId::new(1),
        [
            js_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
            sp_axes[&EventCode::EV_REL(EV_REL::REL_RX)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // Roll
        OutputAxisId::new(2),
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
        OutputAxisId::new(3),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_Y)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_THROTTLE)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // translate l/r
        OutputAxisId::new(4),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_X)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_X)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );
    mux.configure_axis(
        // translate u/d
        OutputAxisId::new(5),
        [
            sp_axes[&EventCode::EV_REL(EV_REL::REL_Z)],
            th_axes[&EventCode::EV_ABS(EV_ABS::ABS_Y)],
        ]
        .into_iter(),
        AxisCombineFn::LargestMagnitude,
    );

    thread::spawn(move || {
        handle_device(js_device, JoystickId(0), js_s);
    });

    thread::spawn(move || {
        handle_device(sp_device, JoystickId(1), sp_s);
    });

    thread::spawn(move || {
        handle_device(th_device, JoystickId(2), th_s);
    });

    thread::spawn(move || loop {
        if let Ok(()) = update_r.recv() {
            dbg!(mux.output(OutputAxisId::new(0)));
        }
    });

    loop {
        select! {
            recv(js_r) -> msg => if let Ok(update) = msg { mux.update(update); },
            recv(sp_r) -> msg => if let Ok(update) = msg { mux.update(update); },
            recv(th_r) -> msg => if let Ok(update) = msg { mux.update(update); },
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::Init => init_gadget(),
        Command::Uninit => uninit_gadget(),
        Command::Run => run_gadget(),
    }
}
