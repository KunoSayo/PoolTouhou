use std::collections::HashSet;

use amethyst::{
    derive::SystemDesc,
    ecs::{Read, System, SystemData, Write},
    input::{InputHandler, StringBindings, VirtualKeyCode},
};

use crate::CoreStorage;

#[derive(Debug)]
pub struct InputData {
    pub x: f32,
    pub y: f32,
    pub pressing: HashSet<VirtualKeyCode>,
}

impl InputData {
    pub fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            pressing: HashSet::new(),
        }
    }
}

impl Clone for InputData {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            pressing: self.pressing.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
        self.pressing = source.pressing.clone();
    }
}

#[derive(SystemDesc)]
pub struct InputDataSystem;

impl<'s> System<'s> for InputDataSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        Write<'s, CoreStorage>
    );
    fn run(&mut self, (data, mut core): Self::SystemData) {
        core.last_input = core.cur_input.take().unwrap();
        let mut cur = InputData::empty();
        for key in data.keys_that_are_down() {
            cur.pressing.insert(key);
        }
        if let Some((x, y)) = data.mouse_position() {
            cur.x = x;
            cur.y = y;
        }
        core.cur_input = Some(cur);
    }
}