mod handles;
mod states;
mod systems;
mod render;

use std::mem::swap;
use winit::event::{VirtualKeyCode, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;
use crate::handles::ResourcesHandles;
use wgpu_glyph::ab_glyph::FontVec;
use std::sync::Arc;
use wgpu::{RenderPassDescriptor, RenderPassColorAttachmentDescriptor, LoadOp, Color, Operations, ShaderModuleDescriptor};
use std::sync::Mutex;
use futures::executor::{LocalPool};
use shaderc::ShaderKind;
use crate::states::GameState;
use std::time::Duration;


// https://doc.rust-lang.org/book/

pub const PLAYER_Z: f32 = 0.0;

// pub struct GameCore {
//     player: Option<Player>,
//     cur_game_input: input::GameInputData,
//     last_input: input::RawInputData,
//     cur_input: input::RawInputData,
//     cache_input: input::RawInputData,
//     last_frame_input: input::RawInputData,
//     cur_frame_input: input::RawInputData,
//     cur_frame_game_input: input::GameInputData,
//     commands: Vec<ScriptGameCommand>,
//     next_tick_time: std::time::SystemTime,
//     tick: u128,
//     al: Option<audio::OpenalData>,
// }
//
// impl Default for GameCore {
//     fn default() -> Self {
//         let alto = match audio::OpenalData::new() {
//             Ok(a) => Some(a),
//             Err(e) => {
//                 eprintln!("load openal failed for {}", e);
//                 None
//             }
//         };
//         Self {
//             player: None,
//             cur_game_input: Default::default(),
//             last_input: input::RawInputData::empty(),
//             cur_input: input::RawInputData::empty(),
//             cache_input: RawInputData::default(),
//             last_frame_input: RawInputData::default(),
//             cur_frame_input: input::RawInputData::empty(),
//             cur_frame_game_input: Default::default(),
//             commands: vec![],
//             next_tick_time: std::time::SystemTime::now(),
//             tick: 0,
//             al: alto,
//         }
//     }
// }
//
// impl GameCore {
//     #[inline]
//     pub fn tick_input(&mut self) {
//         swap(&mut self.last_input, &mut self.cur_input);
//         swap(&mut self.cur_input, &mut self.cache_input);
//         self.cur_game_input.tick_mut(&self.cur_input);
//         self.cache_input.pressing.clear();
//     }
//
//     #[inline]
//     pub fn swap_frame_input(&mut self) {
//         swap(&mut self.cur_frame_input, &mut self.last_frame_input);
//     }
//
//     #[inline]
//     pub fn tick_game_frame_input(&mut self) {
//         self.cur_frame_game_input.tick_mut(&self.cur_frame_input);
//     }
//
//
//     pub fn is_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
//         let last_input = &self.last_frame_input;
//         let cur_input = &self.cur_frame_input;
//
//         let any_last_not_input = keys.iter().any(|key| !last_input.pressing.contains(key));
//         let all_cur_input = keys.iter().all(|key| cur_input.pressing.contains(key));
//
//         return any_last_not_input && all_cur_input;
//     }
// }


pub struct GraphicsState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    staging_belt: wgpu::util::StagingBelt,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    handles: ResourcesHandles,
}

impl GraphicsState {
    async fn new(window: &Window) -> Self {
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

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swapchain_desc);

        let staging_belt = wgpu::util::StagingBelt::new(1024);

        res.load_font("default", "cjkFonts_allseto_v1.11.ttf");
        res.load_with_compile_shader("2dt.v", "normal2dtexture.vert", "main", ShaderKind::Vertex);
        res.load_with_compile_shader("2dt.f", "normal2dtexture.frag", "main", ShaderKind::Fragment);

        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(res.fonts.read().unwrap()
                .get("default").unwrap().clone()).build(&device, swapchain_desc.format);

        Self {
            surface,
            device,
            queue,
            swapchain_desc,
            swap_chain,
            size,
            staging_belt,
            glyph_brush,
            handles: res,
        }
    }
}

