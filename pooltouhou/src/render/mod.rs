use std::collections::HashMap;
use std::sync::Arc;

use wgpu::{BindGroup, BindGroupLayout, Extent3d, TextureDimension, TextureUsages};
use wgpu::Buffer;

use root::audio::OpenalData;
use root::handles::{ResourcesHandles, Texture};
use root::render::texture2d::Texture2DRender;

use crate as root;
use crate::config::Config;
use crate::handles::TextureInfo;

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
    pub config: Config,
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
                info: TextureInfo::new(size.0, size.1),
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
                info: TextureInfo::new(size.0, size.1),
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
            views,
        }
    }
}


impl GlobalState {
    pub fn get_screen_size(&self) -> (u32, u32) {
        (self.surface_cfg.width, self.surface_cfg.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_cfg.width = width;
        self.surface_cfg.height = height;
        self.surface.configure(&self.device, &self.surface_cfg);
        let size = [width as f32, height as f32];
        self.size_scale = [size[0] / 1600.0, size[1] / 900.0];
        self.queue.write_buffer(&self.screen_uni_buffer, 0, bytemuck::cast_slice(&size));
    }
}
