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
use amethyst::core::ecs::Join;

use crate::component::{Enemy, EnemyBullet, PlayerBullet, Sheep};
use crate::CoreStorage;
use crate::handles::TextureHandles;
use crate::script::{ScriptGameData, ScriptManager};
use crate::script::script_context::{ScriptContext, TempGameContext};
use crate::states::pausing::Pausing;
use crate::systems::game_system::CollideType;
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

        let mut script_manager = ScriptManager::default();
        script_manager.load_scripts();
        let script = script_manager.get_script(&"main".to_string()).unwrap();

        let mut context = ScriptContext::new(&script, vec![]);
        let mut game = ScriptGameData {
            player_tran: Transform::default(),
            submit_command: vec![],
            calc_stack: vec![],
        };

        let mut temp = TempGameContext {
            tran: None,
        };
        context.execute_function(&"start".to_string(), &mut game, &mut script_manager, &mut temp);
        for x in game.submit_command {
            match x {
                crate::script::ScriptGameCommand::SummonEnemy(name, x, y, hp, collide, script_name, args) => {
                    setup_enemy(world, &mut script_manager, (name, x, y, hp, collide, script_name, args))
                }
                _ => panic!("没实现哪里来的命令（大声）")
            }
        }
        world.insert(script_manager);
        println!("Gaming state started.");
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let mut core_storage = world.write_resource::<CoreStorage>();
        if !core_storage.tick_sign {
            core_storage.swap_input();
            core_storage.tick_sign = true;
            let mut transforms = world.write_component::<Transform>();
            if let Some(player) = core_storage.player {
                let input = core_storage.cur_input.as_ref().unwrap();

                if let Some(pos) = transforms.get_mut(player) {
                    if input.pressing.contains(&VirtualKeyCode::Q) {
                        pos.prepend_rotation_x_axis(std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
                    }
                    if input.pressing.contains(&VirtualKeyCode::E) {
                        pos.prepend_rotation_x_axis(-std::f32::consts::FRAC_1_PI * 15.0 / 180.0);
                    }
                }
            }

            if core_storage.is_press(Box::from([VirtualKeyCode::Escape])) {
                return Trans::Push(Box::new(Pausing {}));
            }

            let cameras = world.read_component::<Camera>();
            if let Some((camera, transform, _)) = (&cameras, &transforms, &world.entities()).join().next() {
                let mut inverse_args = world.write_resource::<crate::render::CameraUniformArgs>();
                let projection = camera.projection().as_matrix();
                let view = &transform.view_matrix();
                inverse_args.projection = [[projection.m11, projection.m21, projection.m31, projection.m41],
                    [projection.m12, projection.m22, projection.m32, projection.m42],
                    [projection.m13, projection.m23, projection.m33, projection.m43],
                    [projection.m14, projection.m24, projection.m34, projection.m44]].into();
                inverse_args.view = [[view.m11, view.m21, view.m31, view.m41],
                    [view.m12, view.m22, view.m32, view.m42],
                    [view.m13, view.m23, view.m33, view.m43],
                    [view.m14, view.m24, view.m34, view.m44]].into();
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

fn load_bullet(world: &mut World, name: String, ron: String) {
    let handle = load_sprite_sheet(world, &*("texture/".to_owned() + &name + ".png"),
                                   &*("texture/".to_owned() + &ron + ".ron"));
    let mut texture_handle = world.try_fetch_mut::<TextureHandles>().unwrap();
    texture_handle.bullets.insert(name, SpriteRender { sprite_sheet: handle, sprite_number: 0 });
}

fn setup_sheep(world: &mut World) -> Entity {
    let mut pos = Transform::default();

    pos.set_translation_xyz(ARENA_WIDTH * 0.5, 100.0, 0.0125);
    // pos.set_scale(Vector3::new(1.0, 1.0, 1.0));
    let sprite_sheet_handle = load_sprite_sheet(world, "texture/sheep.png", "texture/sheep.ron");
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };
    let sheep_bullet = load_sprite_sheet(world, "texture/sheepBullet.png", "texture/sheepBullet.ron");

    {
        let mut texture_handle = world.try_fetch_mut::<TextureHandles>().unwrap();
        texture_handle.player_bullet = Some(SpriteRender { sprite_sheet: sheep_bullet, sprite_number: 0 });
    }

    load_bullet(world, "bullet".to_string(), "bullet".to_string());
    load_bullet(world, "circle_red".to_string(), "circle".to_string());
    load_bullet(world, "circle_blue".to_string(), "circle".to_string());
    load_bullet(world, "circle_green".to_string(), "circle".to_string());
    load_bullet(world, "circle_yellow".to_string(), "circle".to_string());
    load_bullet(world, "circle_purple".to_string(), "circle".to_string());
    load_bullet(world, "zzzz".to_string(), "zzzz".to_string());


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

fn setup_enemy(world: &mut World, script_manager: &mut ScriptManager, (name, x, y, hp, collide, script_name, args): (String, f32, f32, f32, CollideType, String, Vec<f32>)) {
    let mut pos = Transform::default();
    pos.set_translation_xyz(x, y, 0.0);
    let sprite_sheet_handle = load_sprite_sheet(world,
                                                &*("texture/".to_owned() + &*name + ".png"),
                                                &*("texture/".to_owned() + &*name + ".ron"));
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };

    let ctx;
    if let Some(script) = script_manager.get_script(&script_name) {
        ctx = ScriptContext::new(script, args);
    } else {
        let script = script_manager.load_script(&script_name).unwrap();
        ctx = ScriptContext::new(script, args);
    }
    world.create_entity()
        .with(sprite_render)
        .with(pos.clone())
        .with(Enemy::new(hp, collide, ctx))
        .with(Transparent)
        .build();
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
