use crate::{Error, Sample, Source};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, PlayStreamError, SampleFormat, SupportedStreamConfigsError,
};
use std::sync::{Arc, Mutex};

/// An audio output stream which plays audio sources. Must be used with a Source object.
/// This object will be queried for samples to be played directly to the output device.
pub struct OutputStream<S>
where
    S: Source + Send + Sync + 'static,
{
    _stream: cpal::Stream,
    _source: Arc<Mutex<S>>,
    pub sample_rate: u32,
    pub channel_count: u16,
}

impl<S> OutputStream<S>
where
    S: Source + Send + Sync + 'static,
{
    /// Sets up and returns an OutputStream. Takes a closure which returns a Source, which will be used for
    /// continuous playback until the OutputStream is dropped. The Source must be thread-safe (Send + Sync)
    /// The params to the closure are (u16, u32) which represent the output's channel count and sample rate.
    pub fn with<F>(mixer_setup: F) -> Result<Self, Error>
    where
        F: FnOnce(u16, u32) -> S,
    {
        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err(Error::NoOutputDevice),
        };

        let mut supported_configs_range = match device.supported_output_configs() {
            Ok(r) => r,
            Err(SupportedStreamConfigsError::DeviceNotAvailable) => return Err(Error::DeviceNotAvailable),
            Err(SupportedStreamConfigsError::InvalidArgument) => return Err(Error::InvalidArgument),
            Err(SupportedStreamConfigsError::BackendSpecific { err }) => return Err(Error::CPALError(err)),
        };
        let supported_config = match supported_configs_range.next() {
            Some(c) => c,
            None => return Err(Error::DeviceNotUsable),
        }
        .with_max_sample_rate();

        let sample_rate = supported_config.sample_rate().0;
        let channel_count: u16 = supported_config.channels();

        let source = Arc::new(Mutex::new(mixer_setup(channel_count, sample_rate)));

        let f32_source = source.clone();
        let write_f32 = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            data.iter_mut().for_each(|s| *s = 0.0);
            f32_source.lock().unwrap().write_samples(data);
        };

        let i16_source = source.clone();
        let mut i16_buf: Vec<Sample> = Vec::new();
        let write_i16 = move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            i16_buf.clear();
            i16_buf.reserve(data.len());
            unsafe {
                i16_buf.set_len(data.len());
            }
            i16_buf.iter_mut().for_each(|s| *s = 0.0);
            i16_source.lock().unwrap().write_samples(&mut i16_buf);
            for (in_sample, out_sample) in i16_buf.iter().zip(data.iter_mut()) {
                *out_sample = (in_sample * f32::from(i16::MAX)) as i16;
            }
        };

        let u16_source = source.clone();
        let mut u16_buf: Vec<Sample> = Vec::new();
        let write_u16 = move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
            u16_buf.clear();
            u16_buf.reserve(data.len());
            unsafe {
                u16_buf.set_len(data.len());
            }
            u16_buf.iter_mut().for_each(|s| *s = 0.0);
            u16_source.lock().unwrap().write_samples(&mut u16_buf);
            for (in_sample, out_sample) in u16_buf.iter().zip(data.iter_mut()) {
                *out_sample = ((in_sample + 1.0) * f32::from(i16::MAX)) as u16;
            }
        };

        let sample_format = supported_config.sample_format();
        let config = supported_config.into();
        let stream = match match sample_format {
            SampleFormat::F32 => device.build_output_stream(&config, write_f32, err_fn),
            SampleFormat::I16 => device.build_output_stream(&config, write_i16, err_fn),
            SampleFormat::U16 => device.build_output_stream(&config, write_u16, err_fn),
        } {
            Ok(s) => s,
            Err(BuildStreamError::DeviceNotAvailable) => return Err(Error::DeviceNotAvailable),
            Err(BuildStreamError::StreamConfigNotSupported) => return Err(Error::DeviceNotUsable),
            Err(BuildStreamError::InvalidArgument) => return Err(Error::InvalidArgument),
            Err(BuildStreamError::StreamIdOverflow) => return Err(Error::StreamIdOverflow),
            Err(BuildStreamError::BackendSpecific { err }) => return Err(Error::CPALError(err)),
        };

        match stream.play() {
            Err(PlayStreamError::DeviceNotAvailable) => return Err(Error::DeviceNotAvailable),
            Err(PlayStreamError::BackendSpecific { err }) => return Err(Error::CPALError(err)),
            _ => (),
        }

        Ok(OutputStream { _stream: stream, _source: source, sample_rate, channel_count })
    }
}
