use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::Arc;
use std::time::{Duration, Instant};

use env_logger::Target;
use futures::executor::{LocalPool, LocalSpawner, ThreadPool};
use futures::task::LocalSpawnExt;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use shaderc::ShaderKind;
use wgpu::{BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, LoadOp, Maintain, MapMode, Operations, Origin3d, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor, ShaderStages, TextureAspect, TextureFormat};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;

// use crate as root;
use audio::OpenalData;
use handles::ResourcesHandles;
use pthapi::config::Config;
use render::{GlobalState, MainRendererData, MainRenderViews};
use states::{GameState, StateData, Trans};

mod render;
mod systems;
mod ui;
mod states;
mod input;
mod handles;
mod audio;
mod script;

pub struct Pools {
    pub io_pool: ThreadPool,
    pub render_pool: LocalPool,
    pub render_spawner: LocalSpawner,
}

impl Default for Pools {
    fn default() -> Self {
        let render_pool = LocalPool::new();
        let render_spawner = render_pool.spawner();
        Self {
            io_pool: ThreadPool::builder()
                .pool_size(3)
                .name_prefix("pth io")
                .before_stop(|idx| {
                    // log::info!("IO Thread #{} stop", idx);
                })
                .create()
                .expect("Create pth io thread pool failed"),
            render_pool,
            render_spawner,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct LoopState {
    pub control_flow: ControlFlow,
    pub render: bool,
}

impl LoopState {
    pub const WAIT_ALL: LoopState = LoopState {
        control_flow: ControlFlow::Wait,
        render: false,
    };

    pub const WAIT: LoopState = LoopState {
        control_flow: ControlFlow::Wait,
        render: true,
    };

    pub const POLL: LoopState = LoopState {
        control_flow: ControlFlow::Poll,
        render: true,
    };

    pub const POLL_WITHOUT_RENDER: LoopState = LoopState {
        control_flow: ControlFlow::Poll,
        render: false,
    };

    pub fn wait_until(dur: Duration, render: bool) -> Self {
        Self {
            control_flow: ControlFlow::WaitUntil(std::time::Instant::now() + dur),
            render,
        }
    }
}

impl std::ops::BitOrAssign for LoopState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.render |= rhs.render;
        if self.control_flow != rhs.control_flow {
            match self.control_flow {
                ControlFlow::Wait => self.control_flow = rhs.control_flow,
                ControlFlow::WaitUntil(t1) => match rhs.control_flow {
                    ControlFlow::Wait => {}
                    ControlFlow::WaitUntil(t2) => {
                        self.control_flow = ControlFlow::WaitUntil(t1.min(t2));
                    }
                    _ => {
                        self.control_flow = rhs.control_flow;
                    }
                },
                _ => {}
            }
        }
    }
}


// https://doc.rust-lang.org/book/
pub struct PthData {
    global_state: GlobalState,
    render: MainRendererData,
    pools: Pools,
    states: Vec<Box<dyn GameState>>,
    inputs: input::BakedInputs,
    running_game_thread: bool,
    last_render_time: Instant,
    last_tick_time: Instant,
    tick_interval: Duration,
    config: Config,
}

impl PthData {
    fn start_init(&mut self) {
        log::info!("Init render thread.");
        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                global_state: &mut self.global_state,
                render: &mut self.render,
            };

