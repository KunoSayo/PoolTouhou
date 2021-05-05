use amethyst::{
    ecs::Entity,
    prelude::*,
};

use crate::{GameCore};
use crate::handles::ResourcesHandles;
use amethyst::ui::{UiTransform, Anchor, UiText, LineMode};
use std::convert::TryInto;
use crate::states::Gaming;
use crate::states::load::LoadState;
use amethyst::core::Transform;
use nalgebra::Vector3;


const BUTTON_COUNT: usize = 9;
const BUTTON_NAME: [&str; BUTTON_COUNT] = ["Singleplayer", "Multiplayer", "Extra", "Profile", "Replay", "Music Room", "Option", "Cloud", "Exit"];

pub struct Menu {
    select: u8,
    con: bool,
    time: std::time::SystemTime,
    texts: Option<[Entity; BUTTON_COUNT]>,
    used_e: Vec<Entity>,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            select: 0,
            con: false,
            time: std::time::SystemTime::now(),
            texts: None,
            used_e: vec![],
        }
    }
}

impl SimpleState for Menu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let main_bg = {
            let handles = world.read_resource::<ResourcesHandles>();
            handles.sprites.get("mainbg").unwrap().clone()
        };
        self.used_e.push(world.create_entity().with(main_bg)
            .with({
                let mut tran = Transform::default();
                tran.set_translation_xyz(1600.0 / 2.0, 900.0 / 2.0, 1.0);
                tran
            })
            .build());

        let font = world.write_resource::<ResourcesHandles>().fonts.get("default")
            .expect("get default font to show menu failed").clone();


        let ee = world.create_iter().take(BUTTON_COUNT).collect::<Vec<_>>();
        let mut ui_tran = world.write_component::<UiTransform>();
        let mut ui_text = world.write_component::<UiText>();
        for (i, e) in ee.iter().enumerate() {
            let text = UiText::new(
                font.clone(),
                BUTTON_NAME[i].into(),
                [1., 1., 1., 1.],
                36.,
                LineMode::Wrap,
                Anchor::TopLeft,
            );
            let tran = UiTransform::new(
                "".into(), Anchor::TopLeft, Anchor::TopLeft,
                60., -380.0 - ((i * 55) as f32), 1., 996.1, 55.,
            );
            ui_text.insert(*e, text).unwrap();
            ui_tran.insert(*e, tran).unwrap();
        }

        self.texts = Some(ee.try_into().unwrap());
    }

    fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(texts) = self.texts.take() {
            for e in &texts {
                data.world.delete_entity(*e).unwrap();
            }
        }
        data.world.delete_entities(&self.used_e);
    }


    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let now = std::time::SystemTime::now();
        let core = data.world.read_resource::<GameCore>();
        let input = &core.cur_frame_game_input;

        //make sure the screen is right
        //check enter / shoot first
        if input.shoot > 0 || input.enter > 0 {
            const EXIT_IDX: u8 = (BUTTON_COUNT - 1) as u8;
            match self.select {
                0 => {
                    return LoadState::wait_load(Trans::Push(Box::new(Gaming::default())), 1.0);
                }
                EXIT_IDX => {
                    return Trans::Quit;
                }
                _ => {}
            }
        }

        let just_change = input.up == 1 || input.down == 1;
        if input.up == 1 || input.down == 1 || now.duration_since(self.time).unwrap().as_secs_f32() > if self.con { 1. / 6. } else { 0.5 } {
            match input.direction.1 {
                x if x > 0 => {
                    self.time = now;
                    self.con = !just_change;
                    if self.select == 0 {
                        self.select = (BUTTON_COUNT - 1) as u8;
                    } else {
                        self.select -= 1;
                    }
                }
                x if x < 0 => {
                    self.time = now;
                    self.con = !just_change;
                    self.select += 1;
                    if self.select == BUTTON_COUNT as u8 {
                        self.select = 0;
                    }
                }
                _ => {
                    self.con = false;
                }
            }
        }

        if let Some(text_entities) = self.texts {
            let mut text = data.world.write_component::<UiText>();
            for (i, entity) in text_entities.iter().enumerate() {
                let text = text.get_mut(*entity).unwrap();
                if i as u8 == self.select {
                    text.color = [1., 1., 1., 1.];
                } else {
                    text.color = [0.5, 0.5, 0.5, 1.];
                }
            }
        }
        Trans::None
    }
}
