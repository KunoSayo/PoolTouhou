use crate::states::{GameState, StateData, Trans};

// use amethyst::{
//     ecs::Entity,
//     prelude::*,
// };
//
// use crate::{GameCore};
// use crate::handles::ResourcesHandles;
// use amethyst::ui::{UiTransform, Anchor, UiText, LineMode};
// use std::convert::TryInto;
// use crate::states::Gaming;
// use crate::states::load::LoadState;
// use amethyst::core::Transform;
//
//
const BUTTON_COUNT: usize = 9;
const BUTTON_NAME: [&str; BUTTON_COUNT] = ["Singleplayer", "Multiplayer", "Extra", "Profile", "Replay", "Music Room", "Option", "Cloud", "Exit"];

pub struct Menu {
    select: u8,
    con: bool,
    time: std::time::SystemTime,
    select_text: u8,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            select: 0,
            con: false,
            time: std::time::SystemTime::now(),
            select_text: 0,
        }
    }
}

impl GameState for Menu {
    fn start(&mut self, data: &StateData) {
        //     let world = data.world;
        //
        //     let main_bg = {
        //         let handles = world.read_resource::<ResourcesHandles>();
        //         handles.sprites.get("mainbg").unwrap().clone()
        //     };
        //     self.used_e.push(world.create_entity().with(main_bg)
        //         .with({
        //             let mut tran = Transform::default();
        //             tran.set_translation_xyz(1600.0 / 2.0, 900.0 / 2.0, 1.0);
        //             tran
        //         })
        //         .build());
        //
        //     let font = world.write_resource::<ResourcesHandles>().fonts.get("default")
        //         .expect("get default font to show menu failed").clone();
        //
        //
        //     let ee = world.create_iter().take(BUTTON_COUNT).collect::<Vec<_>>();
        //     let selecting_entity = world.create_entity().build();
        //     {
        //         let mut ui_tran = world.write_component::<UiTransform>();
        //         let mut ui_text = world.write_component::<UiText>();
        //         for (i, e) in ee.iter().enumerate() {
        //             let text = UiText::new(
        //                 font.clone(),
        //                 BUTTON_NAME[i].into(),
        //                 [1., 1., 1., 1.],
        //                 36.,
        //                 LineMode::Wrap,
        //                 Anchor::TopLeft,
        //             );
        //             let tran = UiTransform::new(
        //                 "".into(), Anchor::TopLeft, Anchor::TopLeft,
        //                 60., -380.0 - ((i * 55) as f32), 1., 996.1, 55.,
        //             );
        //             if self.select == i as u8 {
        //                 let mut text = text.clone();
        //                 let mut tran = tran.clone();
        //                 selecting_offset(&mut text, &mut tran);
        //                 ui_text.insert(selecting_entity, text).unwrap();
        //                 ui_tran.insert(selecting_entity, tran).unwrap();
        //             }
        //             ui_text.insert(*e, text).unwrap();
        //             ui_tran.insert(*e, tran).unwrap();
        //         }
        //     }
        //     self.texts = Some(ee.try_into().unwrap());
        //     self.select_text = Some(selecting_entity);
    }


    fn update(&mut self, data: &StateData) -> Trans {
        // const EXIT_IDX: u8 = (BUTTON_COUNT - 1) as u8;
        //
        // let now = std::time::SystemTime::now();
        // let core = data.world.read_resource::<GameCore>();
        // let input = &core.cur_frame_game_input;
        //
        // let last_select = self.select;
        // //make sure the screen is right
        // //check enter / shoot first
        // if input.shoot > 0 || input.enter > 0 {
        //     match self.select {
        //         0 => {
        //             return LoadState::switch_wait_load(Trans::Push(Box::new(Gaming::default())), 1.0);
        //         }
        //         EXIT_IDX => {
        //             return Trans::Quit;
        //         }
        //         _ => {}
        //     }
        // }
        // if input.bomb == 1 {
        //     self.select = EXIT_IDX;
        // }
        //
        // let just_change = input.up == 1 || input.down == 1;
        // if input.up == 1 || input.down == 1 || now.duration_since(self.time).unwrap().as_secs_f32() > if self.con { 1. / 6. } else { 0.5 } {
        //     match input.direction.1 {
        //         x if x > 0 => {
        //             self.time = now;
        //             self.con = !just_change;
        //             self.select = get_previous(self.select, BUTTON_COUNT as _);
        //         }
        //         x if x < 0 => {
        //             self.time = now;
        //             self.con = !just_change;
        //             self.select = get_next(self.select, BUTTON_COUNT as _);
        //         }
        //         _ => {
        //             self.con = false;
        //         }
        //     }
        // }
        //
        // if let Some(text_entities) = self.texts {
        //     let mut text = data.world.write_component::<UiText>();
        //     for (i, entity) in text_entities.iter().enumerate() {
        //         let text = text.get_mut(*entity).unwrap();
        //         if i as u8 == self.select {
        //             text.color = [1., 1., 1., 1.];
        //         } else {
        //             text.color = [0.5, 0.5, 0.5, 1.];
        //         }
        //     }
        //     if last_select != self.select {
        //         if let Some(select_entity) = self.select_text {
        //             let mut trans = data.world.write_component::<UiTransform>();
        //
        //             let n_text = text.get(text_entities[self.select as usize]).unwrap().clone();
        //             let n_tran = trans.get(text_entities[self.select as usize]).unwrap().clone();
        //             let mut text = text.get_mut(select_entity).unwrap();
        //             let mut tran = trans.get_mut(select_entity).unwrap();
        //             *text = n_text;
        //             *tran = n_tran;
        //             selecting_offset(&mut text, &mut tran);
        //         }
        //     }
        // }
        Trans::None
    }
}

// #[inline]
// pub fn selecting_offset(text: &mut UiText, tran: &mut UiTransform) {
//     text.color = [136.0 / 256.0, 136.0 / 256.0, 136.0 / 256.0, 1.0];
//     tran.local_x += 3.0;
//     tran.local_y -= 3.0;
//     tran.local_z -= 0.9961;
// }

#[inline]
pub fn get_previous(cur_idx: u8, max_len: u8) -> u8 {
    if cur_idx == 0 {
        max_len - 1
    } else {
        cur_idx - 1
    }
}

#[inline]
pub fn get_next(cur_idx: u8, max_len: u8) -> u8 {
    let cur_idx = cur_idx + 1;
    if cur_idx == max_len {
        0
    } else {
        cur_idx
    }
}