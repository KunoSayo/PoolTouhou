use amethyst::{
    core::components::Transform,
    ecs::prelude::{Component, DenseVecStorage},
};

#[derive(Default)]
pub struct PlayerBullet {
    pub damage: f32
}

pub struct EnemyBullet {}

impl Component for PlayerBullet {
    type Storage = DenseVecStorage<Self>;
}


impl Component for EnemyBullet {
    type Storage = DenseVecStorage<Self>;
}
