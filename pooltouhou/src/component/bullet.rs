use amethyst::{
    ecs::prelude::{Component, DenseVecStorage},
};

use crate::script::script_context::ScriptContext;
use crate::systems::game_system::CollideType;

#[derive(Default)]
pub struct PlayerBullet {
    pub damage: f32
}

pub struct EnemyBullet {
    pub(crate) collide: CollideType,
    pub(crate) script: ScriptContext,
}

impl Component for PlayerBullet {
    type Storage = DenseVecStorage<Self>;
}


impl Component for EnemyBullet {
    type Storage = DenseVecStorage<Self>;
}