            self.states.last_mut().unwrap().start(&mut state_data);
        }
    }

    fn process_tran(&mut self, tran: Trans) {
        let last = self.states.last_mut().unwrap();
        let mut state_data = StateData {
            pools: &mut self.pools,
            inputs: &self.inputs,
            global_state: &mut self.global_state,
            render: &mut self.render,
        };
        match tran {
            Trans::Push(mut x) => {
                x.start(&mut state_data);
                self.states.push(x);
            }
            Trans::Pop => {
                last.stop(&mut state_data);
                self.states.pop().unwrap();
            }
            Trans::Switch(x) => {
                last.stop(&mut state_data);
                *last = x;
            }
            Trans::Exit => {
                while let Some(mut last) = self.states.pop() {
                    last.stop(&mut state_data);
                }
                self.running_game_thread = false;
            }
            Trans::Vec(ts) => {
                for t in ts {
                    self.process_tran(t);
                }
            }
            Trans::None => {}
        }
    }

    fn loop_once(&mut self) -> LoopState {
        self.inputs.swap_frame();
        let mut loop_result = LoopState::WAIT_ALL;
        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                global_state: &mut self.global_state,
                render: &mut self.render,
            };
            for x in &mut self.states {
                x.shadow_tick(&state_data);
                loop_result |= x.shadow_update();
            }
            if let Some(last) = self.states.last_mut() {
                let (tran, l) = last.update(&mut state_data);
                self.process_tran(tran);
                loop_result |= l;
            }
            let tick_now = std::time::Instant::now();
            let tick_dur = tick_now.duration_since(self.last_tick_time);
            if tick_dur > self.tick_interval {
                self.inputs.tick();

                let mut state_data = StateData {
                    pools: &mut self.pools,
                    inputs: &self.inputs,
                    global_state: &mut self.global_state,
                    render: &mut self.render,
                };

                if let Some(last) = self.states.last_mut() {
                    let tran = last.game_tick(&mut state_data);
                    self.process_tran(tran);
                } else {
                    println!("There is no states to run. Why run states.game thread?");
                    self.running_game_thread = false;
                }

                if tick_dur > 2 * self.tick_interval {
                    self.last_tick_time = std::time::Instant::now();
                } else {
                    self.last_tick_time = tick_now;
                }
            }
        }
        if !loop_result.render && self.inputs.is_pressed(&[VirtualKeyCode::F11]) {
            self.save_screen_shots();
        }

        if self.inputs.is_pressed(&[VirtualKeyCode::F3]) {
            log::info!("{:?}", self.global_state);
        }

        loop_result
    }

    fn render_once(&mut self) {
        let render_now = std::time::Instant::now();
        let render_dur = render_now.duration_since(self.last_render_time);
        let dt = render_dur.as_secs_f32();

        let swap_chain_frame
            = self.global_state.surface.get_current_frame().expect("Failed to acquire next swap chain texture");
        let surface_output = &swap_chain_frame.output;
        {
            let mut encoder = self.global_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Encoder") });
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.render.views.get_screen().view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.global_state.queue.submit(Some(encoder.finish()));
        }
        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                global_state: &mut self.global_state,
                render: &mut self.render,
            };

            for game_state in &mut self.states {
                game_state.shadow_render(&state_data);
            }
            if let Some(g) = self.states.last_mut() {
                let tran = g.render(&mut state_data);
                self.process_tran(tran);
            }
        }

        systems::debug_system::DEBUG.render(&mut self.global_state, &mut self.render, dt);

        {
            let mut encoder = self.global_state.device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy buffer to screen commands")
            });
            let size = self.global_state.get_screen_size();
            encoder.copy_texture_to_texture(ImageCopyTexture {
                texture: &self.render.views.get_screen().texture,
                mip_level: 0,
                origin: Origin3d::default(),
                aspect: TextureAspect::All,
            }, ImageCopyTexture {
                texture: &surface_output.texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: TextureAspect::All,
            }, Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            });
            self.global_state.queue.submit(Some(encoder.finish()));
        }

        if self.inputs.is_pressed(&[VirtualKeyCode::F11]) {
            self.save_screen_shots();
        }

        self.pools.render_pool.try_run_one();
        self.last_render_time = render_now;
    }

    fn save_screen_shots(&mut self) {
        let state = &self.global_state;
        let mut encoder = state.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Save image commands")
        });
        let size = state.get_screen_size();
        let buffer = state.device.create_buffer(&BufferDescriptor {
            label: Some("Save screen buffer"),
            size: ((size.0 * size.1) << 2) as _,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        use std::convert::TryInto;
        encoder.copy_texture_to_buffer(ImageCopyTexture {
            texture: &self.render.views.get_screen().texture,
            mip_level: 0,
            origin: Origin3d::default(),
            aspect: TextureAspect::All,
        }, ImageCopyBuffer {
            buffer: &buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((size.0 * 4).try_into().unwrap()),
                rows_per_image: Some((size.1).try_into().unwrap()),
            },
        }, Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        });
        state.queue.submit(Some(encoder.finish()));
        let buf_slice = buffer.slice(..);
        self.pools.render_spawner.spawn_local_with_handle(buf_slice.map_async(MapMode::Read)).expect("Spawn task failed")
            .forget();
        self.pools.render_pool.try_run_one();
        state.device.poll(Maintain::Wait);
        let mapped_buf = buf_slice.get_mapped_range();
        let image = DynamicImage::ImageBgra8(ImageBuffer::from_raw(size.0, size.1, Vec::from(mapped_buf.as_ref()))
            .expect("Get image from screen failed"));
        log::info!("Saving image");
        self.pools.io_pool.spawn_ok(async move {
            let now = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
            std::fs::create_dir_all("./screenshots").expect("Create screenshots dir failed");
            image.to_rgba8().save_with_format(format!("./screenshots/{}.png", now.format("%y-%m-%d-%H-%M-%S")),
                                              ImageFormat::Png).expect("Save image file failed");
        });
    }

    fn new(graphics_state: GlobalState, config: Config, game_state: impl GameState) -> Self {
        let render = MainRendererData::new(&graphics_state);
        Self {
            global_state: graphics_state,
            render,
            pools: Default::default(),
            states: vec![Box::new(game_state)],
            inputs: Default::default(),
            running_game_thread: true,
            last_render_time: Instant::now(),
            last_tick_time: Instant::now(),
            tick_interval: Duration::from_secs_f64(1.0 / 60.0),
            config,
        }
    }
}

