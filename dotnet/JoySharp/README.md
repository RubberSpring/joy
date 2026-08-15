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

`JoyContext` must remain alive until every `JoyController` opened from it has
been disposed. Call controller methods from one thread at a time.
