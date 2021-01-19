use amethyst::{
    assets::*,
    prelude::*,
    renderer::*,
};

pub use gaming::Gaming;
pub use loading::Loading;

pub type ProgressType = ProgressCounter;

pub mod gaming;
pub mod pausing;
pub mod loading;

pub const ARENA_WIDTH: f32 = 1600.0;
pub const ARENA_HEIGHT: f32 = 900.0;

pub fn load_sprite_sheet(world: &mut World, path: &str, ron_name: &str, progress: Option<&mut ProgressType>) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let is_some = progress.is_some();
    let x = progress.unwrap();
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        if is_some {
            loader.load(
                path,
                ImageFormat::default(),
                x,
                &texture_storage,
            )
        } else {
            loader.load(
                path,
                ImageFormat::default(),
                (),
                &texture_storage,
            )
        }
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    if is_some {
        loader.load(
            ron_name, // Here we load the associated ron file
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_store,
        )
    } else {
        loader.load(
            ron_name, // Here we load the associated ron file
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_store,
        )
    }
}
