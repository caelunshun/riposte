//! C bindings to `rodio` for audio playback.

#![allow(clippy::clippy::missing_safety_doc)]

use std::io::Cursor;

use rodio::{Decoder, OutputStream, Sink};

pub struct OutputDevice {
    stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
}

#[no_mangle]
pub unsafe extern "C" fn rodio_new() -> *mut OutputDevice {
    let (stream, handle) = OutputStream::try_default().expect("failed to initialize audio");
    Box::leak(Box::new(OutputDevice { stream, handle })) as *mut _
}

pub struct SoundHandle {
    source: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn rodio_create_sound(data: *const u8, length: usize) -> *mut SoundHandle {
    let data = std::slice::from_raw_parts(data, length).to_vec();
    Box::leak(Box::new(SoundHandle { source: data })) as *mut _
}

#[derive(Debug)]
pub struct InstanceSettings {
    volume: f32,
}

pub struct InstanceHandle {
    sink: Sink,
}

#[no_mangle]
pub unsafe extern "C" fn rodio_start_sound(
    device: &mut OutputDevice,
    sound: &SoundHandle,
) -> *mut InstanceHandle {
    let sink = Sink::try_new(&device.handle).expect("failed to create sink");
    let source = Decoder::new(Cursor::new(sound.source.clone())).expect("failed to create decoder");
    sink.append(source);
    sink.play();
    Box::leak(Box::new(InstanceHandle { sink })) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn rodio_stop_sound(sound: &mut InstanceHandle) {
    sound.sink.stop();
}

#[no_mangle]
pub unsafe extern "C" fn rodio_sound_is_done(sound: &mut InstanceHandle) -> bool {
    sound.sink.empty()
}
