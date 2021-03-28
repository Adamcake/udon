#![allow(bad_style, dead_code)]

// C Types
use core::ffi::c_void;
pub type c_char = i8;
pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
pub type c_long = i32;
pub type c_ulong = u32;
pub type c_longlong = i64;
pub type c_ulonglong = u64;
pub type wchar_t = u16;

// Windows Types
pub type __int64 = i64;
pub type AUDCLNT_SHAREMODE = u32;
pub type BOOL = c_int;
pub type BYTE = c_uchar;
pub type DWORD = c_ulong;
pub type EDataFlow = u32;
pub type ERole = u32;
pub type HRESULT = u32;
pub type HANDLE = *mut c_void;
pub type IID = GUID;
pub type LONGLONG = __int64;
pub type LPCGUID = *const GUID;
pub type LPCWSTR = *const WCHAR;
pub type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;
pub type LPUNKNOWN = *mut IUnknown;
pub type LPVOID = *mut c_void;
pub type LPWSTR = *mut WCHAR;
pub type REFERENCE_TIME = LONGLONG;
pub type REFCLSID = *const IID;
pub type REFIID = *const IID;
pub type REFPROPERTYKEY = *const PROPERTYKEY;
pub type REFPROPVARIANT = *const PROPVARIANT;
pub type UINT = c_uint;
pub type UINT32 = c_uint;
pub type ULONG = c_ulong;
pub type WCHAR = wchar_t;
pub type WORD = c_ushort;

// Windows Constants
pub const AUDCLNT_SHAREMODE_SHARED: AUDCLNT_SHAREMODE = 0;
pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 0;

// Windows Structs & Unions
#[repr(C)]
pub struct GUID {
    pub data1: c_ulong,
    pub data2: c_ushort,
    pub data3: c_ushort,
    pub data4: [c_uchar; 8],
}
#[repr(C)]
pub struct PROPERTYKEY {
    fmtid: GUID,
    pid: DWORD,
}
#[repr(C)]
pub struct WAVEFORMATEX {
    pub wFormatTag: WORD,
    pub nChannels: WORD,
    pub nSamplesPerSec: DWORD,
    pub nAvgBytesPerSec: DWORD,
    pub nBlockAlign: WORD,
    pub wBitsPerSample: WORD,
    pub cbSize: WORD,
}
#[repr(C)]
pub struct WAVEFORMATEXTENSIBLE {
    pub Format: WAVEFORMATEX,
    pub Samples: WAVEFORMATEXTENSIBLE_Samples,
    pub dwChannelMask: DWORD,
    pub SubFormat: GUID,
}
#[repr(C)]
pub union WAVEFORMATEXTENSIBLE_Samples {
    pub wValidBitsPerSample: WORD,
    pub wSamplesPerBlock: WORD,
    pub wReserved: WORD,
}

// Unused structs we don't define
#[repr(C)]
pub struct IMMNotificationClient { _placeholder: *const c_void }
#[repr(C)]
pub struct PROPVARIANT { _placeholder: *const c_void }
#[repr(C)]
pub struct SECURITY_ATTRIBUTES { _placeholder: *const c_void }

// Windows Functions (Static Linking)
#[link(name = "Kernel32")]
extern "system" {
    fn CreateEventW(
        lpEventAttributes: LPSECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR
    ) -> HANDLE;
    fn WaitForSingleObjectEx(hHandle: HANDLE, dwMilliseconds: DWORD, bAlertable: BOOL) -> DWORD;
}
#[link(name = "Ole32")]
extern "system" {
    fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: DWORD) -> HRESULT;
    fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: LPUNKNOWN,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}

