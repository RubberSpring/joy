//! A small, stable C ABI over the `joycon` crate.
//!
//! Keep Rust types behind opaque handles. This makes the library suitable for
//! P/Invoke and prevents Rust ABI or layout changes leaking into .NET callers.

use std::{
    cell::RefCell,
    ffi::c_char,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

use joycon::{
    hidapi::HidApi,
    joycon_sys::{
        light::{PlayerLight, PlayerLights},
        output::{RumbleData, RumbleSide},
        JOYCON_L_BT, JOYCON_R_BT, NINTENDO_VENDOR_ID, PRO_CONTROLLER,
    },
    JoyCon,
};

const OK: i32 = 0;
const ERROR: i32 = -1;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn set_error(message: impl ToString) -> i32 {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = message.to_string());
    ERROR
}

fn clear_error() {
    LAST_ERROR.with(|slot| slot.borrow_mut().clear());
}

fn ffi_result(f: impl FnOnce() -> Result<(), String>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => {
            clear_error();
            OK
        }
        Ok(Err(message)) => set_error(message),
        Err(_) => set_error("native library panicked"),
    }
}

/// An opaque HID discovery context. It must outlive controllers opened from it.
pub struct Context(HidApi);
pub struct Controller(JoyCon);

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct JoySharpDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    /// 1 = left Joy-Con, 2 = right Joy-Con, 3 = Pro Controller, 0 = other Nintendo HID device.
    pub controller_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct JoySharpMotionSample {
    pub acceleration_x: f32,
    pub acceleration_y: f32,
    pub acceleration_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct JoySharpState {
    /// Bits correspond to the JoySharpButtons enum in JoySharp.cs.
    pub buttons: u32,
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    /// 0 empty, 1 critical, 2 low, 3 medium, 4 full.
    pub battery_level: u8,
    pub is_charging: u8,
    pub is_connected: u8,
    pub _reserved: u8,
    pub motion: [JoySharpMotionSample; 3],
}

fn controller_kind(product_id: u16) -> u32 {
    match product_id {
        JOYCON_L_BT => 1,
        JOYCON_R_BT => 2,
        PRO_CONTROLLER => 3,
        _ => 0,
    }
}

fn device_at(context: &Context, index: usize) -> Result<joycon::hidapi::DeviceInfo, String> {
    context
        .0
        .device_list()
        .filter(|device| device.vendor_id() == NINTENDO_VENDOR_ID)
        .nth(index)
        .cloned()
        .ok_or_else(|| format!("Nintendo HID device index {index} was not found"))
}

/// Returns the ABI version. Increment only when an incompatible change is made.
#[no_mangle]
pub extern "C" fn joysharp_api_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn joysharp_context_create(out_context: *mut *mut Context) -> i32 {
    ffi_result(|| {
        if out_context.is_null() {
            return Err("out_context is null".into());
        }
        let api = HidApi::new().map_err(|e| e.to_string())?;
        unsafe {
            *out_context = Box::into_raw(Box::new(Context(api)));
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_context_destroy(context: *mut Context) {
    if !context.is_null() {
        drop(Box::from_raw(context));
    }
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_device_count(
    context: *const Context,
    out_count: *mut usize,
) -> i32 {
    ffi_result(|| {
        let context = context.as_ref().ok_or("context is null")?;
        let out_count = out_count.as_mut().ok_or("out_count is null")?;
        *out_count = context
            .0
            .device_list()
            .filter(|device| device.vendor_id() == NINTENDO_VENDOR_ID)
            .count();
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_device_get_info(
    context: *const Context,
    index: usize,
    out_info: *mut JoySharpDeviceInfo,
) -> i32 {
    ffi_result(|| {
        let context = context.as_ref().ok_or("context is null")?;
        let out_info = out_info.as_mut().ok_or("out_info is null")?;
        let device = device_at(context, index)?;
        *out_info = JoySharpDeviceInfo {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            controller_kind: controller_kind(device.product_id()),
        };
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_controller_open(
    context: *const Context,
    index: usize,
    out_controller: *mut *mut Controller,
) -> i32 {
    ffi_result(|| {
        let context = context.as_ref().ok_or("context is null")?;
        let out_controller = out_controller.as_mut().ok_or("out_controller is null")?;
        let device_info = device_at(context, index)?;
        if controller_kind(device_info.product_id()) == 0 {
            return Err("the selected Nintendo HID device is not a supported controller".into());
        }
        let device = device_info
            .open_device(&context.0)
            .map_err(|e| e.to_string())?;
        let mut controller = JoyCon::new(device, device_info).map_err(|e| e.to_string())?;
        controller.enable_imu().map_err(|e| e.to_string())?;
        controller.load_calibration().map_err(|e| e.to_string())?;
        *out_controller = Box::into_raw(Box::new(Controller(controller)));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_controller_destroy(controller: *mut Controller) {
    if !controller.is_null() {
        drop(Box::from_raw(controller));
    }
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_controller_read(
    controller: *mut Controller,
    out_state: *mut JoySharpState,
) -> i32 {
    ffi_result(|| {
        let controller = controller.as_mut().ok_or("controller is null")?;
        let out_state = out_state.as_mut().ok_or("out_state is null")?;
        let report = controller.0.tick().map_err(|e| e.to_string())?;
        let b = report.buttons;
        let mut buttons = 0u32;
        for (pressed, bit) in [
            (b.left.up(), 0),
            (b.left.down(), 1),
            (b.left.left(), 2),
            (b.left.right(), 3),
            (b.right.x(), 4),
            (b.right.b(), 5),
            (b.right.a(), 6),
            (b.right.y(), 7),
            (b.left.l(), 8),
            (b.right.r(), 9),
            (b.left.zl(), 10),
            (b.right.zr(), 11),
            (b.left.sl() || b.right.sl(), 12),
            (b.left.sr() || b.right.sr(), 13),
            (b.middle.lstick(), 14),
            (b.middle.rstick(), 15),
            (b.middle.minus(), 16),
            (b.middle.plus(), 17),
            (b.middle.capture(), 18),
            (b.middle.home(), 19),
        ] {
            if pressed {
                buttons |= 1 << bit;
            }
        }
        let mut state = JoySharpState {
            buttons,
            left_stick_x: report.left_stick.x as f32,
            left_stick_y: report.left_stick.y as f32,
            right_stick_x: report.right_stick.x as f32,
            right_stick_y: report.right_stick.y as f32,
            battery_level: report.info.battery_level() as u8,
            is_charging: report.info.charging() as u8,
            is_connected: report.info.connected() as u8,
            ..Default::default()
        };
        if let Some(samples) = report.imu {
            for (target, source) in state.motion.iter_mut().zip(samples) {
                target.acceleration_x = source.accel.x as f32;
                target.acceleration_y = source.accel.y as f32;
                target.acceleration_z = source.accel.z as f32;
                target.rotation_x = source.gyro.x as f32;
                target.rotation_y = source.gyro.y as f32;
                target.rotation_z = source.gyro.z as f32;
            }
        }
        *out_state = state;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_controller_set_player_lights(
    controller: *mut Controller,
    lights: u8,
) -> i32 {
    ffi_result(|| {
        let controller = controller.as_mut().ok_or("controller is null")?;
        let light = |bit| {
            if lights & (1u8 << bit) != 0u8 {
                PlayerLight::On
            } else {
                PlayerLight::Off
            }
        };
        controller
            .0
            .set_player_light(PlayerLights::new(light(0), light(1), light(2), light(3)))
            .map_err(|e| e.to_string())
    })
}

#[no_mangle]
pub unsafe extern "C" fn joysharp_controller_rumble(
    controller: *mut Controller,
    low_frequency: f32,
    high_frequency: f32,
    amplitude: f32,
) -> i32 {
    ffi_result(|| {
        let controller = controller.as_mut().ok_or("controller is null")?;
        let side = RumbleSide::from_freq(high_frequency, amplitude, low_frequency, amplitude);
        controller
            .0
            .set_rumble(RumbleData {
                left: side,
                right: side,
            })
            .map_err(|e| e.to_string())
    })
}

/// Copies the current thread's error message as UTF-8, including a trailing NUL when capacity permits.
/// Returns the message length excluding the trailing NUL.
#[no_mangle]
pub unsafe extern "C" fn joysharp_last_error(buffer: *mut c_char, capacity: usize) -> usize {
    LAST_ERROR.with(|slot| {
        let message = slot.borrow();
        if !buffer.is_null() && capacity != 0 {
            let bytes = message.as_bytes();
            let count = bytes.len().min(capacity - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), count);
            *buffer.add(count) = 0;
        }
        message.len()
    })
}
