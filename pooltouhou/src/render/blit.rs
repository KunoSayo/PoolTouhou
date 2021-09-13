// use amethyst::{
//     core::{
//         ecs::{
//             DispatcherBuilder, World,
//         },
//     },
//     renderer::{
//         bundle::{RenderPlan, RenderPlugin, Target},
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
//             shader::{Shader},
//         },
//         submodules::{DynamicVertexBuffer},
//         types::Backend, util,
//     },
// };
// use amethyst::error::Error;
// use glsl_layout::*;
// use amethyst_rendy::bundle::{TargetImage, TargetPlanOutputs, OutputColor, ImageOptions};
// use amethyst::window::ScreenDimensions;
// use amethyst_rendy::rendy::resource::{Image, Handle, ImageViewInfo, ViewKind, SubresourceRange, SamplerInfo, DescriptorSetLayout, DescriptorSet, Escape, Sampler, ImageView};
// use amethyst_rendy::rendy::graph::ImageId;
// use amethyst_rendy::rendy::hal::command::ClearColor;
// use gfx_hal::command::{ClearValue, ClearDepthStencil};
//
//
// #[derive(Clone, Debug, PartialEq)]
// pub struct BlitDesc {
//     src_id: ImageId,
// }
//
// impl BlitDesc {
//     pub fn new(src: ImageId) -> Self {
//         Self {
//             src_id: src
//         }
//     }
// }
//
// impl<B: Backend> RenderGroupDesc<B, World> for BlitDesc {
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
//         //image0 render to image1
//         //so blit from 0 to 1
//         let src = ctx.get_image(self.src_id).expect("need src");
//
//         let vertex = DynamicVertexBuffer::new();
//
//
//         // setup the offscreen texture descriptor set
//         let texture_layout: Handle<DescriptorSetLayout<B>> = Handle::from(
//             factory
//                 .create_descriptor_set_layout(vec![hal::pso::DescriptorSetLayoutBinding {
//                     binding: 0,
//                     ty: pso::DescriptorType::StorageImage,
//                     count: 1,
//                     stage_flags: pso::ShaderStageFlags::FRAGMENT,
//                     immutable_samplers: false,
//                 }, hal::pso::DescriptorSetLayoutBinding {
//                     binding: 1,
//                     ty: pso::DescriptorType::Sampler,
//                     count: 1,
//                     stage_flags: pso::ShaderStageFlags::FRAGMENT,
//                     immutable_samplers: false,
//                 }])
//                 .unwrap(),
//         );
//
//         let texture_set = factory.create_descriptor_set(texture_layout.clone()).unwrap();
//
//         // make a sampler
//         let sampler = factory.create_sampler(SamplerInfo {
//             min_filter: hal::image::Filter::Nearest,
//             mag_filter: hal::image::Filter::Nearest,
//             mip_filter: hal::image::Filter::Nearest,
//             wrap_mode: (hal::image::WrapMode::Clamp, hal::image::WrapMode::Clamp, hal::image::WrapMode::Clamp),
//             lod_bias: hal::image::Lod::ZERO,
//             lod_range: hal::image::Lod::ZERO..hal::image::Lod::MAX,
//             comparison: None,
//             border: [0.0, 0.0, 0.0, 0.0].into(),
//             normalized: true,
//             anisotropic: hal::image::Anisotropic::Off,
//         }).unwrap();
//
//
//         let (pipeline, pipeline_layout) = build_custom_pipeline(
//             factory,
//             subpass,
//             framebuffer_width,
//             framebuffer_height,
//             vec![texture_layout.raw()],
//         )?;
//
//         Ok(Box::new(Blit::<B> {
//             pipeline,
//             pipeline_layout,
//             vertex,
//             src: (*src).clone(),
//             change: Default::default(),
//             done: false,
//             texture_set,
//             sampler,
//             view: None,
//         }))
//     }
// }
//
// #[derive(Debug)]
// pub struct Blit<B: Backend> {
//     pipeline: B::GraphicsPipeline,
//     pipeline_layout: B::PipelineLayout,
//     vertex: DynamicVertexBuffer<B, BlitVertexArg>,
//     src: Handle<Image<B>>,
//     change: ChangeDetection,
//     done: bool,
//     texture_set: Escape<DescriptorSet<B>>,
//     sampler: Escape<Sampler<B>>,
//     view: Option<Escape<ImageView<B>>>,
// }
//
// impl<B: Backend> RenderGroup<B, World> for Blit<B> {
//     fn prepare(
//         &mut self,
//         factory: &Factory<B>,
//         _queue: QueueId,
//         index: usize,
//         _subpass: hal::pass::Subpass<'_, B>,
//         _world: &World,
//     ) -> PrepareResult {
//
//         //4个顶点画圆
//
//         let left = -0.5;
//         let right = 0.5;
//         let top = 0.5;
//         let bottom = -0.5;
//
//         let coord_left = 0.0;
//         let coord_right = 1.0;
//         let coord_top = 0.0;
//         let coord_bottom = 1.0;
//         let args = [BlitVertexArg {
//             pos: [left, top].into(),
//             coord: [coord_left, coord_top].into(),
//         }, BlitVertexArg {
//             pos: [right, top].into(),
//             coord: [coord_right, coord_top].into(),
//         }, BlitVertexArg {
//             pos: [left, bottom].into(),
//             coord: [coord_left, coord_bottom].into(),
//         }, BlitVertexArg {
//             pos: [right, bottom].into(),
//             coord: [coord_right, coord_bottom].into(),
//         }];
//         self.vertex.write(
//             factory,
//             index,
//             4,
//             vec![args],
//         );
//         let done = !self.done;
//         self.done = true;
//
//
//         let src = self.src.clone();
//
//         self.view.replace(factory.create_image_view(src.clone(), ImageViewInfo {
//             view_kind: ViewKind::D2,
//             format: hal::format::Format::Rgba32Sfloat,
//             swizzle: hal::format::Swizzle::NO,
//             range: SubresourceRange {
//                 aspects: hal::format::Aspects::COLOR | hal::format::Aspects::DEPTH | hal::format::Aspects::STENCIL,
//                 levels: 0..src.levels(),
//                 layers: 0..src.layers(),
//             },
//         }).unwrap());
//
//
//         // write to the texture description set
//         unsafe {
//             factory.device().write_descriptor_sets(vec![
//                 hal::pso::DescriptorSetWrite {
//                     set: self.texture_set.raw(),
//                     binding: 0,
//                     array_offset: 0,
//                     descriptors: Some(pso::Descriptor::Image(
//                         self.view.as_ref().unwrap().raw(),
//                         hal::image::Layout::General,
//                     )),
//                 },
//                 hal::pso::DescriptorSetWrite {
//                     set: self.texture_set.raw(),
//                     binding: 1,
//                     array_offset: 0,
//                     descriptors: Some(pso::Descriptor::Sampler(
//                         self.sampler.raw()
//                     )),
//                 }
//             ]);
//         }
//
//         // Return with we can reuse the draw buffers using the utility struct ChangeDetection
//         self.change.prepare_result(index, done)
//     }
//
//     fn draw_inline(
//         &mut self,
//         mut encoder: RenderPassEncoder<'_, B>,
//         index: usize,
//         _subpass: hal::pass::Subpass<'_, B>,
//         _world: &World,
//     ) {
//         if self.done {
//             encoder.bind_graphics_pipeline(&self.pipeline);
//
//             self.vertex.bind(index, 0, 0, &mut encoder);
//             // Draw the vertices
//             unsafe {
//                 encoder.bind_graphics_descriptor_sets(&self.pipeline_layout, 0, Some(self.texture_set.raw()), std::iter::empty());
//                 encoder.draw(0..4, 0..1);
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
//     let shader_vertex = unsafe { crate::render::BLIT_VERTEX.module(factory).unwrap() };
//     let shader_fragment = unsafe { crate::render::BLIT_FRAG.module(factory).unwrap() };
//
//
//     // Build the pipeline
//     let pipes = PipelinesBuilder::new()
//         .with_pipeline(
//             PipelineDescBuilder::new()
//                 // This Pipeline uses our custom vertex description and does not use instancing
//                 .with_vertex_desc(&[(BlitVertexArg::vertex(), pso::VertexInputRate::Vertex)])
//                 .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
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
// #[derive(Debug)]
// pub struct BlitToWindow {
//     dirty: bool,
//     dimensions: Option<ScreenDimensions>,
//     src: Target,
//     target: Target,
//     define: bool,
// }
//
// impl BlitToWindow {
//     pub fn new(src: Target, target: Target, define: bool) -> Self {
//         Self {
//             dirty: false,
//             dimensions: None,
//             src,
//             target,
//             define,
//         }
//     }
// }
//
// impl<B: Backend> RenderPlugin<B> for BlitToWindow {
//     fn on_build<'a, 'b>(
//         &mut self,
//         _world: &mut World,
//         _builder: &mut DispatcherBuilder<'a, 'b>,
//     ) -> Result<(), Error> {
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
//         plan.add_root(self.src);
//         if self.define {
//             plan.define_pass(self.src, TargetPlanOutputs {
//                 colors: vec![OutputColor::Image(ImageOptions {
//                     kind: hal::image::Kind::D2(self.dimensions.as_ref().unwrap().width() as _, self.dimensions.as_ref().unwrap().height() as _, 1, 1),
//                     levels: 1,
//                     format: hal::format::Format::Rgba32Sfloat,
//                     clear: Some(ClearValue::Color(ClearColor::from([0.0, 0.0, 0.0, 1.0]))),
//                 })],
//                 depth: Some(ImageOptions {
//                     kind: hal::image::Kind::D2(self.dimensions.as_ref().unwrap().width() as _, self.dimensions.as_ref().unwrap().height() as _, 1, 1),
//                     levels: 1,
//                     format: hal::format::Format::D32Sfloat,
//                     clear: Some(ClearValue::DepthStencil(ClearDepthStencil(0.0, 0))),
//                 }),
//             }).expect("define pass failed");
//             println!("defined {:?} as blit to window's src target in {}x{}", self.src,
//                      self.dimensions.as_ref().unwrap().width(), self.dimensions.as_ref().unwrap().height());
//         }
//
//         plan.add_root(self.target);
//         let src = self.src;
//         assert_ne!(src, self.target);
//         plan.extend_target(self.target, move |ctx| {
//             ctx.graph()
//             //blit to swap && draw to main
//             match ctx.get_image(TargetImage::Color(src, 0)) {
//                 Ok(s) => {
//                     let desc = BlitDesc::new(s).builder();
//                     ctx.add(9961, desc)?;
//                     Ok(())
//                 }
//                 Err(e) => {
//                     Err(e)
//                 }
//             }
//         });
//         Ok(())
//     }
// }
//
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
// #[repr(C, align(4))]
// pub struct BlitVertexArg {
//     pub pos: vec2,
//     pub coord: vec2,
// }
//
// /// Required to send data into the shader.
// /// These names must match the shader.
// impl AsVertex for BlitVertexArg {
//     fn vertex() -> VertexFormat {
//         VertexFormat::new((
//             (Format::Rg32Sfloat, "pos"),
//             (Format::Rg32Sfloat, "coord"),
//         ))
//     }
// }
