# JoySharp sample

Build the native bridge from the repository root, then run the sample:

```powershell
cargo build -p joysharp-native --release
dotnet run --project dotnet/JoySharp.Sample
```

The program lists detected Nintendo HID devices, opens device `0` by default,
sets its first player light, briefly rumbles it, and prints each input report.
Pass a device index to select a different controller:

```powershell
dotnet run --project dotnet/JoySharp.Sample -- 1
```

On a right Joy-Con, add `--ir` to enable its infrared camera at 160×120. Each
input report then makes the latest grayscale frame available through
`TryGetInfraredFrame`:

```powershell
dotnet run --project dotnet/JoySharp.Sample -- --ir
```

To retain every frame for analysis, use `--ir-output` with an output directory.
The sample writes lossless binary PGM images (`frame-000000.pgm`, etc.); each
pixel is one infrared grayscale byte and the files can be opened by ImageMagick,
Python/Pillow, MATLAB, and similar tools.

```powershell
dotnet run --project dotnet/JoySharp.Sample -- --ir-output .\ir-frames
```
