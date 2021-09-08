use std::time::Duration;

use crate::handles::Progress;
use crate::LoopState;
use crate::states::{GameState, StateData, Trans};

pub struct LoadState<P: Progress = ()> {
    progress: Option<P>,
    trans: Trans,
    delay: Duration,
    start_time: std::time::Instant,
    should_render: bool,
}

impl LoadState {
    pub fn switch_wait_load(trans: Trans, delay: Duration) -> Trans {
        Trans::Switch(Box::new(
            Self {
                progress: None,
                delay,
                trans,
                start_time: std::time::Instant::now(),
                should_render: true,
            }))
    }
}


impl GameState for LoadState {
    fn start(&mut self, _: &mut StateData) {
        self.start_time = std::time::Instant::now();
    }

    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        let delta = std::time::Instant::now().duration_since(self.start_time);
        if delta >= self.delay {
            if let Some(ref progress) = self.progress {
                if progress.num_loading() == 0 {
                    println!("loaded {} resources.", progress.num_finished());
                    (std::mem::take(&mut self.trans), LoopState::POLL)
                } else {
                    (Trans::None, LoopState::wait_until(Duration::from_millis(50), self.should_render))
                }
            } else {
                (std::mem::take(&mut self.trans), LoopState::POLL)
            }
        } else {
            (Trans::None, LoopState::wait_until(self.delay - delta, self.should_render))
        }
    }

    fn render(&mut self, _: &mut StateData) -> Trans {
        self.should_render = false;
        Trans::None
    }
}