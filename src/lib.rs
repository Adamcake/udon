mod error;
pub mod mixer;
pub mod resampler;
pub mod source;
mod stream;

pub use error::Error;
pub use resampler::Resampler;
pub use source::Source;
pub use stream::OutputStream;

pub type SAMPLE = f32;

/// A basic sound-playing object. When fed to an output stream, will play the samples it contains until it has no more.
/// If the samples have a different sample rate than the output stream, the output will sound sped up or slowed down.
/// Use a resampler (such as boop::resampler::Polyphase, or implement your own) to resample it at the correct rate.
pub struct Player {
    samples: Box<[SAMPLE]>,
    channels: usize,
    offset: usize,
}

impl Player {
    pub fn new(samples: Box<[SAMPLE]>, channels: usize) -> Self {
        Self { samples, channels, offset: 0 }
    }
}

impl Source for Player {
    fn write_samples(&mut self, buffer: &mut [SAMPLE]) -> usize {
        let old_offset = self.offset;
        self.offset += buffer.len();
        if let Some(i) = self.samples.get(old_offset..self.offset) {
            buffer.copy_from_slice(i);
            buffer.len()
        } else {
            if let Some(i) = self.samples.get(old_offset..) {
                buffer[..i.len()].copy_from_slice(i);
                i.len()
            } else {
                0
            }
        }
    }

    fn channel_count(&self) -> usize {
        self.channels
    }
}
