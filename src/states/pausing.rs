use amethyst::{
    input::VirtualKeyCode,
    prelude::*,
};

use crate::CoreStorage;

pub struct Pausing;


impl SimpleState for Pausing {
    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let mut core_storage = world.write_resource::<CoreStorage>();
        core_storage.swap_input();

        if core_storage.is_press(Box::from([VirtualKeyCode::Escape])) {
            Trans::Pop
        } else {
            Trans::None
        }
    }
}