use crate::{error::Error, session, source::Source};

pub struct Device;
pub struct OutputStream;
pub struct Session;

impl Session {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn default_output_device(&self) -> Option<session::Device> {
        session_wrap!(Some(Device), Device(DeviceImpl), Dummy)
    }

    pub fn open_output_stream(
        &self,
        _device: session::Device,
        _source: impl Source + Send + 'static,
    ) -> Result<session::OutputStream, Error> {
        session_wrap!(Ok(OutputStream), OutputStream(OutputStreamImpl), Dummy)
    }
}
