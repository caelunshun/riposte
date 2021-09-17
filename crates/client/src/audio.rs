use std::{cell::RefCell, io::Cursor, rc::Rc, sync::Arc};

use ahash::AHashMap;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::options::Options;

pub type SoundHandle = Arc<Sink>;

/// A type of sound.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SoundCategory {
    /// Game or menu music
    Music,
    /// Sound effects
    Effects,
}

struct PlayingSound {
    sink: Arc<Sink>,
    category: SoundCategory,
    volume: f32,
}

impl PlayingSound {
    pub fn is_stopped(&self) -> bool {
        Arc::strong_count(&self.sink) == 1 || self.sink.empty()
    }
}

/// Manages sounds.
pub struct Audio {
    stream: OutputStreamHandle,
    loaded_sounds: AHashMap<String, Arc<[u8]>>,

    options: Rc<RefCell<Options>>,

    playing_sounds: RefCell<Vec<PlayingSound>>,

    // needs to be kept alive but has no other purpose
    _stream: OutputStream,
}

impl Audio {
    pub fn new(options: Rc<RefCell<Options>>) -> anyhow::Result<Self> {
        let (_stream, stream) = OutputStream::try_default()?;

        Ok(Self {
            _stream,
            stream,
            options,
            playing_sounds: RefCell::new(Vec::new()),
            loaded_sounds: AHashMap::new(),
        })
    }

    pub fn sounds(&self) -> impl Iterator<Item = &str> + '_ {
        self.loaded_sounds.keys().map(|s| s.as_str())
    }

    pub fn update(&mut self) {
        self.playing_sounds.borrow_mut().retain(|s| !s.is_stopped());
    }

    pub fn on_sound_options_changed(&self) {
        for sound in self.playing_sounds.borrow().iter() {
            sound
                .sink
                .set_volume(self.volume_for(sound.category, sound.volume));
        }
    }

    pub fn add_sound(&mut self, id: &str, sound_encoded_bytes: &[u8]) {
        self.loaded_sounds
            .insert(id.to_owned(), Arc::<[u8]>::from(sound_encoded_bytes));
    }

    pub fn play(&self, id: &str, category: SoundCategory, volume: f32) -> SoundHandle {
        let sink = self.play_inner(id, category, volume, |sound_data| {
            Decoder::new(Cursor::new(sound_data))
                .unwrap_or_else(|e| panic!("sound '{}' is malformed: {}", id, e))
        });

        sink
    }

    pub fn play_looping(&self, id: &str, category: SoundCategory, volume: f32) -> SoundHandle {
        let sink = self.play_inner(id, category, volume, |sound_data| {
            Decoder::new_looped(Cursor::new(sound_data))
                .unwrap_or_else(|e| panic!("sound '{}' is malformed: {}", id, e))
        });

        sink
    }

    fn volume_for(&self, category: SoundCategory, multiplier: f32) -> f32 {
        self.options.borrow().sound().volume(category) * multiplier
    }

    fn play_inner<
        Sample: rodio::Sample + Send,
        S: Source + Iterator<Item = Sample> + Send + 'static,
    >(
        &self,
        id: &str,
        category: SoundCategory,
        volume: f32,
        build_source: impl Fn(Arc<[u8]>) -> S,
    ) -> SoundHandle {
        let sink = Sink::try_new(&self.stream).expect("failed to create new sound");
        let sound = self
            .loaded_sounds
            .get(id)
            .unwrap_or_else(|| panic!("missing sound '{}'", id));
        sink.append(build_source(Arc::clone(sound)));

        sink.set_volume(self.volume_for(category, volume));

        let handle = Arc::new(sink);
        self.playing_sounds.borrow_mut().push(PlayingSound {
            sink: Arc::clone(&handle),
            category,
            volume,
        });

        handle
    }
}
