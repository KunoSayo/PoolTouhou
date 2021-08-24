use std::cell::Cell;
use std::sync::atomic::{AtomicU16, Ordering};

use wgpu_glyph::GlyphCruncher;

use crate::{GlobalState, MainRendererData};

pub struct DebugSystem {
    count: AtomicU16,
    delta: Cell<f32>,
    fps: Cell<f32>,
}

pub static DEBUG: DebugSystem = DebugSystem {
    count: AtomicU16::new(0),
    delta: Cell::new(0.0),
    fps: Cell::new(60.0),
};

//What's the difference between use static mut and this?
unsafe impl Sync for DebugSystem {}

impl DebugSystem {
    pub(crate) fn render(&self, state: &mut GlobalState, render: &mut MainRendererData, dt: f32) {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Debug Encoder") });

        let delta = self.delta.get() + dt;

        self.count.fetch_add(1, Ordering::Relaxed);
        if delta >= 1.0 {
            self.fps.set(self.count.load(Ordering::Relaxed) as f32 / delta);
            self.delta.set(0.0);
            self.count.store(0, Ordering::Relaxed);
        } else {
            self.delta.set(delta);
        }


        {
            let text = format!("fps:{:.2} ", self.fps.get());
            let mut section = wgpu_glyph::Section {
                screen_position: (0.0, 0.0),
                bounds: (
                    state.surface_cfg.width as f32,
                    state.surface_cfg.height as f32,
                ),
                text: vec![
                    wgpu_glyph::Text::new(&text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(20.0),
                ],
                ..wgpu_glyph::Section::default()
            };

            if let Some(rect) = render.glyph_brush.glyph_bounds(section.clone()) {
                section.screen_position.0 = state.surface_cfg.width as f32 - rect.width();
                section.screen_position.1 = state.surface_cfg.height as f32 - rect.height();
            }
            render.glyph_brush.queue(section);
            render.glyph_brush
                .draw_queued(
                    &state.device,
                    &mut render.staging_belt,
                    &mut encoder,
                    &render.views.get_screen().view,
                    state.surface_cfg.width,
                    state.surface_cfg.height,
                )
                .expect("Draw queued!");
        }
        render.staging_belt.finish();
        state.queue.submit(Some(encoder.finish()));
    }
}
