use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use rayon::iter::*;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry,
           BindGroupLayout, BindGroupLayoutDescriptor,
           BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
           BufferBinding, BufferBindingType, BufferDescriptor, BufferUsage,
           IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
           RenderPassDescriptor, RenderPipeline, ShaderFlags, ShaderStage, TextureSampleType,
           TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::GraphicsState;
use crate::handles::ResourcesHandles;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(4))]
pub struct Texture2DVertexData {
    pub pos: [f32; 2],
    pub coord: [f32; 2],
}

const VERTEX_DATA_SIZE: usize = std::mem::size_of::<Texture2DVertexData>();
const OBJ_COUNT_IN_BUFFER: usize = 8192;

pub struct Texture2DObject {
    pub vertex: [Texture2DVertexData; 4],
    pub z: f32,
    pub tex: usize,
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
    vertex_uni_buffer: Buffer,
    vertex_uni_bind_group: BindGroup,
    frag_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    bind_groups: HashMap<usize, BindGroup>,
}

impl Texture2DRender {
    pub fn new(device: &wgpu::Device, target_color_state: wgpu::ColorTargetState, handles: &Arc<ResourcesHandles>, size: [f32; 2]) -> Self {
        let vertex_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let frag_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::FRAGMENT,
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
            size: (std::mem::size_of::<Texture2DVertexData>() * OBJ_COUNT_IN_BUFFER) as u64,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_uni_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsage::UNIFORM,
            contents: bytemuck::cast_slice(&size),
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&(0..OBJ_COUNT_IN_BUFFER).map(|obj_idx| {
                let offset = obj_idx as u16 * 6;
                [offset, offset + 1, offset + 2, offset + 1, offset + 2, offset + 3]
            }).collect::<Vec<_>>()),
            usage: BufferUsage::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&vertex_bind_group_layout, &frag_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_uni_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &vertex_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &vertex_uni_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let vert = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.v").unwrap())),
            flags: ShaderFlags::all(),
        });

        let frag = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.f").unwrap())),
            flags: ShaderFlags::all(),
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
            vertex_uni_buffer,
            vertex_uni_bind_group,
            frag_bind_group_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_groups: Default::default(),
        }
    }

    pub fn add_tex(&mut self, state: &mut GraphicsState, tex: usize) {
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

    pub fn render<'a>(&'a mut self, state: &mut GraphicsState, render_target: &TextureView, sorted_obj: &[&Texture2DObject]) {
        let mut iter = sorted_obj.iter().enumerate();
        if let Some((_, fst)) = iter.next() {
            let mut last_tex = fst.tex;
            let mut start_idx = 0;

            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("2D Render Encoder") });
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
            rp.set_pipeline(&self.render_pipeline);

            let mut draw = |tex, start_idx, end_idx| {
                if let Some(bind_group) = self.bind_groups.get(&tex) {
                    let mut cur = start_idx;
                    loop {
                        let mut end = cur + OBJ_COUNT_IN_BUFFER;

                        if end > end_idx {
                            end = end_idx;
                        }

                        sorted_obj[cur..end].par_iter().enumerate().for_each(|(obj_idx, obj)| {
                            let mut data = Vec::with_capacity(VERTEX_DATA_SIZE << 2);
                            for x in obj.vertex.iter() {
                                data.extend_from_slice(bytemuck::cast_slice(&x.pos));
                                data.extend_from_slice(bytemuck::cast_slice(&x.coord));
                            }
                            state.queue.write_buffer(&self.vertex_buffer, ((obj_idx << 2) * VERTEX_DATA_SIZE) as u64, &data);
                        });
                        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
                        rp.set_bind_group(0, &self.vertex_uni_bind_group, &[]);
                        rp.set_bind_group(1, &bind_group, &[]);

                        rp.draw_indexed(0..((end - cur) * 6) as u32, 0, 0..1);
                        cur = end;
                        if cur >= end_idx {
                            break;
                        }
                    }
                } else {
                    log::warn!("Tried to render not added tex handle by: {}", tex);
                }
            };
            let mut last_idx = 0;
            while let Some((idx, cur)) = iter.next() {
                if cur.tex != last_tex {
                    //here to render
                    draw(cur.tex, start_idx, idx);
                    //end render
                    last_tex = cur.tex;
                    start_idx = idx;
                }
                last_idx = idx;
            }

            draw(last_tex, start_idx, last_idx + 1);
            std::mem::drop(rp);
            state.queue.submit(Some(encoder.finish()));
        }
    }
}