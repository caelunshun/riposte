//! C bindings to `rodio` for audio playback.

#![allow(clippy::clippy::missing_safety_doc)]

use std::io::Cursor;

use rodio::{Decoder, OutputStream, Sink};

pub struct OutputDevice {
    #[allow(unused)]
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
    volume: f32,
) -> *mut InstanceHandle {
    let sink = Sink::try_new(&device.handle).expect("failed to create sink");
    let source = Decoder::new(Cursor::new(sound.source.clone())).expect("failed to create decoder");
    sink.append(source);
    sink.set_volume(volume);
    sink.play();
    Box::leak(Box::new(InstanceHandle { sink })) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn rodio_stop_sound(sound: &mut InstanceHandle) {
    sound.sink.stop();
}

#[no_mangle]
pub unsafe extern "C" fn rodio_is_sound_done(sound: &mut InstanceHandle) -> bool {
    sound.sink.empty()
}

#[no_mangle]
pub unsafe extern "C" fn rodio_sound_set_volume(sound: &mut InstanceHandle, volume: f32) {
    sound.sink.set_volume(volume);
}

#[no_mangle]
pub unsafe extern "C" fn rodio_free_sound(sound: *mut InstanceHandle) {
    let _ = Box::from_raw(sound);
}

use std::ffi::CString;
use std::os::raw::c_char;

static DATA_DIR: once_cell::sync::Lazy<CString> = once_cell::sync::Lazy::new(|| {
    use directories_next::ProjectDirs;
    let project_dirs =
        ProjectDirs::from("me.caelunshun", "", "Riposte").expect("failed to get project dir");
    CString::new(
        project_dirs
            .data_dir()
            .to_str()
            .expect("path contains invalid UTF-8"),
    )
    .unwrap()
});

#[no_mangle]
pub unsafe extern "C" fn riposte_data_dir() -> *const c_char {
    DATA_DIR.as_ptr()
}
