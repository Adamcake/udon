use crate::{error::Error, source::{ChannelCount, SampleRate, Source}, session::{self, SampleFormat}};
use std::{any, mem, ops, ptr, slice, sync::atomic::{self, AtomicBool}, thread};

use super::ffi::*;

const CLSCTX_ALL: u32 = 23; // (CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER)

pub struct Device {
    audio_client: IPtr<IAudioClient>,
    sample_format: SampleFormat,

    // Invariant: The channel count or sample rate must not be 0.
    wave_format: CoTaskMem<WAVEFORMATEX>,
}

impl Device {
    pub fn channel_count(&self) -> ChannelCount {
        unsafe {
            ChannelCount::new_unchecked((&*(self.wave_format.0)).nChannels)
        }
    }

    pub fn sample_rate(&self) -> SampleRate {
        unsafe {
            SampleRate::new_unchecked((&*(self.wave_format.0)).nSamplesPerSec)
        }
    }
}

impl Device {
    pub fn default_output() -> Option<Self> {
        unsafe {
            // TODO: "In Windows 8, the first use of IAudioClient
            // to access the audio device should be on the STA thread.
            // Calls from an MTA thread may result in undefined behavior."
            //
            // ^ WHAT

            let mut enumerator = IPtr::<IMMDeviceEnumerator>::null();
            let _err1 = CoInitializeEx(ptr::null_mut(), 0);
            let _err2 = CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                ptr::null_mut(),
                CLSCTX_ALL,
                &IID_IMMDeviceEnumerator,
                (&mut enumerator.ptr) as *mut *mut _ as *mut LPVOID,
            );

            // TODO: Check for DEVICE_STATE_NOTPRESENT or DEVICE_STATE_DISABLED or DEVICE_STATE_UNPLUGGED please
            // like if not here, in the general iterator
            let mut device = IPtr::<IMMDevice>::null();
            let _err3 = enumerator.GetDefaultAudioEndpoint(eRender, eConsole, &mut device.ptr); // TODO: eConsole

            // TODO: IAudioClient2, IAudioClient3
            let mut audio_client = IPtr::<IAudioClient>::null();
            let _err4 = device.Activate(
                (&IID_IAudioClient) as *const GUID as _,
                CLSCTX_ALL,
                ptr::null_mut(),
                (&mut audio_client.ptr) as *mut *mut IAudioClient as *mut LPVOID,
            );

            let mut wave_format = CoTaskMem::<WAVEFORMATEX>(ptr::null_mut());
            let _err6 = audio_client.GetMixFormat(&mut wave_format.0);

            // TODO: What about *unsigned* 16-bit?
            let format_info = &*wave_format.0;
            let sample_format = match (format_info.wFormatTag, format_info.wBitsPerSample) {
                (WAVE_FORMAT_PCM, 16) => SampleFormat::I16,
                (WAVE_FORMAT_IEEE_FLOAT, 32) => SampleFormat::F32,
                (WAVE_FORMAT_EXTENSIBLE, bps) => {
                    let format_info_extended = &*(wave_format.0 as *mut WAVEFORMATEXTENSIBLE);
                    match (&format_info_extended.SubFormat, bps) {
                        (x, 16) if x.eq(&KSDATAFORMAT_SUBTYPE_PCM) => SampleFormat::I16,
                        (x, 32) if x.eq(&KSDATAFORMAT_SUBTYPE_IEEE_FLOAT) => SampleFormat::F32,
                        _ => return None, // TODO: err
                    }
                },
                (_, _) => return None, // TODO: err
            };

            Some(Device {
                audio_client,
                sample_format,
                wave_format,
            })
        }
    }
}


pub struct OutputStream {
    //kill_switch: AtomicBool,
}

unsafe fn write_source(
    format: SampleFormat,
    buffer: *mut u8,
    sample_count: usize,
    source: &mut dyn Source,
) -> usize {
    match format {
        SampleFormat::I16 => todo!(), // TODO: big
        SampleFormat::F32 => {
            let buf = slice::from_raw_parts_mut(buffer as *mut f32, sample_count);
            source.write_samples(buf)
        },
    }
}

impl OutputStream {
    pub fn new(device: session::Device, source: impl Source + Send + 'static) -> Result<session::OutputStream, Error> {
        #[allow(irrefutable_let_patterns)] // TODO: yeah only wasapi right now
        unsafe {
            if let session::Device(session::DeviceImpl::Wasapi(device)) = device {
                // TODO: `Box::try_new` once `allocator_api` hits
                session_wrap!(Self::new_(device, Box::new(source)), OutputStream(OutputStreamImpl), Wasapi)
            } else {
                todo!("piss off");
            }
        }
    }

