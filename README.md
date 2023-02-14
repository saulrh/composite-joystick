# Overview

Composite Joystick uses the [Linux USB Gadget
system](https://docs.kernel.org/usb/gadget_configfs.html) to make a
Linux computer pretend to be a USB Joystick whose output is the
combination of several physical joysticks. Want to use a stick and
throttle but the game will only bind to a single joystick at a time?
Elite: Dangerous will only let you bind a single axis to pitch but you
want to be able to use either your joystick _or_ the 6DOF controller
that you usually use for CAD? Plug all your joysticks into your
Raspberry Pi and use Composite Joystick to combine them into one!

# Use and Configuration

Not particularly easy at the present moment, sadly, because I haven't
written a dynamic configuration system so the configuration is
hard-coded.

Plug everything together. Physical joysticks get plugged into normal
USB ports on the machine running Composite Joystick (the "JS
Host"). The JS Host will have a single special USB port with a UDC
(USB Device Controller) that allows it to act as a USB client rather
than a USB host; use this special port to connect it to a standard USB
port on the computer that you're actually using. For example, on a
Raspberry Pi 4B, the USB-C port has UDC so you'll use a USB-A to USB-C
cable to plug it into your gaming desktop, then you'll plug your
joysticks into the Raspberry Pi's four USB-A ports.

Install your toolchain: Rust. I recommend
[rustup](https://rustup.rs/).

Obtain the code:

```sh
git clone https://github.com/saulrh/composite-joystick.git
```

Edit the code to reflect your joysticks. In `main.rs`:

```rust
    let (js_idx, js_device, js_axes) =
        make_device("/dev/input/by-id/usb-Thrustmaster_T.16000M-event-joystick")
            .context("while opening joystick")?;
```

Find the evdev device for your joystick by inspecting
`/dev/input/by-id`. If you have less than three joysticks you'll also
want to change the total number of devices by removing one set of
`(idx, dev, axes)` variables, removing a `thread::spawn`, and removing
an argument from `configure_mux` in `configuration.rs`.

Edit the configuration. Open `configuration.rs` and, in
`configure_mux`, change the `configure_axis` invocations to bind
inputs to outputs. `AxisCombineFn::LargestMagnitude` takes the largest
value among all input axes as the output
value. `AxisCombineFn::Button` takes any nonzero value as a `1` and
all zero values as a `0`. You can find out how your joysticks' buttons
are mapped by running programs like `evtest` on the SBC that your
joysticks are all plugged into. Consult the giant match statement in
`report.rs` to determine what event codes to bind as outputs to drive
particular axes/buttons on the emulated composite joystick.

Handle quirks if necessary. In particular, `main.rs`,
`lower_bound_for` and `upper_bound_for` are used for devices that
report relative axes rather than absolute axes, such as mice and
3dconnexion spacemouse 6DOF controllers. These have "maximum" and
"minimum" values in practical terms, but relative axes report no
max/min values so we can't autoconfigure them like we can absolute
axes.

Build:

```sh
cargo build --release
```

Initialize the USB HID gadget:

```sh
sudo ./target/release/composite_joystick init
```

Run:

```sh
./target/release/composite_joystick run
```

# Current problems and plans

Some (many) games seem to stop paying attention after the twentieth or
thirtieth button (looking at you, Elite: Dangerous! you can't even pay
attention to enough buttons to bind everything meaningful on a single
stick!). In order to deal with this I'm going to have to get Composite
Joystick to pretend to be multiple joysticks on the output side. Which
isn't bad anyway, since it'll mean more axes to bind, which we're
running fairly low on.

Configuration is Painful right now. Being able to deserialize configs
from something like toml would be great. That would also make this
suitable for binary distribution and easy installation through
`crates.io`, which would be fantastic.

# Development Notes

I used [hrdc](https://github.com/nipo/hrdc/tree/master/hrdc) to
compile the report descriptor in `descriptor.hex`. If you change this,
in addition to updating the appropriate structures in `report.rs`, you
_must_ change the value of `report_length` in `gadget.rs`. Otherwise
you'll get really weird non-obvious misbehavior.

Best of luck if you decide to try changing the joystick report struct
in `report.rs`. The biggest problem: `endian` doesn't apply to arrays,
so there's Some Annoying Fanciness at the end of `make_report` to get
the bits in the right order.

