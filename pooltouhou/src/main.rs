use std::collections::HashSet;
use std::fs::OpenOptions;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use env_logger::Target;
use futures::executor::{LocalPool, LocalSpawner, ThreadPool};
use wgpu::{Color, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::render::{GraphicsState, MainRendererData, RenderViews};
use crate::render::texture2d::Texture2DRender;
use crate::states::{GameState, StateData, Trans};

mod handles;
mod states;
mod systems;
mod render;
mod input;

// https://doc.rust-lang.org/book/

pub const PLAYER_Z: f32 = 0.0;


enum WindowEventSync {
    ///(pressed keys, released keys)
    KeysChange(Box<HashSet<VirtualKeyCode>>, Box<HashSet<VirtualKeyCode>>),
    ChangeSize(u32, u32),
}

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

pub struct PthData {
    graphics_state: GraphicsState,
    render: MainRendererData,
    pools: Pools,
    states: Vec<Box<dyn GameState>>,
    receiver: Receiver<WindowEventSync>,
    inputs: input::BakedInputs,
    running_game_thread: bool,
}

impl PthData {
    fn game_thread_run(&mut self) {
        log::info!("created render thread.");
        let mut last_render_time = std::time::Instant::now();
        let mut last_tick_time = std::time::Instant::now();
        let tick_interval = Duration::from_secs_f64(1.0 / 60.0);

        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                graphics_state: &mut self.graphics_state,
                render: &mut self.render,
                screens: None,
            };

            self.states.last_mut().unwrap().start(&mut state_data);
        }

        while self.running_game_thread {
            while let Ok(event) = self.receiver.try_recv() {
                match event {
                    WindowEventSync::ChangeSize(width, height) => {
                        let swapchain_desc = wgpu::SwapChainDescriptor {
                            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                            format: self.graphics_state.swapchain_desc.format,
                            width,
                            height,
                            present_mode: wgpu::PresentMode::Fifo,
                        };
                        self.graphics_state.swap_chain = self.graphics_state.device.create_swap_chain(&self.graphics_state.surface, &swapchain_desc);
                    }
                    WindowEventSync::KeysChange(pressed, released) => {
                        self.inputs.process(pressed, released);
                    }
                }
            }
            self.inputs.swap_frame();

            {
                let tick_now = std::time::Instant::now();

                let tick_dur = tick_now.duration_since(last_tick_time);
                if tick_dur > tick_interval {
                    self.inputs.tick();

                    let mut state_data = StateData {
                        pools: &mut self.pools,
                        inputs: &self.inputs,
                        graphics_state: &mut self.graphics_state,
                        render: &mut self.render,
                        screens: None,
                    };
                    for x in &mut self.states {
                        x.shadow_update(&state_data);
                    }


                    if let Some(last) = self.states.last_mut() {
                        match last.game_tick(&mut state_data) {
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
                                *self.states.last_mut().unwrap() = x;
                            }
                            Trans::Exit => {
                                while let Some(mut last) = self.states.pop() {
                                    last.stop(&mut state_data);
                                }
                                self.running_game_thread = false;
                                break;
                            }
                            Trans::None => {}
                        }
                    } else {
                        println!("There is no states to run. Why run game thread?");
                        self.running_game_thread = false;
                    }

                    if tick_dur > 2 * tick_interval {
                        last_tick_time = std::time::Instant::now();
                    } else {
                        last_tick_time = tick_now;
                    }
                }
            }

            {
                let render_now = std::time::Instant::now();
                let render_dur = render_now.duration_since(last_render_time);
                self.render_once(render_dur.as_secs_f32());
                self.pools.render_pool.try_run_one();
                last_render_time = render_now;
            }

            std::thread::yield_now();
        }
    }

    fn render_once(&mut self, dt: f32) {
        let state = &mut self.graphics_state;
        let frame = state.swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture")
            .output;

        {
            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Encoder") });
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &frame.view,
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
            state.queue.submit(Some(encoder.finish()));
        }
        let views = RenderViews {
            screen: &frame.view
        };
        {
            let mut state_data = StateData {
                pools: &mut self.pools,
                inputs: &self.inputs,
                graphics_state: state,
                render: &mut self.render,
                screens: Some(&views),
            };

            for game_state in &mut self.states {
                game_state.shadow_render(&state_data);
            }
            if let Some(g) = self.states.last_mut() {
                match g.render(&mut state_data) {
                    Trans::Push(mut x) => {
                        x.start(&mut state_data);
                        self.states.push(x);
                    }
                    Trans::Pop => {
                        g.stop(&mut state_data);
                        self.states.pop().unwrap();
                    }
                    Trans::Switch(x) => {
                        g.stop(&mut state_data);
                        *self.states.last_mut().unwrap() = x;
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
        }


        systems::debug_system::DEBUG.render(state, &mut self.render, dt, &frame.view);
        state.queue.submit(None);
    }

    fn new(graphics_state: GraphicsState, game_state: impl GameState, receiver: Receiver<WindowEventSync>) -> Self {
        let render = MainRendererData::new(&graphics_state);
        Self {
            graphics_state,
            render,
            pools: Default::default(),
            states: vec![Box::new(game_state)],
            receiver,
            inputs: Default::default(),
            running_game_thread: true,
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

    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let state = pollster::block_on(GraphicsState::new(&window));
        let mut pth = PthData::new(state, crate::states::init::Loading::default(), receiver);
        pth.game_thread_run();
    });
    log::info!("going to run event loop");
    let mut pressed_keys = Box::new(HashSet::new());
    let mut released_keys = Box::new(HashSet::new());
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
                match sender.send(WindowEventSync::ChangeSize(size.width, size.height)) {
                    Ok(_) => {}
                    Err(e) => {
                        log::warn!("send window event failed: {}", e);
                    }
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
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                //todo: process text input
            }
            Event::MainEventsCleared => {
                if !pressed_keys.is_empty() || !released_keys.is_empty() {
                    match sender.send(WindowEventSync::KeysChange(std::mem::take(&mut pressed_keys), std::mem::take(&mut released_keys))) {
                        Ok(_) => {}
                        Err(e) => {
                            log::warn!("send window event failed: {}", e);
                        }
                    }
                }
            }
            _ => {
                *control_flow = ControlFlow::Wait
            }
        }
    });
}
