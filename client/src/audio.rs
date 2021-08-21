use std::{io::Cursor, sync::Arc};

use ahash::AHashMap;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub type SoundHandle = Sink;

/// Manages sounds.
pub struct Audio {
    stream: OutputStreamHandle,
    loaded_sounds: AHashMap<String, Arc<[u8]>>,

    // needs to be kept alive but has no other purpose
    _stream: OutputStream,
}

impl Audio {
    pub fn new() -> anyhow::Result<Self> {
        let (_stream, stream) = OutputStream::try_default()?;

        Ok(Self {
            _stream,
            stream,
            loaded_sounds: AHashMap::new(),
        })
    }

    pub fn add_sound(&mut self, id: &str, sound_encoded_bytes: &[u8]) {
        self.loaded_sounds
            .insert(id.to_owned(), Arc::<[u8]>::from(sound_encoded_bytes));
    }

    pub fn play(&self, id: &str, volume: f32) -> SoundHandle {
        let sink = self.play_inner(id, |sound_data| {
            Decoder::new(Cursor::new(sound_data))
                .unwrap_or_else(|e| panic!("sound '{}' is malformed: {}", id, e))
        });
        sink.set_volume(volume);
        sink
    }

    pub fn play_looping(&self, id: &str, volume: f32) -> SoundHandle {
        let sink = self.play_inner(id, |sound_data| {
            Decoder::new_looped(Cursor::new(sound_data))
                .unwrap_or_else(|e| panic!("sound '{}' is malformed: {}", id, e))
        });
        sink.set_volume(volume);
        sink
    }

    fn play_inner<
        Sample: rodio::Sample + Send,
        S: Source + Iterator<Item = Sample> + Send + 'static,
    >(
        &self,
        id: &str,
        build_source: impl Fn(Arc<[u8]>) -> S,
    ) -> SoundHandle {
        let sink = Sink::try_new(&self.stream).expect("failed to create new sound");
        let sound = self
            .loaded_sounds
            .get(id)
            .unwrap_or_else(|| panic!("missing sound '{}'", id));
        sink.append(build_source(Arc::clone(sound)));
        sink
    }
}