impl GraphicsState {
    pub fn render_once(&mut self, dt: f32) {
        let frame = self.swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture")
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        {
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            systems::debug_system::DEBUG.render(self, dt, &frame.view, &mut encoder)
        }
        self.queue.submit(Some(encoder.finish()));
    }
}

impl PthData {
    pub fn render_thread(&self) {
        log::info!("created render thread.");
        let mut last_render_time = std::time::Instant::now();
        loop {
            let now = std::time::Instant::now();
            let dur = now.duration_since(last_render_time);

            {
                let mut state = self.graphics_state.lock()
                    .expect("lock graphics failed");
                state.render_once(dur.as_secs_f32());
            }
            last_render_time = now;
            std::thread::yield_now();
        }
    }

    pub fn logic_thread(&self) {
        log::info!("created logic thread.");

        let mut last_tick_time = std::time::SystemTime::now();

        let interval = Duration::from_secs_f64(1.0 / 60.0);
        let one_ms = Duration::from_millis(1);
        loop {
            //tick here

            let now = std::time::SystemTime::now();
            let dur = match now.duration_since(last_tick_time) {
                Ok(dur) => dur,
                Err(e) => Duration::from_secs(1)
            };
            if dur < interval {
                if let Some(d) = (dur - interval).checked_sub(one_ms) {
                    std::thread::sleep(d);
                }
            }
            if dur > 2 * interval {
                last_tick_time = std::time::SystemTime::now();
            } else {
                last_tick_time = now;
            }

            std::thread::yield_now();
        }
    }
}

pub struct PthData {
    graphics_state: Mutex<GraphicsState>,
    states: Mutex<Vec<Box<dyn GameState>>>,
}

pub struct LogicData {
    futures: futures::executor::LocalPool,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(if cfg!(feature = "debug-game") { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .init();
    log::info!("Starting up...");
    // let app_root = application_root_dir().expect("get app root dir failed.");
    // let res_root = if app_root.join("res").exists() { app_root.join("res") } else { app_root };
    // let display_config_path = res_root.join("config").join("display.ron");
    // let assets_dir = res_root.join("assets");
    // let game_data = GameDataBuilder::default()
    //     .with_bundle(RenderingBundle::<DefaultBackend>::new()
    //                      .with_plugin(render::blit::BlitToWindow::new(amethyst::renderer::bundle::Target::Main, render::WINDOW, true))
    //                      .with_plugin(
    //                          RenderToWindow::from_config_path(display_config_path)?
    //                              .with_clear([0.0, 0.0, 0.0, 1.0])
    //                              .with_target(render::WINDOW)
    //                      )
    //                      .with_plugin(RenderFlat2D::default())
    //                      .with_plugin(RenderFlat3D::default())
    //                      .with_plugin(RenderUi::default())
    //                      .with_plugin(render::RenderInvertColorCircle::default())
    //                  // .with_plugin(render::water_wave::RenderWaterWave::default().with_target(render::PTH_MAIN))
    //     )?
    //     .with_bundle(TransformBundle::new())?
    //     .with_bundle(InputBundle::<StringBindings>::new())?
    //     .with_bundle(UiBundle::<StringBindings>::new())?
    //     .with(systems::AnimationSystem, "main_anime_system", &[])
    //     .with(systems::DebugSystem::default(), "debug_system", &[]);
    // let mut game = Application::build(assets_dir, states::Loading::default())?
    //     .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
    //     .build(game_data)?;
    // game.run();

    let event_loop = winit::event_loop::EventLoop::new();

    log::info!("going to build window");
    let window = winit::window::WindowBuilder::new()
        .with_title("PoolTouhou")
        .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .with_min_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .with_max_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();


    log::info!("building graphics state.");
    let state = pollster::block_on(GraphicsState::new(&window));

    let pth = Arc::new(PthData {
        graphics_state: Mutex::new(state),
        states: Mutex::new(vec![Box::new(crate::states::init::Loading::default())]),
    });

    {
        let c_pth = pth.clone();
        std::thread::spawn(move || { c_pth.render_thread() });
    }
    log::info!("going to run event loop");
    std::thread::spawn(move || { pth.logic_thread() });
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
            _ => {
                *control_flow = ControlFlow::Wait
            }
        }
    });
}
