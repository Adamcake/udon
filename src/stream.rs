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
                Self::new(Backend::WASAPI).expect("no backends available (enable them via cargo features)")
            }
        }

        #[allow(unused_doc_comments)]
        impl Api {
            pub fn default_output_device(&self) -> Option<Device> {
                match self.0 {
                    $(
                        #[cfg($cfg)]
                        $(#[$outer])*
                        ApiImpl::$variant(ref imp) => imp.default_output_device(),
                    )*
                }
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

backends! {
    /// Windows Audio Session API
    mod wasapi => WASAPI if all(target_os = "windows", feature = "wasapi"),
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
