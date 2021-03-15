pub use crate::Sample;

/// An audio source. Anything implementing this trait may be played to an output stream.
pub trait Source {
    /// Writes the next set of samples to an output buffer.
    /// The Source object is expected to "remember" its progress through the sound it's playing,
    /// such that it will continue where it left off on subsequent calls to this function.
    /// If the Source has multiple channels then the samples will be interleaved.
    /// Returns the number of samples which were written to the buffer. Values must be written contiguously from
    /// the start of the buffer. A value lower than buffer.len() indicates the sound has ended.
    fn write_samples(&mut self, buffer: &mut [Sample]) -> usize;

    /// Returns the number of channels in this Source object's audio data.
    fn channel_count(&self) -> usize;
}
