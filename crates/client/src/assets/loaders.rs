use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc, sync::Arc};

use dume::{Canvas, SpriteData, SpriteDescriptor};
use serde::de::DeserializeOwned;

use crate::audio::Audio;

use super::Loader;

pub struct ImageLoader {
    canvas: Rc<RefCell<Canvas>>,
}

impl ImageLoader {
    pub fn new(canvas: Rc<RefCell<Canvas>>) -> Self {
        Self { canvas }
    }
}

impl Loader for ImageLoader {
    fn load_from_bytes(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.canvas.borrow_mut().create_sprite(SpriteDescriptor {
            name: id,
            data: SpriteData::Encoded(bytes),
        });
        Ok(None)
    }
}

pub struct FontLoader {
    canvas: Rc<RefCell<Canvas>>,
}

impl FontLoader {
    pub fn new(canvas: Rc<RefCell<Canvas>>) -> Self {
        Self { canvas }
    }
}

impl Loader for FontLoader {
    fn load_from_bytes(
        &self,
        _id: &str,
        bytes: &[u8],
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.canvas.borrow_mut().load_font(bytes.to_owned());
        Ok(None)
    }
}

pub struct JsonLoader<T> {
    _marker: PhantomData<T>,
}

impl<T> JsonLoader<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Loader for JsonLoader<T>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    fn load_from_bytes(
        &self,
        _id: &str,
        bytes: &[u8],
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>> {
        let value: T = serde_json::from_slice(bytes)?;
        Ok(Some(Arc::new(value)))
    }
}

pub struct SoundLoader {
    audio: Rc<RefCell<Audio>>,
}

impl SoundLoader {
    pub fn new(audio: Rc<RefCell<Audio>>) -> Self {
        Self { audio }
    }
}

impl Loader for SoundLoader {
    fn load_from_bytes(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.audio.borrow_mut().add_sound(id, bytes);
        Ok(None)
    }
}
