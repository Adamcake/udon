# hi

This is a hacked up fork of [libsoundio](https://github.com/andrewrk/libsoundio.git):

- changed CMakeLists.txt to work with MSVC 23

... combined with a modification of [libsoundio-sys](https://github.com/RamiHg/soundio-rs.git):

- changed build.rs to accomodate shipping sources
- apparently it's "stolen from libgit2-sys" so there's that too
- changed callback signature to `unsafe extern` instead of `extern`
- removed subprocessing git when it gets confused (lol)
