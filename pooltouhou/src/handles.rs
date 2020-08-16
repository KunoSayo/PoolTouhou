use std::collections::HashMap;

use amethyst::{
    renderer::*,
};

pub struct TextureHandles {
    pub player_bullet: Option<SpriteRender>,
    pub bullets: HashMap<String, SpriteRender>,
}

impl Default for TextureHandles {
    fn default() -> Self {
        Self {
            player_bullet: None,
            bullets: Default::default(),
        }
    }
}