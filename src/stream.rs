#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
pub use win32::{Device, OutputStream};

#[derive(Debug)]
pub enum Format {
    F32,
    I16,
}
