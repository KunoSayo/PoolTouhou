use amethyst::{
    core::TransformBundle,
    ecs::Entity,
    input::{InputBundle, StringBindings, VirtualKeyCode},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderFlat3D, RenderToWindow},
        RenderingBundle,
        types::DefaultBackend,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;

use crate::script::ScriptGameCommand;
use crate::input::RawInputData;
use std::mem::swap;

mod script;

mod render;
mod ui;
mod handles;
pub mod systems;
mod states;
mod component;
mod input;
mod audio;


// https://doc.rust-lang.org/book/

pub const PLAYER_Z: f32 = 0.0;


pub struct GameCore {
    player: Option<Entity>,
    cur_game_input: input::GameInputData,
    last_input: input::RawInputData,
    cur_input: input::RawInputData,
    cache_input: input::RawInputData,
    last_frame_input: input::RawInputData,
    cur_frame_input: input::RawInputData,
    cur_frame_game_input: input::GameInputData,
    commands: Vec<ScriptGameCommand>,
    next_tick_time: std::time::SystemTime,
    tick: u128,
    al: Option<audio::OpenalData>,
}

impl Default for GameCore {
    fn default() -> Self {
        let alto = match audio::OpenalData::new() {
            Ok(a) => Some(a),
            Err(e) => {
                eprintln!("load openal failed for {}", e);
                None
            }
        };
        Self {
            player: None,
            cur_game_input: Default::default(),
            last_input: input::RawInputData::empty(),
            cur_input: input::RawInputData::empty(),
            cache_input: RawInputData::default(),
            last_frame_input: RawInputData::default(),
            cur_frame_input: input::RawInputData::empty(),
            cur_frame_game_input: Default::default(),
            commands: vec![],
            next_tick_time: std::time::SystemTime::now(),
            tick: 0,
            al: alto,
        }
    }
}

impl GameCore {
    #[inline]
    pub fn tick_input(&mut self) {
        swap(&mut self.last_input, &mut self.cur_input);
        swap(&mut self.cur_input, &mut self.cache_input);
        self.cur_game_input.tick_mut(&self.cur_input);
        self.cache_input.pressing.clear();
    }

    #[inline]
    pub fn swap_frame_input(&mut self) {
        swap(&mut self.cur_frame_input, &mut self.last_frame_input);
    }

    #[inline]
    pub fn tick_game_frame_input(&mut self) {
        self.cur_frame_game_input.tick_mut(&self.cur_frame_input);
    }


    pub fn is_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        let last_input = &self.last_frame_input;
        let cur_input = &self.cur_frame_input;

        let any_last_not_input = keys.iter().any(|key| !last_input.pressing.contains(key));
        let all_cur_input = keys.iter().all(|key| cur_input.pressing.contains(key));

        return any_last_not_input && all_cur_input;
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let app_root = application_root_dir()?;
    let res_root = if app_root.join("res").exists() { app_root.join("res") } else { app_root };
    let display_config_path = res_root.join("config").join("display.ron");
    let assets_dir = res_root.join("assets");
    let game_data = GameDataBuilder::default()
        .with_bundle(RenderingBundle::<DefaultBackend>::new()
                         .with_plugin(render::blit::BlitToWindow::new(amethyst::renderer::bundle::Target::Main, render::WINDOW, true))
                         .with_plugin(
                             RenderToWindow::from_config_path(display_config_path)?
                                 .with_clear([0.0, 0.0, 0.0, 1.0])
                                 .with_target(render::WINDOW)
                         )
                         .with_plugin(RenderFlat2D::default())
                         .with_plugin(RenderFlat3D::default())
                         .with_plugin(RenderUi::default())
                         .with_plugin(render::RenderInvertColorCircle::default())
                     // .with_plugin(render::water_wave::RenderWaterWave::default().with_target(render::PTH_MAIN))
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(systems::AnimationSystem, "main_anime_system", &[])
        .with(systems::DebugSystem::default(), "debug_system", &[]);
    let mut game = Application::build(assets_dir, states::Loading::default())?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .build(game_data)?;
    game.run();
    Ok(())
}
