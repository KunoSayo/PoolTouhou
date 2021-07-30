use std::collections::HashMap;
use std::sync::Arc;

use shaderc::ShaderKind;
use wgpu::{TextureFormat, TextureView};
use winit::window::Window;

use crate::handles::ResourcesHandles;
use crate::render::texture2d::Texture2DRender;

pub mod texture2d;

// pub use invert_color::InvertColorCircle;
// pub use invert_color::RenderInvertColorCircle;
// use glsl_layout::*;
//
// use std::path::PathBuf;
//
// pub mod blit;
// pub mod invert_color;
// pub mod water_wave;
//

pub struct GraphicsState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub handles: Arc<ResourcesHandles>,
}

pub struct MainRendererData {
    pub render2d: Texture2DRender,
    pub staging_belt: wgpu::util::StagingBelt,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    pub views: HashMap<String, TextureView>,
}

pub struct RenderViews<'a> {
    pub screen: &'a TextureView,
}

impl GraphicsState {
    pub(super) async fn new(window: &Window) -> Self {
        let mut res = ResourcesHandles::default();
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

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
        Self {
            surface,
            device,
            queue,
            swapchain_desc,
            swap_chain,
            handles: Arc::new(res),
        }
    }
}
