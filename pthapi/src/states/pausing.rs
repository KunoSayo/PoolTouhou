// use amethyst::{
//     input::VirtualKeyCode,
//     prelude::*,
// };
//
// use crate::GameCore;
// use crate::states::Gaming;
//
// #[derive(Default)]
// pub struct Pausing {
//     choosing: u8,
// }
//
// impl SimpleState for Pausing {
//     fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
//         let world = &mut data.world;
//         let core_storage = world.read_resource::<GameCore>();
//
//         if core_storage.is_pressed(&[VirtualKeyCode::Escape]) {
//             Trans::Pop
//         } else if core_storage.is_pressed(&[VirtualKeyCode::X]) {
//             if self.choosing == 1 {
//                 //pop pause. switch gaming.
//                 Trans::Sequence(vec![Trans::Pop, Trans::Switch(Box::new(Gaming::default()))])
//             } else {
//                 self.choosing = 1;
//                 Trans::None
//             }
//         } else {
//             Trans::None
//         }
//     }
// }