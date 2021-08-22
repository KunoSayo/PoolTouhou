use crate::{GlobalState, LoopState, MainRendererData, Pools};
use crate::input::BakedInputs;

// pub use gaming::Gaming;
// pub use init::Loading;
//
//
// pub mod gaming;
// pub mod pausing;
pub mod init;
pub mod menu;
pub mod load;
mod gaming;
// pub mod load;

pub const ARENA_WIDTH: f32 = 1600.0;
pub const ARENA_HEIGHT: f32 = 900.0;

pub enum StateEvent {
    Resize {
        width: u32,
        height: u32,
    }
}

pub enum Trans {
    None,
    Push(Box<dyn GameState>),
    Pop,
    Switch(Box<dyn GameState>),
    Exit,
    Vec(Vec<Trans>),
}

impl Default for Trans {
    fn default() -> Self {
        Self::None
    }
}

pub struct StateData<'a> {
    pub(crate) pools: &'a mut Pools,
    pub(crate) inputs: &'a BakedInputs,
    pub(crate) global_state: &'a mut GlobalState,
    pub(crate) render: &'a mut MainRendererData,
}

pub trait GameState: Send + 'static {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) { (Trans::None, LoopState::WAIT) }

    fn shadow_update(&mut self) -> LoopState { LoopState::WAIT_ALL }

    fn start(&mut self, _: &mut StateData) {}

    fn game_tick(&mut self, _: &mut StateData) -> Trans { Trans::None }

    fn shadow_tick(&mut self, _: &StateData) {}

    fn render(&mut self, _: &mut StateData) -> Trans { Trans::None }

    fn shadow_render(&mut self, _: &StateData) {}

    fn stop(&mut self, _: &mut StateData) {}

    fn on_event(&mut self, _: &StateEvent) {}
}
