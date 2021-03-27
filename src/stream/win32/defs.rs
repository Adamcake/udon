use std::ffi;

pub type HRESULT = u32;
pub type HANDLE = *mut ffi::c_void;

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[repr(C)]
#[derive(Debug)]
pub struct WAVEFORMATEX {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub avg_bytes_per_sec: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub cb_size: u16,
    pub _union: u16,
    pub channel_mask: u32,
    pub sub_format: Guid,
}

#[repr(C)]
pub struct IUnknown(*const IUnknownVtable);

#[repr(C)]
pub struct IUnknownVtable {
    pub query_interface: unsafe extern "system" fn(this: *mut IUnknown, riid: *mut ffi::c_void, ppv: *mut *mut ffi::c_void) -> HRESULT,
    pub add_ref: unsafe extern "system" fn(this: *mut IUnknown) -> u32,
    pub release: unsafe extern "system" fn(this: *mut IUnknown) -> u32,
}

#[repr(C)]
pub struct IMMDeviceEnumerator(*const IMMDeviceEnumeratorVtable);

#[repr(C)]
pub struct IMMDeviceEnumeratorVtable {
    pub parent: IUnknownVtable,
    pub enum_audio_endpoints: unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, data_flow: u32, state_mask: u32, devices: *mut *mut IMMDeviceCollection) -> HRESULT,
    pub get_default_audio_endpoint: unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, data_flow: u32, role: u32, endpoint: *mut *mut IMMDevice) -> HRESULT,
    pub get_device: unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, str_id: *const u16, devices: *mut *mut IMMDevice) -> HRESULT,
    pub register_endpoint_notification_callback: unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, client: *mut ffi::c_void) -> HRESULT,
    pub unregister_endpoint_notification_callback: unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, client: *mut ffi::c_void) -> HRESULT,
}

#[repr(C)]
pub struct IMMDeviceCollection(*const IMMDeviceCollectionVtable);

#[repr(C)]
pub struct IMMDeviceCollectionVtable {
    pub parent: IUnknownVtable,
    pub get_count: unsafe extern "system" fn(this: *mut IMMDeviceCollection, devices: *const u32) -> HRESULT,
    pub item: unsafe extern "system" fn(this: *mut IMMDeviceCollection, device_index: u32, device: *mut *mut IMMDevice) -> HRESULT,
}

#[repr(C)]
pub struct IMMDevice(*const IMMDeviceVtable);

#[repr(C)]
pub struct IMMDeviceVtable {
    pub parent: IUnknownVtable,
    pub activate: unsafe extern "system" fn(this: *mut IMMDevice, iid: *mut ffi::c_void, clsctx: u32, activation_params: *mut ffi::c_void, interface: *mut *mut ffi::c_void) -> HRESULT,
    pub open_property_store: unsafe extern "system" fn(this: *mut IMMDevice, stgm_access: u32, properties: *mut *mut IPropertyStore) -> HRESULT,
    pub get_id: unsafe extern "system" fn(this: *mut IMMDevice, str_id: *mut *mut u16) -> HRESULT,
    pub get_state: unsafe extern "system" fn(this: *mut IMMDevice, state: *mut u32) -> HRESULT,
}

#[repr(C)]
pub struct IPropertyStore(*const IPropertyStoreVtable);

#[repr(C)]
pub struct IPropertyStoreVtable {
    pub parent: IUnknownVtable,
    pub get_count: unsafe extern "system" fn(this: *mut IPropertyStore, props: *mut u32) -> HRESULT,
    pub get_at: unsafe extern "system" fn(this: *mut IPropertyStore, prop: u32, pkey: *mut ffi::c_void) -> HRESULT,
    pub get_value: unsafe extern "system" fn(this: *mut IPropertyStore, key: *const ffi::c_void, pv: *mut PROPVARIANT) -> HRESULT,
    pub set_value: unsafe extern "system" fn(this: *mut IPropertyStore, key: *const ffi::c_void, propvar: *const ffi::c_void) -> HRESULT,
    pub commit: unsafe extern "system" fn(this: *mut IPropertyStore) -> HRESULT,
}

