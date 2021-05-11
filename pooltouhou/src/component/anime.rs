use amethyst::{
    core::components::Transform,
    ecs::prelude::{Component, DenseVecStorage},
};

#[derive(Default)]
pub struct InvertColorAnimation {
    pub last_seconds: f32,
    pub spread_per_second: f32,
    pub delay_second: f32,
    pub transform: Option<Transform>,
}

impl Component for InvertColorAnimation {
    type Storage = DenseVecStorage<Self>;
}