use std::time::{Duration, Instant};

use crate::handles::{CounterProgress, Progress};
use crate::LoopState;
use crate::states::{GameState, StateData, Trans};
use crate::states::menu::Menu;

pub struct Loading {
    progress: CounterProgress,
    start: Instant,
    fst: bool,
}

impl Default for Loading {
    fn default() -> Self {
        Self {
            progress: Default::default(),
            start: Instant::now(),
            fst: true
        }
    }
}

impl GameState for Loading {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        if self.fst {
            self.fst = false;
            (Trans::None, LoopState::WaitTimed(Duration::from_millis(250)))
        } else if self.progress.num_loading() == 0 {
            (Trans::Push(Box::new(Menu::default())), LoopState::Wait)
        } else {
            (Trans::None, LoopState::WaitAllTimed(Duration::from_millis(50)))
        }
    }

    fn start(&mut self, data: &mut StateData) {
        log::info!("loading state start");
        self.start = Instant::now();
        let graphics_state = &data.global_state;
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
        if let Some(al) = &data.global_state.al {
            handles.load_bgm_static("title", "title.mp3", al.ctx.clone(), &data.pools, self.progress.create_tracker());
        }
    }

    fn shadow_tick(&mut self, _data: &StateData) {
        //todo: reload
    }
}