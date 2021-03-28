#[cfg(all(target_os = "windows", feature = "wasapi"))]
mod wasapi;

#[derive(Debug)]
pub enum Format {
    F32,
    I16,
}