struct LogTarget<Console: std::io::Write> {
    log_file: Option<std::fs::File>,
    c: Console,
}

impl<Console: std::io::Write> LogTarget<Console> {
    fn new(c: Console) -> Self {
        let log_file = std::fs::OpenOptions::new().read(true).write(true).truncate(true).create(true).open("latest.log");
        if let Err(ref e) = log_file {
            eprintln!("Open log file failed for {}", e);
        }
        Self {
            log_file: log_file.ok(),
            c,
        }
    }
}

impl<Console: std::io::Write> std::io::Write for LogTarget<Console> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(file) = self.log_file.as_mut() {
            if let Err(e) = file.write_all(&buf) {
                eprintln!("Log into file failed for {}", e);
            }
        }
        self.c.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(file) = self.log_file.as_mut() {
            if let Err(e) = file.flush() {
                eprintln!("Flush log into file failed for {}", e);
            }
        }
        self.c.flush()
    }
}

async fn new_global(window: &Window) -> GlobalState {
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
    res.load_with_compile_shader("n2dt.v", "normal2dtexture.vert", "main", ShaderKind::Vertex).unwrap();
    res.load_with_compile_shader("n2dt.f", "normal2dtexture.frag", "main", ShaderKind::Fragment).unwrap();

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

    GlobalState {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::read_from_path("opt.cfg")?;
    env_logger::Builder::default()
        .filter_module("wgpu_core::device", log::LevelFilter::Warn)
        .filter_module("pool_script", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .target(Target::Pipe(Box::new(LogTarget::new(std::io::stderr()))))
        .parse_filters(config.or_default("log_filters", ""))
        .parse_default_env()
        .init();
    log::info!("Starting up...");
    for arg in std::env::args_os() {
        log::info!("arg {:?}", arg);
    }
    if let Err(e) = config.save() {
        log::warn!("Save config file failed for {:?}", e);
    }
    let event_loop = winit::event_loop::EventLoop::new();

    let width: u32 = config.parse_or_default("width", "1600");
    let height: u32 = config.parse_or_default("height", "900");
    log::info!("going to build window");
    let window = winit::window::WindowBuilder::new()
        .with_title("PoolTouhou")
        .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    log::info!("building graphics state.");


    let state = pollster::block_on(new_global(&window));
    let mut pth = PthData::new(state, config, crate::states::init::Loading::default());
    pth.start_init();

    log::info!("going to run event loop");
    let mut pressed_keys = HashSet::new();
    let mut released_keys = HashSet::new();
    let mut focused = true;
    let mut game_draw_requested = false;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let (width, height) = (size.width, size.height);
                log::info!("Changed windows size to {}, {}", width, height);
                if width != 0 && height != 0 {
                    pth.global_state.resize(width, height);
                    pth.render.views = MainRenderViews::new(&pth.global_state);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input,
                    is_synthetic,
                    ..
                }, ..
            } => {
                if !is_synthetic {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => {
                                pressed_keys.insert(key);
                            }
                            ElementState::Released => {
                                released_keys.insert(key);
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(f),
                ..
            } => { focused = f }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(_c),
                ..
            } => {}
            Event::RedrawRequested(_) => {
                if !game_draw_requested {
                    log::trace!("System Redraw Requested");
                }
                pth.render_once();
                game_draw_requested = false;
            }
            Event::MainEventsCleared => {
                if !pressed_keys.is_empty() || !released_keys.is_empty() {
                    log::trace!("process pressed_key {:?} and released {:?}", pressed_keys, released_keys);
                    pth.inputs.process(&pressed_keys, &released_keys);
                    pressed_keys.clear();
                    released_keys.clear();
                }
                if pth.running_game_thread {
                    let LoopState {
                        control_flow: c_f,
                        render
                    } = pth.loop_once();
                    if render {
                        game_draw_requested = true;
                        window.request_redraw();
                    }
                    *control_flow = c_f;
                } else {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
        log::trace!("got event {:?}", event);
    });
}
