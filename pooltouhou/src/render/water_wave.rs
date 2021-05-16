// use std::path::PathBuf;
//
// use amethyst::{
//     core::{
//         ecs::{
//             DispatcherBuilder, Join, ReadStorage, SystemData, World,
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
//             hal::{self, device::Device, format::Format, pso},
//             mesh::{AsVertex, VertexFormat},
//             shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
//         },
//         submodules::{DynamicUniform, DynamicVertexBuffer},
//         types::Backend, util,
//     },
// };
// use amethyst::error::Error;
// use glsl_layout::*;
// use crate::render::{PthCameraUniformArgs, SWAP};
// use crate::component::anime::WaterWave;
// use amethyst_rendy::bundle::{TargetImage, TargetPlanOutputs, OutputColor, ImageOptions};
// use amethyst::window::ScreenDimensions;
// use amethyst_rendy::rendy::resource::{Handle, SamplerInfo, DescriptorSetLayout, SubresourceRange, ViewKind, ImageViewInfo, DescriptorSet, Escape, ImageView, Sampler};
// use crate::render::blit::BlitDesc;
// use amethyst_rendy::rendy::graph::ImageId;
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
// #[derive(Clone, Debug, PartialEq)]
// pub struct WaterWaveDesc {
//     src_id: ImageId,
// }
//
// impl WaterWaveDesc {
//     pub fn new(src_id: ImageId) -> Self {
//         Self {
//             src_id
//         }
//     }
// }
//
// impl<B: Backend> RenderGroupDesc<B, World> for WaterWaveDesc {
//     fn build(
//         self,
//         ctx: &GraphContext<B>,
//         factory: &mut Factory<B>,
//         _queue: QueueId,
//         _world: &World,
//         framebuffer_width: u32,
//         framebuffer_height: u32,
//         subpass: hal::pass::Subpass<'_, B>,
//         _buffers: Vec<NodeBuffer>,
//         _images: Vec<NodeImage>,
//     ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
//         let src = ctx.get_image(self.src_id).expect("need src");
//
//         let view = factory.create_image_view((*src).clone(), ImageViewInfo {
//             view_kind: ViewKind::D2,
//             format: hal::format::Format::Rgba32Sfloat,
//             swizzle: hal::format::Swizzle::NO,
//             range: SubresourceRange {
//                 aspects: hal::format::Aspects::COLOR,
//                 levels: 0..src.levels(),
//                 layers: 0..src.layers(),
//             },
//         }).unwrap();
//
//         // setup the offscreen texture descriptor set
//         let texture_layout: Handle<DescriptorSetLayout<B>> = Handle::from(
//             factory
//                 .create_descriptor_set_layout(vec![hal::pso::DescriptorSetLayoutBinding {
//                     binding: 0,
//                     ty: pso::DescriptorType::CombinedImageSampler,
//                     count: 1,
//                     stage_flags: pso::ShaderStageFlags::FRAGMENT,
//                     immutable_samplers: false,
//                 }])
//                 .unwrap()
//         );
//
//         let texture_set = factory.create_descriptor_set(texture_layout.clone()).unwrap();
//
//         // write to the texture description set
//
//         // make a sampler
//         let sampler = factory.create_sampler(SamplerInfo {
//             min_filter: hal::image::Filter::Nearest,
//             mag_filter: hal::image::Filter::Nearest,
//             mip_filter: hal::image::Filter::Nearest,
//             wrap_mode: (hal::image::WrapMode::Border, hal::image::WrapMode::Border, hal::image::WrapMode::Border),
//             lod_bias: hal::image::Lod::ZERO,
//             lod_range: hal::image::Lod::ZERO..hal::image::Lod::MAX,
//             comparison: None,
//             border: [0.0, 0.0, 0.0, 0.0].into(),
//             normalized: true,
//             anisotropic: hal::image::Anisotropic::Off,
//         }).unwrap();
//
//         unsafe {
//             factory.device().write_descriptor_sets(vec![
//                 hal::pso::DescriptorSetWrite {
//                     set: texture_set.raw(),
//                     binding: 0,
//                     array_offset: 0,
//                     descriptors: Some(pso::Descriptor::CombinedImageSampler(
//                         view.raw(),
//                         hal::image::Layout::Present,
//                         sampler.raw(),
//                     )),
//                 }
//             ]);
//         }
//
//         let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX | pso::ShaderStageFlags::FRAGMENT)?;
//         let vertex = DynamicVertexBuffer::new();
//         let (pipeline, pipeline_layout) = build_custom_pipeline(
//             factory,
//             subpass,
//             framebuffer_width,
//             framebuffer_height,
//             vec![env.raw_layout()],
//         )?;
//
//         Ok(Box::new(DrawWaterWave::<B> {
//             pipeline,
//             pipeline_layout,
//             camera_u: env,
//             vertex,
//             change: Default::default(),
//             texture_set,
//             view,
//             vertex_count: 0,
//             sampler,
//         }))
//     }
// }
//
// #[derive(Debug)]
// pub struct DrawWaterWave<B: Backend> {
//     pipeline: B::GraphicsPipeline,
//     pipeline_layout: B::PipelineLayout,
//     camera_u: DynamicUniform<B, PthCameraUniformArgs>,
//     vertex: DynamicVertexBuffer<B, WaterWaveVertexArg>,
//     vertex_count: usize,
//     change: ChangeDetection,
//     texture_set: Escape<DescriptorSet<B>>,
//     view: Escape<ImageView<B>>,
//     sampler: Escape<Sampler<B>>,
// }
//
// impl<B: Backend> RenderGroup<B, World> for DrawWaterWave<B> {
//     fn prepare(
//         &mut self,
//         factory: &Factory<B>,
//         _queue: QueueId,
//         index: usize,
//         _subpass: hal::pass::Subpass<'_, B>,
//         world: &World,
//     ) -> PrepareResult {
//         let (water_waves, ) = <(ReadStorage<'_, WaterWave>, )>::fetch(world);
//
//         let uniform_args = world.read_resource::<PthCameraUniformArgs>();
//         self.camera_u.write(factory, index, uniform_args.std140());
//         //Update vertex count and see if it has changed
//         let old_vertex_count = self.vertex_count;
//         self.vertex_count = (water_waves.count() * 6) as usize;
//
//         let changed = old_vertex_count != self.vertex_count;
//         let vertex_data_iter = (&water_waves).join().flat_map(|w| { w.get_args() });
//
//         self.vertex.write(
//             factory,
//             index,
//             self.vertex_count as u64,
//             Some(vertex_data_iter.collect::<Box<[WaterWaveVertexArg]>>()),
//         );
//
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
//         if self.vertex_count == 0 {
//             return;
//         }
//
//         encoder.bind_graphics_pipeline(&self.pipeline);
//
//         self.camera_u.bind(index, &self.pipeline_layout, 0, &mut encoder);
//
//         self.vertex.bind(index, 0, 0, &mut encoder);
//         // Draw the vertices
//         unsafe {
//             encoder.bind_graphics_descriptor_sets(&self.pipeline_layout, 0, Some(self.texture_set.raw()), std::iter::empty());
//
//             encoder.draw(0..self.vertex_count as u32, 0..1);
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
//                 .with_vertex_desc(&[(WaterWaveVertexArg::vertex(), pso::VertexInputRate::Vertex)])
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
//                     blend: None,
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
// pub struct RenderWaterWave {
//     dirty: bool,
//     dimensions: Option<ScreenDimensions>,
//     target: Target,
// }
//
// impl RenderWaterWave {
//     pub fn with_target(mut self, target: Target) -> Self {
//         self.target = target;
//         self
//     }
// }
//
// impl<B: Backend> RenderPlugin<B> for RenderWaterWave {
//     fn on_build<'a, 'b>(
//         &mut self,
//         _world: &mut World,
//         _builder: &mut DispatcherBuilder<'a, 'b>,
//     ) -> Result<(), Error> {
//         // Add the required components to the world ECS
//
//         Ok(())
//     }
//
//     fn should_rebuild(&mut self, world: &World) -> bool {
//         let new_dimensions = world.try_fetch::<ScreenDimensions>();
//         if self.dimensions.as_ref() != new_dimensions.as_deref() {
//             self.dirty = true;
//             self.dimensions = new_dimensions.map(|d| (*d).clone());
//             return false;
//         }
//         self.dirty
//     }
//
//     fn on_plan(
//         &mut self,
//         plan: &mut RenderPlan<B>,
//         _factory: &mut Factory<B>,
//         world: &World,
//     ) -> Result<(), Error> {
//         let new_dimensions = world.try_fetch::<ScreenDimensions>();
//         self.dimensions = new_dimensions.map(|d| (*d).clone());
//         self.dirty = false;
//
//         plan.define_pass(SWAP, TargetPlanOutputs {
//             colors: vec![OutputColor::Image(ImageOptions {
//                 kind: hal::image::Kind::D2(self.dimensions.as_ref().unwrap().width() as _, self.dimensions.as_ref().unwrap().height() as _, 1, 1),
//                 levels: 1,
//                 format: hal::format::Format::Rgba32Sfloat,
//                 clear: None,
//             })],
//             depth: Some(ImageOptions {
//                 kind: hal::image::Kind::D2(self.dimensions.as_ref().unwrap().width() as _, self.dimensions.as_ref().unwrap().height() as _, 1, 1),
//                 levels: 1,
//                 format: hal::format::Format::D32Sfloat,
//                 clear: None,
//             }),
//         }).expect("define swap pass failed");
//
//
//         let target = self.target;
//
//         plan.extend_target(SWAP, move |ctx| {
//             let src = ctx.get_image(TargetImage::Color(target, 0)).expect("get src image failed.");
//             ctx.add(RenderOrder::DisplayPostEffects, BlitDesc::new(src).builder())?;
//             Ok(())
//         });
//         // plan.extend_target(target, move |ctx| {
//         //     let src = ctx.get_image(TargetImage::Color(SWAP, 0)).expect("get swap image failed.");
//         //
//         //     ctx.add(RenderOrder::DisplayPostEffects as i32 + 1i32, WaterWaveDesc::new(src).builder())?;
//         //     Ok(())
//         // });
//         Ok(())
//     }
// }
//
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
// #[repr(C, align(4))]
// pub struct WaterWaveVertexArg {
//     pub pos: vec3,
//     pub coord: vec2,
// }
//
// /// Required to send data into the shader.
// /// These names must match the shader.
// impl AsVertex for WaterWaveVertexArg {
//     fn vertex() -> VertexFormat {
//         VertexFormat::new((
//             (Format::Rgb32Sfloat, "pos"),
//             (Format::Rg32Sfloat, "coord"),
//         ))
//     }
// }
//
//
// impl WaterWave {
//     fn get_args(&self) -> Vec<WaterWaveVertexArg> {
//         let mut vec = Vec::new();
//         let tran = self.src.translation();
//         vec.extend((0..4).map(|i| WaterWaveVertexArg {
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