// Windows COM Interfaces
#[repr(C)]
pub struct IUnknown(*const IUnknownVtable);
#[repr(C)]
pub struct IUnknownVtable {
    pub QueryInterface: unsafe extern "system" fn(this: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
}
macro_rules! com_interface {
    (
        $(
            $(#[$outer:meta])*
            $v:vis interface $name:ident ( $vt_name:ident ) {
                $(
                    $(#[$fn_outer:meta])*
                    fn $fn_name:ident ( $( $arg_name:ident : $arg_ty:ty ),* $(,)? ) -> $ret:ty;
                )*
            }
        )*
    ) => {
        $(
            $(#[$outer])*
            #[repr(C)]
            $v struct $name(pub *const $vt_name);

            #[repr(C)]
            $v struct $vt_name {
                __iunknown_vtable: IUnknownVtable,
                $( $fn_name: unsafe extern "system" fn( *mut $name , $($arg_ty),* ) -> $ret ),*
            }

            impl $name {
                $(
                    $(#[$fn_outer])*
                    #[inline]
                    pub unsafe fn $fn_name ( &mut self , $( $arg_name : $arg_ty ),* ) -> $ret {
                        ((*self.0).$fn_name)( self , $( $arg_name ),* )
                    }
                )*
            }

            impl ::core::ops::Drop for $name {
                fn drop(&mut self) {
                    unsafe {
                        ((*self.0).__iunknown_vtable.Release)(self as *mut Self as *mut IUnknown);
                    }
                }
            }
        )*
    };
}

com_interface! {
    pub interface IMMDeviceEnumerator(IMMDeviceEnumeratorVtable) {
        fn EnumAudioEndpoints(
            dataFlow: EDataFlow,
            dwStateMask: DWORD,
            ppDevices: *mut *mut IMMDeviceCollection,
        ) -> HRESULT;
        fn GetDefaultAudioEndpoint(dataFlow: EDataFlow, role: ERole, ppEndpoint: *mut *mut IMMDevice) -> HRESULT;
        fn GetDevice(pwstrId: LPCWSTR, ppDevices: *mut *mut IMMDevice) -> HRESULT;
        fn RegisterEndpointNotificationCallback(pClient: *mut IMMNotificationClient) -> HRESULT;
        fn UnregisterEndpointNotificationCallback(pClient: *mut IMMNotificationClient) -> HRESULT;
    }

    pub interface IMMDeviceCollection(IMMDeviceCollectionVtable) {
        fn GetCount(pcDevices: *const UINT) -> HRESULT;
        fn Item(nDevice: UINT, ppDevice: *mut *mut IMMDevice) -> HRESULT;
    }

    pub interface IMMDevice(IMMDeviceVtable) {
        fn Activate(
            iid: REFIID,
            dwClsCtx: DWORD,
            pActivationParams: *mut PROPVARIANT,
            ppInterface: *mut LPVOID,
        ) -> HRESULT;
        fn OpenPropertyStore(stgmAccess: DWORD, ppProperties: *mut *mut IPropertyStore) -> HRESULT;
        fn GetId(ppstrId: *mut LPWSTR) -> HRESULT;
        fn GetState(pdwState: *mut DWORD) -> HRESULT;
    }

    pub interface IPropertyStore(IPropertyStoreVtable) {
        fn GetCount(cProps: *mut DWORD) -> HRESULT;
        fn GetAt(iProp: DWORD, pkey: *mut PROPERTYKEY) -> HRESULT;
        fn GetValue(key: REFPROPERTYKEY, pv: *mut PROPVARIANT) -> HRESULT;
        fn SetValue(key: REFPROPERTYKEY, propvar: REFPROPVARIANT) -> HRESULT;
        fn Commit() -> HRESULT;
    }

    pub interface IAudioClient(IAudioClientVtable) {
        fn Initialize(
            ShareMode: AUDCLNT_SHAREMODE,
            StreamFlags: DWORD,
            hnsBufferDuration: REFERENCE_TIME,
            hnsPeriodicity: REFERENCE_TIME,
            pFormat: *const WAVEFORMATEX,
             AudioSessionGuid: LPCGUID,
        ) -> HRESULT;
        fn GetBufferSize(pNumBufferFrames: *mut UINT32) -> HRESULT;
        fn GetStreamLatency(phnsLatency: *mut REFERENCE_TIME) -> HRESULT;
        fn GetCurrentPadding(pNumPaddingFrames: *mut UINT32) -> HRESULT;
        fn IsFormatSupported(
            ShareMode: AUDCLNT_SHAREMODE,
            pFormat: *const WAVEFORMATEX,
            ppClosestMatch: *mut *mut WAVEFORMATEX,
        ) -> HRESULT;
        fn GetMixFormat(ppDeviceFormat: *mut *mut WAVEFORMATEX) -> HRESULT;
        fn GetDevicePeriod(
            phnsDefaultDevicePeriod: *mut REFERENCE_TIME,
            phnsMinimumDevicePeriod: *mut REFERENCE_TIME,
        ) -> HRESULT;
        fn Start() -> HRESULT;
        fn Stop() -> HRESULT;
        fn Reset() -> HRESULT;
        fn SetEventHandle(eventHandle: HANDLE) -> HRESULT;
        fn GetService(riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    }

    pub interface IAudioRenderClient(IAudioRenderClientVtable) {
        fn GetBuffer(NumFramesRequested: UINT32, ppData: *mut *mut BYTE) -> HRESULT;
        fn ReleaseBuffer(NumFramesWritten: UINT32, dwFlags: DWORD) -> HRESULT;
    }
}
