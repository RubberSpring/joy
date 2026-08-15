# JoySharp .NET wrapper

Build the native bridge from the repository root:

```powershell
cargo build -p joysharp-native --release
```

Place `joysharp_native.dll` (Windows), `libjoysharp_native.so` (Linux), or
`libjoysharp_native.dylib` (macOS) beside your application executable, then
reference this project. The wrapper uses source-generated P/Invoke and has no
managed dependencies.

```csharp
using JoySharp;

using var context = new JoyContext();
for (var i = 0; i < context.DeviceCount; i++)
    Console.WriteLine(context.GetDeviceInfo(i).ControllerKind);

using var controller = context.Open(0);
controller.SetPlayerLights(0b0001);
var state = controller.Read(); // blocks until a controller input report arrives
if (state.Buttons.HasFlag(JoySharpButtons.A))
controller.Rumble(0.5f);
```

The right Joy-Con's infrared camera is available through an opt-in API. After
each `Read`, retrieve a newly received grayscale frame as one byte per pixel:

```csharp
if (controller.SupportsInfrared)
{
    controller.EnableInfrared(JoySharpInfraredResolution.R160x120);
    controller.SetInfraredExposureMode(JoySharpInfraredExposureMode.Manual);
    controller.SetInfraredExposure(400); // 1-600 microseconds
    var state = controller.Read();
    if (controller.TryGetInfraredFrame(out var frame, out var pixels))
        Console.WriteLine($"{frame.Width}x{frame.Height}: {pixels.Length} bytes");
    controller.DisableInfrared();
}
```

Use `SetInfraredExposureMode(JoySharpInfraredExposureMode.Max)` to let the
sensor use its maximum exposure instead.

`JoyContext` must remain alive until every `JoyController` opened from it has
been disposed. Call controller methods from one thread at a time.
