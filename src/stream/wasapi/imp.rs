use crate::{source::Source, stream::{self, SampleFormat}};
use std::{any, mem, ops, ptr, thread};
use super::ffi::*;

const CLSCTX_ALL: u32 = 23; // (CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER)

struct CoTaskMem<T>(*mut T);

unsafe impl<T> Send for CoTaskMem<T> {}

impl<T> ops::Drop for CoTaskMem<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.0 as LPVOID);
        }
    }
}

struct IPtr<T> {
    ptr: *mut T,
}

unsafe impl<T> Send for IPtr<T> {}

impl<T> IPtr<T> {
    fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }
}

impl<T> ops::Deref for IPtr<T> {
    type Target = T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        if !self.ptr.is_null() {
            unsafe { &*self.ptr }
        } else {
            panic!("{} deref when null", any::type_name::<Self>());
        }
        #[cfg(not(debug_assertions))]
        unsafe { &*self.ptr }
    }
}

impl<T> ops::DerefMut for IPtr<T> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    fn deref_mut(&mut self) -> &mut <Self as ops::Deref>::Target {
        #[cfg(debug_assertions)]
        if !self.ptr.is_null() {
            unsafe { &mut *self.ptr }
        } else {
            panic!("{} deref-mut when null", any::type_name::<Self>());
        }
        #[cfg(not(debug_assertions))]
        unsafe { &mut *self.ptr }
    }
}

impl<T> ops::Drop for IPtr<T> {
    #[inline]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ptr::drop_in_place(self.ptr) }
        }
    }
}

pub struct Api;

impl Api {
    pub fn new() -> Self {
        Self
    }

    pub fn default_output_device(&self) -> Option<stream::Device> {
        Device::default_output().map(|dev| stream::Device(stream::DeviceImpl::Wasapi(dev)))
    }
}

pub struct Device {
    audio_client: IPtr<IAudioClient>,
    sample_format: SampleFormat,
    wave_format: CoTaskMem<WAVEFORMATEX>,
}

// impl stream::DeviceImpl for Device {

// }

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

            let mut default_period: REFERENCE_TIME = 0;
            let mut min_period: REFERENCE_TIME = 0;
            let _err5 = audio_client.GetDevicePeriod(&mut default_period, &mut min_period);

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

}

impl OutputStream {
    pub fn new(mut device: Device, mut source: impl Source + Send + 'static) -> Self {
        unsafe {
            let handle = CreateEventW(ptr::null_mut(), 0, 0, ptr::null());
            let _err = device.audio_client.SetEventHandle(handle);
            let mut buffer_frame_count: u32 = 0;
            let _err = device.audio_client.GetBufferSize(&mut buffer_frame_count);
            let mut render_client: *mut IAudioRenderClient = ptr::null_mut();
            let _err = device.audio_client.GetService((&IID_IAudioRenderClient) as *const GUID as _, (&mut render_client) as *mut *mut IAudioRenderClient as _);
            let mut buffer_data: *mut u8 = ptr::null_mut();
            let _err = (*render_client).GetBuffer(buffer_frame_count, &mut buffer_data);

            let written_count = match device.sample_format {
                SampleFormat::F32 => {
                    let buf_slice = std::slice::from_raw_parts_mut(buffer_data as *mut f32, (buffer_frame_count * u32::from((*device.wave_format.0).nChannels)) as _);
                    source.write_samples(buf_slice)
                },
                SampleFormat::I16 => {
                    todo!()
                },
            };

            let written_count = (written_count / usize::from((*device.wave_format.0).nChannels)) as u32;
            let _err = (*render_client).ReleaseBuffer(written_count, 0);
            let _err = device.audio_client.Start();

            /*
            thread::spawn(move || {
                if written_count >= buffer_frame_count {
                    loop {
                        WaitForSingleObjectEx(handle, 0xFFFFFFFF, FALSE);
                        let mut padding: u32 = 0;
                        let _err = device.audio_client.GetCurrentPadding(&mut padding);
                        let frame_count = buffer_frame_count - padding;
                        if frame_count > 0 {
                            let _err = (*render_client).GetBuffer(frame_count, &mut buffer_data);

                            let written_count = match device.sample_format {
                                SampleFormat::F32 => {
                                    let buf_slice = std::slice::from_raw_parts_mut(buffer_data as *mut f32, (frame_count * u32::from((*device.wave_format.0).nChannels)) as _);
                                    source.write_samples(buf_slice)
                                },
                                SampleFormat::I16 => {
                                    todo!()
                                },
                            };

                            let written_frames = (written_count / usize::from((*device.wave_format.0).nChannels)) as u32;
                            let _err = (*render_client).ReleaseBuffer(written_frames, 0);

                            if written_frames < frame_count {
                                break;
                            }
                        }
                    }
                }

                loop {
                    WaitForSingleObjectEx(handle, 0xFFFFFFFF, FALSE);
                    let mut padding: u32 = 0;
                    let _err = device.audio_client.GetCurrentPadding(&mut padding);
                    let frame_count = buffer_frame_count - padding;
                    let _err = (*render_client).GetBuffer(frame_count, &mut buffer_data);
                    let _err = (*render_client).ReleaseBuffer(frame_count, 2); // AUDCLNT_BUFFERFLAGS_SILENT
                }
            });*/
            todo!()
        }
    }
}
