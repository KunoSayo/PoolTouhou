use std::path::PathBuf;

use amethyst::{
    core::{
        components::Transform,
        ecs::{
            Component, DenseVecStorage, DispatcherBuilder, Join, ReadStorage, SystemData, World,
        },
    },
    prelude::*,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        ChangeDetection,
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                GraphContext,
                NodeBuffer, NodeImage, render::{PrepareResult, RenderGroup, RenderGroupDesc},
            },
            hal::{self, device::Device, format::Format, pso, pso::*},
            mesh::{AsVertex, VertexFormat},
            shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
        },
        submodules::{DynamicUniform, DynamicVertexBuffer},
        types::Backend, util,
    },
};
use amethyst::error::Error;
use amethyst_rendy::submodules::DynamicIndexBuffer;
use derivative::*;
use glsl_layout::*;

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/circle.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
       "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/circle.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
}


/// Draw triangles.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct InverseColorDesc;

impl InverseColorDesc {
    /// Create instance of `DrawCustomDesc` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for InverseColorDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let vertex = DynamicVertexBuffer::new();
        let index = DynamicIndexBuffer::new();

        let (pipeline, pipeline_layout) = build_custom_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout()],
        )?;

        Ok(Box::new(DrawInverseColor::<B> {
            pipeline,
            pipeline_layout,
            env,
            vertex,
            index,
            vertex_count: 0,
            change: Default::default(),
        }))
    }
}

#[derive(Debug)]
pub struct DrawInverseColor<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, InverseColorUniformArgs>,
    vertex: DynamicVertexBuffer<B, InverseColorVertexArg>,
    index: DynamicIndexBuffer<B, u32>,
    vertex_count: usize,
    change: ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for DrawInverseColor<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let inverse_color_circles = <ReadStorage<'_, InverseCircle>>::fetch(world);

        // Get our scale value
        let uniform_args = world.read_resource::<InverseColorUniformArgs>();

        // Write to our DynamicUniform
        self.env.write(factory, index, uniform_args.std140());

        //Update vertex count and see if it has changed
        let old_vertex_count = self.vertex_count;
        //4个顶点画圆
        self.vertex_count = (inverse_color_circles.count() * 4) as usize;
        let changed = old_vertex_count != self.vertex_count;

        let vertex_data_iter = inverse_color_circles.join().map(InverseCircle::get_args);
        self.vertex.write(
            factory,
            index,
            self.vertex_count as u64,
            vertex_data_iter,
        );
        let mut index_vec: Vec<u32> = Vec::new();
        let mut cur = 0;
        while cur + 3 <= self.vertex_count {
            index_vec.push(cur as u32);
            index_vec.push((cur + 1) as u32);
            index_vec.push((cur + 2) as u32);
            cur += 1;
            index_vec.push(cur as u32);
            index_vec.push((cur + 1) as u32);
            index_vec.push((cur + 2) as u32);
        }
        self.index.write(factory, index, 6, Some(index_vec));
        // Return with we can reuse the draw buffers using the utility struct ChangeDetection
        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        if self.vertex_count == 0 {
            return;
        }

        encoder.bind_graphics_pipeline(&self.pipeline);

        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);

        self.vertex.bind(index, 0, 0, &mut encoder);
        self.index.bind(index, 0, &mut encoder);
        // Draw the vertices
        unsafe {
            encoder.draw_indexed(0..(self.vertex_count + self.vertex_count / 2) as u32, 0, 0..1);
            // encoder.draw(0..self.vertex_count as u32, 0..1);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_custom_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    // Load the shaders
    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };

    // Build the pipeline
    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                // This Pipeline uses our custom vertex description and does not use instancing
                .with_vertex_desc(&[(InverseColorVertexArg::vertex(), pso::VertexInputRate::Vertex)])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
                // Add the shaders
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                // We are using alpha blending
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState {
                        color: BlendOp::Sub { src: Factor::Zero, dst: Factor::One },
                        alpha: BlendOp::Add { src: Factor::Zero, dst: Factor::One },
                    }),
                }]),
        )
        .build(factory, None);

    // Destoy the shaders once loaded
    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    // Handle the Errors
    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

/// A [RenderPlugin] for our custom plugin
#[derive(Default, Debug)]
pub struct RenderInverseColorCircle {}

impl<B: Backend> RenderPlugin<B> for RenderInverseColorCircle {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Add the required components to the world ECS
        world.register::<InverseCircle>();
        world.insert(InverseColorUniformArgs { projection: Default::default(), view: Default::default(), model: Default::default() });
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(Target::Main, |ctx| {
            // Add our Description
            ctx.add(RenderOrder::DisplayPostEffects, InverseColorDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct InverseColorVertexArg {
    pub pos: vec3,
    pub coord: vec2,
}

/// Required to send data into the shader.
/// These names must match the shader.
impl AsVertex for InverseColorVertexArg {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rgb32Sfloat, "pos"),
            (Format::Rg32Sfloat, "coord"),
        ))
    }
}


#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(4))]
pub struct InverseColorUniformArgs {
    pub projection: mat4,
    pub view: mat4,
    pub model: mat4,
}

#[derive(Debug, Default)]
pub struct InverseCircle {
    pub pos: Transform,
    pub radius: f32,
}

impl Component for InverseCircle {
    type Storage = DenseVecStorage<Self>;
}

impl InverseCircle {
    pub fn get_args(&self) -> Vec<InverseColorVertexArg> {
        let mut vec = Vec::new();
        vec.extend((0..3).map(|i| InverseColorVertexArg {
            pos: match i {
                0 => [-self.radius, self.radius, 0.0].into(),
                1 => [self.radius, self.radius, 0.0].into(),
                2 => [-self.radius, -self.radius, 0.0].into(),
                3 => [self.radius, -self.radius, 0.0].into(),
                _ => panic!("?")
            },
            coord: match i {
                0 => [0.0, 1.0].into(),
                1 => [1.0, 1.0].into(),
                2 => [0.0, 0.0].into(),
                3 => [1.0, 0.0].into(),
                _ => panic!("?")
            },
        }));
        vec
    }
}