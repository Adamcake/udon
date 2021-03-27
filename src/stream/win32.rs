use std::{ffi, ptr};

mod defs;

#[link(name = "Ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *const ffi::c_void, coinit: u32) -> u32;
    fn CoCreateInstance(rclsid: *const defs::Guid, outer: *const ffi::c_void, cls_context: u32, riid: *const defs::Guid, ppv: *mut *const ffi::c_void) -> u32;
}

#[link(name = "Kernel32")]
extern "system" {
    fn CreateEventW(lpEventAttributes: *const ffi::c_void, bManualReset: i32, bInitialState: i32, lpName: *const ffi::c_void) -> defs::HANDLE;
    fn WaitForSingleObject(handle: defs::HANDLE, ms: u32) -> u32;
}

pub struct Device {
    
}

pub struct OutputStream {

}

impl Device {
    pub fn default() -> Option<Self> {
        todo!()
    }
}