use std::time::Duration;

pub use futures;
use futures::executor::{LocalPool, LocalSpawner, ThreadPool};
pub use wgpu;
pub use winit;
use winit::event_loop::ControlFlow;

pub use pool_script;
pub use render::*;

pub mod handles;
pub mod input;
pub mod config;
pub mod render;
pub mod audio;
pub mod states;


pub const PLAYER_Z: f32 = 0.0;

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
            io_pool: ThreadPool::builder().pool_size(3).name_prefix("pth io")
                .before_stop(|idx| {
                    log::info!("IO Thread #{} stop", idx);
                })
                .create().expect("Create pth io thread pool failed"),
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
                ControlFlow::Wait => {
                    self.control_flow = rhs.control_flow
                }
                ControlFlow::WaitUntil(t1) => {
                    match rhs.control_flow {
                        ControlFlow::Wait => {}
                        ControlFlow::WaitUntil(t2) => {
                            self.control_flow = ControlFlow::WaitUntil(t1.min(t2));
                        }
                        _ => {
                            self.control_flow = rhs.control_flow;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
