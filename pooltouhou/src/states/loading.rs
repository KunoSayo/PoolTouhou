use amethyst::{
    assets::*,
    core::{
        components::Transform
    },
    prelude::*,
    renderer::*,
};
use amethyst::audio::{FlacFormat, Mp3Format, OggFormat, SourceHandle, WavFormat};

use crate::component::{Enemy, EnemyBullet, PlayerBullet, Sheep};
use crate::CoreStorage;
use crate::handles::ResourcesHandles;
use crate::script::ScriptManager;
use crate::states::{ARENA_HEIGHT, ARENA_WIDTH, Gaming, load_sprite_sheet};

pub type ProgressType = ProgressCounter;

#[derive(Default)]
pub struct Loading {
    progress: ProgressType
}


impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Sheep>();
        world.register::<Enemy>();
        world.register::<PlayerBullet>();
        world.register::<EnemyBullet>();
        world.insert(ResourcesHandles::default());

        load_sheep(world, &mut self.progress);


        load_texture(world, "bullet".into(), "bullet.ron".into(), &mut self.progress);
        load_texture(world, "circle_red".into(), "circle.ron".into(), &mut self.progress);
        load_texture(world, "circle_blue".into(), "circle.ron".into(), &mut self.progress);
        load_texture(world, "circle_green".into(), "circle.ron".into(), &mut self.progress);
        load_texture(world, "circle_yellow".into(), "circle.ron".into(), &mut self.progress);
        load_texture(world, "circle_purple".into(), "circle.ron".into(), &mut self.progress);
        load_texture(world, "zzzz".into(), "zzzz.ron".to_string(), &mut self.progress);


        setup_camera(world);

        crate::ui::debug::setup_debug_text(world);

        let mut script_manager = ScriptManager::default();
        script_manager.load_scripts();


        world.insert(script_manager);


        println!("Loading state started.");
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress.num_loading() == 0 {
            println!("loaded {} resources.", self.progress.num_finished());
            match self.progress.complete() {
                Completion::Failed => {
                    for x in self.progress.errors() {
                        eprintln!("load {} failed for {}", x.asset_name, x.error);
                    }
                }
                _ => {}
            }
            Trans::Push(Box::new(Gaming::default()))
        } else {
            Trans::None
        }
    }


    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut core_storage = data.world.write_resource::<CoreStorage>();
        core_storage.swap_input();
    }
}

fn setup_camera(world: &mut World) {
    let mut transform = Transform::default();
    // transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 16.0);
    // let camera = Camera::from(camera::Projection::from(camera::Perspective
    // ::new(ARENA_WIDTH / ARENA_HEIGHT,
    //       std::f32::consts::FRAC_PI_6,
    //       0.1,
    //       3200.0)));
    transform.set_translation_xyz(0.0, 0.0, 16.0);
    let camera = camera::Camera::orthographic(0.0, ARENA_WIDTH, -ARENA_HEIGHT, 0.0,
                                              0.1, 32.0);
    world
        .create_entity()
        .with(camera)
        .with(transform)
        .build();
}


fn load_sound(world: &mut World, name: String) {
    let get_format = |s: &str| -> Box<dyn amethyst::assets::Format<_>> {
        match s {
            "wav" => Box::new(WavFormat),
            "ogg" => Box::new(OggFormat),
            "mp3" => Box::new(Mp3Format),
            "flac" => Box::new(FlacFormat),
            _ => {
                panic!("Not supported format!");
            }
        }
    };
    let loader = world.read_resource::<Loader>();
    let handle: SourceHandle = loader
        .load(format!("sounds/{}", name),
              get_format(*name.split(".").collect::<Vec<&str>>().last().unwrap()),
              (), &world.read_resource());
    let mut handles = world.fetch_mut::<ResourcesHandles>();
    handles.sounds.insert(name, handle);
}

fn load_sheep(world: &mut World, progress: &mut ProgressType) {
    let mut pos = Transform::default();

    pos.set_translation_xyz(ARENA_WIDTH * 0.5, 100.0, crate::PLAYER_Z);
    // pos.set_scale(Vector3::new(1.0, 1.0, 1.0));
    load_texture(world, "sheep".to_string(), "sheep.ron".to_string(), progress);
    load_texture(world, "sheepBullet".into(), "sheepBullet.ron".into(), progress);
    {
        let mut texture_handle = world.try_fetch_mut::<ResourcesHandles>().unwrap();
        let ss = texture_handle.textures.get("sheepBullet").unwrap();
        texture_handle.player_bullet = Some(ss.clone());
    }
}


fn load_texture(world: &mut World, name: String, ron: String, progress: &mut ProgressType) {
    let path = "texture/".to_owned() + &name + ".png";
    load_texture_with_path(world, name, path, ron, progress);
}

fn load_texture_with_path(world: &mut World, name: String, path: String, ron: String, progress: &mut ProgressType) {
    let handle = load_sprite_sheet(world, &path,
                                   &("texture/".to_owned() + &ron), progress);
    let mut texture_handle = world.try_fetch_mut::<ResourcesHandles>().unwrap();
    texture_handle.textures.insert(name, SpriteRender { sprite_sheet: handle, sprite_number: 0 });
}

