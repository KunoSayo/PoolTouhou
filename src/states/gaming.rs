use amethyst::{
    assets::*,
    core::*,
    ecs::Entity,
    input::VirtualKeyCode,
    prelude::*,
    renderer::*,
};

use crate::CoreStorage;
use crate::entities::Sheep;

pub struct Gaming;

impl SimpleState for Gaming {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Sheep>();
        let player = init_sheep(world);
        {
            let mut core_storage = world.write_resource::<CoreStorage>();
            core_storage.player = Some(player);
        }
        init_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let core_storage = data.world.read_resource::<CoreStorage>();
        let mut transform = data.world.write_component::<Transform>();
        let input = core_storage.cur_input.as_ref().unwrap();
        const MOVE_SPEED: f32 = 2.5;
        if input.pressing.contains(&VirtualKeyCode::Up) {
            let pos = transform.get_mut(core_storage.player.unwrap()).expect("Where is my sheep");
            pos.set_translation_y((pos.translation().y + MOVE_SPEED).min(850.0));
        }
        if input.pressing.contains(&VirtualKeyCode::Down) {
            let pos = transform.get_mut(core_storage.player.unwrap()).expect("Where is my sheep");
            pos.set_translation_y((pos.translation().y - MOVE_SPEED).max(50.0));
        }
        if input.pressing.contains(&VirtualKeyCode::Left) {
            let pos = transform.get_mut(core_storage.player.unwrap()).expect("Where is my sheep");
            pos.set_translation_x((pos.translation().x - MOVE_SPEED).max(50.0));
        }
        if input.pressing.contains(&VirtualKeyCode::Right) {
            let pos = transform.get_mut(core_storage.player.unwrap()).expect("Where is my sheep");
            pos.set_translation_x((pos.translation().x + MOVE_SPEED).min(1550.0));
        }
        Trans::None
    }
}

const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 900.0;

fn init_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);
    world
        .create_entity()
        .with(Camera::standard_2d(1600.0, 900.0))
        .with(transform)
        .build();
}

fn init_sheep(world: &mut World) -> Entity {
    let mut pos = Transform::default();
    pos.set_translation_xyz(50.0, 50.0, 0.5);
    let sprite_sheet_handle = load_sprite_sheet(world);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    };
    world.create_entity()
        .with(sprite_render)
        .with(Sheep::new())
        .with(pos)
        .build()
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/sheep.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/sheep.ron", // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}
