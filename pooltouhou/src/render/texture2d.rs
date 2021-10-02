use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use bytemuck::Pod;
use bytemuck::Zeroable;
use rayon::prelude::*;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry,
           BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
           BindingResource, BindingType, Buffer, BufferDescriptor, BufferUsages,
           IndexFormat, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
           RenderPipeline, ShaderStages, TextureSampleType, TextureView, TextureViewDimension,
           VertexAttribute, VertexBufferLayout, VertexFormat};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use pthapi::{PosType, TexHandle};

use crate::GlobalState;
use crate::handles::ResourcesHandles;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Pod, Zeroable)]
#[repr(C, align(4))]
pub struct Texture2DVertexData {
    pub pos: [f32; 2],
    pub coord: [f32; 2],
}

const VERTEX_DATA_SIZE: usize = std::mem::size_of::<Texture2DVertexData>();

#[derive(Clone, Debug)]
pub struct Texture2DObject {
    pub vertex: [Texture2DVertexData; 4],
    pub z: f32,
    pub tex: TexHandle,
}

pub trait AsTexture2DObject {
    fn vertex(&self) -> &[Texture2DVertexData; 4];
    fn z(&self) -> f32;
    fn tex(&self) -> TexHandle;
}

impl AsTexture2DObject for Texture2DObject {
    fn vertex(&self) -> &[Texture2DVertexData; 4] {
        &self.vertex
    }

    fn z(&self) -> f32 {
        self.z
    }

    fn tex(&self) -> TexHandle {
        self.tex
    }
}

impl Texture2DObject {
    #[inline]
    pub fn with_game_pos(mut center: PosType, width: f32, height: f32, tex: TexHandle) -> Self {
        center.0 += 800.0;
        center.1 += 450.0;
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        Self {
            vertex: (0..4).map(|x|
                Texture2DVertexData {
                    pos: match x {
                        0 => [center.0 - half_width, center.1 + half_height],
                        1 => [center.0 + half_width, center.1 + half_height],
                        2 => [center.0 - half_width, center.1 - half_height],
                        3 => [center.0 + half_width, center.1 - half_height],
                        _ => unreachable!()
                    },
                    coord: match x {
                        0 => [0.0, 0.0],
                        1 => [1.0, 0.0],
                        2 => [0.0, 1.0],
                        3 => [1.0, 1.0],
                        _ => unreachable!()
                    },
                }).collect::<Vec<_>>().try_into().unwrap(),
            z: center.2,
            tex,
        }
    }
}

impl PartialEq for Texture2DObject {
    fn eq(&self, other: &Self) -> bool {
        self.z == other.z && self.tex == other.tex
    }
}

impl Eq for Texture2DObject {}

impl PartialOrd for Texture2DObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.z.partial_cmp(&other.z).map(|x| match x {
            Ordering::Equal => { self.tex.cmp(&other.tex) }
            _ => x
        })
    }
}

