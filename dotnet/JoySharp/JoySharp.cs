using System;
using System.Runtime.InteropServices;

namespace JoySharp;

[Flags]
public enum JoySharpButtons : uint
{
    Up = 1 << 0, Down = 1 << 1, Left = 1 << 2, Right = 1 << 3,
    X = 1 << 4, B = 1 << 5, A = 1 << 6, Y = 1 << 7,
    L = 1 << 8, R = 1 << 9, ZL = 1 << 10, ZR = 1 << 11,
    SL = 1 << 12, SR = 1 << 13, LeftStick = 1 << 14, RightStick = 1 << 15,
    Minus = 1 << 16, Plus = 1 << 17, Capture = 1 << 18, Home = 1 << 19,
}

public enum JoySharpInfraredResolution : uint
{
    R320x240 = 0,
    R160x120 = 1,
    R80x60 = 2,
    R40x30 = 3,
}

public enum JoySharpInfraredExposureMode : uint
{
    Manual = 0,
    Max = 1,
}

[StructLayout(LayoutKind.Sequential)]
public struct DeviceInfo { public ushort VendorId, ProductId; public uint ControllerKind; }
[StructLayout(LayoutKind.Sequential)]
public struct MotionSample { public float AccelerationX, AccelerationY, AccelerationZ, RotationX, RotationY, RotationZ; }
[StructLayout(LayoutKind.Sequential)]
public struct InfraredFrameInfo { public uint Width, Height; public UIntPtr ByteCount; }
[StructLayout(LayoutKind.Sequential)]
public struct ControllerState
{
    public JoySharpButtons Buttons;
    public float LeftStickX, LeftStickY, RightStickX, RightStickY;
    public byte BatteryLevel, IsCharging, IsConnected, Reserved;
    public MotionSample Motion0, Motion1, Motion2;
}

public sealed class JoyContext : IDisposable
{
    private IntPtr _handle;
    public JoyContext() { Native.Check(Native.ContextCreate(out _handle)); }
    public int DeviceCount { get { Native.Check(Native.DeviceCount(_handle, out var count)); return checked((int)count.ToUInt64()); } }
    public DeviceInfo GetDeviceInfo(int index) { Native.Check(Native.DeviceGetInfo(_handle, ToSize(index), out var info)); return info; }
    public JoyController Open(int index) { Native.Check(Native.ControllerOpen(_handle, ToSize(index), out var controller)); return new JoyController(controller); }
    private static UIntPtr ToSize(int value) => new UIntPtr(checked((ulong)value));
    public void Dispose() { if (_handle != IntPtr.Zero) { Native.ContextDestroy(_handle); _handle = IntPtr.Zero; GC.SuppressFinalize(this); } }
}

public sealed class JoyController : IDisposable
{
    private IntPtr _handle;
    internal JoyController(IntPtr handle) => _handle = handle;
    public ControllerState Read() { Native.Check(Native.ControllerRead(_handle, out var state)); return state; }
    public bool SupportsInfrared { get { Native.Check(Native.ControllerSupportsInfrared(_handle, out var supported)); return supported != 0; } }
    public void EnableInfrared(JoySharpInfraredResolution resolution = JoySharpInfraredResolution.R160x120) => Native.Check(Native.ControllerEnableInfrared(_handle, resolution));
    public void DisableInfrared() => Native.Check(Native.ControllerDisableInfrared(_handle));
    public void SetInfraredExposureMode(JoySharpInfraredExposureMode mode) => Native.Check(Native.ControllerSetInfraredExposureMode(_handle, mode));
    public void SetInfraredExposure(uint microseconds) => Native.Check(Native.ControllerSetInfraredExposure(_handle, microseconds));
    public bool TryGetInfraredFrame(out InfraredFrameInfo info, out byte[] pixels)
    {
        Native.Check(Native.ControllerInfraredFrameInfo(_handle, out info));
        if (info.ByteCount == UIntPtr.Zero) { pixels = Array.Empty<byte>(); return false; }
        pixels = new byte[checked((int)info.ByteCount.ToUInt64())];
        Native.Check(Native.ControllerCopyInfraredFrame(_handle, pixels, info.ByteCount));
        return true;
    }
    public void SetPlayerLights(byte mask) => Native.Check(Native.SetPlayerLights(_handle, mask));
    public void Rumble(float amplitude, float lowFrequency = 160, float highFrequency = 320) => Native.Check(Native.Rumble(_handle, lowFrequency, highFrequency, amplitude));
    public void Dispose() { if (_handle != IntPtr.Zero) { Native.ControllerDestroy(_handle); _handle = IntPtr.Zero; GC.SuppressFinalize(this); } }
}

