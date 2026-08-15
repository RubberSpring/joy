using JoySharp;

var useInfrared = false;
string? infraredOutputDirectory = null;
var selectedIndex = 0;

for (var argumentIndex = 0; argumentIndex < args.Length; argumentIndex++)
{
    switch (args[argumentIndex])
    {
        case "--ir":
            useInfrared = true;
            break;
        case "--ir-output":
            if (++argumentIndex >= args.Length)
            {
                Console.Error.WriteLine("--ir-output requires a directory path.");
                return;
            }
            infraredOutputDirectory = args[argumentIndex];
            useInfrared = true;
            break;
        default:
            if (!int.TryParse(args[argumentIndex], out selectedIndex))
            {
                Console.Error.WriteLine($"Unknown argument: {args[argumentIndex]}");
                return;
            }
            break;
    }
}

using var context = new JoyContext();
Console.WriteLine($"Nintendo HID devices found: {context.DeviceCount}");

for (var index = 0; index < context.DeviceCount; index++)
{
    var device = context.GetDeviceInfo(index);
    Console.WriteLine($"[{index}] {ControllerName(device.ControllerKind)} " +
                      $"(VID: {device.VendorId:X4}, PID: {device.ProductId:X4})");
}

if (context.DeviceCount == 0)
{
    Console.WriteLine("Connect or pair a Joy-Con or Pro Controller, then run this sample again.");
    return;
}

if (selectedIndex < 0 || selectedIndex >= context.DeviceCount)
{
    Console.Error.WriteLine($"Device index must be between 0 and {context.DeviceCount - 1}.");
    return;
}

using var controller = context.Open(selectedIndex);
controller.SetPlayerLights(0b0001);
controller.Rumble(0.25f);
if (useInfrared)
{
    if (!controller.SupportsInfrared)
    {
        Console.Error.WriteLine("The selected controller does not have an infrared camera.");
        return;
    }
    controller.EnableInfrared(JoySharpInfraredResolution.R160x120);
    Console.WriteLine("Infrared camera enabled at 160x120.");
    if (infraredOutputDirectory is not null)
    {
        Directory.CreateDirectory(infraredOutputDirectory);
        Console.WriteLine($"Writing infrared frames to {Path.GetFullPath(infraredOutputDirectory)}");
    }
}
Console.WriteLine("Reading controller input. Press Ctrl+C to exit.");

var stopping = false;
var infraredFrameNumber = 0;
Console.CancelKeyPress += (_, eventArgs) =>
{
    eventArgs.Cancel = true;
    stopping = true;
};

while (!stopping)
{
    // Read blocks until the controller sends its next input report.
    var state = controller.Read();
    var infrared = "";
    if (useInfrared && controller.TryGetInfraredFrame(out var frame, out var pixels))
    {
        infrared = $" IR={frame.Width}x{frame.Height} ({pixels.Length} grayscale bytes)";
        if (infraredOutputDirectory is not null)
        {
            var outputPath = Path.Combine(infraredOutputDirectory, $"frame-{infraredFrameNumber++:D6}.pgm");
            WritePgm(outputPath, frame, pixels);
        }
    }
    Console.WriteLine(
        $"Buttons={state.Buttons,-18} " +
        $"L=({state.LeftStickX,6:F0}, {state.LeftStickY,6:F0}) " +
        $"R=({state.RightStickX,6:F0}, {state.RightStickY,6:F0}) " +
        $"Battery={state.BatteryLevel}/4 " +
        $"Charging={state.IsCharging != 0}{infrared}");
}

return;

static string ControllerName(uint kind) => kind switch
{
    1 => "Left Joy-Con",
    2 => "Right Joy-Con",
    3 => "Pro Controller",
    _ => "Nintendo HID device",
};

static void WritePgm(string path, InfraredFrameInfo frame, byte[] pixels)
{
    var header = System.Text.Encoding.ASCII.GetBytes($"P5\n{frame.Width} {frame.Height}\n255\n");
    var file = new byte[header.Length + pixels.Length];
    Buffer.BlockCopy(header, 0, file, 0, header.Length);
    Buffer.BlockCopy(pixels, 0, file, header.Length, pixels.Length);
    File.WriteAllBytes(path, file);
}
