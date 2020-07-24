use amethyst::{
    audio::AudioBundle,
    core::TransformBundle,
    ecs::Entity,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        RenderingBundle,
        types::DefaultBackend,
    },
    utils::application_root_dir,
};

// https://doc.rust-lang.org/book/
mod states;
mod entities;
mod input;

pub struct CoreStorage {
    player: Option<Entity>,
    last_input: input::InputData,
    cur_input: Option<input::InputData>,
}

impl Default for CoreStorage {
    fn default() -> Self {
        Self {
            player: None,
            last_input: input::InputData::empty(),
            cur_input: Some(input::InputData::empty()),
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");
    let assets_dir = app_root.join("assets");
    let game_data = GameDataBuilder::default()
        .with_bundle(RenderingBundle::<DefaultBackend>::new()
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)?
                    .with_clear([0, 0, 0, 1])
            )
            .with_plugin(RenderFlat2D::default())
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(AudioBundle::default())?
        .with(input::InputDataSystem, "main_input_system", &["input_system"]);
    let mut game = Application::new(assets_dir, states::Gaming, game_data)?;
    game.run();
    Ok(())
}