#[repr(C)]
pub struct PROPVARIANT {
    pub vt: u16,
    pub reserved1: u16,
    pub reserved2: u16,
    pub reserved3: u16,
    pub data: PROPVARIANT_data,
}

#[repr(C)]
pub struct PROPVARIANT_data;

#[repr(C)]
pub struct IAudioClient(*const IAudioClientVtable);

#[repr(C)]
pub struct IAudioClientVtable {
    pub parent: IUnknownVtable,
    pub initialize: unsafe extern "system" fn(this: *mut IAudioClient, share_mode: u32, stream_flags: u32, buffer_duration: i64, periodicity: i64, format: *const WAVEFORMATEX, AudioSessionGuid: *const Guid) -> HRESULT,
    pub get_buffer_size: unsafe extern "system" fn(this: *mut IAudioClient, frame_count: *mut u32) -> HRESULT,
    pub get_stream_latency: unsafe extern "system" fn(this: *mut IAudioClient, latency: *mut i64) -> HRESULT,
    pub get_current_padding: unsafe extern "system" fn(this: *mut IAudioClient, padding_frames: *mut u32) -> HRESULT,
    pub is_format_supported: unsafe extern "system" fn(this: *mut IAudioClient, share_mode: u32, format: *const WAVEFORMATEX, closest_match: *mut *mut WAVEFORMATEX) -> HRESULT,
    pub get_mix_format: unsafe extern "system" fn(this: *mut IAudioClient, device_format: *mut *mut WAVEFORMATEX) -> HRESULT,
    pub get_device_period: unsafe extern "system" fn(this: *mut IAudioClient, default_device_period: *mut i64, minimum_device_period: *mut i64) -> HRESULT,
    pub start: unsafe extern "system" fn(this: *mut IAudioClient) -> HRESULT,
    pub stop: unsafe extern "system" fn(this: *mut IAudioClient) -> HRESULT,
    pub reset: unsafe extern "system" fn(this: *mut IAudioClient) -> HRESULT,
    pub set_event_handle: unsafe extern "system" fn(this: *mut IAudioClient, event_handle: *mut ffi::c_void) -> HRESULT,
    pub get_service: unsafe extern "system" fn(this: *mut IAudioClient, riid: *const ffi::c_void, ppv: *mut ffi::c_void) -> HRESULT,
}

#[repr(C)]
pub struct IAudioRenderClient(*const IAudioRenderClientVtable);

#[repr(C)]
pub struct IAudioRenderClientVtable {
    pub parent: IUnknownVtable,
    pub get_buffer: unsafe extern "system" fn(this: *mut IAudioRenderClient, num_frames_requested: u32, data_ptr: *mut *mut u8) -> HRESULT,
    pub release_buffer: unsafe extern "system" fn(this: *mut IAudioRenderClient, num_frames_written: u32, flags: u32) -> HRESULT,
}



pub const CLSCTX_ALL: u32 = 23; // (CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER)

impl IMMDeviceEnumerator {
    // TODO
}

impl IMMDeviceCollection {
    // TODO
}

impl IMMDevice {
    // TODO
}

impl IAudioClient {
    // TODO
}

impl IAudioRenderClient {
    // TODO
}

macro_rules! iunknown_drop {
    ($t: ty) => {
        impl ::std::ops::Drop for $t {
            fn drop(&mut self) {
                unsafe { ((*self.0).parent.release)(self as *mut Self as _) };
            }
        }
    };
}
iunknown_drop!(IMMDeviceEnumerator);
iunknown_drop!(IMMDeviceCollection);
iunknown_drop!(IMMDevice);
iunknown_drop!(IAudioClient);
iunknown_drop!(IAudioRenderClient);
