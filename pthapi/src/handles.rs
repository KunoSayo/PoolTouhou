use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU16, Ordering};

use alto::Buffer;
use image::GenericImageView;
use lewton::inside_ogg::OggStreamReader;
use shaderc::ShaderKind;
use wgpu::{Extent3d, ImageCopyTexture, Origin3d, TextureAspect, TextureDimension, TextureFormat, TextureUsages};
use wgpu_glyph::ab_glyph::FontArc;

use crate::Pools;
use crate::render::GlobalState;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub struct ResourcesHandles {
    pub res_root: PathBuf,
    assets_dir: PathBuf,
    pub fonts: RwLock<HashMap<String, FontArc>>,
    pub shaders: RwLock<HashMap<String, Vec<u32>>>,
    pub textures: RwLock<Vec<Texture>>,
    pub texture_map: RwLock<HashMap<String, usize>>,

    pub bgm_map: RwLock<HashMap<String, Arc<Buffer>>>,
}

impl std::fmt::Debug for ResourcesHandles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourcesHandle")
            .field("res_root", &self.res_root)
            .field("assets_dir", &self.assets_dir)
            .field("fonts", &self.fonts)
            .field("shaders", &self.shaders)
            .field("textures", &self.textures)
            .field("textures_map", &self.texture_map)
            .field("bgm_map", &self.bgm_map.read().map(|m| m.keys().cloned().collect::<Vec<_>>()))
            .finish()
    }
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

pub trait ProgressTracker: 'static + Send {
    fn end_loading(&mut self) {}

    fn new_error_num(&mut self) {}
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

impl Progress for () {
    type Tracker = ();

    fn num_loading(&self) -> u16 {
        0
    }

    fn num_finished(&self) -> u16 {
        0
    }

    fn error_nums(&self) -> u16 {
        0
    }

    fn create_tracker(&self) -> Self::Tracker {
        ()
    }
}

impl ProgressTracker for () {}

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
            bgm_map: Default::default(),
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

    pub fn load_with_compile_shader(&mut self, name: &str, file_path: &str, entry: &str, shader_kind: ShaderKind) -> shaderc::Result<()> {
        let target = self.assets_dir.join("shaders").join(file_path);
        let s = std::fs::read_to_string(target).expect("read shader file failed.");
        let compile_result = shaderc::Compiler::new().unwrap()
            .compile_into_spirv(&s, shader_kind, name, entry, None);
        let compile = compile_result?;
        if compile.get_num_warnings() > 0 {
            // log::warn!("compile shader warnings: {}", compile.get_warning_messages())
        }
        self.shaders.get_mut().unwrap().insert(name.to_string(), compile.as_binary().to_vec());
        Ok(())
    }

    fn load_texture_static_inner(self: Arc<Self>, name: &'static str, file_path: &'static str,
                                 state: &GlobalState, pools: &Pools, mut progress: impl ProgressTracker) {
        let state = unsafe { std::mem::transmute::<_, &'static GlobalState>(state) };
        let target = self.assets_dir.join("texture").join(file_path);
        pools.io_pool.spawn_ok(async move {
            let image = image::load_from_memory(&std::fs::read(target)
                .expect("read texture file failed"));

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
                        format: TextureFormat::Rgba8Unorm,
                        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                    });
                    state.queue.write_texture(
                        ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                            aspect: TextureAspect::All,
                        },
                        &rgba,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some((width * 4).try_into().unwrap()),
                            rows_per_image: Some((height).try_into().unwrap()),
                        },
                        size);
                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        compare: None,
                        lod_min_clamp: -100.0,
                        lod_max_clamp: 100.0,
                        ..wgpu::SamplerDescriptor::default()
                    });
                    {
                        let idx = {
                            let mut textures = self.textures.write().unwrap();
                            let idx = textures.len();
                            textures.push(Texture {
                                texture,
                                view,
                                sampler,
                            });
                            idx
                        };
                        let mut map = self.texture_map.write().unwrap();
                        map.insert(name.to_string(), idx);
                    }
                    state.queue.submit(None);
                }
                Err(e) => {
                    //todo: log here
                    progress.new_error_num();
                }
            }
        });
    }

    pub fn load_bgm_static(self: &Arc<Self>, name: &'static str, file_path: &'static str,
                           context: alto::Context, pools: &Pools, mut progress: impl ProgressTracker) {
        let this = self.clone();
        pools.io_pool.spawn_ok(async move {
            let target = this.assets_dir.join("sounds").join(file_path);
            let (audio_bin, freq, channel) = match file_path.rsplitn(2, ".").next().unwrap_or("ogg") {
                "mp3" => {
                    let mut decoder = minimp3::Decoder::new(std::fs::File::open(target).unwrap());
                    let mut fst = match decoder.next_frame() {
                        Ok(f) => f,
                        Err(e) => {
                            progress.new_error_num();
                            //todo: log here
                            panic!("Decoder mp3 file first audio frame failed for {:?}", e);
                        }
                    };
                    let freq = fst.sample_rate;
                    let channel = fst.channels;
                    let mut audio_bin = Vec::with_capacity(8 * 1024 * 1024);
                    audio_bin.append(&mut fst.data);
                    while let Ok(mut frame) = decoder.next_frame() {
                        debug_assert!(frame.channels == channel);
                        debug_assert!(frame.sample_rate == freq);
                        audio_bin.append(&mut frame.data);
                    }
                    audio_bin.resize(audio_bin.len(), 0);
                    (audio_bin, freq, channel as _)
                }
                _ => {
                    let mut sr = match OggStreamReader::new(std::fs::File::open(target).unwrap()) {
                        Ok(sr) => sr,
                        Err(e) => {
                            progress.new_error_num();
                            //todo: log here
                            panic!("Decode ogg file failed for {:?}", e);
                        }
                    };
                    let mut audio_bin = match sr.read_dec_packet_itl() {
                        Ok(Some(d)) => d,
                        _ => Vec::with_capacity(8 * 1024 * 1024),
                    };
                    if let Ok(Some(mut d)) = sr.read_dec_packet_itl() {
                        audio_bin.append(&mut d);
                    }
                    let freq = sr.ident_hdr.audio_sample_rate;
                    let channel = sr.ident_hdr.audio_channels;
                    while let Ok(Some(mut d)) = sr.read_dec_packet_itl() {
                        debug_assert!(sr.ident_hdr.audio_channels == channel);
                        debug_assert!(sr.ident_hdr.audio_sample_rate == freq);
                        audio_bin.append(&mut d);
                    }
                    audio_bin.resize(audio_bin.len(), 0);
                    (audio_bin, freq as _, channel)
                }
            };
            // log::info!("Loaded bgm {} and it has {} channels", name, channel);

            let buf = if channel == 1 {
                Arc::new(context.new_buffer::<alto::Mono<i16>, _>(&audio_bin, freq).unwrap())
            } else {
                Arc::new(context.new_buffer::<alto::Stereo<i16>, _>(&audio_bin, freq).unwrap())
            };
            let mut map = this.bgm_map.write().unwrap();
            map.insert(name.into(), buf);
        });
    }

    pub fn load_texture_static(self: &Arc<Self>, name: &'static str, file_path: &'static str,
                               state: &GlobalState, pools: &Pools, progress: impl ProgressTracker) {
        self.clone().load_texture_static_inner(name, file_path, state, pools, progress);
    }
}