    unsafe fn new_(mut device: Device, mut source: Box<dyn Source + Send + 'static>) -> Result<Self, Error> {
        let mut default_period: REFERENCE_TIME = 0;
        let mut min_period: REFERENCE_TIME = 0;
        let _err_minus1 = device.audio_client.GetDevicePeriod(&mut default_period, &mut min_period);

        // initialize audio client
        let _err0 = device.audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
            0, // not in exclusive mode
            default_period,
            device.wave_format.0,
            ptr::null_mut(),
        );
        // create a nameless handle for audio render events, defaulted to non-signaled
        let event_handle = CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null());
        if event_handle.is_null() {
            panic!(); // TODO:
        }
        let _err2 = device.audio_client.SetEventHandle(event_handle);

        let mut buffer_frame_count: UINT32 = 0;
        let _err3 = device.audio_client.GetBufferSize(&mut buffer_frame_count);

        let mut render_client = IPtr::<IAudioRenderClient>::null();
        let _err4 = device.audio_client.GetService(
            &IID_IAudioRenderClient,
            &mut render_client.ptr as *mut *mut _ as *mut *mut c_void,
        );

        println!("ERR4: {}", _err4);

        // immediately write first chunk
        let mut buffer_data: *mut BYTE = ptr::null_mut();
        let _err5 = render_client.GetBuffer(buffer_frame_count, &mut buffer_data);

        let samples_to_write = (buffer_frame_count * UINT32::from((*device.wave_format.0).nChannels)) as usize;
        let samples_written = write_source(device.sample_format, buffer_data, samples_to_write, source.as_mut());
        let samples_written = (samples_written / (*device.wave_format.0).nChannels as usize) as UINT32;
        let _err6 = render_client.ReleaseBuffer(samples_written, 0);
        let _err7 = device.audio_client.Start();

        struct OutputStreamThreadData {
            buffer_frame_count: UINT32,
            device: Device,
            event_handle: HANDLE,
            samples_written: UINT32,
        }

        let data = OutputStreamThreadData {
            buffer_frame_count,
            event_handle,
            device,
            samples_written,
        };

        unsafe impl Send for OutputStreamThreadData {}
        let thread = thread::spawn(move || {
            // TODO: Maybe a bit too much, but it makes `loop {}` based testing nice :)
            SetThreadPriority(GetCurrentThread(), 15);

            let OutputStreamThreadData { buffer_frame_count, mut device, event_handle, samples_written } = data;
            if samples_written >= buffer_frame_count {
                loop {
                    let _err8 = WaitForSingleObjectEx(event_handle, INFINITE, FALSE);

                    let mut padding: UINT32 = 0;
                    let _err9 = device.audio_client.GetCurrentPadding(&mut padding);
                    let frame_count = buffer_frame_count - padding;

                    if frame_count > 0 {
                        let mut buffer_data: *mut BYTE = ptr::null_mut();
                        let _err10 = render_client.GetBuffer(frame_count, &mut buffer_data);
                        let samples_to_write = (frame_count * u32::from((*device.wave_format.0).nChannels)) as usize;
                        let frames_written = write_source(device.sample_format, buffer_data, samples_to_write, source.as_mut());
                        let frames_written = (frames_written / (*device.wave_format.0).nChannels as usize) as UINT32;

                        let _err11 = render_client.ReleaseBuffer(frames_written, 0);
                        if frames_written < frame_count {
                            break
                        }
                    }
                }
            }

            loop {
                let _err12 = WaitForSingleObjectEx(event_handle, INFINITE, FALSE);
                let mut padding: UINT32 = 0;
                let _err13 = device.audio_client.GetCurrentPadding(&mut padding);
                let frame_count = buffer_frame_count - padding;
                let mut buffer_data: *mut BYTE = ptr::null_mut();
                let _err14 = render_client.GetBuffer(frame_count, &mut buffer_data);
                let _err15 = render_client.ReleaseBuffer(frame_count, AUDCLNT_BUFFERFLAGS_SILENT);
            }

            // TODO: do this!
            // CloseHandle(event_handle);
        });

        Ok(Self {})
    }
}

pub struct Session;

impl Session {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn default_output_device(&self) -> Option<session::Device> {
        session_wrap!(Device::default_output(), Device(DeviceImpl), Wasapi)
    }

    pub fn open_output_stream(
        &self,
        device: session::Device,
        source: impl Source + Send + 'static,
    ) -> Result<session::OutputStream, Error> {
        OutputStream::new(device, source)
    }
}
