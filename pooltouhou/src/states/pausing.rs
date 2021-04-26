use amethyst::{
    input::VirtualKeyCode,
    prelude::*,
};

use crate::GameCore;

#[derive(Default)]
pub struct Pausing {
    choosing: u8
}

impl SimpleState for Pausing {
    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let core_storage = world.read_resource::<GameCore>();

        if core_storage.is_pressed(&[VirtualKeyCode::Escape]) {
            Trans::Pop
        } else if core_storage.is_pressed(&[VirtualKeyCode::X]) {
            if self.choosing == 1 {
                Trans::Sequence(vec![Trans::Pop, Trans::Pop])
            } else {
                self.choosing = 1;
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}