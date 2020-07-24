use amethyst::{
    ecs::prelude::{Component, DenseVecStorage},
};

pub struct Sheep;

impl Sheep {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Sheep {
    type Storage = DenseVecStorage<Self>;
}