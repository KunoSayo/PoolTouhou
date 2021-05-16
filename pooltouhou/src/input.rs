// use std::collections::HashSet;
//
// use amethyst::{
//     derive::SystemDesc,
//     ecs::{Read, System, SystemData, Write},
//     input::{InputHandler, StringBindings, VirtualKeyCode},
// };
//
// use crate::GameCore;
//
// #[derive(Debug, Default)]
// pub struct RawInputData {
//     pub x: f32,
//     pub y: f32,
//     pub pressing: HashSet<VirtualKeyCode>,
// }
//
// #[derive(Debug, Default)]
// pub struct GameInputData {
//     pub shoot: u32,
//     pub slow: u32,
//     pub bomb: u32,
//     pub sp: u32,
//     pub up: u32,
//     pub down: u32,
//     pub left: u32,
//     pub right: u32,
//     pub direction: (i32, i32),
//     pub enter: u32,
//     pub esc: u32,
// }
//
// fn get_direction(up: u32, down: u32, left: u32, right: u32) -> (i32, i32) {
//     //left x-
//     //right x+
//     //up y+
//     //down y-
//     match (up, down, left, right) {
//         (x, y, w, z) if x == y && w == z => (0, 0),
//         (x, y, 0, _) if x == y => (1, 0),
//         (x, y, _, 0) if x == y => (-1, 0),
//         (0, _, x, y) if x == y => (0, -1),
//         (_, 0, x, y) if x == y => (0, 1),
//         _ => {
//             if up > down {
//                 //go down
//                 if left > right {
//                     //go right
//                     (1, -1)
//                 } else {
//                     //go left
//                     (-1, -1)
//                 }
//             } else {
//                 //go up
//                 if left > right {
//                     //go right
//                     (1, 1)
//                 } else {
//                     //go left
//                     (-1, 1)
//                 }
//             }
//         }
//     }
// }
//
// impl From<&RawInputData> for GameInputData {
//     fn from(r: &RawInputData) -> Self {
//         let up = r.pressing.contains(&VirtualKeyCode::Up) as u32;
//         let down = r.pressing.contains(&VirtualKeyCode::Down) as u32;
//         let left = r.pressing.contains(&VirtualKeyCode::Left) as u32;
//         let right = r.pressing.contains(&VirtualKeyCode::Right) as u32;
//         let direction = get_direction(up, down, left, right);
//         Self {
//             shoot: r.pressing.contains(&VirtualKeyCode::Z) as u32,
//             slow: r.pressing.contains(&VirtualKeyCode::LShift) as u32,
//             bomb: r.pressing.contains(&VirtualKeyCode::X) as u32,
//             sp: r.pressing.contains(&VirtualKeyCode::C) as u32,
//             up,
//             down,
//             left,
//             right,
//             direction,
//             enter: (r.pressing.contains(&VirtualKeyCode::Return) || r.pressing.contains(&VirtualKeyCode::NumpadEnter)) as u32,
//             esc: r.pressing.contains(&VirtualKeyCode::Escape) as u32,
//         }
//     }
// }
//
// macro_rules! inc_or_zero {
//     ($e: expr, $b: expr) => {
//         if $b {
//             $e += 1;
//         } else {
//             $e = 0;
//         }
//     };
// }
//
// impl GameInputData {
//     pub fn tick_mut(&mut self, r: &RawInputData) {
//         inc_or_zero!(self.shoot, r.pressing.contains(&VirtualKeyCode::Z));
//         inc_or_zero!(self.slow, r.pressing.contains(&VirtualKeyCode::LShift));
//         inc_or_zero!(self.bomb, r.pressing.contains(&VirtualKeyCode::X));
//         inc_or_zero!(self.sp, r.pressing.contains(&VirtualKeyCode::C));
//         inc_or_zero!(self.up, r.pressing.contains(&VirtualKeyCode::Up));
//         inc_or_zero!(self.down, r.pressing.contains(&VirtualKeyCode::Down));
//         inc_or_zero!(self.left, r.pressing.contains(&VirtualKeyCode::Left));
//         inc_or_zero!(self.right, r.pressing.contains(&VirtualKeyCode::Right));
//         inc_or_zero!(self.enter, r.pressing.contains(&VirtualKeyCode::Return) || r.pressing.contains(&VirtualKeyCode::NumpadEnter));
//         inc_or_zero!(self.esc, r.pressing.contains(&VirtualKeyCode::Escape));
//         self.direction = get_direction(self.up, self.down, self.left, self.right);
//     }
//
//     pub fn clear(&mut self) {
//         *self = Default::default();
//     }
// }
//
// impl RawInputData {
//     pub fn empty() -> Self {
//         Self {
//             x: 0.0,
//             y: 0.0,
//             pressing: HashSet::new(),
//         }
//     }
//
//     pub fn get_move(&self, base_speed: f32) -> (f32, f32) {
//         let up = self.pressing.contains(&VirtualKeyCode::Up);
//         let down = self.pressing.contains(&VirtualKeyCode::Down);
//         let left = self.pressing.contains(&VirtualKeyCode::Left);
//         let right = self.pressing.contains(&VirtualKeyCode::Right);
//         if !(up ^ down) && !(left ^ right) {
//             (0.0, 0.0)
//         } else if up ^ down {
//             if left ^ right {
//                 let corner_speed = base_speed * 2.0_f32.sqrt() * 0.5;
//                 (if left { -corner_speed } else { corner_speed }, if up { corner_speed } else { -corner_speed })
//             } else {
//                 (0.0, if up { base_speed } else { -base_speed })
//             }
//         } else {
//             (if left { -base_speed } else if right { base_speed } else { 0.0 }, 0.0)
//         }
//     }
// }
//
// impl Clone for RawInputData {
//     fn clone(&self) -> Self {
//         Self {
//             x: self.x,
//             y: self.y,
//             pressing: self.pressing.clone(),
//         }
//     }
//
//     fn clone_from(&mut self, source: &Self) {
//         self.x = source.x;
//         self.y = source.y;
//         self.pressing = source.pressing.clone();
//     }
// }
//
// #[derive(SystemDesc)]
// pub struct InputDataSystem;
//
// impl<'s> System<'s> for InputDataSystem {
//     type SystemData = (
//         Read<'s, InputHandler<StringBindings>>,
//         Write<'s, GameCore>
//     );
//     fn run(&mut self, (data, mut core): Self::SystemData) {
//         core.swap_frame_input();
//
//         for ref x in data.keys_that_are_down() {
//             core.cache_input.pressing.insert(*x);
//         }
//
//         let mut cur = &mut core.cur_frame_input;
//
//         cur.pressing.clear();
//         cur.pressing.extend(data.keys_that_are_down());
//         if let Some((x, y)) = data.mouse_position() {
//             cur.x = x;
//             cur.y = y;
//         }
//         core.tick_game_frame_input();
//     }
// }