use crate::{Source, stream::Format};
use std::{any, ptr, ops};
use super::ffi::*;

const CLSCTX_ALL: u32 = 23; // (CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER)

const clsid: GUID = GUID { data1: 0xBCDE0395, data2: 0xE52F, data3: 0x467C, data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E] };
const imm_device_enumerator: GUID = GUID { data1: 0xA95664D2, data2: 0x9614, data3: 0x4F35, data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6] };
const iaudioclient: GUID = GUID { data1: 0x1CB9AD4C, data2: 0xDBFA, data3: 0x4c32, data4: [0xB1, 0x78, 0xC2, 0xF5, 0x68, 0xA7, 0x03, 0xB2] };
const iaudiorenderclient: GUID = GUID { data1: 0xf294acfc, data2: 0x3146, data3: 0x4483, data4: [0xa7, 0xbf, 0xad, 0xdc, 0xa7, 0xc2, 0x60, 0xe2] };

struct IPtr<T> {
    ptr: *mut T,
}

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

pub struct Device {
    audio_client_ptr: *mut IAudioClient,
    waveformat_ptr: *mut WAVEFORMATEX,
    sample_format: Format,
}

pub struct OutputStream {

}

impl Device {
    pub fn default() -> Option<Self> {
        unsafe {
            let mut enumerator: *mut IMMDeviceEnumerator = ptr::null_mut();
            let _err = CoInitializeEx(ptr::null_mut(), 0);
            let _err = CoCreateInstance(&clsid, ptr::null_mut(), CLSCTX_ALL, &imm_device_enumerator, (&mut enumerator) as *mut *mut _ as *mut LPVOID);
            let mut device: *mut IMMDevice = ptr::null_mut();
            let _err = (*enumerator).GetDefaultAudioEndpoint(0, 0, (&mut device) as _);
            let mut audio_client: *mut IAudioClient = ptr::null_mut();
            let _err = (*device).Activate((&iaudioclient) as *const GUID as _, CLSCTX_ALL, ptr::null_mut(), (&mut audio_client) as *mut *mut IAudioClient as _);
            let mut default_period: i64 = 0;
            let mut min_period: i64 = 0;
            let _err = (*audio_client).GetDevicePeriod(&mut default_period, &mut min_period);
            let mut format_info: *mut WAVEFORMATEX = ptr::null_mut();
            let _err = (*audio_client).GetMixFormat(&mut format_info);
            
            let sample_format = match ((*format_info).wFormatTag, (*format_info).wBitsPerSample) {
                (1, 16) => Format::I16,
                (3, 32) => Format::F32,
                (0xFFFE, 16) if (*(format_info as *mut WAVEFORMATEXTENSIBLE)).SubFormat == GUID { data1: 1, data2: 0, data3: 16, data4: [128, 0, 0, 170, 0, 56, 155, 113] } => Format::I16,
                (0xFFFE, 32) if (*(format_info as *mut WAVEFORMATEXTENSIBLE)).SubFormat == GUID { data1: 3, data2: 0, data3: 16, data4: [128, 0, 0, 170, 0, 56, 155, 113] } => Format::F32,
                _ => return None,
            };

            Some(Device {
                audio_client_ptr: audio_client,
                waveformat_ptr: format_info,
                sample_format,
            })
        }
    }
}

impl OutputStream {
    pub fn new(device: Device, mut source: impl Source + Send + 'static) -> Self {
        unsafe {
            let handle = CreateEventW(ptr::null_mut(), 0, 0, ptr::null());
            let _err = (*device.audio_client_ptr).SetEventHandle(handle);
            let mut buffer_frame_count: u32 = 0;
            let _err = (*device.audio_client_ptr).GetBufferSize(&mut buffer_frame_count);
            let mut render_client: *mut IAudioRenderClient = ptr::null_mut();
            let _err = (*device.audio_client_ptr).GetService((&iaudiorenderclient) as *const GUID as _, (&mut render_client) as *mut *mut IAudioRenderClient as _);
            let mut buffer_data: *mut u8 = ptr::null_mut();
            let _err = (*render_client).GetBuffer(buffer_frame_count, &mut buffer_data);

            let written_count = match device.sample_format {
                Format::F32 => {
                    let buf_slice = std::slice::from_raw_parts_mut(buffer_data as *mut f32, (buffer_frame_count * u32::from((*device.waveformat_ptr).nChannels)) as _);
                    source.write_samples(buf_slice)
                },
                Format::I16 => {
                    todo!()
                },
            };

            let written_count = (written_count / usize::from((*device.waveformat_ptr).nChannels)) as u32;
            let _err = (*render_client).ReleaseBuffer(written_count, 0);
            let _err = (*device.audio_client_ptr).Start();

            /*
            std::thread::spawn(move || {
                if written_count >= buffer_frame_count {
                    loop {
                        WaitForSingleObjectEx(handle, 0xFFFFFFFF, FALSE);
                        let mut padding: u32 = 0;
                        let _err = (*device.audio_client_ptr).GetCurrentPadding(&mut padding);
                        let frame_count = buffer_frame_count - padding;
                        if frame_count > 0 {
                            let _err = (*render_client).GetBuffer(frame_count, &mut buffer_data);

                            let written_count = match device.sample_format {
                                Format::F32 => {
                                    let buf_slice = std::slice::from_raw_parts_mut(buffer_data as *mut f32, (frame_count * u32::from((*device.waveformat_ptr).nChannels)) as _);
                                    source.write_samples(buf_slice)
                                },
                                Format::I16 => {
                                    todo!()
                                },
                            };

                            let written_frames = (written_count / usize::from((*device.waveformat_ptr).nChannels)) as u32;
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
                    let _err = (*device.audio_client_ptr).GetCurrentPadding(&mut padding);
                    let frame_count = buffer_frame_count - padding;
                    let _err = (*render_client).GetBuffer(frame_count, &mut buffer_data);
                    let _err = (*render_client).ReleaseBuffer(frame_count, 2); // AUDCLNT_BUFFERFLAGS_SILENT
                }
            });
            */

            todo!()
        }
    }
}
