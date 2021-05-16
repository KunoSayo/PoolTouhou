// use amethyst::{
//     assets::*,
//     prelude::*,
// };
//
// use crate::states::{ProgressType};
//
//
// pub struct LoadState {
//     progress: Option<ProgressType>,
//     trans: SimpleTrans,
//     seconds: f32,
//     start_time: std::time::SystemTime,
// }
//
// impl LoadState {
//     pub fn switch_wait_load(trans: SimpleTrans, seconds: f32) -> SimpleTrans {
//         Trans::Switch(Box::new(
//             Self {
//                 progress: None,
//                 seconds,
//                 trans,
//                 start_time: std::time::SystemTime::now(),
//             }))
//     }
// }
//
//
// impl SimpleState for LoadState {
//     fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
//         self.start_time = std::time::SystemTime::now();
//     }
//
//
//     fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
//         if let Some(ref progress) = self.progress {
//             if progress.num_loading() == 0 {
//                 println!("loaded {} resources.", progress.num_finished());
//                 match progress.complete() {
//                     Completion::Failed => {
//                         for x in progress.errors() {
//                             eprintln!("load {} failed for {}", x.asset_name, x.error);
//                         }
//                     }
//                     _ => {}
//                 }
//                 if std::time::SystemTime::now().duration_since(self.start_time).unwrap().as_secs_f32() >= self.seconds {
//                     let mut trans = Trans::None;
//                     std::mem::swap(&mut trans, &mut self.trans);
//                     Trans::Sequence(vec![Trans::Pop, trans])
//                 } else {
//                     Trans::None
//                 }
//             } else {
//                 Trans::None
//             }
//         } else if std::time::SystemTime::now().duration_since(self.start_time).unwrap().as_secs_f32() >= self.seconds {
//             let mut trans = Trans::None;
//             std::mem::swap(&mut trans, &mut self.trans);
//             Trans::Sequence(vec![Trans::Pop, trans])
//         } else {
//             Trans::None
//         }
//     }
// }