use amethyst::{
    ecs::prelude::Component,
    renderer::SpriteRender,
};
use amethyst::core::ecs::DenseVecStorage;

pub use bullet::EnemyBullet;
pub use bullet::PlayerBullet;
pub use invert_color_anime::InvertColorAnimation;
pub use sheep::Sheep;

use crate::script::script_context::ScriptContext;
use crate::systems::game_system::CollideType;

pub mod bullet;
pub mod sheep;
pub mod invert_color_anime;

pub struct Enemy {
    pub hp: f32,
    pub collide: CollideType,
    pub script: ScriptContext,
    pub sprite_render: Option<SpriteRender>,
}

impl Enemy {
    pub fn new(hp: f32, collide: CollideType, script: ScriptContext) -> Self {
        Self {
            hp,
            collide,
            script,
            sprite_render: None,
        }
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
