use ahash::AHashMap;
use riposte_common::Era;

use crate::{
    audio::{SoundCategory, SoundHandle},
    context::Context,
    fair_random::FairRandomTable,
    game::Game,
    volumes,
};

// Plays music depending on the current era.
pub struct GameMusic {
    current_song: Option<SoundHandle>,
    current_song_era: Era,

    song_picker: Option<FairRandomTable>,
    era_songs: AHashMap<Era, Vec<String>>,
}

impl GameMusic {
    pub fn new(cx: &Context) -> Self {
        let mut era_songs: AHashMap<Era, Vec<String>> = AHashMap::new();

        for sound in cx.audio().sounds() {
            // Extract the era
            let mut components = sound.split("/");
            if matches!(components.next(), Some("music")) {
                if let Some(era) = components.next() {
                    let era = match era {
                        "ancient" => Era::Ancient,
                        "classical" => Era::Classical,
                        "medieval" => Era::Medieval,
                        "renaissance" => Era::Renaissance,
                        "industrial" => Era::Industrial,
                        "modern" => Era::Modern,
                        "future" => Era::Future,
                        _ => continue,
                    };

                    log::info!("Loaded song {} for era {:?}", sound, era);
                    era_songs.entry(era).or_default().push(sound.to_owned());
                }
            }
        }

        Self {
            current_song: None,
            current_song_era: Era::Ancient,
            song_picker: None,
            era_songs,
        }
    }

    pub fn update(&mut self, cx: &Context, game: &Game) {
        if self.current_song.is_none()
            || game.era() != self.current_song_era
            || self.current_song.as_ref().unwrap().empty()
        {
            self.pick_new_song(cx, game.era());
        }
    }

    fn pick_new_song(&mut self, cx: &Context, current_era: Era) {
        if self.current_song_era != current_era {
            self.song_picker = None;
        }

        let era_songs = &self.era_songs;
        let picker = self
            .song_picker
            .get_or_insert_with(|| FairRandomTable::new(era_songs[&current_era].len()));

        let index = picker.sample();

        let song = &self.era_songs[&current_era][index];

        log::info!("Set game music to {}", song);
        self.current_song = Some(
            cx.audio()
                .play(song, SoundCategory::Music, volumes::GAME_MUSIC),
        );
        self.current_song_era = current_era;
    }
}
