// pub use gaming::Gaming;
// pub use init::Loading;
//
//
// pub mod gaming;
// pub mod pausing;
// pub mod init;
// pub mod menu;
// pub mod load;

pub const ARENA_WIDTH: f32 = 1600.0;
pub const ARENA_HEIGHT: f32 = 900.0;

pub enum StateTransform {
    Push(Box<dyn GameState>),
    Pop,
    Switch(Box<dyn GameState>),
    Exit,
    None,
}

pub trait GameState {
    fn start(&mut self) {}

    fn update(&mut self) -> StateTransform { StateTransform::None }

    fn shadow_update(&mut self) {}

    fn render(&mut self) {}

    fn shadow_render(&mut self) {}

    fn stop(&mut self) {}
}
