use std::collections::HashSet;
use std::time::{Duration, Instant};

use env_logger::Target;
use futures::executor::{LocalPool, LocalSpawner, ThreadPool};
use futures::task::LocalSpawnExt;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use wgpu::{BufferDescriptor, BufferUsage, Color, CommandEncoderDescriptor, Extent3d,
           ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, LoadOp,
           Maintain, MapMode, Operations, Origin3d, RenderPassColorAttachment,
           RenderPassDescriptor};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::LoopState::WaitAllTimed;
use crate::render::{GlobalState, MainRendererData};
use crate::states::{GameState, StateData, Trans};

mod handles;
mod states;
mod systems;
mod render;
mod input;
mod script;
mod audio;
mod config;

// https://doc.rust-lang.org/book/

pub const PLAYER_Z: f32 = 0.0;

pub struct Pools {
    io_pool: ThreadPool,
    render_pool: LocalPool,
    render_spawner: LocalSpawner,
}

impl Default for Pools {
    fn default() -> Self {
        let render_pool = LocalPool::new();
        let render_spawner = render_pool.spawner();
        Self {
            io_pool: ThreadPool::builder().pool_size(3).name_prefix("pth io").create().expect("Create pth io thread pool failed"),
            render_pool,
            render_spawner,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum LoopState {
    /// Wait event and do not draw
    WaitAll,
    /// Wait event and may be timed out
    WaitAllTimed(Duration),
    /// Wait for next event but draw once
    Wait,
    /// Wait for next event or timed out but draw once
    WaitTimed(Duration),
    /// Poll next event but do not draw
    PollNoRender,
    /// Pool next event with render
    Poll,
}

impl LoopState {
    #[inline]
    fn turn_on_render(&mut self) {
        match self {
            LoopState::WaitAll => { *self = Self::Wait }
            LoopState::WaitAllTimed(d) => { *self = Self::WaitTimed(*d) }
            LoopState::PollNoRender => { *self = Self::Poll }
            _ => {}
        }
    }
    #[inline]
    fn should_render(&self) -> bool {
        match self {
            Self::Wait | Self::WaitTimed(_) | Self::Poll => true,
            _ => false
        }
    }

    #[inline]
    fn take_dur(self) -> Option<Duration> {
        match self {
            WaitAllTimed(d) => Some(d),
            LoopState::WaitTimed(d) => Some(d),
            _ => None
        }
    }

    fn min_dur(self, dur: Duration) -> Self {
        match self {
            WaitAllTimed(d) => {
                Self::WaitAllTimed(d.min(dur))
            }
            LoopState::WaitTimed(d) => {
                Self::WaitTimed(d.min(dur))
            }
            _ => self
        }
    }

    #[inline]
    fn get_priority(&self) -> u8 {
        match self {
            LoopState::WaitAll => 0,
            WaitAllTimed(_) => 1,
            LoopState::Wait => 2,
            LoopState::WaitTimed(_) => 3,
            LoopState::PollNoRender => 4,
            LoopState::Poll => 5
        }
    }

    fn into_control_flow(self) -> ControlFlow {
        match self {
            LoopState::Wait | LoopState::WaitAll => {
                ControlFlow::Wait
            }
            WaitAllTimed(d) | LoopState::WaitTimed(d) => {
                ControlFlow::WaitUntil(std::time::Instant::now() + d)
            }
            LoopState::PollNoRender | Self::Poll => {
                ControlFlow::Poll
            }
        }
    }
}

impl std::ops::BitOrAssign for LoopState {
    fn bitor_assign(&mut self, mut rhs: Self) {
        if self.should_render() {
            rhs.turn_on_render();
        } else if rhs.should_render() {
            self.turn_on_render();
        }
        if self != &rhs {
            let idx = self.get_priority();
            let o_idx = rhs.get_priority();
            if idx == o_idx {
                *self = self.min_dur(rhs.take_dur().unwrap())
            } else if idx < o_idx {
                if let Some(d) = self.take_dur() {
                    *self = rhs.min_dur(d);
                } else {
                    *self = rhs;
                }
            } else {
                if let Some(d) = rhs.take_dur() {
                    self.min_dur(d);
                }
            }
        }
    }
}

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
        let mut last = self.states.last_mut().unwrap();
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
            Trans::None => {}
        }
    }

    fn loop_once(&mut self) -> LoopState {
        self.inputs.swap_frame();
        let mut dirty = LoopState::WaitAll;
        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                global_state: &mut self.global_state,
                render: &mut self.render,
            };
            for x in &mut self.states {
                x.shadow_tick(&state_data);
                dirty |= x.shadow_update();
            }
            if let Some(last) = self.states.last_mut() {
                let (tran, l) = last.update(&mut state_data);
                self.process_tran(tran);
                dirty |= l;
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
                    println!("There is no states to run. Why run game thread?");
                    self.running_game_thread = false;
                }

                if tick_dur > 2 * self.tick_interval {
                    self.last_tick_time = std::time::Instant::now();
                } else {
                    self.last_tick_time = tick_now;
                }
            }
        }

        dirty
    }

    fn render_once(&mut self) {
        let render_now = std::time::Instant::now();
        let render_dur = render_now.duration_since(self.last_render_time);
        let dt = render_dur.as_secs_f32();

        let swap_chain_frame
            = self.global_state.swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture");
        let output_tex = &swap_chain_frame.output;

        {
            let mut encoder = self.global_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Encoder") });
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.render.views.screen.view,
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

        self.render.render2d.blit(&self.global_state, &self.render.views.screen.view, &output_tex.view);

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
            usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
            mapped_at_creation: false,
        });
        use std::convert::TryInto;
        encoder.copy_texture_to_buffer(ImageCopyTexture {
            texture: &self.render.views.screen.texture,
            mip_level: 0,
            origin: Origin3d::default(),
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

    fn new(graphics_state: GlobalState, game_state: impl GameState) -> Self {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::default()
        .filter_module("wgpu_core::device", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::Info)
        .target(Target::Pipe(Box::new(LogTarget::new(std::io::stderr()))))
        .parse_default_env()
        .init();
    log::info!("Starting up...");

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

    // std::thread::spawn(move || {
    let state = pollster::block_on(GlobalState::new(&window));
    let mut pth = PthData::new(state, crate::states::init::Loading::default());
    pth.start_init();
    // });
    log::info!("going to run event loop");
    let mut pressed_keys = HashSet::new();
    let mut released_keys = HashSet::new();
    let mut focused = true;
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
                    let swapchain_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                        format: pth.global_state.swapchain_desc.format,
                        width,
                        height,
                        present_mode: wgpu::PresentMode::Fifo,
                    };
                    pth.global_state.swap_chain = pth.global_state.device.create_swap_chain(&pth.global_state.surface, &swapchain_desc);
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
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                //todo: process text input
            }
            Event::RedrawRequested(_) => {
                if pth.running_game_thread {
                    let loop_state = pth.loop_once();
                    if loop_state.should_render() {
                        pth.render_once();
                    }
                    *control_flow = loop_state.into_control_flow();
                } else {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                if !pressed_keys.is_empty() || !released_keys.is_empty() {
                    log::trace!("process pressed_key {:?} and released {:?}", pressed_keys, released_keys);
                    pth.inputs.process(&pressed_keys, &released_keys);
                    pressed_keys.clear();
                    released_keys.clear();
                }
                window.request_redraw();
            }
            _ => {}
        }
    });
}
