use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use futures::task::{LocalSpawn, LocalSpawnExt, SpawnExt};
use image::GenericImageView;
use shaderc::ShaderKind;
use wgpu::{Extent3d, ImageCopyTexture, Origin3d, Texture, TextureDimension, TextureFormat, TextureUsage};
use wgpu_glyph::ab_glyph::FontArc;

use crate::{GraphicsState, Pools};

pub struct ResourcesHandles {
    pub res_root: PathBuf,
    assets_dir: PathBuf,
    pub fonts: RefCell<HashMap<String, FontArc>>,
    pub shaders: RefCell<HashMap<String, Vec<u32>>>,
    pub textures: RefCell<Vec<Texture>>,
    pub texture_map: RefCell<HashMap<String, usize>>,
}

#[derive(Default)]
struct CounterInner {
    loading: AtomicU16,
    finished: AtomicU16,
    errors: AtomicU16,
}

#[derive(Default)]
pub struct CounterProgress {
    inner: Arc<CounterInner>,
}

pub struct CounterProgressTracker {
    loaded: bool,
    inner: Arc<CounterInner>,
}

pub trait Progress {
    type Tracker: ProgressTracker;
    fn num_loading(&self) -> u16;

    fn num_finished(&self) -> u16;

    fn error_nums(&self) -> u16;
    fn create_tracker(&self) -> Self::Tracker;
}

pub trait ProgressTracker: 'static {
    fn end_loading(&mut self);

    fn new_error_num(&mut self);
}

impl Progress for CounterProgress {
    type Tracker = CounterProgressTracker;

    fn num_loading(&self) -> u16 {
        self.inner.loading.load(Ordering::Acquire)
    }

    fn num_finished(&self) -> u16 {
        self.inner.finished.load(Ordering::Acquire)
    }

    fn error_nums(&self) -> u16 {
        self.inner.errors.load(Ordering::Acquire)
    }

    fn create_tracker(&self) -> Self::Tracker {
        self.inner.loading.fetch_add(1, Ordering::AcqRel);
        CounterProgressTracker {
            loaded: false,
            inner: self.inner.clone(),
        }
    }
}

impl ProgressTracker for () {
    fn end_loading(&mut self) {}

    fn new_error_num(&mut self) {}
}

impl ProgressTracker for CounterProgressTracker {
    fn end_loading(&mut self) {
        self.loaded = true;
        self.inner.loading.fetch_sub(1, Ordering::AcqRel);
        self.inner.finished.fetch_add(1, Ordering::AcqRel);
    }

    fn new_error_num(&mut self) {
        self.end_loading();
        self.inner.errors.fetch_add(1, Ordering::AcqRel);
    }
}

impl Drop for CounterProgressTracker {
    fn drop(&mut self) {
        if !self.loaded {
            //now loaded.
            self.end_loading();
        }
    }
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
        self.fonts.get_mut().insert(name.to_string(), font_arc);
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
                self.shaders.get_mut().insert(name.to_string(), compile.as_binary().to_vec());
            }
            Err(e) => {
                log::warn!("compile shader {} error: {}", file_path, e);
            }
        }
    }

    fn load_texture_static_inner(self: Rc<Self>, name: &'static str, file_path: &'static str,
                                 state: &GraphicsState, pools: &Pools, mut progress: impl ProgressTracker) {
        let state = unsafe { std::mem::transmute::<_, &'static GraphicsState>(state) };
        let target = self.assets_dir.join("texture").join(file_path);
        let load_future = pools.io_pool.spawn_with_handle(async {
            image::load_from_memory(&std::fs::read(target)
                .expect("read texture file failed"))
        }).expect("use io pool to read image failed");

        pools.render_spawner.spawn_local(async move {
            let image = load_future.await;
            match image {
                Ok(image) => {
                    let rgba = image.to_rgba8();
                    let (width, height) = image.dimensions();

                    let size = Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    };
                    let texture = state.device.create_texture(&wgpu::TextureDescriptor {
                        label: None,
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Rgba8UnormSrgb,
                        usage: TextureUsage::COPY_SRC | TextureUsage::COPY_DST,
                    });
                    state.queue.write_texture(
                        ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                        },
                        &rgba,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some((width * 4).try_into().unwrap()),
                            rows_per_image: Some((height).try_into().unwrap()),
                        },
                        size);
                    {
                        let mut textures = self.textures.borrow_mut();
                        let mut map = self.texture_map.borrow_mut();
                        let idx = textures.len();
                        textures.push(texture);
                        map.insert(name.to_string(), idx);
                    }
                }
                Err(e) => {
                    log::warn!("load image error: {}", e);
                    progress.new_error_num();
                }
            }
        }).expect("use render pool to spawn ");
    }

    pub fn load_texture_static(self: &Rc<Self>, name: &'static str, file_path: &'static str,
                               state: &GraphicsState, pools: &Pools, mut progress: impl ProgressTracker) {
        self.clone().load_texture_static_inner(name, file_path, state, pools, progress);
    }
}