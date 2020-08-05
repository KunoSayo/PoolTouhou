use amethyst::{
    renderer::*,
};

pub struct TextureHandles {
    pub player_bullet: Option<SpriteRender>
}

impl Default for TextureHandles {
    fn default() -> Self {
        Self {
            player_bullet: None
        }
    }
}