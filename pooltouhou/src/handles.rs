use std::collections::HashMap;

use amethyst::{
    renderer::*,
};
use amethyst::audio::SourceHandle;

pub struct ResourcesHandles {
    pub player_bullet: Option<SpriteRender>,
    pub textures: HashMap<String, SpriteRender>,
    pub sounds: HashMap<String, SourceHandle>,
}

impl Default for ResourcesHandles {
    fn default() -> Self {
        Self {
            player_bullet: None,
            textures: Default::default(),
            sounds: Default::default(),
        }
    }
}