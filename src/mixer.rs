use crate::{Sample, source::Source};
use std::sync::mpsc::{self, Receiver, Sender};

const INIT_CAPACITY: usize = 16;

/// A simple additive mixer. Construct with `Mixer::new()`. This will return a Mixer and a MixerHandle.
/// The Mixer is a Source object, and intended to be attached (directly or indirectly) to an OutputStream
/// or any other place where a Source is expected.
/// The MixerHandle is kept and used for dynamically adding Sources to the Mixer.
pub struct Mixer {
    channels: usize,
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
    /// Constructs a new Mixer and MixerHandle. `channels` is the number of channels wanted in the output data.
    pub fn new(channels: usize) -> (Self, MixerHandle) {
        let (sender, receiver) = mpsc::channel();
        (
            Self { channels, sources: Vec::with_capacity(INIT_CAPACITY), input_buffer: Vec::new(), receiver },
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
        let output_channel_count = self.channels;

        self.sources.retain_mut(|source| {
            let source_channel_count = source.channel_count();
            input_buffer.resize_with(buffer.len() * source_channel_count / output_channel_count, Default::default);
            let count = source.write_samples(input_buffer);

            if source_channel_count == output_channel_count {
                // Firstly, if the input and output channel counts are the same, pass straight through.
                for (in_sample, out_sample) in input_buffer.iter().take(count).copied().zip(buffer.iter_mut()) {
                    *out_sample += in_sample;
                }
            } else if source_channel_count == 1 {
                // Next, if the input is 1-channel, duplicate the next sample across all output channels.
                for (in_sample, out_samples) in
                    input_buffer.iter().take(count).copied().zip(buffer.chunks_exact_mut(output_channel_count))
                {
                    out_samples.iter_mut().for_each(|s| *s = in_sample);
                }
            } else {
                // Different multi-channel counts. What do we do here!?
                todo!("multi-channel mixing")
            }

            count == input_buffer.len()
        });

        buffer.len()
    }

    fn channel_count(&self) -> usize {
        self.channels
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
