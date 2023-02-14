use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

static GADGET_DIR: &str = "/sys/kernel/config/usb_gadget/composite_joystick";

pub fn init_gadget() -> Result<()> {
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
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("report_length"),
        "22",
    )
    .context("Failed to set report length")?;
    let descriptor = include_str!("descriptor.hex");
    let descriptor = descriptor.replace(' ', "");
    fs::write(
        PathBuf::from(GADGET_DIR)
            .join("functions")
            .join("hid.usb0")
            .join("report_desc"),
        hex::decode(descriptor)?,
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

pub fn uninit_gadget() -> Result<()> {
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

pub fn get_gadget_device() -> io::Result<fs::File> {
    fs::File::create("/dev/hidg0")
}
