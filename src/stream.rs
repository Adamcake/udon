#[cfg(all(target_os = "windows", feature = "wasapi"))]
mod wasapi;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Backend {
    WASAPI,
}

#[derive(Debug)]
pub enum Format {
    F32,
    I16,
}
