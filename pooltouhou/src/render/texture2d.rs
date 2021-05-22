use wgpu::{RenderPipeline, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStage, BindingType, BufferBindingType, BufferSize, TextureSampleType, TextureViewDimension, PipelineLayout, ShaderFlags, CommandEncoder, RenderPass, VertexBufferLayout, VertexAttribute, VertexFormat, BindGroupEntry, BindingResource, BufferDescriptor, BufferUsage, Buffer, Texture, Sampler, BindGroup};
use crate::GraphicsState;
use crate::handles::ResourcesHandles;
use std::borrow::Cow;

use glsl_layout::*;
use wgpu::util::{DeviceExt, BufferInitDescriptor};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct Texture2DVertexData {
    pub pos: vec2,
    pub coord: vec2,
    pub color: vec4,
}

pub struct Texture2DObject {
    vertex: [Texture2DVertexData; 4],
    z: f32,
    tex: u32,
}

pub struct Texture2DRender {
    vertex_bind_group: BindGroup,
    pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    data: Vec<Texture2DObject>,
}

const OBJ_COUNT_IN_BUFFER: usize = 4096;

impl Texture2DRender {
    pub fn new(state: &mut GraphicsState, handles: &mut ResourcesHandles) -> Self {
        let vertex_bind_group_layout = state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let frag_bind_group_layout = state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                    filtering: false,
                    comparison: false,
                },
                count: None,
            }],
        });
        //done bind group

        let vertex_buffer = state.device.create_buffer(&BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<Texture2DVertexData>() * OBJ_COUNT_IN_BUFFER) as u64,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        });

        let index_buffer = state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &[0, 1, 2, 1, 2, 3],
            usage: BufferUsage::INDEX,
        });

        let pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&vertex_bind_group_layout, &frag_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &vertex_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &vertex_buffer,
                    offset: 0,
                    size: None,
                },
            }],
        });

        let vert = state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.v").unwrap())),
            flags: ShaderFlags::all(),
        });

        let frag = state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(handles.shaders.read().unwrap().get("n2dt.f").unwrap())),
            flags: ShaderFlags::all(),
        });

        let vertex_len = std::mem::size_of::<Texture2DVertexData>();
        let render_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: vertex_len as u64,
                    step_mode: Default::default(),
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }, VertexBufferLayout {
                    array_stride: vertex_len as u64,
                    step_mode: Default::default(),
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float2,
                        offset: 2 * 4,
                        shader_location: 1,
                    }],
                }, VertexBufferLayout {
                    array_stride: vertex_len as u64,
                    step_mode: Default::default(),
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float4,
                        offset: (2 + 2) * 4,
                        shader_location: 2,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag,
                entry_point: "main",
                targets: &[state.swapchain_desc.format.into()],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
        });

        Self {
            vertex_bind_group,
            pipeline_layout,
            render_pipeline,
            vertex_buffer,
            data: Vec::with_capacity(4096),
        }
    }

    pub fn render<'a>(&'a self, state: &mut GraphicsState, rp: &'a mut RenderPass<'a>) {
        rp.set_pipeline(&self.render_pipeline);
    }
}