use std::iter::FromIterator;
use std::num::NonZeroUsize;
use std::ops::Range;

use wgpu;
use wgpu_glyph::{GlyphCruncher, HorizontalAlign, Layout, VerticalAlign};
use wgpu_glyph;
use wgpu_glyph::ab_glyph::{Point, Rect};

use crate::render::{GlobalState, MainRendererData};

#[derive(Debug)]
pub enum InputResult {
    Ignored,
    Normal,
    Copy,
    Paste,
    Esc,
    Back,
    Del,
    Enter,
}

pub struct TextInput {
    pub chars: Vec<char>,
    pub limit: Option<NonZeroUsize>,
    pub cursor: usize,
    pub select: Option<Range<usize>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            chars: vec![],
            limit: None,
            cursor: 0,
            select: None,
        }
    }
}

impl TextInput {
    pub fn input(&mut self, c: char) -> InputResult {
        match c {
            '\t' | '\r' => {
                InputResult::Ignored
            }
            '\n' => {
                InputResult::Enter
            }
            '\x1a' => {
                InputResult::Esc
            }
            '\x03' => {
                // maybe copy the text
                InputResult::Copy
            }
            '\x16' => {
                // maybe paste the text
                InputResult::Paste
            }
            '\x08' => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.chars.remove(self.cursor);
                }
                InputResult::Back
            }
            '\x7f' => {
                if self.chars.len() > self.cursor {
                    self.chars.remove(self.cursor);
                }
                InputResult::Del
            }
            c if c.is_control() => {
                InputResult::Ignored
            }
            _ => {
                log::debug!("got char {} [as {}]", c, c as u128);
                if self.limit.filter(|x| x.get() < self.chars.len()).is_none() {
                    self.chars.insert(self.cursor, c);
                    self.cursor += 1;
                    InputResult::Normal
                } else {
                    InputResult::Ignored
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.chars.len() {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn render_to_screen(&self, state: &mut GlobalState, render: &mut MainRendererData, scale: f32, pos: (f32, f32), bounds: (f32, f32)) {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Text Render Encoder") });

        {
            let text = String::from_iter(&self.chars[..self.cursor]);
            let text_after = String::from_iter(&self.chars[self.cursor..]);
            let section = wgpu_glyph::Section {
                screen_position: pos,
                bounds,
                text: vec![
                    wgpu_glyph::Text::new(&text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(scale),
                ],
                layout: Layout::default_single_line().v_align(VerticalAlign::Bottom).h_align(HorizontalAlign::Left),
            };
            let bound = render.glyph_brush.glyph_bounds(&section).unwrap_or(Rect {
                min: Point::from(pos),
                max: Point::from(pos),
            });
            let split = wgpu_glyph::Section {
                screen_position: (bound.max.x, pos.1),
                bounds,
                text: vec![
                    wgpu_glyph::Text::new("_")
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(scale),
                ],
                layout: Layout::default_single_line().v_align(VerticalAlign::Bottom).h_align(HorizontalAlign::Left),
            };
            let section_after = wgpu_glyph::Section {
                screen_position: (bound.max.x, pos.1),
                bounds,
                text: vec![
                    wgpu_glyph::Text::new(&text_after)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(scale),
                ],
                layout: Layout::default_single_line().v_align(VerticalAlign::Bottom).h_align(HorizontalAlign::Left),
            };
            render.glyph_brush.queue(section);
            render.glyph_brush.queue(split);
            render.glyph_brush.queue(section_after);
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