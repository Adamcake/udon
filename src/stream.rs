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

        /// Represents a native API to request devices and streams from.
        pub struct Api(pub(crate) ApiImpl);

        pub(crate) enum ApiImpl {
            $(
                #[cfg($cfg)]
                $(#[$outer])*
                $variant ( $name::Api )
            ),*
        }

        impl Api {
            /// Creates an API handle for the selected backend, if available.
            pub fn new(backend: Backend) -> Option<Self> {
                match backend {
                    $(
                        Backend::$variant => {
                            #[cfg($cfg)]
                            { Some(Self(ApiImpl::$variant($name::Api::new()))) }
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
            impl Api(ApiImpl) <- $( $variant if $cfg ),* {
                fn default_output_device(&self) -> Option<Device>;
            }
        }

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

        pub struct OutputStream(pub(crate) OutputStreamImpl);

        pub(crate) enum OutputStreamImpl {
            $(
                #[cfg($cfg)]
                $(#[$outer])*
                $variant ( $name::OutputStream )
            ),*
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
            backend_wrap_fns!(@wrap $enum $($variant if $cfg),* $($rest)*);
        }
    };
    (
        @wrap
        $enum:ident $( $variant:ident if $cfg:meta ),*
        $(#[$fn_outer:meta])*
        fn $fn_name:ident ( & self $( , $arg_name:ident : $arg_ty:ty )* ) -> $( $ret:ty )? ;
        $( $rest:tt )*
    ) => {
        $(#[$fn_outer])*
        fn $fn_name ( & self $( , $arg_name : $arg_ty)* ) $(-> $ret)? {
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
        backend_wrap_fns!(@wrap $enum $($variant if $cfg),* $($rest)*);
    };
    ( @wrap $enum:ident $( $variant:ident if $cfg:meta ),* ) => {};
}

backends! {
    /// Windows Audio Session API
    mod wasapi => Wasapi if all(target_os = "windows", feature = "wasapi"),
}

#[derive(Debug)]
pub enum SampleFormat {
    // /// Unsigned 16-bit integer PCM
    // U16,

    // Signed 16-bit integer PCM
    I16,

    // IEEE single-precision float PCM
    F32,
}
