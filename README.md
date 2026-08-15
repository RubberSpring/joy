# joy

Suite of tools and libraries for interactions with Nintendo Switch and DualShock 4 controllers.

fork made by RubberSpring, i dont know if this will get updates or something.

## External dependencies

On Linux, you'll need `libusb`, `libbluetooth` and `libudev`. On Ubuntu, you can install these by running:

```sh
sudo apt-get install libusb-1.0-0-dev libbluetooth-dev libudev-dev
```

## Tools

The tools can be run with `cargo run --bin <tool>`.

- `joytk`: main front-facing tool.
- `joy-infrared`: visualize the images captured by the infrared camera of the Joycon(R) as a realtime 3D view.

## Libraries

- [`joycon-sys`](https://yamakaky.github.io/joy/joycon_sys): decoding and encoding HID reports. Doesn't include any I/O.
- [`joycon`](https://yamakaky.github.io/joy/joycon): implements I/O and communication protocols on top of `joycon-sys`.
- [`dualshock`](https://yamakaky.github.io/joy/dualshock): decoding HID reports from the DS4 controller.
- [`hid-gamepad`](https://yamakaky.github.io/joy/hid_gamepad): abstraction above `dualshock` and `joycon`.

## C# / .NET

This fork includes `joysharp-native`, a stable native C ABI over `joycon`, and
`dotnet/JoySharp`, its .NET 8 wrapper. It supports Nintendo-controller
discovery, polling buttons/sticks/IMU/battery state, player LEDs, and rumble.

Build the native DLL with:

```powershell
cargo build -p joysharp-native --release
```

Then place the produced native library beside your .NET application and add a
project reference to `dotnet/JoySharp/JoySharp.csproj`. See the
[`JoySharp` wrapper README](dotnet/JoySharp/README.md) for a complete example.
