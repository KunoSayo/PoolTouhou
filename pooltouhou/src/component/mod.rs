use amethyst::{
    ecs::prelude::{Component, FlaggedStorage},
    renderer::SpriteRender,
};

pub use bullet::EnemyBullet;
pub use bullet::PlayerBullet;
pub use invert_color_anime::InvertColorAnimation;
pub use sheep::Sheep;

pub mod bullet;
pub mod sheep;
pub mod invert_color_anime;

#[derive(Default)]
pub struct Enemy {
    pub hp: f32,
    pub rad_p2: f32,
    pub sprite_render: Option<SpriteRender>,
}

impl Enemy {
    pub fn new(hp: f32, rad_p2: f32) -> Self {
        Self {
            hp,
            rad_p2,
            sprite_render: None,
        }
    }
}

impl Component for Enemy {
    type Storage = FlaggedStorage<Self>;
}
