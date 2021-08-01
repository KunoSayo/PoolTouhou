use std::time::Instant;

use crate::handles::{CounterProgress, Progress};
use crate::states::{GameState, StateData, Trans};
use crate::states::menu::Menu;

pub struct Loading {
    progress: CounterProgress,
    start: Instant,
}

impl Default for Loading {
    fn default() -> Self {
        Self {
            progress: Default::default(),
            start: Instant::now(),
        }
    }
}

impl GameState for Loading {
    fn start(&mut self, data: &mut StateData) {
        log::info!("loading state start");
        self.start = Instant::now();
        let graphics_state = &data.graphics_state;
        let handles = &graphics_state.handles;
        let pools = &data.pools;
        handles.load_texture_static("bullet", "bullet.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("circle_red", "circle_red.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("circle_blue", "circle_blue.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("circle_green", "circle_green.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("circle_yellow", "circle_yellow.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("circle_purple", "circle_purple.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("zzzz", "zzzz.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("mainbg", "mainbg.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("暗夜", "暗夜.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("sheepBullet", "sheepBullet.png", graphics_state, pools, self.progress.create_tracker());
        handles.load_texture_static("sheep", "sheep.png", graphics_state, pools, self.progress.create_tracker());
    }

    fn game_tick(&mut self, _: &mut StateData) -> Trans {
        if self.progress.num_loading() == 0 {
            log::info!("loaded {} resources in {}ms.", self.progress.num_finished(), self.start.elapsed().as_millis());
            Trans::Push(Box::new(Menu::default()))
        } else {
            Trans::None
        }
    }

    fn shadow_update(&mut self, _data: &StateData) {
        //todo: reload
    }
}