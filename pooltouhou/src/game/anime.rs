// use amethyst::{
//     core::components::Transform,
//     ecs::prelude::{Component, HashMapStorage},
// };
//
// #[derive(Default)]
// pub struct InvertColorAnimation {
//     pub last_seconds: f32,
//     pub spread_per_second: f32,
//     pub delay_second: f32,
//     pub transform: Option<Transform>,
// }
//
// impl Component for InvertColorAnimation {
//     type Storage = HashMapStorage<Self>;
// }
//
// #[derive(Default)]
// pub struct WaterWave {
//     pub radius: f32,
//     pub lambda: f32,
//     pub src: Transform,
// }
//
// impl Component for WaterWave {
//     type Storage = HashMapStorage<Self>;
// }