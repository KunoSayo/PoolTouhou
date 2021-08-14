use std::convert::TryInto;

use wgpu_glyph::Text;

use crate::LoopState;
use crate::render::texture2d::{Texture2DObject, Texture2DVertexData};
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
    texts: Vec<wgpu_glyph::Section<'static>>,
    background: Option<Texture2DObject>,
}

impl Default for Menu {
    fn default() -> Self {
        let mut texts = Vec::with_capacity(BUTTON_NAME.len());
        for (i, text) in BUTTON_NAME.iter().enumerate() {
            let color = if i == 0 { 1.0 } else { 0.5 };
            texts.push(wgpu_glyph::Section {
                screen_position: (60.0, 380.0 + i as f32 * 55.0),
                bounds: (9961.0, 9961.0),
                layout: Default::default(),
                text: vec![Text::new(text).with_color([color, color, color, 1.0])
                    .with_scale(36.0)],
            })
        }
        Self {
            select: 0,
            con: false,
            time: std::time::SystemTime::now(),
            texts,
            background: None,
        }
    }
}

impl GameState for Menu {
    fn update(&mut self, data: &mut StateData) -> (Trans, LoopState) {
        let mut loop_state = LoopState::WAIT_ALL;
        const EXIT_IDX: u8 = (BUTTON_COUNT - 1) as u8;

        let now = std::time::SystemTime::now();
        let input = &data.inputs.cur_frame_game_input;

        //make sure the screen is right
        //check enter / shoot first
        if input.shoot > 0 || input.enter > 0 {
            match self.select {
                0 => {
                    // return LoadState::switch_wait_load(Trans::Push(Box::new(Gaming::default())), 1.0);
                }
                EXIT_IDX => {
                    return (Trans::Exit, loop_state);
                }
                _ => {}
            }
        }
        if input.bomb == 1 {
            loop_state = LoopState::WAIT;
            self.select = EXIT_IDX;
        }

        let just_change = input.up == 1 || input.down == 1;
        if input.up == 1 || input.down == 1 || now.duration_since(self.time).unwrap().as_secs_f32() > if self.con { 1. / 6. } else { 0.5 } {
            match input.direction.1 {
                x if x > 0 => {
                    self.time = now;
                    self.con = !just_change;
                    log::trace!("Select previous button");
                    self.select = get_previous(self.select, BUTTON_COUNT as _);
                    loop_state = LoopState::WAIT;
                }
                x if x < 0 => {
                    self.time = now;
                    self.con = !just_change;
                    log::trace!("Select next button");
                    self.select = get_next(self.select, BUTTON_COUNT as _);
                    loop_state = LoopState::WAIT;
                }
                _ => {
                    self.con = false;
                }
            }
        }

        for (i, s) in self.texts.iter_mut().enumerate() {
            if i as u8 == self.select {
                s.text[0].extra.color = [1., 1., 1., 1.];
            } else {
                s.text[0].extra.color = [0.5, 0.5, 0.5, 1.];
            }
        }
        (Trans::None, loop_state)
    }

    fn start(&mut self, data: &mut StateData) {
        let tex = *data.global_state.handles.texture_map.read().unwrap().get("mainbg").expect("Where is the bg tex?");
        let w = data.global_state.swapchain_desc.width as f32;
        let h = data.global_state.swapchain_desc.height as f32;
        self.background = Some(Texture2DObject {
            vertex: (0..4).map(|x| {
                Texture2DVertexData {
                    pos: match x {
                        0 => [0.0, h],
                        1 => [w, h],
                        2 => [0.0, 0.0],
                        3 => [w, 0.0],
                        _ => unreachable!()
                    },
                    coord: match x {
                        0 => [0.0, 0.0],
                        1 => [1.0, 0.0],
                        2 => [0.0, 1.0],
                        3 => [1.0, 1.0],
                        _ => unreachable!()
                    },
                }
            }).collect::<Vec<_>>().try_into().unwrap(),
            z: 0.0,
            tex,
        });

        data.render.render2d.add_tex(data.global_state, tex);

        if let Some(al) = &mut data.global_state.al {
            al.play_bgm(data.global_state.handles.bgm_map.read().unwrap()["title"].clone());
        }
    }


    fn render(&mut self, data: &mut StateData) -> Trans {
        let screen = &data.render.views.get_screen().view;

        data.render.render2d.render(data.global_state, screen, &[self.background.as_ref().unwrap()]);
        {
            let mut encoder = data.global_state.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Menu Text Encoder") });
            let mut tran = self.texts[self.select as usize].screen_position;
            tran.0 += 3.0;
            tran.1 += 3.0;
            let shadow = wgpu_glyph::Section {
                screen_position: tran,
                bounds: (9961.0, 9961.0),
                layout: Default::default(),
                text: vec![Text::new(BUTTON_NAME[self.select as usize])
                    .with_color([136.0 / 256.0, 136.0 / 256.0, 136.0 / 256.0, 1.0])
                    .with_scale(36.0)],
            };
            data.render.glyph_brush.queue(shadow);

            for s in &self.texts {
                data.render.glyph_brush.queue(s);
            }

            if let Err(e) = data.render.glyph_brush
                .draw_queued(&data.global_state.device, &mut data.render.staging_belt, &mut encoder, screen,
                             data.global_state.swapchain_desc.width,
                             data.global_state.swapchain_desc.height) {
                log::warn!("Render menu text failed for {}", e);
            }
            data.render.staging_belt.finish();
            data.global_state.queue.submit(Some(encoder.finish()));
        }
        Trans::None
    }
}

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