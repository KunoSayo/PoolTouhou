use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use image::GenericImageView;
use shaderc::ShaderKind;
use wgpu::{Extent3d, Origin3d, Texture, TextureCopyView, TextureDimension, TextureFormat, TextureUsage};
use wgpu_glyph::ab_glyph::FontArc;

use crate::GraphicsState;

pub struct ResourcesHandles {
    pub res_root: PathBuf,
    assets_dir: PathBuf,
    pub fonts: RwLock<HashMap<String, FontArc>>,
    pub shaders: RwLock<HashMap<String, Vec<u32>>>,
    pub textures: RwLock<Vec<Texture>>,
    pub texture_map: RwLock<HashMap<String, usize>>,
}

impl Default for ResourcesHandles {
    fn default() -> Self {
        let app_root = std::env::current_dir().unwrap();
        let res_root = if app_root.join("res").exists() { app_root.join("res") } else { app_root };
        let assets_dir = res_root.join("assets");
        Self {
            res_root,
            assets_dir,
            fonts: Default::default(),
            shaders: Default::default(),
            textures: Default::default(),
            texture_map: Default::default(),
        }
    }
}

impl ResourcesHandles {
    pub fn load_font(&mut self, name: &str, file_path: &str) {
        let target = self.assets_dir.join("font").join(file_path);
        let font_arc = wgpu_glyph::ab_glyph::FontArc::try_from_vec(
            std::fs::read(target)
                .expect("read font file failed")).unwrap();
        self.fonts.get_mut().unwrap().insert(name.to_string(), font_arc);
    }

    pub fn load_with_compile_shader(&mut self, name: &str, file_path: &str, entry: &str, shader_kind: ShaderKind) {
        let target = self.assets_dir.join("shaders").join(file_path);
        let s = std::fs::read_to_string(target).expect("read shader file failed.");
        let compile_result = shaderc::Compiler::new().unwrap()
            .compile_into_spirv(&s, shader_kind, name, entry, None);
        match compile_result {
            Ok(compile) => {
                if compile.get_num_warnings() > 0 {
                    log::warn!("compile shader warnings: {}", compile.get_warning_messages())
                }
                self.shaders.get_mut().unwrap().insert(name.to_string(), compile.as_binary().to_vec());
            }
            Err(e) => {
                log::warn!("compile shader error: {}", e);
            }
        }
    }

    pub fn load_texture(&mut self, name: &str, file_path: &str, state: &mut GraphicsState) {
        let target = self.assets_dir.join("texture").join(file_path);

        let image = image::load_from_memory(&std::fs::read(target)
            .expect("read texture file failed"));
        match image {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let (width, height) = image.dimensions();

                let size = Extent3d {
                    width,
                    height,
                    depth: 1,
                };
                let texture = state.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    usage: TextureUsage::STORAGE,
                });
                state.queue.write_texture(
                    TextureCopyView {
                        texture: &texture,
                        mip_level: 1,
                        origin: Origin3d::ZERO,
                    },
                    &rgba,
                    wgpu::TextureDataLayout {
                        offset: 0,
                        bytes_per_row: width * 4,
                        rows_per_image: height,
                    },
                    size);
                let textures = self.textures.get_mut().unwrap();
                let map = self.texture_map.get_mut().unwrap();
                let idx = textures.len();
                textures.push(texture);
                map.insert(name.to_string(), idx);
            }
            Err(e) => {
                log::warn!("load image error: {}", e);
            }
        }
    }
}