internal static partial class Native
{
    private const string Library = "joysharp_native";
    internal static void Check(int result) { if (result != 0) throw new InvalidOperationException(GetLastError()); }
    private static unsafe string GetLastError()
    {
        var length = LastError(null, UIntPtr.Zero).ToUInt64();
        var buffer = new byte[checked((int)length + 1)];
        fixed (byte* pointer = buffer) { LastError(pointer, new UIntPtr((uint)buffer.Length)); }
        return System.Text.Encoding.UTF8.GetString(buffer, 0, checked((int)length));
    }
    [DllImport(Library, EntryPoint = "joysharp_context_create", CallingConvention = CallingConvention.Cdecl)] internal static extern int ContextCreate(out IntPtr context);
    [DllImport(Library, EntryPoint = "joysharp_context_destroy", CallingConvention = CallingConvention.Cdecl)] internal static extern void ContextDestroy(IntPtr context);
    [DllImport(Library, EntryPoint = "joysharp_device_count", CallingConvention = CallingConvention.Cdecl)] internal static extern int DeviceCount(IntPtr context, out UIntPtr count);
    [DllImport(Library, EntryPoint = "joysharp_device_get_info", CallingConvention = CallingConvention.Cdecl)] internal static extern int DeviceGetInfo(IntPtr context, UIntPtr index, out DeviceInfo info);
    [DllImport(Library, EntryPoint = "joysharp_controller_open", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerOpen(IntPtr context, UIntPtr index, out IntPtr controller);
    [DllImport(Library, EntryPoint = "joysharp_controller_destroy", CallingConvention = CallingConvention.Cdecl)] internal static extern void ControllerDestroy(IntPtr controller);
    [DllImport(Library, EntryPoint = "joysharp_controller_read", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerRead(IntPtr controller, out ControllerState state);
    [DllImport(Library, EntryPoint = "joysharp_controller_set_player_lights", CallingConvention = CallingConvention.Cdecl)] internal static extern int SetPlayerLights(IntPtr controller, byte lights);
    [DllImport(Library, EntryPoint = "joysharp_controller_rumble", CallingConvention = CallingConvention.Cdecl)] internal static extern int Rumble(IntPtr controller, float lowFrequency, float highFrequency, float amplitude);
    [DllImport(Library, EntryPoint = "joysharp_controller_supports_infrared", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerSupportsInfrared(IntPtr controller, out byte supported);
    [DllImport(Library, EntryPoint = "joysharp_controller_enable_infrared", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerEnableInfrared(IntPtr controller, JoySharpInfraredResolution resolution);
    [DllImport(Library, EntryPoint = "joysharp_controller_disable_infrared", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerDisableInfrared(IntPtr controller);
    [DllImport(Library, EntryPoint = "joysharp_controller_infrared_frame_info", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerInfraredFrameInfo(IntPtr controller, out InfraredFrameInfo info);
    [DllImport(Library, EntryPoint = "joysharp_controller_copy_infrared_frame", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerCopyInfraredFrame(IntPtr controller, [Out] byte[] buffer, UIntPtr capacity);
    [DllImport(Library, EntryPoint = "joysharp_controller_set_infrared_exposure_mode", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerSetInfraredExposureMode(IntPtr controller, JoySharpInfraredExposureMode mode);
    [DllImport(Library, EntryPoint = "joysharp_controller_set_infrared_exposure", CallingConvention = CallingConvention.Cdecl)] internal static extern int ControllerSetInfraredExposure(IntPtr controller, uint microseconds);
    [DllImport(Library, EntryPoint = "joysharp_last_error", CallingConvention = CallingConvention.Cdecl)] private static unsafe extern UIntPtr LastError(byte* buffer, UIntPtr capacity);
}
