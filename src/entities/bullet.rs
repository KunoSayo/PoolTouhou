use amethyst::{
    ecs::prelude::{Component, DenseVecStorage},
};

#[derive(Default)]
pub struct PlayerBullet;

#[derive(Default)]
pub struct EnemyBullet;


impl Component for PlayerBullet {
    type Storage = DenseVecStorage<Self>;
}


impl Component for EnemyBullet {
    type Storage = DenseVecStorage<Self>;
}
