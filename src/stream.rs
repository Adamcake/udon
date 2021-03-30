use crate::{error::Error, source::Source};

macro_rules! backends {
    (
        $(
            $(#[$outer:meta])*
            mod $name:ident => $variant:ident if $cfg:meta
        ),* $(,)?
    ) => {
        const _CALL_BACKENDS_MACRO_ONLY_ONCE: () = ();

        $(
            #[cfg($cfg)]
            $(#[$outer])*
            mod $name;
        )*

        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum Backend {
            $(
                $(#[$outer])*
                $variant
            ),*
        }

        /// Handle to an audio input/output device.
        ///
        /// To acquire a [`Device`], request one from an [`Api`].
        pub struct Device(pub(crate) DeviceImpl);

        pub(crate) enum DeviceImpl {
            $(
                #[cfg($cfg)]
                $(#[$outer])*
                $variant ( $name::Device )
            ),*
        }

        /// Handle to an audio output stream.
        pub struct OutputStream(pub(crate) OutputStreamImpl);

        pub(crate) enum OutputStreamImpl {
            $(
                #[cfg($cfg)]
                $(#[$outer])*
                $variant ( $name::OutputStream )
            ),*
        }


        /// Represents a native API to request devices and streams from.
        pub struct Session(pub(crate) SessionImpl);

        pub(crate) enum SessionImpl {
            $(
                #[cfg($cfg)]
                $(#[$outer])*
                $variant ( $name::Session )
            ),*
        }

        impl Session {
            /// Creates an API handle for the selected backend, if available.
            pub fn new(backend: Backend) -> Option<Self> {
                match backend {
                    $(
                        Backend::$variant => {
                            #[cfg($cfg)]
                            { Some(Self(SessionImpl::$variant($name::Session::new()))) }
                            #[cfg(not($cfg))]
                            { None }
                        },
                    )*
                }
            }

            /// Creates an API handle for the default backend.
            pub fn default() -> Self {
                // TODO: Don't be stupid, and also document what the defaults are
                // Defaults shouldn't change with feature switches because that's non-additive
                Self::new(Backend::Wasapi).expect("no backends available (enable them via cargo features)")
            }
        }

        backend_wrap_fns! {
            impl Session(SessionImpl) <- $( $variant if $cfg ),* {
                pub fn default_output_device(&self) -> Option<Device>;

                pub fn open_output_stream(
                    &self,
                    device: Device,
                    source: impl Source + Send + 'static,
                ) -> Result<OutputStream, Error>;
            }
        }
    };
}

macro_rules! backend_wrap_fns {
    (
        impl $target:ident ( $enum:ident ) <- $( $variant:ident if $cfg:meta ),* {
            $( $rest:tt )*
        }
    ) => {
        impl $target {
            backend_wrap_fns!(@wrap $enum ( $($variant if $cfg),* ) $($rest)*);
        }
    };
    (
        @wrap $enum:ident ( $( $variant:ident if $cfg:meta ),* )
        $(#[$fn_outer:meta])*
        $v:vis fn $fn_name:ident ( & self $( , $arg_name:ident : $arg_ty:ty )* $(,)? ) $( -> $ret:ty )? ;
        $( $rest:tt )*
    ) => {
        $(#[$fn_outer])*
        $v fn $fn_name ( & self $( , $arg_name : $arg_ty)* ) $( -> $ret )? {
            macro_rules! _invoke_hack {
                ($imp:expr) => { $imp . $fn_name ( $($arg_name),* ) };
            }
            match self.0 {
                $(
                    #[cfg($cfg)]
                    $enum :: $variant (ref imp) => _invoke_hack!(imp)
                ),*
            }
        }
        backend_wrap_fns!(@wrap $enum ( $($variant if $cfg),* ) $($rest)*);
    };
    ( @wrap $enum:ident ( $( $variant:ident if $cfg:meta ),* ) ) => {};
}

backends! {
    /// Windows Audio Session API
    mod wasapi => Wasapi if all(target_os = "windows", feature = "wasapi"),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SampleFormat {
    // /// Unsigned 16-bit integer PCM
    // U16,

    /// Signed 16-bit integer PCM
    I16,

    /// IEEE 754 32-bit float PCM
    F32,
}
