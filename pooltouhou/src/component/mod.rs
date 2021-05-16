// use amethyst::{
//     ecs::prelude::Component,
// };
// use amethyst::core::ecs::DenseVecStorage;
//
// pub use bullet::EnemyBullet;
// pub use bullet::PlayerBullet;
// pub use anime::InvertColorAnimation;
// pub use sheep::Sheep;
//
// use crate::script::script_context::ScriptContext;
// use crate::systems::game_system::CollideType;
//
// pub mod bullet;
// pub mod sheep;
// pub mod anime;
//
// pub struct Enemy {
//     pub hp: f32,
//     pub collide: CollideType,
//     pub script: ScriptContext,
// }
//
// impl Enemy {
//     pub fn new(hp: f32, collide: CollideType, script: ScriptContext) -> Self {
//         Self {
//             hp,
//             collide,
//             script,
//         }
//     }
// }
//
// impl Component for Enemy {
//     type Storage = DenseVecStorage<Self>;
// }
