#[derive(Debug)]
pub enum Error {
    /// "catch-all" error type returned by CPAL in cases of unknown or unexpected errors
    CPALError(cpal::BackendSpecificError),

    /// The device no longer exists (ie. it has been disabled or unplugged)
    DeviceNotAvailable,

    /// The device doesn't support any of the playback configurations we can use
    DeviceNotUsable,

    /// An invalid argument was provided somewhere in the CPAL backend
    InvalidArgument,

    /// There is no output device available
    NoOutputDevice,

    /// Occurs if adding a new Stream ID would cause an integer overflow.
    StreamIdOverflow,
}
