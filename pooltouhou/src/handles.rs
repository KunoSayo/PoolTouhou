use std::collections::HashMap;

use amethyst::{
    renderer::*,
};
use amethyst::audio::SourceHandle;
use amethyst::ui::FontHandle;

#[derive(Default)]
pub struct ResourcesHandles {
    pub player_bullet: Option<SpriteRender>,
    pub textures: HashMap<String, SpriteRender>,
    pub sounds: HashMap<String, SourceHandle>,
    pub fonts: HashMap<String, FontHandle>,
}