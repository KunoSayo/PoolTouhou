use amethyst::{
    assets::*,
    core::{
        components::Transform
    },
    ecs::Entity,
    input::VirtualKeyCode,
    prelude::*,
    renderer::*,
};

use crate::CoreStorage;
use crate::entities::{Enemy, EnemyBullet, PlayerBullet, Sheep};
use crate::handles::TextureHandles;
use crate::states::pausing::Pausing;
use crate::systems::Player;

#[derive(Default)]
pub struct Gaming;

impl SimpleState for Gaming {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Sheep>();
        world.register::<Enemy>();
        world.register::<PlayerBullet>();
        world.register::<EnemyBullet>();
        world.insert(TextureHandles::default());
        let player = setup_sheep(world);
        {
            //immutable borrow
            let mut core_storage = world.write_resource::<CoreStorage>();
            core_storage.player = Some(player);
        }
        setup_camera(world);

        crate::ui::debug::setup_debug_text(world);
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let mut core_storage = world.write_resource::<CoreStorage>();
        if !core_storage.tick_sign {
            core_storage.swap_input();
            core_storage.tick_sign = true;
            let player = core_storage.player.unwrap();

            let mut transform = world.write_component::<Transform>();
            let input = core_storage.cur_input.as_ref().unwrap();

            if let Some(pos) = transform.get_mut(player) {
                if input.pressing.contains(&VirtualKeyCode::Q) {
                    pos.prepend_rotation_x_axis(std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
                }
                if input.pressing.contains(&VirtualKeyCode::E) {
                    pos.prepend_rotation_x_axis(-std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
                }
            }

            if core_storage.is_press(Box::from([VirtualKeyCode::Escape])) {
                return Trans::Push(Box::new(Pausing {}));
            }
        }
        Trans::None
    }
}

const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 900.0;

fn setup_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1600.0);
    let camera = Camera::from(camera::Projection::from(camera::Perspective
    ::new(ARENA_WIDTH / ARENA_HEIGHT,
          std::f32::consts::FRAC_PI_6,
          0.1,
          3200.0)));
    world
        .create_entity()
        .with(camera)
        .with(transform)
        .build();
}

fn setup_sheep(world: &mut World) -> Entity {
    let mut pos = Transform::default();
    pos.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);
    let sprite_sheet_handle = load_sprite_sheet(world, "texture/暗夜.png", "texture/sheep.ron");
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };
    world.create_entity()
        .with(sprite_render)
        .with(pos.clone())
        .with(Enemy::new(5000.0, 30. * 30.))
        .with(Transparent)
        .build();
    pos.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);
    // pos.set_scale(Vector3::new(1.0, 1.0, 1.0));
    let sprite_sheet_handle = load_sprite_sheet(world, "texture/sheep.png", "texture/sheep.ron");
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };

    let sprite_sheet_handle = load_sprite_sheet(world, "texture/sheepBullet.png", "texture/sheepBullet.ron");
    {
        let mut texture_handle = world.try_fetch_mut::<TextureHandles>().unwrap();
        texture_handle.player_bullet = Some(SpriteRender { sprite_sheet: sprite_sheet_handle, sprite_number: 0 });
    }

    world.create_entity()
        .with(sprite_render)
        .with(Sheep {
            sprite_render: None
        })
        .with(Player::new(5.0))
        .with(Transparent)
        .with(pos)
        .build()
}

fn load_sprite_sheet(world: &mut World, name: &str, ron_name: &str) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            name,
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_name, // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}
