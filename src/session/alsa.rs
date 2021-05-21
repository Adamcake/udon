use crate::{error::Error, source::{ChannelCount, SampleRate, Source}, session};
use std::num::{NonZeroU16, NonZeroU32};

pub struct Device {
    pcm: alsa_rs::pcm::PCM,
    sample_rate: SampleRate,
    channel_count: ChannelCount,
}

impl Device {
    pub fn channel_count(&self) -> ChannelCount {
        self.channel_count
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn default_output() -> Result<Self, Error> {
        let req_samplerate = 48000;
        let req_bufsize = 256;

        // Open the device
        let pcm = match alsa_rs::PCM::new("default", alsa_rs::Direction::Playback, false) {
            Ok(p) => p,
            Err(_) => return Err(Error::NoOutputDevice),
        };

        // Hardware parameters
        let hwp = alsa_rs::pcm::HwParams::any(&pcm).unwrap();
        hwp.set_channels(2).unwrap();
        hwp.set_rate(req_samplerate, alsa_rs::ValueOr::Nearest).unwrap();
        hwp.set_format(alsa_rs::pcm::Format::FloatLE).unwrap();
        hwp.set_access(alsa_rs::pcm::Access::RWInterleaved).unwrap(); // mmap would be better but doesn't work
        hwp.set_buffer_size(req_bufsize).unwrap();
        hwp.set_period_size(req_bufsize / 4, alsa_rs::ValueOr::Nearest).unwrap();
        pcm.hw_params(&hwp).unwrap();
        std::mem::drop(hwp); // because rust

        // Software parameters
        let hwp = pcm.hw_params_current().unwrap();
        let swp = pcm.sw_params_current().unwrap();
        swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
        swp.set_avail_min(hwp.get_period_size().unwrap()).unwrap();
        pcm.sw_params(&swp).unwrap();
        let rate = hwp.get_rate().ok().and_then(NonZeroU32::new).unwrap();
        std::mem::drop(swp); // because rust 2 electric boolgaoo
        std::mem::drop(hwp); // because rust (2016) (remastered for ps5)

        Ok(Self {
            pcm,
            sample_rate: rate,
            channel_count: unsafe { NonZeroU16::new_unchecked(2) },
        })
    }
}

pub struct OutputStream(Device);

impl OutputStream {
    pub fn new(device: session::Device) -> Result<session::OutputStream, Error> {
        match device {
            session::Device(session::DeviceImpl::Alsa(device)) => session_wrap!(Ok(Self(device)), OutputStream(OutputStreamImpl), Alsa),
            _ => todo!(), // TODO: what?
        }
    }

    pub fn play(&self, mut source: impl Source + Send + 'static) -> Result<(), Error> {
        // TODO: might it be better to use io.mmap here instead of io.writei?
        let io = self.0.pcm.io_f32().unwrap();
        let mut samples = Vec::with_capacity(96000);
        samples.resize_with(96000, Default::default);
        source.write_samples(&mut samples);
        let written = io.writei(&samples).unwrap();
        println!("Wrote {} samples?", written);
        dbg!(self.0.pcm.state());
        self.0.pcm.drain().unwrap();
        Ok(())
    }
}

pub struct Session;

impl Session {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn default_output_device(&self) -> Result<session::Device, Error> {
        session_wrap!(Device::default_output(), Device(DeviceImpl), Alsa)
    }

    pub fn open_output_stream(
        &self,
        device: session::Device,
    ) -> Result<session::OutputStream, Error> {
        OutputStream::new(device)
    }
}
