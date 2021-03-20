use crate::{Sample, Source};

const INIT_CAPACITY: usize = 16;

/// A simple additive mixer.
/// TODO: can't be used yet, document this when it can be used...
pub struct Mixer {
    channels: usize,
    sources: Vec<Box<dyn Source + Send + Sync>>,
    input_buffer: Vec<Sample>,
}

impl Mixer {
    /// Constructs a new Mixer. `channels` is the number of channels wanted in the output data.
    pub fn new(channels: usize) -> Self {
        Self { channels, sources: Vec::with_capacity(INIT_CAPACITY), input_buffer: Vec::new() }
    }
}

impl Source for Mixer {
    fn write_samples(&mut self, buffer: &mut [Sample]) -> usize {
        let input_buffer = &mut self.input_buffer;
        let output_channel_count = self.channels;

        self.sources.retain_mut(|source| {
            let source_channel_count = source.channel_count();
            input_buffer.resize_with(buffer.len() * source_channel_count / output_channel_count, Default::default);
            let count = source.write_samples(input_buffer);

            if source_channel_count == output_channel_count {
                // Firstly, if the input and output channel counts are the same, pass straight through.
                for (in_sample, out_sample) in input_buffer[..count].iter().copied().zip(buffer.iter_mut()) {
                    *out_sample += in_sample;
                }
            } else if source_channel_count == 1 {
                // Next, if the input is 1-channel, duplicate the next sample across all output channels.
                for (in_sample, out_samples) in
                    input_buffer[..count].iter().copied().zip(buffer.chunks_exact_mut(output_channel_count))
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
