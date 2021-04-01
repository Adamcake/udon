use crate::{Sample, source::{ChannelCount, SampleRate, Source}};
use std::sync::mpsc::{self, Receiver, Sender};

const INIT_CAPACITY: usize = 16;

/// A simple additive mixer. Construct with `Mixer::new()`. This will return a Mixer and a MixerHandle.
/// The Mixer is a Source object, and intended to be attached (directly or indirectly) to an OutputStream
/// or any other place where a Source is expected.
/// The MixerHandle is kept and used for dynamically adding Sources to the Mixer.
pub struct Mixer {
    channels: ChannelCount,
    sample_rate: SampleRate,
    sources: Vec<Box<dyn Source + Send + Sync>>,
    input_buffer: Vec<Sample>,
    receiver: Receiver<Box<dyn Source + Send + Sync + 'static>>,
}

/// Returned from Mixer::new(), and permanently associated with the Mixer created alongside it.
/// Used for dynamically adding sounds to the Mixer with `handle.add()`
pub struct MixerHandle(Sender<Box<dyn Source + Send + Sync + 'static>>);

/// Error type for Mixer calls
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// Indicates that something could not be sent to the Mixer via a MixerHandle.
    /// This usually happens because the Mixer no longer exists.
    SendError,
}

impl Mixer {
    /// Constructs a new Mixer and MixerHandle.
    /// 
    /// `Mixer` does not make any changes to the channel count or sample rate of its Sources. As such, it needs to know
    /// its expected output rate and channel count on construction.
    /// 
    /// If Sources with a different sample rate or channel count than this are subsequently added to the Mixer,
    /// they will sound wrong. For resampling and rechanneling, see `Resampler` and `Rechanneler`.
    pub fn new(sample_rate: SampleRate, channels: ChannelCount) -> (Self, MixerHandle) {
        let (sender, receiver) = mpsc::channel();
        (
            Self { channels, sample_rate, sources: Vec::with_capacity(INIT_CAPACITY), input_buffer: Vec::new(), receiver },
            MixerHandle(sender),
        )
    }
}

impl Source for Mixer {
    fn write_samples(&mut self, buffer: &mut [Sample]) -> usize {
        buffer.iter_mut().for_each(|x| *x = 0.0);

        // Check for new sources...
        while let Ok(source) = self.receiver.try_recv() {
            self.sources.push(source);
        }

        let input_buffer = &mut self.input_buffer;

        self.sources.retain_mut(|source| {
            input_buffer.resize_with(buffer.len(), Default::default);
            let count = source.write_samples(input_buffer);

            for (in_sample, out_sample) in input_buffer.iter().take(count).copied().zip(buffer.iter_mut()) {
                *out_sample += in_sample;
            }

            count == input_buffer.len()
        });

        buffer.len()
    }

    fn channel_count(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

impl MixerHandle {
    /// Adds a Source to the Mixer associated with this handle. The Mixer will play the Source until it ends,
    /// then discard it.
    pub fn add(&self, source: impl Source + Send + Sync + 'static) -> Result<(), Error> {
        self.0.send(Box::new(source)).ok().ok_or(Error::SendError)
    }
}

trait RetainMut<T> {
    fn retain_mut(&mut self, f: impl FnMut(&mut T) -> bool);
}

impl<T> RetainMut<T> for Vec<T> {
    fn retain_mut(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