impl Ord for Texture2DObject {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.z > other.z {
            Ordering::Greater
        } else if self.z < other.z {
            Ordering::Less
        } else {
            if self.tex > other.tex {
                Ordering::Greater
            } else if self.tex < other.tex {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    }
}

pub struct Texture2DRender {
    frag_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    bind_groups: HashMap<usize, BindGroup>,
    obj_count_in_buffer: usize,
}

impl Texture2DRender {
    pub fn new(state: &GlobalState, target_color_state: wgpu::ColorTargetState, handles: &Arc<ResourcesHandles>) -> Self {
        let obj_count_in_buffer = state.config.get_or_default("obj2d_count_once", 8192);
        let device = &state.device;
        let frag_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler {
                    filtering: true,
                    comparison: false,
                },
                count: None,
            }],
        });
        //done bind group

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<Texture2DVertexData>() * obj_count_in_buffer * 4) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&(0..obj_count_in_buffer).map(|obj_idx| {
                let offset = obj_idx as u16 * 4;
                [offset, offset + 1, offset + 2, offset + 1, offset + 2, offset + 3]
            }).collect::<Vec<_>>()),
            usage: BufferUsages::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&state.screen_uni_bind_layout, &frag_bind_group_layout],
            push_constant_ranges: &[],
        });


        let vert = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.v").unwrap())),
        });

        let frag = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.f").unwrap())),
        });

        let vertex_len = std::mem::size_of::<Texture2DVertexData>();
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: vertex_len as u64,
                    step_mode: Default::default(),
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }, VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 2 * 4,
                        shader_location: 1,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag,
                entry_point: "main",
                targets: &[target_color_state],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
        });

        Self {
            frag_bind_group_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_groups: Default::default(),
            obj_count_in_buffer
        }
    }

    pub fn add_tex(&mut self, state: &mut GlobalState, tex: usize) {
        if !self.bind_groups.contains_key(&tex) {
            let textures = state.handles.textures.read().unwrap();
            let tex_bind = state.device.create_bind_group(&BindGroupDescriptor {
                label: Some("texture bind"),
                layout: &self.frag_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&textures[tex as usize].view),
                }, BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&textures[tex as usize].sampler),
                }],
            });
            self.bind_groups.insert(tex, tex_bind);
        }
    }

    pub fn render<'a>(&'a self, state: &GlobalState, render_target: &TextureView, sorted_obj: &[Texture2DObject]) {
        let mut iter = sorted_obj.iter().enumerate();
        if let Some((_, fst)) = iter.next() {
            let mut cur_tex = fst.tex;
            let mut last_tex = fst.tex;
            let mut start_idx = 0;
            let mut last_idx = 0;

            let chunk_size = (self.obj_count_in_buffer >> 6) + 64;

            'rp_loop:
            loop {
                let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("2D Render Encoder") });
                let mut once_rp_offset = 0;
                let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("t2d rp"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: render_target,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                rp.set_pipeline(&self.render_pipeline);
                rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
                rp.set_bind_group(0, &state.screen_uni_bind, &[]);

                let mut draw = |tex, start_idx, end_idx, drew_obj| -> usize {
                    if let Some(bind_group) = self.bind_groups.get(&tex) {
                        let mut end = end_idx;

                        if end - start_idx + drew_obj > self.obj_count_in_buffer {
                            end = self.obj_count_in_buffer - drew_obj + start_idx;
                        }
                        if end <= start_idx {
                            return 0;
                        }
                        //16 Bytes per obj
                        sorted_obj[start_idx..end].par_chunks(chunk_size).enumerate().for_each(|(obj_idx, obj)| {
                            let mut data: Vec<u8> = Vec::with_capacity(VERTEX_DATA_SIZE << 8);
                            for x in obj {
                                for x in &x.vertex {
                                    data.extend_from_slice(bytemuck::cast_slice(&x.pos));
                                    data.extend_from_slice(bytemuck::cast_slice(&x.coord));
                                }
                            }
                            state.queue.write_buffer(&self.vertex_buffer, (((drew_obj + (obj_idx * chunk_size)) << 2) * VERTEX_DATA_SIZE) as _, &data);
                        });

                        state.queue.submit(None);
                        rp.set_bind_group(1, &bind_group, &[]);
                        rp.draw_indexed(0..((end - start_idx) * 6) as u32, drew_obj as i32 * 4, 0..1);
                        end - start_idx
                    } else {
                        log::warn!("Tried to render not added tex handle by: {}", tex);
                        0
                    }
                };
                if last_idx != start_idx {
                    //here to render
                    let rendered = draw(last_tex, start_idx, last_idx, once_rp_offset);
                    once_rp_offset += rendered;
                    if rendered < last_idx - start_idx {
                        start_idx += rendered;
                        std::mem::drop(rp);
                        state.queue.submit(Some(encoder.finish()));
                        continue 'rp_loop;
                    }
                    start_idx = last_idx;
                    last_tex = cur_tex;
                }
                while let Some((idx, cur)) = iter.next() {
                    last_idx = idx;
                    if cur.tex != last_tex {
                        //here to render
                        let rendered = draw(last_tex, start_idx, last_idx, once_rp_offset);
                        once_rp_offset += rendered;
                        cur_tex = cur.tex;
                        //end render
                        if rendered < idx - start_idx {
                            start_idx += rendered;
                            std::mem::drop(rp);
                            state.queue.submit(Some(encoder.finish()));
                            continue 'rp_loop;
                        }
                        last_tex = cur.tex;
                        start_idx = idx;
                    }
                }

                let rendered = draw(last_tex, start_idx, last_idx + 1, once_rp_offset);
                std::mem::drop(rp);
                state.queue.submit(Some(encoder.finish()));
                if start_idx + rendered == last_idx + 1 {
                    break;
                }
            }
        }
    }


    pub fn blit<'a>(&'a self, state: &GlobalState, src: &TextureView, render_target: &TextureView) {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("2D Render Encoder") });
        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
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
        });
        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("blit texture bind"),
            layout: &self.frag_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(src),
            }, BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            }],
        });
        {
            let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            let (w, h) = state.get_screen_size();
            let (w, h) = (w as f32, h as f32);
            state.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&(0..4).map(|x|
                Texture2DVertexData {
                    pos: match x {
                        0 => [0.0, h],
                        1 => [w, h],
                        2 => [0.0, 0.0],
                        3 => [w, 0.0],
                        _ => unreachable!()
                    },
                    coord: match x {
                        0 => [0.0, 0.0],
                        1 => [1.0, 0.0],
                        2 => [0.0, 1.0],
                        3 => [1.0, 1.0],
                        _ => unreachable!()
                    },
                }).collect::<Vec<_>>()));
            rp.set_pipeline(&self.render_pipeline);
            rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            rp.set_bind_group(0, &state.screen_uni_bind, &[]);
            rp.set_bind_group(1, &bind_group, &[]);

            rp.draw_indexed(0..6, 0, 0..1);
        }
        state.queue.submit(Some(encoder.finish()));
    }
}