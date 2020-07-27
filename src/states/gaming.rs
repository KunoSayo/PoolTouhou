use amethyst::{
    assets::*,
    core::{
        components::Transform,
    },
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
            //immutable borrow
            let mut core_storage = world.write_resource::<CoreStorage>();
            core_storage.player = Some(player);
        }
        init_camera(world);
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;

        let core_storage = world.read_resource::<CoreStorage>();
        let player = core_storage.player.unwrap();

        let mut transform = world.write_component::<Transform>();
        let input = core_storage.cur_input.as_ref().unwrap();
        const MOVE_SPEED: f32 = 5.0;
        let (mov_x, mov_y) = input.get_move(MOVE_SPEED);

        if let Some(pos) = transform.get_mut(player) {
            let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
            pos.set_translation_x((mov_x + raw_x).max(0.0).min(1600.0))
                .set_translation_y((mov_y + raw_y).max(0.0).min(900.0));
            if input.pressing.contains(&VirtualKeyCode::Q) {
                pos.prepend_rotation_x_axis(std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
            }
            if input.pressing.contains(&VirtualKeyCode::E) {
                pos.prepend_rotation_x_axis(-std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
            }
        }

        Trans::None
    }
}

const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 900.0;

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1600.0);
    let camera = Camera::from(camera::Projection::from(camera::Perspective
    ::new(ARENA_WIDTH / ARENA_HEIGHT,
          std::f32::consts::FRAC_PI_6,
          0.1,
          2000.0)));
    world
        .create_entity()
        .with(camera)
        .with(transform)
        .build();
}

fn init_sheep(world: &mut World) -> Entity {
    let mut pos = Transform::default();
    pos.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);
    let sprite_sheet_handle = load_sprite_sheet(world);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };
    world.create_entity()
        .with(sprite_render)
        .with(Sheep {
            sprite_render: None
        })
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
