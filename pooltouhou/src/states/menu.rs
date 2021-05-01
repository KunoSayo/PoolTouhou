use amethyst::{
    core::{
        components::Transform
    },
    ecs::Entity,
    input::VirtualKeyCode,
    prelude::*,
    renderer::*,
};
use amethyst::core::ecs::{Join, DispatcherBuilder};

use crate::{GameCore};
use crate::handles::ResourcesHandles;
use amethyst::ui::{UiTransform, Anchor, UiText, LineMode};
use std::convert::TryInto;


const BUTTON_COUNT: usize = 6;
const BUTTON_NAME: [&str; BUTTON_COUNT] = ["Start", "Network", "Profile", "Option", "我觉得这里有个按钮", "Exit"];

pub struct Menu {
    select: u8,
    con: bool,
    time: std::time::SystemTime,
    texts: Option<[Entity; BUTTON_COUNT]>,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            select: 0,
            con: false,
            time: std::time::SystemTime::now(),
            texts: None,
        }
    }
}

impl SimpleState for Menu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
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
                20.,
                LineMode::Wrap,
                Anchor::MiddleRight,
            );
            let tran = UiTransform::new(
                "".into(), Anchor::Middle, Anchor::Middle,
                0., -((i * 40) as f32), 1., 200., 40.,
            );
            ui_text.insert(*e, text);
            ui_tran.insert(*e, tran);
        }

        self.texts = Some(ee.try_into().unwrap())
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {}


    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let now = std::time::SystemTime::now();
        let core = data.world.read_resource::<GameCore>();
        let input = &core.cur_frame_game_input;

        //make sure the screen is right
        //check enter / shoot first
        if input.shoot > 0 || input.enter > 0 {
            if self.select == BUTTON_COUNT as u8 - 1 {
                return Trans::Quit;
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
