// use std::path::PathBuf;
//
// use amethyst::{
//     core::{
//         components::Transform,
//         ecs::{
//             Component, DispatcherBuilder, Join, ReadStorage, SystemData, World,
//         },
//     },
//     prelude::*,
//     renderer::{
//         bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
//         ChangeDetection,
//         pipeline::{PipelineDescBuilder, PipelinesBuilder},
//         rendy::{
//             command::{QueueId, RenderPassEncoder},
//             factory::Factory,
//             graph::{
//                 GraphContext,
//                 NodeBuffer, NodeImage, render::{PrepareResult, RenderGroup, RenderGroupDesc},
//             },
//             hal::{self, device::Device, format::Format, pso, pso::*},
//             mesh::{AsVertex, VertexFormat},
//             shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
//         },
//         submodules::{DynamicUniform, DynamicVertexBuffer},
//         types::Backend, util,
//     },
// };
// use amethyst::error::Error;
// use amethyst_rendy::submodules::DynamicIndexBuffer;
// use derivative::*;
// use glsl_layout::*;
// use crate::render::PthCameraUniformArgs;
// use amethyst::core::ecs::HashMapStorage;
//
// lazy_static::lazy_static! {
//     static ref VERTEX: SpirvShader = PathBufShaderInfo::new(
//         PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/normal3d.vert"),
//         ShaderKind::Vertex,
//         SourceLanguage::GLSL,
//        "main",
//     ).precompile().unwrap();
//
//     static ref FRAGMENT: SpirvShader = PathBufShaderInfo::new(
//         PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/circle.frag"),
//         ShaderKind::Fragment,
//         SourceLanguage::GLSL,
//         "main",
//     ).precompile().unwrap();
// }
//
//
// #[derive(Clone, Debug, PartialEq, Derivative)]
// #[derivative(Default(bound = ""))]
// pub struct InvertColorDesc;
//
// impl InvertColorDesc {
//     pub fn new() -> Self {
//         Default::default()
//     }
// }
//
// impl<B: Backend> RenderGroupDesc<B, World> for InvertColorDesc {
//     fn build(
//         self,
//         _ctx: &GraphContext<B>,
//         factory: &mut Factory<B>,
//         _queue: QueueId,
//         _world: &World,
//         framebuffer_width: u32,
//         framebuffer_height: u32,
//         subpass: hal::pass::Subpass<'_, B>,
//         _buffers: Vec<NodeBuffer>,
//         _images: Vec<NodeImage>,
//     ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
//         let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX | pso::ShaderStageFlags::FRAGMENT)?;
//         let vertex = DynamicVertexBuffer::new();
//         let index = DynamicIndexBuffer::new();
//
//         let (pipeline, pipeline_layout) = build_custom_pipeline(
//             factory,
//             subpass,
//             framebuffer_width,
//             framebuffer_height,
//             vec![env.raw_layout()],
//         )?;
//
//         Ok(Box::new(DrawInvertColor::<B> {
//             pipeline,
//             pipeline_layout,
//             env,
//             vertex,
//             indices: index,
//             index_count: 0,
//             change: Default::default(),
//         }))
//     }
// }
//
// #[derive(Debug)]
// pub struct DrawInvertColor<B: Backend> {
//     pipeline: B::GraphicsPipeline,
//     pipeline_layout: B::PipelineLayout,
//     env: DynamicUniform<B, PthCameraUniformArgs>,
//     vertex: DynamicVertexBuffer<B, InvertColorVertexArg>,
//     indices: DynamicIndexBuffer<B, u16>,
//     index_count: usize,
//     change: ChangeDetection,
// }
//
// impl<B: Backend> RenderGroup<B, World> for DrawInvertColor<B> {
//     fn prepare(
//         &mut self,
//         factory: &Factory<B>,
//         _queue: QueueId,
//         index: usize,
//         _subpass: hal::pass::Subpass<'_, B>,
//         world: &World,
//     ) -> PrepareResult {
//         let (inverse_color_circles, ) = <(ReadStorage<'_, InvertColorCircle>, )>::fetch(world);
//
//         let uniform_args = world.read_resource::<PthCameraUniformArgs>();
//
//         // Write to our DynamicUniform
//         self.env.write(factory, index, uniform_args.std140());
//
//         //Update vertex count and see if it has changed
//         let old_index_count = self.index_count;
//         //4个顶点画圆
//         let vertex_count = (inverse_color_circles.count() * 4) as usize;
//         //这不就是raw_count *6么 【恼】
//         //当初什么破代码 就留在这里看戏好了 :qp:
//         self.index_count = vertex_count + vertex_count >> 1;
//         let changed = old_index_count != self.index_count;
//         let vertex_data_iter = (&inverse_color_circles).join().flat_map(|circle| { circle.get_args() });
//
//         self.vertex.write(
//             factory,
//             index,
//             self.index_count as u64,
//             Some(vertex_data_iter.collect::<Box<[InvertColorVertexArg]>>()),
//         );
//         let mut index_vec: Vec<u16> = Vec::with_capacity(self.index_count.min(65532));
//         let mut cur = 0;
//         while cur + 4 <= vertex_count && cur + 3 <= 65532 {
//             index_vec.push(cur as u16);
//             index_vec.push((cur + 1) as u16);
//             index_vec.push((cur + 2) as u16);
//             index_vec.push((cur + 1) as u16);
//             index_vec.push((cur + 2) as u16);
//             index_vec.push((cur + 3) as u16);
//             cur += 4;
//         }
//         self.indices.write(factory, index, index_vec.len() as u64, Some(index_vec));
//         // Return with we can reuse the draw buffers using the utility struct ChangeDetection
//         self.change.prepare_result(index, changed)
//     }
//
//     fn draw_inline(
//         &mut self,
//         mut encoder: RenderPassEncoder<'_, B>,
//         index: usize,
//         _subpass: hal::pass::Subpass<'_, B>,
//         _world: &World,
//     ) {
//         if self.index_count == 0 {
//             return;
//         }
//
//         encoder.bind_graphics_pipeline(&self.pipeline);
//
//         self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
//
//         self.indices.bind(index, 0, &mut encoder);
//         self.vertex.bind(index, 0, 0, &mut encoder);
//         // Draw the vertices
//         unsafe {
//             let mut vertex_offset = 0;
//             let mut left = self.index_count;
//             while left > 0 {
//                 let rendered_indices_count = left.min(65532 as usize);
//                 encoder.draw_indexed(0..rendered_indices_count as u32, vertex_offset, 0..1);
//                 left -= rendered_indices_count;
//                 vertex_offset += rendered_indices_count as i32 * 4 / 6;
//             }
//         }
//     }
//
//     fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
//         unsafe {
//             factory.device().destroy_graphics_pipeline(self.pipeline);
//             factory
//                 .device()
//                 .destroy_pipeline_layout(self.pipeline_layout);
//         }
//     }
// }
//
// fn build_custom_pipeline<B: Backend>(
//     factory: &Factory<B>,
//     subpass: hal::pass::Subpass<'_, B>,
//     framebuffer_width: u32,
//     framebuffer_height: u32,
//     layouts: Vec<&B::DescriptorSetLayout>,
// ) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
//     let pipeline_layout = unsafe {
//         factory
//             .device()
//             .create_pipeline_layout(layouts, None as Option<(_, _)>)
//     }?;
//     // Load the shaders
//     let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
//     let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };
//
//
//     // Build the pipeline
//     let pipes = PipelinesBuilder::new()
//         .with_pipeline(
//             PipelineDescBuilder::new()
//                 // This Pipeline uses our custom vertex description and does not use instancing
//                 .with_vertex_desc(&[(InvertColorVertexArg::vertex(), pso::VertexInputRate::Vertex)])
//                 .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
//                 // Add the shaders
//                 .with_shaders(util::simple_shader_set(
//                     &shader_vertex,
//                     Some(&shader_fragment),
//                 ))
//                 .with_layout(&pipeline_layout)
//                 .with_subpass(subpass)
//                 .with_framebuffer_size(framebuffer_width, framebuffer_height)
//                 // We are using alpha blending
//                 .with_blend_targets(vec![pso::ColorBlendDesc {
//                     mask: pso::ColorMask::ALL,
//                     blend: Some(pso::BlendState {
//                         color: BlendOp::Sub { src: Factor::One, dst: Factor::One },
//                         alpha: BlendOp::Add { src: Factor::Zero, dst: Factor::One },
//                     }),
//                 }]),
//         )
//         .build(factory, None);
//
//     // Destoy the shaders once loaded
//     unsafe {
//         factory.destroy_shader_module(shader_vertex);
//         factory.destroy_shader_module(shader_fragment);
//     }
//
//     // Handle the Errors
//     match pipes {
//         Err(e) => {
//             unsafe {
//                 factory.device().destroy_pipeline_layout(pipeline_layout);
//             }
//             Err(e)
//         }
//         Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
//     }
// }
//
// /// A [RenderPlugin] for our custom plugin
// #[derive(Default, Debug)]
// pub struct RenderInvertColorCircle {
//     target: Target,
// }
//
// impl RenderInvertColorCircle {
//     pub fn with_target(mut self, target: Target) -> Self {
//         self.target = target;
//         self
//     }
// }
//
// impl<B: Backend> RenderPlugin<B> for RenderInvertColorCircle {
//     fn on_build<'a, 'b>(
//         &mut self,
//         world: &mut World,
//         _builder: &mut DispatcherBuilder<'a, 'b>,
//     ) -> Result<(), Error> {
//         // Add the required components to the world ECS
//         world.register::<InvertColorCircle>();
//         world.insert(PthCameraUniformArgs {
//             projection: [[1.0, 0.0, 0.0, 0.0],
//                 [0.0, 1.0, 0.0, 0.0],
//                 [0.0, 0.0, 1.0, 0.0],
//                 [0.0, 0.0, 0.0, 1.0]].into(),
//             view: Default::default(),
//         });
//         Ok(())
//     }
//
//     fn on_plan(
//         &mut self,
//         plan: &mut RenderPlan<B>,
//         _factory: &mut Factory<B>,
//         _world: &World,
//     ) -> Result<(), Error> {
//         plan.extend_target(self.target, |ctx| {
//             // Add our Description
//             ctx.add(RenderOrder::DisplayPostEffects, InvertColorDesc::new().builder())?;
//             Ok(())
//         });
//         Ok(())
//     }
// }
//
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
// #[repr(C, align(4))]
// pub struct InvertColorVertexArg {
//     pub pos: vec3,
//     pub coord: vec2,
// }
//
// /// Required to send data into the shader.
// /// These names must match the shader.
// impl AsVertex for InvertColorVertexArg {
//     fn vertex() -> VertexFormat {
//         VertexFormat::new((
//             (Format::Rgb32Sfloat, "pos"),
//             (Format::Rg32Sfloat, "coord"),
//         ))
//     }
// }
//
//
// #[derive(Debug, Default)]
// pub struct InvertColorCircle {
//     pub pos: Transform,
//     pub radius: f32,
// }
//
// impl Component for InvertColorCircle {
//     type Storage = HashMapStorage<Self>;
// }
//
// impl InvertColorCircle {
//     pub fn get_args(&self) -> Vec<InvertColorVertexArg> {
//         let mut vec = Vec::new();
//         let tran = self.pos.translation();
//         vec.extend((0..4).map(|i| InvertColorVertexArg {
//             pos: {
//                 match i {
//                     0 => [-self.radius + tran.x, self.radius + tran.y, tran.z].into(),
//                     1 => [self.radius + tran.x, self.radius + tran.y, tran.z].into(),
//                     2 => [-self.radius + tran.x, -self.radius + tran.y, tran.z].into(),
//                     3 => [self.radius + tran.x, -self.radius + tran.y, tran.z].into(),
//                     _ => unreachable!("?")
//                 }
//             }
//             ,
//             coord: match i {
//                 0 => [0.0, 1.0].into(),
//                 1 => [1.0, 1.0].into(),
//                 2 => [0.0, 0.0].into(),
//                 3 => [1.0, 0.0].into(),
//                 _ => unreachable!("?")
//             },
//         }));
//         vec
//     }
// }