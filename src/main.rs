use udon::session::{Session, Device, Api};

fn main() {
    let sesh = Session::new(Api::SoundIo).unwrap();
    let dev = sesh.default_output_device().unwrap();
    _ = dev;
}