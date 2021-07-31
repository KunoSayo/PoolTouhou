use std::collections::HashMap;
use std::sync::Arc;

use shaderc::ShaderKind;
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout,
           BindGroupLayoutDescriptor, BindGroupLayoutEntry,
           BindingResource, BindingType, Buffer, BufferBinding,
           BufferBindingType, BufferUsage, ShaderStage, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::window::Window;

use crate::handles::ResourcesHandles;
use crate::render::texture2d::Texture2DRender;

pub mod texture2d;
pub mod water_wave;

pub trait RenderEffect {
    fn render(&self, src: &[&TextureView], dest: &TextureView);
}

pub struct GraphicsState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub handles: Arc<ResourcesHandles>,
    pub views: HashMap<String, crate::handles::Texture>,
    pub screen_uni_buffer: Buffer,
    pub screen_uni_bind_layout: BindGroupLayout,
    pub screen_uni_bind: BindGroup,
}

pub struct MainRendererData {
    pub render2d: Texture2DRender,
    pub staging_belt: wgpu::util::StagingBelt,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
}

impl MainRendererData {
    pub fn new(state: &GraphicsState) -> Self {
        let staging_belt = wgpu::util::StagingBelt::new(2048);
        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(state.handles.fonts.read().unwrap()
                .get("default").unwrap().clone())
                .build(&state.device, state.swapchain_desc.format);


        let render2d = Texture2DRender::new(&state, state.swapchain_desc.format.into(), &state.handles);

        Self {
            render2d,
            staging_belt,
            glyph_brush,
        }
    }
}


pub struct RenderViews<'a> {
    pub screen: &'a TextureView,
}

impl GraphicsState {
    pub(super) async fn new(window: &Window) -> Self {
        log::debug!("New graphics state");
        let mut res = ResourcesHandles::default();
        let size = window.inner_size();
        log::debug!("Got window inner size {:?}", size);

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        log::debug!("Got wgpu  instance {:?}", instance);

        let surface = unsafe { instance.create_surface(window) };
        log::debug!("Created surface {:?}", surface);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        log::debug!("Got adapter {:?}", adapter);

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
        log::debug!("Requested device {:?} and queue {:?}", device, queue);

        let mut format = adapter.get_swap_chain_preferred_format(&surface).expect("get format from swap chain failed");

        log::info!("Adapter chose {:?} for swap chain format", format);
        if format.describe().srgb {
            unsafe {
                let idx: i32 = std::mem::transmute(format);
                format = std::mem::transmute(idx - 1);
            }
        }

        log::info!("Using {:?} for swap chain format", format);

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swapchain_desc);

        res.load_font("default", "cjkFonts_allseto_v1.11.ttf");
        res.load_with_compile_shader("n2dt.v", "normal2dtexture.vert", "main", ShaderKind::Vertex);
        res.load_with_compile_shader("n2dt.f", "normal2dtexture.frag", "main", ShaderKind::Fragment);

        let screen_uni_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let size = [swapchain_desc.width as f32, swapchain_desc.height as f32];
        let screen_uni_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsage::UNIFORM,
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
            surface,
            device,
            queue,
            swapchain_desc,
            swap_chain,
            handles: Arc::new(res),
            views: Default::default(),
            screen_uni_buffer,
            screen_uni_bind_layout,
            screen_uni_bind,
        }
    }
}
