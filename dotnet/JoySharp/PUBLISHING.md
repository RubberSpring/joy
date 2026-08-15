# Publishing JoySharp to NuGet

The first package targets `netstandard2.0` and includes the native Windows x64
library. A Windows machine with the Rust MSVC toolchain and the .NET SDK is
required to create it.

## Create a package

From the repository root, build and inspect the package:

```powershell
dotnet pack dotnet/JoySharp/JoySharp.csproj --configuration Release --output .\artifacts\nuget
```

`dotnet pack` automatically builds `joysharp_native.dll` for
`x86_64-pc-windows-msvc` and places it in the package under
`runtimes/win-x64/native`. To build that library without packing, run:

```powershell
.\dotnet\JoySharp\build-native.ps1
```
yay u build da package!!!! wow!!!