// use amethyst::{
//     core::timing::Time,
//     derive::SystemDesc,
//     ecs::{Entities, Read, System, SystemData, WriteStorage},
// };
// use amethyst::core::ecs::Join;
//
// use crate::render::InvertColorCircle;
//
// #[derive(SystemDesc)]
// pub struct AnimationSystem;
//
// impl<'a> System<'a> for AnimationSystem {
//     type SystemData = (
//         Entities<'a>,
//         WriteStorage<'a, crate::render::InvertColorCircle>,
//         WriteStorage<'a, crate::component::InvertColorAnimation>,
//         Read<'a, Time>
//     );
//     fn run(&mut self, (entities, mut invert_color_circles, mut invert_color_animations, time): Self::SystemData) {
//         let delta_second = time.delta_time().as_secs_f32();
//         //invert color (biu)
//         for (entity, mut anime) in (&entities, &mut invert_color_animations).join() {
//             if let Some(circle) = invert_color_circles.get_mut(entity) {
//                 anime.last_seconds -= delta_second;
//                 if anime.last_seconds < 0.0 {
//                     entities.delete(entity).expect("delete anime entity failed");
//                 } else {
//                     circle.radius += anime.spread_per_second * delta_second;
//                 }
//             } else {
//                 anime.delay_second -= delta_second;
//                 if anime.delay_second <= 0.0 {
//                     anime.last_seconds += anime.delay_second;
//                     invert_color_circles.insert(entity, InvertColorCircle {
//                         pos: anime.transform.take().unwrap(),
//                         radius: anime.spread_per_second * -anime.delay_second,
//                     }).expect("insert anime entity failed");
//                 }
//             }
//         }
//     }
// }