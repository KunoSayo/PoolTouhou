use std::collections::HashMap;
use std::sync::Arc;

use shaderc::ShaderKind;
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout,
           BindGroupLayoutDescriptor, BindGroupLayoutEntry,
           BindingResource, BindingType, Buffer, BufferBinding,
           BufferBindingType, BufferUsages, Extent3d, PowerPreference,
           ShaderStages, TextureDimension, TextureFormat, TextureUsages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::window::Window;

use crate::audio::OpenalData;
use crate::handles::{ResourcesHandles, Texture};
use crate::render::texture2d::Texture2DRender;

pub mod texture2d;
pub mod water_wave;

pub trait EffectRenderer: Send + Sync + std::fmt::Debug + 'static {
    fn alive(&self) -> bool {
        true
    }

    fn render(&mut self, state: &GlobalState, renderer: &MainRendererData);
}

#[derive(Default, Debug)]
pub struct DynamicData {
    pub msgs: Vec<String>,
    pub effects: Vec<Box<dyn EffectRenderer>>,
}

#[derive(Debug)]
pub struct GlobalState {
    pub surface: wgpu::Surface,
    pub surface_cfg: wgpu::SurfaceConfiguration,
    pub size_scale: [f32; 2],
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub handles: Arc<ResourcesHandles>,
    pub views: HashMap<String, crate::handles::Texture>,
    pub screen_uni_buffer: Buffer,
    pub screen_uni_bind_layout: BindGroupLayout,
    pub screen_uni_bind: BindGroup,

    pub dyn_data: DynamicData,
    pub al: Option<OpenalData>,
}

pub struct MainRenderViews {
    buffers: [Texture; 2],
    main: usize,
}

impl MainRenderViews {
    pub fn new(state: &GlobalState) -> Self {
        let size = state.get_screen_size();
        let texture_desc = wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.surface_cfg.format,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        };
        let sampler_desc = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..wgpu::SamplerDescriptor::default()
        };
        let buffer_a = {
            let texture = state.device.create_texture(&texture_desc);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let sampler = state.device.create_sampler(&sampler_desc);
            Texture {
                texture,
                view,
                sampler,
            }
        };

        let buffer_b = {
            let texture = state.device.create_texture(&texture_desc);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let sampler = state.device.create_sampler(&sampler_desc);
            Texture {
                texture,
                view,
                sampler,
            }
        };

        Self {
            buffers: [buffer_a, buffer_b],
            main: 0,
        }
    }

    pub fn get_screen(&self) -> &Texture {
        &self.buffers[self.main]
    }

    pub fn swap_screen(&mut self) -> (&Texture, &Texture) {
        let src = self.main;
        self.main = (self.main + 1) & 1;
        let dst = self.main;
        (&self.buffers[src], &self.buffers[dst])
    }
}

pub struct MainRendererData {
    pub render2d: Texture2DRender,
    pub staging_belt: wgpu::util::StagingBelt,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    pub views: MainRenderViews,
}

impl MainRendererData {
    pub fn new(state: &GlobalState) -> Self {
        let staging_belt = wgpu::util::StagingBelt::new(2048);
        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(state.handles.fonts.read().unwrap()
                .get("default").unwrap().clone())
                .build(&state.device, state.surface_cfg.format);

        let render2d = Texture2DRender::new(&state, state.surface_cfg.format.into(), &state.handles);
        let views = MainRenderViews::new(state);
        Self {
            render2d,
            staging_belt,
            glyph_brush,
            views
        }
    }
}


impl GlobalState {
    pub fn get_screen_size(&self) -> (u32, u32) {
        (self.surface_cfg.width, self.surface_cfg.height)
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.surface_cfg.width = width;
        self.surface_cfg.height = height;
        self.surface.configure(&self.device, &self.surface_cfg);
        let size = [width as f32, height as f32];
        self.size_scale = [size[0] / 1600.0, size[1] / 900.0];
        self.queue.write_buffer(&self.screen_uni_buffer, 0, bytemuck::cast_slice(&size));
    }

    pub(super) async fn new(window: &Window) -> Self {
        log::info!("New graphics state");
        let mut res = ResourcesHandles::default();
        let size = window.inner_size();
        log::info!("Got window inner size {:?}", size);

        let instance = wgpu::Instance::new(wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY));
        log::info!("Got wgpu  instance {:?}", instance);
        let surface = unsafe { instance.create_surface(window) };
        log::info!("Created surface {:?}", surface);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::util::power_preference_from_env().unwrap_or(PowerPreference::HighPerformance),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        log::info!("Got adapter {:?}", adapter);
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 5,
                        ..wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();
        log::info!("Requested device {:?} and queue {:?}", device, queue);

        let mut format = surface.get_preferred_format(&adapter)
            .expect("get format from swap chain failed");
        log::info!("Adapter chose {:?} for swap chain format", format);
        format = TextureFormat::Bgra8Unorm;
        log::info!("Using {:?} for swap chain format", format);

        let surface_cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_DST,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_cfg);

        res.load_font("default", "cjkFonts_allseto_v1.11.ttf");
        res.load_with_compile_shader("n2dt.v", "normal2dtexture.vert", "main", ShaderKind::Vertex);
        res.load_with_compile_shader("n2dt.f", "normal2dtexture.frag", "main", ShaderKind::Fragment);

        let screen_uni_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let size = [size.width as f32, size.height as f32];
        let screen_uni_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&size),
        });
        let screen_uni_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &screen_uni_bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &screen_uni_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            size_scale: [surface_cfg.width as f32 / 1600.0, surface_cfg.height as f32 / 900.0],
            surface,
            device,
            queue,
            surface_cfg,
            handles: Arc::new(res),
            views: Default::default(),
            screen_uni_buffer,
            screen_uni_bind_layout,
            screen_uni_bind,
            dyn_data: Default::default(),
            al: match OpenalData::new() {
                Ok(data) => Some(data),
                Err(e) => {
                    log::warn!("Cannot create openal context for {:?}" , e);
                    None
                }
            },
        }
    }
}
