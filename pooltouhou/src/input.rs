use std::collections::HashSet;

use amethyst::{
    derive::SystemDesc,
    ecs::{Read, System, SystemData, Write},
    input::{InputHandler, StringBindings, VirtualKeyCode},
};

use crate::GameCore;

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

    pub fn get_move(&self, base_speed: f32) -> (f32, f32) {
        let up = self.pressing.contains(&VirtualKeyCode::Up);
        let down = self.pressing.contains(&VirtualKeyCode::Down);
        let left = self.pressing.contains(&VirtualKeyCode::Left);
        let right = self.pressing.contains(&VirtualKeyCode::Right);
        if !(up ^ down) && !(left ^ right) {
            (0.0, 0.0)
        } else if up ^ down {
            if left ^ right {
                let corner_speed = base_speed * 2.0_f32.sqrt() * 0.5;
                (if left { -corner_speed } else { corner_speed }, if up { corner_speed } else { -corner_speed })
            } else {
                (0.0, if up { base_speed } else { -base_speed })
            }
        } else {
            (if left { -base_speed } else if right { base_speed } else { 0.0 }, 0.0)
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
        Write<'s, GameCore>
    );
    fn run(&mut self, (data, mut core): Self::SystemData) {
        let mut cur = &mut core.temp_input;
        cur.pressing.extend(data.keys_that_are_down());
        if let Some((x, y)) = data.mouse_position() {
            cur.x = x;
            cur.y = y;
        }
    }
}